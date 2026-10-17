//! 邮件与附件的写入。
//!
//! 入参结构体定义在本模块，而非直接复用 `mail::parser::ParsedMessage`：
//! `DESIGN.md` 第三章的调用方向是 `mail → storage`，storage 不应反向依赖 mail。
//! 映射由 `mail` 层负责。

use rusqlite::{params, Connection, OptionalExtension, Transaction};

use super::error::StorageError;

/// 待写入的附件元数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewAttachment {
    pub filename: String,
    pub content_type: String,
    pub size: i64,
}

/// 待写入的一封邮件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewMessage {
    /// IMAP UID。同一 `folder_id` 内唯一，是去重键。
    pub uid: i64,
    pub message_id: Option<String>,
    pub subject: Option<String>,
    pub from_name: Option<String>,
    pub from_address: Option<String>,
    /// 收件人以逗号拼接，P0 不做规范化。
    pub to_addresses: Option<String>,
    /// Unix 秒。
    pub date_unix: i64,
    pub text_body: Option<String>,
    pub html_body: Option<String>,
    pub attachments: Vec<NewAttachment>,
}

/// 按 `(folder_id, uid)` 插入或更新一封邮件及其附件，返回本地 `messages.id`。
///
/// 去重键是联合唯一索引 `UNIQUE(folder_id, uid)`；重复同步同一封信只会更新，
/// 不会产生重复行。`is_seen` 不在更新列内 —— 本地已读状态不被同步覆盖。
/// 附件采取「全量替换」策略。
pub fn upsert_message(
    conn: &mut Connection,
    folder_id: i64,
    msg: &NewMessage,
) -> Result<i64, StorageError> {
    let tx = conn.transaction()?;
    let message_id = write_one(&tx, folder_id, msg, now_unix())?;
    tx.commit()?;
    Ok(message_id)
}

/// 把一封邮件标记为已读。
///
/// 用途：本地「已发送」归档刚写进去时 `is_seen` 默认是 0，
/// 会让发件箱永远挂着一堆「未读」——自己发的信当然已经读过了。
///
/// 不动 `upsert_message` 的插入默认值：那个默认值对**收到的**邮件是对的。
pub fn mark_seen(conn: &Connection, message_id: i64) -> Result<(), StorageError> {
    conn.execute(
        "UPDATE messages SET is_seen = 1 WHERE id = ?1",
        params![message_id],
    )?;
    Ok(())
}

/// 批量写入同一文件夹的多封邮件，返回写入条数。
///
/// **整个批次共用一个事务** —— 逐封调用 [`upsert_message`] 会开 N 个事务，
/// 同步一个文件夹时开销显著。批次失败则整批回滚。
pub fn upsert_messages(
    conn: &mut Connection,
    folder_id: i64,
    msgs: &[NewMessage],
) -> Result<u32, StorageError> {
    // 同一批次共用一个时间戳，避免同批邮件出现毫秒级前后不一致。
    let fetched_at = now_unix();

    let tx = conn.transaction()?;
    for msg in msgs {
        write_one(&tx, folder_id, msg, fetched_at)?;
    }
    tx.commit()?;

    Ok(msgs.len() as u32)
}

