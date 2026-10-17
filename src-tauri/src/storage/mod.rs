//! 存储模块：表结构 / CRUD / FTS5。
//!
//! 对外接口以 `DESIGN.md` 第三章调用契约表为准。
//! FTS5 搜索属 P1 范围，本阶段不建虚表。

use std::path::Path;

use rusqlite::Connection;

pub mod account_store;
pub mod error;
pub mod folder_store;
pub mod message_store;
pub mod schema;
pub mod search_store;

pub use account_store::{
    create_account, delete_account, list_accounts, load_account_credentials, load_encrypted_secret,
    update_account, AccountDeletionImpact,
    AccountCredentials, AccountRecord,
};
pub use error::StorageError;
pub use folder_store::{ensure_folder, list_folders, FolderRecord};
pub use message_store::{
    list_messages, list_unified_messages, load_message_detail, upsert_message, upsert_messages,
    AttachmentMeta, MessageDetail, MessageSummary, NewAttachment, NewMessage,
    UnifiedMessageSummary,
};
pub use schema::{init_schema, SCHEMA_VERSION};
pub use search_store::{search_messages, MAX_SEARCH_LIMIT};

/// 打开（或创建）数据库文件，设好连接级 PRAGMA，并确保表结构就绪。
///
/// `foreign_keys` 是**每连接**设置且**在事务内无效**的，所以必须在这里、
/// 事务之外执行 —— 放进 [`init_schema`] 会是静默失效的空操作。
pub fn open_database(path: &Path) -> Result<Connection, StorageError> {
    let mut conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")?;
    init_schema(&mut conn)?;
    Ok(conn)
}
