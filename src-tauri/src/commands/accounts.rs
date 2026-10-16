//! 账户相关命令。

use crate::mail::provider::{self, ProviderConfig};
use crate::security;
use crate::state::AppState;
use crate::storage;

/// 返回给前端的账户摘要。
///
/// **不是**数据库行 —— `DESIGN.md` P0-04 禁止命令层返回原始 DB 行。
/// 放在命令层而非 `storage`，是因为 `provider_name` 需要查 `mail::provider`，
/// 而 `DESIGN.md` 第三章的调用方向是 `mail → storage`，storage 不应反向依赖 mail。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct AccountView {
    pub id: i64,
    pub email: String,
    pub display_name: Option<String>,
    pub provider_id: String,
    pub provider_name: String,
    /// 是否已存有加密凭证。**绝不返回凭证本身**。
    pub has_secret: bool,
}

/// 校验新增账户的入参，并解析出服务商。
///
/// 这是命令层唯一有逻辑的地方（参数校验），故单独成函数以便独立测试。
pub fn validate_new_account(
    email: &str,
    secret: &str,
) -> Result<(String, &'static ProviderConfig), String> {
    let email = email.trim();
    if email.is_empty() {
        return Err("邮箱地址不能为空".to_owned());
    }
    if secret.is_empty() {
        return Err("授权码/密码不能为空".to_owned());
    }

    // 服务商自动识别（DESIGN.md P0-05）。未收录的域名交给用户另想办法，不猜。
    let provider = provider::resolve_by_email(email)
        .ok_or_else(|| format!("暂不支持的邮箱服务商：{email}"))?;

    Ok((email.to_owned(), provider))
}

/// 新增一个邮箱账户。密码/授权码经 AES-256-GCM 加密后落库，**不存明文**。
#[tauri::command]
pub fn add_account(
    state: tauri::State<'_, AppState>,
    email: String,
    secret: String,
    display_name: Option<String>,
) -> Result<AccountView, String> {
    let (email, provider) = validate_new_account(&email, &secret)?;

    let encrypted =
        security::encrypt_secret(&secret, &state.crypto_key).map_err(|err| err.to_string())?;

    let conn = state
        .db
        .lock()
        .map_err(|err| format!("数据库连接不可用：{err}"))?;

    let id = storage::create_account(
        &conn,
        &email,
        display_name.as_deref(),
        provider.id,
        Some(&encrypted),
    )
    .map_err(|err| err.to_string())?;

    Ok(AccountView {
        id,
        email,
        display_name,
        provider_id: provider.id.to_owned(),
        provider_name: provider.display_name.to_owned(),
        has_secret: true,
    })
}
