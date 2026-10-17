//! 账户读写。
//!
//! 凭证一律以**密文**形态经过本模块 —— 明文密码不得进入 `storage`
//! （`AGENTS.md` 禁止项）。加解密由调用方用 `crate::security` 完成。

use rusqlite::{params, Connection, OptionalExtension};

use super::error::StorageError;

/// 新建账户，返回 `accounts.id`。
///
/// `secret_encrypted` 是 AES-256-GCM 密文（布局见 `ADR-0002`），**不是明文**。
/// `email` 重复会触发 `UNIQUE` 约束错误。
pub fn create_account(
    conn: &Connection,
    email: &str,
    display_name: Option<&str>,
    provider_id: &str,
    secret_encrypted: Option<&[u8]>,
) -> Result<i64, StorageError> {
    let created_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    conn.execute(
        "INSERT INTO accounts (email, display_name, provider_id, created_at, secret_encrypted)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            email,
            display_name,
            provider_id,
            created_at,
            secret_encrypted
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// 读取账户的加密凭证密文。
///
/// 账户不存在、或账户未设置凭证，都返回 `None`。
pub fn load_encrypted_secret(
    conn: &Connection,
    account_id: i64,
) -> Result<Option<Vec<u8>>, StorageError> {
    let found = conn
        .query_row(
            "SELECT secret_encrypted FROM accounts WHERE id = ?1",
            params![account_id],
            |row| row.get::<_, Option<Vec<u8>>>(0),
        )
        .optional()?;

    Ok(found.flatten())
}

/// 账户的同步所需信息：身份 + 加密凭证。
///
/// **一次查询取齐**，避免「先读身份、再读凭证」之间出现状态不一致。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountCredentials {
    pub email: String,
    pub provider_id: String,
    /// AES-256-GCM 密文；账户未设凭证时为 `None`。
    pub secret_encrypted: Option<Vec<u8>>,
}

/// 读取账户的同步所需信息。账户不存在返回 `None`。
pub fn load_account_credentials(
    conn: &Connection,
    account_id: i64,
) -> Result<Option<AccountCredentials>, StorageError> {
    conn.query_row(
        "SELECT email, provider_id, secret_encrypted FROM accounts WHERE id = ?1",
        params![account_id],
        |row| {
            Ok(AccountCredentials {
                email: row.get(0)?,
                provider_id: row.get(1)?,
                secret_encrypted: row.get(2)?,
            })
        },
    )
    .optional()
    .map_err(StorageError::from)
}

/// 账户摘要（**不含凭证**）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountRecord {
    pub id: i64,
    pub email: String,
    pub display_name: Option<String>,
    pub provider_id: String,
    /// 是否已存有加密凭证。只暴露布尔，**绝不暴露密文本身**。
    pub has_secret: bool,
}

/// 列出全部账户，按创建顺序。
///
/// 用 `secret_encrypted IS NOT NULL` 在 SQL 里算出布尔值，
/// 而不是把密文读出来再在 Rust 侧判空 —— 密文根本不该进入这一层之上。
pub fn list_accounts(conn: &Connection) -> Result<Vec<AccountRecord>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT id, email, display_name, provider_id, secret_encrypted IS NOT NULL
         FROM accounts ORDER BY id",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(AccountRecord {
            id: row.get(0)?,
            email: row.get(1)?,
            display_name: row.get(2)?,
            provider_id: row.get(3)?,
            has_secret: row.get::<_, i64>(4)? != 0,
        })
    })?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

/// 删除账户会连带删掉多少东西。**先算清楚再删** —— 删是不可逆的。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccountDeletionImpact {
    pub messages: i64,
    pub attachments: i64,
}

/// 删除一个账户，连带它的文件夹、邮件与附件。
///
/// 靠 schema 里的 `ON DELETE CASCADE` 链式删除（`folders → messages → attachments`），
/// 因此**必须**在 `PRAGMA foreign_keys = ON` 的连接上调用 ——
/// 该 pragma 由 `storage::open_database` 在事务外设置。
///
/// 邮件的 FTS 索引由 `messages_fts_ad` 触发器跟着删，无需手工清。
///
/// 账户不存在返回 `None` 且不删任何东西。
pub fn delete_account(
    conn: &mut Connection,
    account_id: i64,
) -> Result<Option<AccountDeletionImpact>, StorageError> {
    let tx = conn.transaction()?;

    let messages: i64 = tx.query_row(
        "SELECT COUNT(*) FROM messages
         WHERE folder_id IN (SELECT id FROM folders WHERE account_id = ?1)",
        params![account_id],
        |row| row.get(0),
    )?;

    let attachments: i64 = tx.query_row(
        "SELECT COUNT(*) FROM attachments
         WHERE message_id IN (
             SELECT m.id FROM messages m
             JOIN folders f ON f.id = m.folder_id
             WHERE f.account_id = ?1
         )",
        params![account_id],
        |row| row.get(0),
    )?;

    let changed = tx.execute("DELETE FROM accounts WHERE id = ?1", params![account_id])?;

    // 没这个账户：回滚（其实什么都没改），报告未命中。
    if changed == 0 {
        return Ok(None);
    }

    tx.commit()?;
    Ok(Some(AccountDeletionImpact {
        messages,
        attachments,
    }))
}

/// 修改账户的可修改字段，返回是否命中该账户。
///
/// `new_secret_encrypted` 为 `None` 表示**不改凭证**。
/// 想清空凭证不该用空数组表达 —— 空数组会被当成"设了一个空密文"，
/// 那是坏数据；清空凭证目前没有产品需求，`None` / `Some(有效密文)` 两种状态足够。
///
/// 改邮箱会连带改服务商（`provider_id` 由调用方重新解析后传入），
/// 否则会出现「地址是 A 家的、连接参数还是 B 家的」这种不一致。
pub fn update_account(
    conn: &Connection,
    account_id: i64,
    email: &str,
    display_name: Option<&str>,
    provider_id: &str,
    new_secret_encrypted: Option<&[u8]>,
) -> Result<bool, StorageError> {
    let changed = match new_secret_encrypted {
        Some(secret) => conn.execute(
            "UPDATE accounts
             SET email = ?2, display_name = ?3, provider_id = ?4, secret_encrypted = ?5
             WHERE id = ?1",
            params![account_id, email, display_name, provider_id, secret],
        )?,
        None => conn.execute(
            "UPDATE accounts
             SET email = ?2, display_name = ?3, provider_id = ?4
             WHERE id = ?1",
            params![account_id, email, display_name, provider_id],
        )?,
    };

    Ok(changed != 0)
}