/// 当前 Unix 秒。
fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// 单封邮件的写入。**必须在调用方的事务里执行**，自身不开事务。
fn write_one(
    tx: &Transaction<'_>,
    folder_id: i64,
    msg: &NewMessage,
    fetched_at: i64,
) -> Result<i64, StorageError> {
    tx.execute(
        "INSERT INTO messages (
             folder_id, uid, message_id, subject, from_name, from_address,
             to_addresses, date_unix, text_body, html_body, fetched_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
         ON CONFLICT(folder_id, uid) DO UPDATE SET
             message_id   = excluded.message_id,
             subject      = excluded.subject,
             from_name    = excluded.from_name,
             from_address = excluded.from_address,
             to_addresses = excluded.to_addresses,
             date_unix    = excluded.date_unix,
             text_body    = excluded.text_body,
             html_body    = excluded.html_body,
             fetched_at   = excluded.fetched_at",
        params![
            folder_id,
            msg.uid,
            msg.message_id,
            msg.subject,
            msg.from_name,
            msg.from_address,
            msg.to_addresses,
            msg.date_unix,
            msg.text_body,
            msg.html_body,
            fetched_at,
        ],
    )?;

    let message_id: i64 = tx.query_row(
        "SELECT id FROM messages WHERE folder_id = ?1 AND uid = ?2",
        params![folder_id, msg.uid],
        |row| row.get(0),
    )?;

    tx.execute(
        "DELETE FROM attachments WHERE message_id = ?1",
        params![message_id],
    )?;
    for att in &msg.attachments {
        tx.execute(
            "INSERT INTO attachments (message_id, filename, content_type, size)
             VALUES (?1, ?2, ?3, ?4)",
            params![message_id, att.filename, att.content_type, att.size],
        )?;
    }

    Ok(message_id)
}

/// 列表用摘要。
///
/// **不是**数据库行 —— `DESIGN.md` P0-04 明确禁止命令层返回原始 DB 行，
/// 故这里只暴露前端渲染需要的字段。`Serialize` 是为了让 Tauri 命令直接返回它，
/// 避免再复制一份逐字段等同的 DTO。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct MessageSummary {
    pub id: i64,
    pub uid: i64,
    pub subject: Option<String>,
    pub from_name: Option<String>,
    pub from_address: Option<String>,
    /// Unix 秒。
    pub date_unix: i64,
    pub is_seen: bool,
    pub attachment_count: i64,
}

/// 按收件时间倒序列出某文件夹的邮件摘要，支持分页。
///
/// 排序加 `m.id` 作为次序键，保证 `date_unix` 相同的多封邮件顺序稳定，
/// 否则分页会漏信或重信。
pub fn list_messages(
    conn: &Connection,
    folder_id: i64,
    limit: u32,
    offset: u32,
) -> Result<Vec<MessageSummary>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT m.id, m.uid, m.subject, m.from_name, m.from_address, m.date_unix, m.is_seen,
                (SELECT COUNT(*) FROM attachments a WHERE a.message_id = m.id) AS attachment_count
         FROM messages m
         WHERE m.folder_id = ?1
         ORDER BY m.date_unix DESC, m.id DESC
         LIMIT ?2 OFFSET ?3",
    )?;

    let rows = stmt.query_map(params![folder_id, limit, offset], |row| {
        Ok(MessageSummary {
            id: row.get(0)?,
            uid: row.get(1)?,
            subject: row.get(2)?,
            from_name: row.get(3)?,
            from_address: row.get(4)?,
            date_unix: row.get(5)?,
            is_seen: row.get::<_, i64>(6)? != 0,
            attachment_count: row.get(7)?,
        })
    })?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

/// 附件元数据（详情面板展示用）。P0 阶段不取附件正文。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentMeta {
    pub filename: String,
    pub content_type: String,
    pub size: i64,
}

/// 邮件详情。
///
/// `html_body` 是**未净化**的原始 HTML。本层不做净化 ——
/// 净化属 `security` 的职责，且原始 HTML **绝不允许**直接交给前端渲染。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageDetail {
    pub id: i64,
    pub uid: i64,
    pub subject: Option<String>,
    pub from_name: Option<String>,
    pub from_address: Option<String>,
    pub to_addresses: Option<String>,
    /// Unix 秒。
    pub date_unix: i64,
    pub is_seen: bool,
    pub text_body: Option<String>,
    pub html_body: Option<String>,
    pub attachments: Vec<AttachmentMeta>,
}

