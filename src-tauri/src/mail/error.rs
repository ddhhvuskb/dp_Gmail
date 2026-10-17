//! `mail` 模块的错误类型。

/// 邮件模块统一错误。
#[derive(Debug, thiserror::Error)]
pub enum MailError {
    #[error("连接超时（{0} 秒）")]
    Timeout(u64),

    #[error("暂不支持的 TLS 模式：{0}")]
    UnsupportedTlsMode(&'static str),

    #[error("邮件解析失败：不是合法的 RFC5322 报文")]
    Parse,

    /// TLS 失败。
    ///
    /// 存 `String` 而不是接具体后端错误，两个原因：
    /// 1. 桌面用 native-tls、Android 用 rustls，两者的错误类型不同（ADR-0006 §9.6）；
    /// 2. `Io` 已经占了 `From<std::io::Error>`，再加一个同类型的 `#[from]` 会冲突。
    #[error("TLS 错误：{0}")]
    Tls(String),

    #[error("IMAP 协议错误：{0}")]
    Imap(#[from] async_imap::error::Error),

    #[error("网络错误：{0}")]
    Io(#[from] std::io::Error),

    #[error("收件人不能为空")]
    NoRecipients,

    #[error("邮箱地址格式非法：{0}")]
    BadAddress(String),

    #[error("附件的 Content-Type 非法：{0}")]
    BadContentType(String),

    #[error("邮件组装失败：{0}")]
    BuildMail(#[from] lettre::error::Error),

    #[error("SMTP 错误：{0}")]
    Smtp(#[from] lettre::transport::smtp::Error),
}
