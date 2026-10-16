//! 应用全局状态。
//!
//! 由 `run()` 在 `setup` 阶段注入，命令层用 `tauri::State<'_, AppState>` 取用。

use std::sync::Mutex;

use rusqlite::Connection;

use crate::security::KEY_LEN;

/// 应用状态：数据库连接 + 加密密钥。
pub struct AppState {
    /// `rusqlite::Connection` 是 `Send` 但**不是** `Sync`，共享必须加锁。
    pub db: Mutex<Connection>,

    /// AES-256-GCM 密钥，启动时由密钥文件读出或生成。
    ///
    /// 只在进程内存中，**不落日志、不进 `Debug`**（见 `ADR-0002` §2.4）。
    pub crypto_key: [u8; KEY_LEN],
}

impl AppState {
    pub fn new(db: Connection, crypto_key: [u8; KEY_LEN]) -> Self {
        Self {
            db: Mutex::new(db),
            crypto_key,
        }
    }
}
