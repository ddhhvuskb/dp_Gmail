//! RFC822 报文解析。
//!
//! 编码转换（base64 / quoted-printable / RFC2047 编码字 / charset）一律交给
//! `mail-parser`，本模块**不得**手写任何解码逻辑（见 `AGENTS.md` 禁止项）。

use mail_parser::{Addr, Address, DateTime, MessageParser, MessagePart, MimeHeaders};

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

/// 解析原始 RFC822 报文。解析器返回 `None` 时给 [`MailError::Parse`]。
pub fn parse_message(raw: &[u8]) -> Result<ParsedMessage, MailError> {
    let message = MessageParser::default()
        .parse(raw)
        .ok_or(MailError::Parse)?;

    let to_mail_address = |addr: &Addr<'_>| {
        addr.address().map(|address| MailAddress {
            name: addr.name().map(str::to_owned),
            address: address.to_owned(),
        })
    };

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
        from: message
            .from()
            .and_then(Address::first)
            .and_then(to_mail_address),
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
