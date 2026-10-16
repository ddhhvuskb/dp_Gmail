//! 邮件与附件的写入。
//!
//! 入参结构体定义在本模块，而非直接复用 `mail::parser::ParsedMessage`：
//! `DESIGN.md` 第三章的调用方向是 `mail → storage`，storage 不应反向依赖 mail。
//! 映射由 `mail` 层负责。

use rusqlite::{params, Connection};

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
    let fetched_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let tx = conn.transaction()?;

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

    tx.commit()?;
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
