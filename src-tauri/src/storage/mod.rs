//! 存储模块：表结构 / CRUD / FTS5。
//!
//! 对外接口以 `DESIGN.md` 第三章调用契约表为准。
//! FTS5 搜索属 P1 范围，本阶段不建虚表。

use std::path::Path;

use rusqlite::Connection;

pub mod error;
pub mod message_store;
pub mod schema;

pub use error::StorageError;
pub use message_store::{list_messages, upsert_message, MessageSummary, NewAttachment, NewMessage};
pub use schema::init_schema;

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
