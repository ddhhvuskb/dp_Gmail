//! `mail::parser` 的单元测试。
//!
//! 覆盖 `docs/tasks/P0-02-mime-parse.md` §5.1 约定的用例。

use dp_gmail::mail::error::MailError;
use dp_gmail::mail::parser::{parse_message, MailAddress};

/// 把 LF 分隔的报文转成 CRLF 分隔的字节串，避免源码里堆 `\r\n`。
fn crlf(s: &str) -> Vec<u8> {
    s.replace('\n', "\r\n").into_bytes()
}

const PLAIN: &str = "Subject: Hello\n\
From: Alice <alice@example.com>\n\
To: bob@example.com\n\
Date: Mon, 13 Oct 2025 10:00:00 +0800\n\
Message-ID: <abc@example.com>\n\
\n\
Hi there\n";

#[test]
fn parses_plain_text() {
    let parsed = parse_message(&crlf(PLAIN)).expect("应能解析纯文本邮件");

    assert_eq!(parsed.subject.as_deref(), Some("Hello"));
    // mail-parser 会剥掉尖括号，此处记录其实际行为
    assert_eq!(parsed.message_id.as_deref(), Some("abc@example.com"));
    assert_eq!(
        parsed.from,
        Some(MailAddress {
            name: Some("Alice".to_owned()),
            address: "alice@example.com".to_owned(),
        })
    );
    assert_eq!(parsed.to.len(), 1);
    assert_eq!(parsed.to[0].address, "bob@example.com");
    assert_eq!(parsed.text_body.as_deref(), Some("Hi there\r\n"));
    assert!(parsed.attachments.is_empty());
}

#[test]
fn converts_date_header_to_unix_integer() {
    let parsed = parse_message(&crlf(PLAIN)).expect("应能解析");
    // Mon, 13 Oct 2025 10:00:00 +0800 == 2025-10-13T02:00:00Z
    assert_eq!(parsed.date_unix, Some(1_760_320_800));
}

#[test]
fn decodes_rfc2047_subject() {
    // =?utf-8?B?5L2g5aW9?= 是「你好」的 base64 编码字
    let raw = crlf("Subject: =?utf-8?B?5L2g5aW9?=\nFrom: a@qq.com\n\nbody\n");
    let parsed = parse_message(&raw).expect("应能解析");
    assert_eq!(parsed.subject.as_deref(), Some("你好"));
}

#[test]
fn parses_multiple_recipients() {
    let raw = crlf("Subject: x\nTo: a@x.com, Bob <b@y.com>\n\nbody\n");
    let parsed = parse_message(&raw).expect("应能解析");

    let addresses: Vec<&str> = parsed.to.iter().map(|a| a.address.as_str()).collect();
    assert_eq!(addresses, vec!["a@x.com", "b@y.com"]);
    assert_eq!(parsed.to[1].name.as_deref(), Some("Bob"));
}

#[test]
fn parses_attachment_metadata() {
    let raw = crlf(
        "Subject: with attachment\n\
Content-Type: multipart/mixed; boundary=\"BOUND\"\n\
\n\
--BOUND\n\
Content-Type: text/plain\n\
\n\
hello\n\
--BOUND\n\
Content-Type: application/pdf; name=\"report.pdf\"\n\
Content-Disposition: attachment; filename=\"report.pdf\"\n\
Content-Transfer-Encoding: base64\n\
\n\
SGVsbG8=\n\
--BOUND--\n",
    );

    let parsed = parse_message(&raw).expect("应能解析带附件的邮件");
    assert_eq!(parsed.attachments.len(), 1);

    let att = &parsed.attachments[0];
    assert_eq!(att.filename, "report.pdf");
    assert_eq!(att.content_type, "application/pdf");
    // base64 "SGVsbG8=" -> "Hello"，5 字节
    assert_eq!(att.size, 5);
}

#[test]
fn tolerates_missing_subject() {
    let raw = crlf("From: a@qq.com\n\nbody\n");
    let parsed = parse_message(&raw).expect("无 Subject 也应能解析");
    assert_eq!(parsed.subject, None);
    assert_eq!(parsed.from.unwrap().address, "a@qq.com");
}

#[test]
fn handles_unknown_8bit_charset() {
    // GBK 的「你好」，配以非标准 charset 声明，靠 full_encoding 兜底
    let mut raw =
        b"Subject: legacy\nContent-Type: text/plain; charset=\"unknown-8bit\"\n\n".to_vec();
    raw.extend_from_slice(&[0xC4, 0xE3, 0xBA, 0xC3]);

    let parsed = parse_message(&raw).expect("非标准编码不应导致解析失败");
    assert!(
        parsed.text_body.as_deref().is_some_and(|t| !t.is_empty()),
        "unknown-8bit 正文应被兜底解码，实际：{:?}",
        parsed.text_body
    );
}

/// 空输入是唯一确定返回 `None` 的情形：
/// `mail-parser` 的 `parse_` 仅在「未解析出任何头部 且 消息为空」时返回 None
/// （见 `parsers/message.rs:553-569`）。
#[test]
fn rejects_empty_input() {
    let err = parse_message(b"").expect_err("空输入应判定为解析失败");
    assert!(matches!(err, MailError::Parse), "错误类型不符：{err:?}");
}

/// 二进制垃圾**不会**报错：mail-parser 走 best-effort，产出一条空消息。
/// 记录这条行为，避免后续误以为垃圾输入会返回 `MailError::Parse`。
#[test]
fn parses_binary_garbage_best_effort() {
    let parsed = parse_message(b"\x00\x01\x02\x03").expect("垃圾输入走 best-effort，不报错");
    assert_eq!(parsed.subject, None);
    assert_eq!(parsed.from, None);
    assert!(parsed.attachments.is_empty());
}
