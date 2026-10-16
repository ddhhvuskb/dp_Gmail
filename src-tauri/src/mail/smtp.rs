//! SMTP 发信。
//!
//! 用 `lettre` 而非手写 SMTP：EHLO / STARTTLS / AUTH / MIME 组装的边界情况极多，
//! 手写必然在某个服务商的某个扩展上翻车。决策记录见 `docs/adr/ADR-0003`。
//!
//! 服务器地址一律由调用方从 [`crate::mail::provider`] 取，本模块不得出现地址字面量。

use std::time::Duration;

use lettre::message::{Mailbox, Message};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};

use super::error::MailError;
use super::provider::{ProviderConfig, TlsMode};

/// 发信超时。与 IMAP 保持一致。
pub const SMTP_TIMEOUT_SECS: u64 = 30;

/// 待发送的邮件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutgoingMail {
    /// 形如 `张三 <zhangsan@qq.com>` 或裸地址 `zhangsan@qq.com`。
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    pub body_text: String,
}

/// 经 SMTP 投递一封邮件。
///
/// 凭证只在内存中传递，**不得**被日志记录。
pub async fn send_mail(
    cfg: &ProviderConfig,
    user: &str,
    secret: &str,
    mail: &OutgoingMail,
) -> Result<(), MailError> {
    if cfg.smtp_tls != TlsMode::Implicit {
        return Err(MailError::UnsupportedTlsMode("SMTP STARTTLS"));
    }
    if mail.to.is_empty() {
        return Err(MailError::NoRecipients);
    }

    let mut builder = Message::builder()
        .from(parse_mailbox(&mail.from)?)
        .subject(mail.subject.clone());
    for recipient in &mail.to {
        builder = builder.to(parse_mailbox(recipient)?);
    }
    let email = builder.body(mail.body_text.clone())?;

    let transport = AsyncSmtpTransport::<Tokio1Executor>::relay(cfg.smtp_host)?
        .port(cfg.smtp_port)
        .timeout(Some(Duration::from_secs(SMTP_TIMEOUT_SECS)))
        .credentials(Credentials::new(user.to_owned(), secret.to_owned()))
        .build::<Tokio1Executor>();

    transport.send(email).await?;
    Ok(())
}

/// 解析 `Name <addr>` 或裸地址。
fn parse_mailbox(raw: &str) -> Result<Mailbox, MailError> {
    raw.trim()
        .parse::<Mailbox>()
        .map_err(|_| MailError::BadAddress(raw.trim().to_owned()))
}
