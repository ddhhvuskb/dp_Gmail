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
