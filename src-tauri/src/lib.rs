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

/// 密钥文件名。与数据库同目录，方案见 `docs/adr/ADR-0002-credential-encryption.md`。
const KEY_FILE_NAME: &str = "key.json";

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
            let crypto_key = security::load_or_create_key(&data_dir.join(KEY_FILE_NAME))?;

            app.manage(AppState::new(conn, crypto_key));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::accounts::add_account,
            commands::accounts::list_accounts,
            commands::accounts::resolve_provider,
            commands::compose::send_mail,
            commands::folders::list_folders,
            commands::messages::get_message,
            commands::messages::get_messages,
            commands::messages::list_unified_messages,
            commands::search::search_messages,
            commands::sync::sync_folder,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri 应用启动失败");
}
