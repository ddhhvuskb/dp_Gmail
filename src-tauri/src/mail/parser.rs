//! RFC822 报文解析。
//!
//! 编码转换（base64 / quoted-printable / RFC2047 编码字 / charset）一律交给
//! `mail-parser`，本模块**不得**手写任何解码逻辑（见 `AGENTS.md` 禁止项）。

use mail_parser::{Addr, Address, DateTime, Message, MessageParser, MessagePart, MimeHeaders};

use super::error::MailError;

/// 邮件地址（显示名 + 地址）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailAddress {
    pub name: Option<String>,
    pub address: String,
}

/// 附件元数据。P0 阶段只取元数据，不落附件正文。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedAttachment {
    pub filename: String,
    pub content_type: String,
    pub size: usize,
}

/// 解析结果。时间一律 Unix 整数（`DESIGN.md` P0-03 约束）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedMessage {
    pub subject: Option<String>,
    pub from: Option<MailAddress>,
    pub to: Vec<MailAddress>,
    pub date_unix: Option<i64>,
    pub message_id: Option<String>,
    pub text_body: Option<String>,
    pub html_body: Option<String>,
    pub attachments: Vec<ParsedAttachment>,
}

/// 把一个 `Addr` 转成 [`MailAddress`]。
///
/// **没有地址的返回 `None`** —— `From: 某某` 这类只有显示名的写法真实存在，
/// 见 [`resolve_from`]。
fn to_mail_address(addr: &Addr<'_>) -> Option<MailAddress> {
    addr.address().map(|address| MailAddress {
        name: addr.name().map(str::to_owned),
        address: address.to_owned(),
    })
}

/// 取发件人，`From` 里没有地址时回退到 `Sender`。
///
/// RFC 5322 要求 `From` 必须有地址，但**现实不遵守**。2026-10-17 拿真实收件箱
/// 做解析体检时撞到 2 例（31 封里占 2 封）：
///
/// ```text
/// From: =?utf-8?b?Tk9J56ue6LWb5Yqe5YWs5a6k?=     ← 解码后是「NOI竞赛办公室」
/// Sender: reg@notice.noi.cn
/// ```
///
/// 只认 `From` 会让这类邮件的发件人**整个变成空**，界面上就成了无主邮件。
/// 回退到 `Sender` 取地址的**同时保留 `From` 的显示名** —— 「NOI竞赛办公室」
/// 比裸地址对用户有用得多。
fn resolve_from(message: &Message<'_>) -> Option<MailAddress> {
    let from_first = message.from().and_then(Address::first);

    if let Some(addr) = from_first.and_then(to_mail_address) {
        return Some(addr);
    }

    let sender = message
        .sender()
        .and_then(Address::first)
        .and_then(to_mail_address)?;

    let name = from_first
        .and_then(Addr::name)
        .map(str::to_owned)
        .or(sender.name);

    Some(MailAddress {
        name,
        address: sender.address,
    })
}

/// 解析原始 RFC822 报文。解析器返回 `None` 时给 [`MailError::Parse`]。
pub fn parse_message(raw: &[u8]) -> Result<ParsedMessage, MailError> {
    let message = MessageParser::default()
        .parse(raw)
        .ok_or(MailError::Parse)?;

    let to_attachment = |part: &MessagePart<'_>| ParsedAttachment {
        filename: part
            .content_disposition()
            .and_then(|cd| cd.attribute("filename"))
            .or_else(|| part.content_type().and_then(|ct| ct.attribute("name")))
            .unwrap_or("unnamed")
            .to_owned(),
        content_type: part
            .content_type()
            .map(|ct| match ct.subtype() {
                Some(sub) => format!("{}/{}", ct.ctype(), sub),
                None => ct.ctype().to_owned(),
            })
            .unwrap_or_else(|| "application/octet-stream".to_owned()),
        size: part.contents().len(),
    };

    Ok(ParsedMessage {
        subject: message.subject().map(str::to_owned),
        from: resolve_from(&message),
        to: message
            .to()
            .map(|list| list.iter().filter_map(to_mail_address).collect())
            .unwrap_or_default(),
        date_unix: message.date().map(DateTime::to_timestamp),
        message_id: message.message_id().map(str::to_owned),
        text_body: message
            .text_part(0)
            .and_then(MessagePart::text_contents)
            .map(str::to_owned),
        html_body: message
            .html_part(0)
            .and_then(MessagePart::text_contents)
            .map(str::to_owned),
        attachments: message.attachments().map(to_attachment).collect(),
    })
}
