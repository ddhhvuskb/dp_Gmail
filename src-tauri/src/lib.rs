//! 本地优先跨平台桌面邮件客户端 —— 后端库。
//!
//! 模块边界与调用契约见 `DESIGN.md` 第三章。

use tauri::Manager;

pub mod commands;
pub mod mail;
pub mod security;
pub mod state;
pub mod storage;

use state::AppState;

/// 数据库文件名。放在应用数据目录下。
const DB_FILE_NAME: &str = "dp_gmail.sqlite";

/// 启动 Tauri 应用。
///
/// 命令清单见 `DESIGN.md` P0-04。命令只做参数校验和转发，不写业务逻辑。
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let conn = storage::open_database(&data_dir.join(DB_FILE_NAME))?;
            app.manage(AppState::new(conn));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![commands::messages::get_messages])
        .run(tauri::generate_context!())
        .expect("Tauri 应用启动失败");
}
