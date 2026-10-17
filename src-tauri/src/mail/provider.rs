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
///
/// 表里有两类条目，区别就在 `domains`：
///
/// | 类别 | `domains` | 怎么被选中 |
/// | --- | --- | --- |
/// | 个人邮箱 | 非空 | [`resolve_by_email`] 按域名**自动识别** |
/// | 企业邮箱 | **空** | 用客户自己的域名，自动识别无解 ⇒ 由用户**手动指定 id** |
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
    // ---- 以下为企业邮箱：**无法**用域名自动识别，只能由用户手动选择 ----
    //
    // 企业邮箱用的是客户自己的域名，故 `domains` 留空 ⇒ `resolve_by_email`
    // 永不匹配它们，只能经 `find_by_id` 由用户显式指定。
    //
    // 地址来源：2026-10-17 用 TLS **实测**（TCP 连通 + 证书主机名校验 + 协议 banner），
    // 不是转述第三方页面 —— 证书主机名匹配即证明该地址确实是这家服务商的。
    // 实测记录见 `docs/tasks/P3-01-provider-compat.md` §2.2。
    //
    // **新增条目必须追加在末尾**：多处测试以 `PROVIDERS[0]` 为基准，插到前面会改基准。
    ProviderConfig {
        id: "exmail_qq",
        display_name: "腾讯企业邮",
        domains: &[],
        imap_host: "imap.exmail.qq.com",
        imap_port: 993,
        imap_tls: TlsMode::Implicit,
        smtp_host: "smtp.exmail.qq.com",
        smtp_port: 465,
        smtp_tls: TlsMode::Implicit,
    },
    ProviderConfig {
        id: "qiye_163",
        display_name: "网易企业邮",
        domains: &[],
        imap_host: "imap.qiye.163.com",
        imap_port: 993,
        imap_tls: TlsMode::Implicit,
        // 994 是实测确认的端口。**别照抄常见的 465** —— 这条是撞过的坑。
        smtp_host: "smtp.qiye.163.com",
        smtp_port: 994,
        smtp_tls: TlsMode::Implicit,
    },
    ProviderConfig {
        id: "aliyun_mail",
        display_name: "阿里云企业邮箱",
        domains: &[],
        imap_host: "imap.mxhichina.com",
        imap_port: 993,
        imap_tls: TlsMode::Implicit,
        smtp_host: "smtp.mxhichina.com",
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
