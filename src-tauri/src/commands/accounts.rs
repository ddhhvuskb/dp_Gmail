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

/// 服务商识别结果，供前端在用户输入邮箱时实时展示。
///
/// 服务器地址只用于**显示**（让用户看清将要连到哪里），
/// 前端不得据此硬编码 —— 真正的连接参数始终由后端从 `provider.rs` 取。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ProviderView {
    pub provider_id: String,
    pub provider_name: String,
    pub imap_host: String,
    pub imap_port: u16,
    pub smtp_host: String,
    pub smtp_port: u16,
}

/// `ProviderConfig` → 前端视图。两条命令共用，避免字段各写一遍之后漂移。
fn to_view(cfg: &'static ProviderConfig) -> ProviderView {
    ProviderView {
        provider_id: cfg.id.to_owned(),
        provider_name: cfg.display_name.to_owned(),
        imap_host: cfg.imap_host.to_owned(),
        imap_port: cfg.imap_port,
        smtp_host: cfg.smtp_host.to_owned(),
        smtp_port: cfg.smtp_port,
    }
}

/// 由邮箱地址识别服务商。未收录的域名返回 `null`（前端据此提示"暂不支持"）。
///
/// 只读、不涉密码、不落库。
#[tauri::command]
pub fn resolve_provider(email: String) -> Option<ProviderView> {
    provider::resolve_by_email(&email).map(to_view)
}

/// 列出全部服务商，供「自动识别不出来时手动选择」用。
///
/// 企业邮箱用的是客户自己的域名，[`provider::resolve_by_email`] 对它**永远**
/// 返回 `None`。这条命令让那个场景有路可走，而不是死路一条。
///
/// 只读、不涉密码、不落库。
#[tauri::command]
pub fn list_providers() -> Vec<ProviderView> {
    provider::PROVIDERS.iter().map(to_view).collect()
}

/// 解析服务商：**用户显式指定优先，否则按域名自动识别**。
///
/// 抽出来是因为新增与编辑都要用同一套判定 —— 两处各写一遍必然漂移。
fn resolve_for(email: &str, provider_id: Option<&str>) -> Result<&'static ProviderConfig, String> {
    match provider_id {
        Some(id) => provider::find_by_id(id).ok_or_else(|| format!("未知的服务商标识：{id}")),
        // 服务商自动识别（DESIGN.md P0-05）。未收录的域名交给用户另想办法，不猜。
        None => provider::resolve_by_email(email)
            .ok_or_else(|| format!("暂不支持的邮箱服务商：{email}")),
    }
}

/// 校验新增账户的入参，并解析出服务商。
///
/// 这是命令层唯一有逻辑的地方（参数校验），故单独成函数以便独立测试。
///
/// `provider_id` 是**用户显式指定**的服务商，优先级高于自动识别：
/// 企业邮箱用客户自己的域名，自动识别对它无解，只能由用户选。
pub fn validate_new_account(
    email: &str,
    secret: &str,
    provider_id: Option<&str>,
) -> Result<(String, &'static ProviderConfig), String> {
    let email = email.trim();
    if email.is_empty() {
        return Err("邮箱地址不能为空".to_owned());
    }
    if secret.is_empty() {
        return Err("授权码/密码不能为空".to_owned());
    }

    Ok((email.to_owned(), resolve_for(email, provider_id)?))
}

/// 校验编辑账户的入参。与新增的唯一区别：**授权码是可选的**（不传＝不改凭证）。
pub fn validate_account_update(
    email: &str,
    provider_id: Option<&str>,
) -> Result<(String, &'static ProviderConfig), String> {
    let email = email.trim();
    if email.is_empty() {
        return Err("邮箱地址不能为空".to_owned());
    }

    Ok((email.to_owned(), resolve_for(email, provider_id)?))
}

