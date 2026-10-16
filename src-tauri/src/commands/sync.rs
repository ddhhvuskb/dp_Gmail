//! 同步命令。

use crate::mail::{parser, provider, sync};
use crate::security;
use crate::state::AppState;
use crate::storage;

/// 单次同步最多拉取的邮件数。
pub const MAX_SYNC_BATCH: u32 = 50;

/// 一次同步的结果。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SyncReport {
    pub folder_id: i64,
    pub fetched: u32,
    pub stored: u32,
}

/// 归一化同步批大小，收敛到 `1..=MAX_SYNC_BATCH`。
pub fn normalize_batch(limit: u32) -> u32 {
    limit.clamp(1, MAX_SYNC_BATCH)
}

/// 同步一个文件夹的最近邮件到本地。
///
/// **关键约束**：`rusqlite` 的 `MutexGuard` 绝不跨 `.await` 存活，
/// 否则本命令的 future 不是 `Send`，Tauri 无法调度。
/// 因此流程被切成「同步段 → 异步段 → 同步段」，中间那段全程不持锁。
#[tauri::command]
pub async fn sync_folder(
    state: tauri::State<'_, AppState>,
    account_id: i64,
    mailbox: String,
    limit: u32,
) -> Result<SyncReport, String> {
    let limit = normalize_batch(limit);

    // ── 第一段（同步）：读身份、解密凭证。锁在这一段结束时释放。
    let (email, provider_id, secret) = {
        let conn = state
            .db
            .lock()
            .map_err(|err| format!("数据库连接不可用：{err}"))?;

        let account = storage::load_account_credentials(&conn, account_id)
            .map_err(|err| err.to_string())?
            .ok_or_else(|| format!("账户不存在：{account_id}"))?;

        let blob = account
            .secret_encrypted
            .ok_or_else(|| format!("账户 {account_id} 未设置凭证"))?;

        let secret = security::decrypt_secret(&blob, &state.crypto_key)
            .map_err(|err| format!("凭证解密失败：{err}"))?;

        (account.email, account.provider_id, secret)
    };

    let cfg = provider::find_by_id(&provider_id)
        .ok_or_else(|| format!("未知的服务商标识：{provider_id}"))?;

    // ── 第二段（异步）：全程不持锁。
    let fetched = sync::fetch_recent(cfg, &email, &secret, &mailbox, limit)
        .await
        .map_err(|err| err.to_string())?;

    let mut messages = Vec::with_capacity(fetched.len());
    for item in &fetched {
        let parsed = parser::parse_message(&item.raw).map_err(|err| err.to_string())?;
        messages.push(sync::to_new_message(item.uid, &parsed));
    }

    // ── 第三段（同步）：重新加锁，建文件夹行 + 批量落库。
    let mut conn = state
        .db
        .lock()
        .map_err(|err| format!("数据库连接不可用：{err}"))?;

    let folder_id =
        storage::ensure_folder(&conn, account_id, &mailbox).map_err(|err| err.to_string())?;

    let stored =
        storage::upsert_messages(&mut conn, folder_id, &messages).map_err(|err| err.to_string())?;

    Ok(SyncReport {
        folder_id,
        fetched: fetched.len() as u32,
        stored,
    })
}
