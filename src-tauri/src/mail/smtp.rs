//! SMTP 发信。
//!
//! 用 `lettre` 而非手写 SMTP：EHLO / STARTTLS / AUTH / MIME 组装的边界情况极多，
//! 手写必然在某个服务商的某个扩展上翻车。决策记录见 `docs/adr/ADR-0003`。
//!
//! 服务器地址一律由调用方从 [`crate::mail::provider`] 取，本模块不得出现地址字面量。

use std::time::Duration;

use lettre::message::{Mailbox, Message, MultiPart};
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
    /// 提供时以 `multipart/alternative` 同时发送纯文本与 HTML。
    ///
    /// 纯文本那一份**不可省**：部分客户端与无障碍工具只读纯文本，
    /// 也影响垃圾邮件评分。
    pub body_html: Option<String>,
}

/// 经 SMTP 投递一封邮件。
///
/// 校验一律排在建连之前 —— 非法地址不该先白白建立一次 TLS 连接。
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

    let email = build_message(mail)?;

    let transport = AsyncSmtpTransport::<Tokio1Executor>::relay(cfg.smtp_host)?
        .port(cfg.smtp_port)
        .timeout(Some(Duration::from_secs(SMTP_TIMEOUT_SECS)))
        .credentials(Credentials::new(user.to_owned(), secret.to_owned()))
        .build::<Tokio1Executor>();

    transport.send(email).await?;
    Ok(())
}

/// 组装 RFC5322 报文。
///
/// 同时有纯文本与 HTML 时用 `multipart/alternative`；只有纯文本时直接单部分，
/// 不额外套一层 multipart —— 单部分报文兼容性最好，没必要为不存在的第二种格式付代价。
///
/// 公开此函数是为了让它**可被单测直接验证**：组装结果不依赖网络，
/// 不该因为 `send_mail` 会建连就连带失去测试覆盖。
pub fn build_message(mail: &OutgoingMail) -> Result<Message, MailError> {
    let mut builder = Message::builder()
        .from(parse_mailbox(&mail.from)?)
        .subject(mail.subject.clone());

    for recipient in &mail.to {
        builder = builder.to(parse_mailbox(recipient)?);
    }

    match &mail.body_html {
        Some(html) => {
            let alternative =
                MultiPart::alternative_plain_html(mail.body_text.clone(), html.clone());
            Ok(builder.multipart(alternative)?)
        }
        None => Ok(builder.body(mail.body_text.clone())?),
    }
}

/// 解析 `Name <addr>` 或裸地址。
fn parse_mailbox(raw: &str) -> Result<Mailbox, MailError> {
    raw.trim()
        .parse::<Mailbox>()
        .map_err(|_| MailError::BadAddress(raw.trim().to_owned()))
}