/// 读取单封邮件的完整内容。邮件不存在返回 `None`。
pub fn load_message_detail(
    conn: &Connection,
    message_id: i64,
) -> Result<Option<MessageDetail>, StorageError> {
    let found = conn
        .query_row(
            "SELECT id, uid, subject, from_name, from_address, to_addresses,
                    date_unix, is_seen, text_body, html_body
             FROM messages WHERE id = ?1",
            params![message_id],
            |row| {
                Ok(MessageDetail {
                    id: row.get(0)?,
                    uid: row.get(1)?,
                    subject: row.get(2)?,
                    from_name: row.get(3)?,
                    from_address: row.get(4)?,
                    to_addresses: row.get(5)?,
                    date_unix: row.get(6)?,
                    is_seen: row.get::<_, i64>(7)? != 0,
                    text_body: row.get(8)?,
                    html_body: row.get(9)?,
                    attachments: Vec::new(),
                })
            },
        )
        .optional()?;

    let Some(mut detail) = found else {
        return Ok(None);
    };

    let mut stmt = conn.prepare(
        "SELECT filename, content_type, size FROM attachments
         WHERE message_id = ?1 ORDER BY id",
    )?;
    let rows = stmt.query_map(params![message_id], |row| {
        Ok(AttachmentMeta {
            filename: row.get(0)?,
            content_type: row.get(1)?,
            size: row.get(2)?,
        })
    })?;
    for row in rows {
        detail.attachments.push(row?);
    }

    Ok(Some(detail))
}

/// 统一收件箱用的摘要：在 [`MessageSummary`] 之上补出来源信息。
///
/// 跨文件夹聚合时「这封信属于哪个账户/文件夹」是必需信息，
/// 而 [`MessageSummary`] 里没有 —— 故单独一个 DTO，而不是去改既有结构。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct UnifiedMessageSummary {
    pub id: i64,
    pub uid: i64,
    pub folder_id: i64,
    pub folder_name: String,
    pub account_id: i64,
    pub account_email: String,
    pub subject: Option<String>,
    pub from_name: Option<String>,
    pub from_address: Option<String>,
    /// Unix 秒。
    pub date_unix: i64,
    pub is_seen: bool,
    pub attachment_count: i64,
}

/// 统一收件箱：跨文件夹（可再按账户、信箱名收窄）聚合邮件。
///
/// - `account_id = None`：全部账户
/// - `mailbox = Some("INBOX")`：只看各账户的 INBOX；`None` 则不限文件夹
///
/// 两个过滤条件都用 `?N IS NULL OR …` 表达，一条 SQL 覆盖四种组合。
pub fn list_unified_messages(
    conn: &Connection,
    account_id: Option<i64>,
    mailbox: Option<&str>,
    limit: u32,
    offset: u32,
) -> Result<Vec<UnifiedMessageSummary>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT m.id, m.uid, m.folder_id, f.name, f.account_id, a.email,
                m.subject, m.from_name, m.from_address, m.date_unix, m.is_seen,
                (SELECT COUNT(*) FROM attachments att WHERE att.message_id = m.id)
         FROM messages m
         JOIN folders  f ON f.id = m.folder_id
         JOIN accounts a ON a.id = f.account_id
         WHERE (?1 IS NULL OR f.account_id = ?1)
           AND (?2 IS NULL OR f.name = ?2 COLLATE NOCASE)
         ORDER BY m.date_unix DESC, m.id DESC
         LIMIT ?3 OFFSET ?4",
    )?;

    let rows = stmt.query_map(params![account_id, mailbox, limit, offset], |row| {
        Ok(UnifiedMessageSummary {
            id: row.get(0)?,
            uid: row.get(1)?,
            folder_id: row.get(2)?,
            folder_name: row.get(3)?,
            account_id: row.get(4)?,
            account_email: row.get(5)?,
            subject: row.get(6)?,
            from_name: row.get(7)?,
            from_address: row.get(8)?,
            date_unix: row.get(9)?,
            is_seen: row.get::<_, i64>(10)? != 0,
            attachment_count: row.get(11)?,
        })
    })?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}