/// 新增一个邮箱账户。密码/授权码经 AES-256-GCM 加密后落库，**不存明文**。
///
/// `provider_id` 可选：自动识别不出来时（企业邮箱）由用户显式指定。
#[tauri::command]
pub fn add_account(
    state: tauri::State<'_, AppState>,
    email: String,
    secret: String,
    display_name: Option<String>,
    provider_id: Option<String>,
) -> Result<AccountView, String> {
    let (email, provider) = validate_new_account(&email, &secret, provider_id.as_deref())?;

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

/// 列出全部账户。
///
/// 服务商展示名由命令层补上（`storage` 不认识 `mail::provider`）。
/// 遇到已下线的旧服务商标识时，展示名回退为标识本身而不是报错 ——
/// 列表里少一个名字，总好过整页打不开。
#[tauri::command]
pub fn list_accounts(state: tauri::State<'_, AppState>) -> Result<Vec<AccountView>, String> {
    let conn = state
        .db
        .lock()
        .map_err(|err| format!("数据库连接不可用：{err}"))?;

    let records = storage::list_accounts(&conn).map_err(|err| err.to_string())?;

    Ok(records
        .into_iter()
        .map(|record| {
            let provider_name = provider::find_by_id(&record.provider_id).map_or_else(
                || record.provider_id.clone(),
                |cfg| cfg.display_name.to_owned(),
            );

            AccountView {
                id: record.id,
                email: record.email,
                display_name: record.display_name,
                provider_id: record.provider_id,
                provider_name,
                has_secret: record.has_secret,
            }
        })
        .collect())
}

/// 修改一个已有账户。
///
/// `secret` 为 `None` 表示**不改凭证**（用户只改显示名或邮箱时不必重填授权码）。
/// 改了邮箱就重新解析服务商 —— 否则会出现「地址是 A 家的、连接参数还是 B 家的」。
#[tauri::command]
pub fn update_account(
    state: tauri::State<'_, AppState>,
    account_id: i64,
    email: String,
    display_name: Option<String>,
    secret: Option<String>,
    provider_id: Option<String>,
) -> Result<AccountView, String> {
    let (email, provider) = validate_account_update(&email, provider_id.as_deref())?;

    let encrypted = match secret.as_deref() {
        Some(plain) if plain.is_empty() => {
            return Err("授权码/密码不能为空；不想改动就不要填".to_owned())
        }
        Some(plain) => Some(
            security::encrypt_secret(plain, &state.crypto_key).map_err(|err| err.to_string())?,
        ),
        None => None,
    };

    let conn = state
        .db
        .lock()
        .map_err(|err| format!("数据库连接不可用：{err}"))?;

    let found = storage::update_account(
        &conn,
        account_id,
        &email,
        display_name.as_deref(),
        provider.id,
        encrypted.as_deref(),
    )
    .map_err(|err| err.to_string())?;

    if !found {
        return Err(format!("账户不存在：{account_id}"));
    }

    // 没改凭证时也要如实报告库里到底有没有凭证，不能靠猜
    let has_secret = storage::load_encrypted_secret(&conn, account_id)
        .map_err(|err| err.to_string())?
        .is_some();

    Ok(AccountView {
        id: account_id,
        email,
        display_name,
        provider_id: provider.id.to_owned(),
        provider_name: provider.display_name.to_owned(),
        has_secret,
    })
}

/// 删除账户时**实际删掉了什么**。回给前端用于提示，也用于测试断言。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct AccountDeletionReport {
    pub messages: i64,
    pub attachments: i64,
}

/// 删除一个账户，连带它已下载的文件夹、邮件与附件。
///
/// **不可逆**。前端必须先让用户确认，并把这里返回的数量显示出来。
#[tauri::command]
pub fn delete_account(
    state: tauri::State<'_, AppState>,
    account_id: i64,
) -> Result<AccountDeletionReport, String> {
    let mut conn = state
        .db
        .lock()
        .map_err(|err| format!("数据库连接不可用：{err}"))?;

    let impact = storage::delete_account(&mut conn, account_id).map_err(|err| err.to_string())?;

    impact
        .map(|it| AccountDeletionReport {
            messages: it.messages,
            attachments: it.attachments,
        })
        .ok_or_else(|| format!("账户不存在：{account_id}"))
}
