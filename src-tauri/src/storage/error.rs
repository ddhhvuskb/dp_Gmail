//! `storage` 模块的错误类型。

/// 存储模块统一错误。
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("数据库错误：{0}")]
    Sqlite(#[from] rusqlite::Error),
}
