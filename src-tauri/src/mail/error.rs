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

    #[error("TLS 错误：{0}")]
    Tls(#[from] async_native_tls::Error),

    #[error("IMAP 协议错误：{0}")]
    Imap(#[from] async_imap::error::Error),

    #[error("网络错误：{0}")]
    Io(#[from] std::io::Error),

    #[error("收件人不能为空")]
    NoRecipients,

    #[error("邮箱地址格式非法：{0}")]
    BadAddress(String),

    #[error("邮件组装失败：{0}")]
    BuildMail(#[from] lettre::error::Error),

    #[error("SMTP 错误：{0}")]
    Smtp(#[from] lettre::transport::smtp::Error),
}
