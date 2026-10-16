//! 服务商配置表。
//!
//! 本模块是全项目**唯一**允许出现服务器地址字面量的地方（见 `AGENTS.md` 禁止项）。
//!
//! **收录标准：地址必须来自可核实的公开来源，不得凭记忆填写。**
//! 每一条的出处记在 `docs/tasks/P3-01-provider-compat.md`。

/// TLS 连接模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsMode {
    /// 连上即 TLS。QQ：IMAP 993 / SMTP 465。
    Implicit,
    /// 明文连接后用 STARTTLS 升级。QQ：SMTP 587。
    StartTls,
}

/// 单个服务商的连接参数。
#[derive(Debug, Clone, Copy)]
pub struct ProviderConfig {
    pub id: &'static str,
    pub display_name: &'static str,
    pub domains: &'static [&'static str],
    pub imap_host: &'static str,
    pub imap_port: u16,
    pub imap_tls: TlsMode,
    pub smtp_host: &'static str,
    pub smtp_port: u16,
    pub smtp_tls: TlsMode,
}

/// 已收录的服务商。**服务器地址的唯一来源。**
///
/// 每条地址的核实来源见 `docs/tasks/P3-01-provider-compat.md` §2。
/// 未核实的服务商**不得**加入本表 —— 填错一个主机名会让用户以为是自己密码错了。
pub const PROVIDERS: &[ProviderConfig] = &[
    ProviderConfig {
        id: "qq",
        display_name: "QQ邮箱",
        domains: &["qq.com", "vip.qq.com", "foxmail.com"],
        imap_host: "imap.qq.com",
        imap_port: 993,
        imap_tls: TlsMode::Implicit,
        smtp_host: "smtp.qq.com",
        smtp_port: 465,
        smtp_tls: TlsMode::Implicit,
    },
    ProviderConfig {
        id: "netease163",
        display_name: "网易163邮箱",
        domains: &["163.com"],
        imap_host: "imap.163.com",
        imap_port: 993,
        imap_tls: TlsMode::Implicit,
        smtp_host: "smtp.163.com",
        smtp_port: 465,
        smtp_tls: TlsMode::Implicit,
    },
];

/// 由邮箱地址解析服务商配置。
///
/// 域名大小写不敏感，允许首尾空白；本地部分或域名为空、未收录的域名返回 `None`，
/// 由调用方交给用户手填，不 panic。
pub fn resolve_by_email(email: &str) -> Option<&'static ProviderConfig> {
    let (local, domain) = email.trim().rsplit_once('@')?;
    let domain = domain.trim().to_ascii_lowercase();
    if local.is_empty() || domain.is_empty() {
        return None;
    }
    PROVIDERS
        .iter()
        .find(|p| p.domains.contains(&domain.as_str()))
}

/// 按服务商标识取配置。
///
/// 同步时手里只有已落库的 `accounts.provider_id`，没有邮箱地址可反查，
/// 故需要这个按 id 的入口。找不到返回 `None`，由调用方报错，不 panic。
pub fn find_by_id(id: &str) -> Option<&'static ProviderConfig> {
    PROVIDERS.iter().find(|p| p.id == id)
}
