//! `mail::smtp` 的单元测试。
//!
//! **刻意只测网络请求之前的路径**：真正的投递需要真实 SMTP 服务器，
//! 在单测里连 smtp.qq.com 既慢又没意义。这里验证参数校验、校验顺序与报文组装。

use dp_gmail::mail::error::MailError;
use dp_gmail::mail::provider::{ProviderConfig, TlsMode, PROVIDERS};
use dp_gmail::mail::smtp::{build_message, send_mail, OutgoingMail};

fn draft(from: &str, to: &[&str]) -> OutgoingMail {
    OutgoingMail {
        from: from.to_owned(),
        to: to.iter().map(|s| (*s).to_owned()).collect(),
        subject: "测试主题".to_owned(),
        body_text: "测试正文".to_owned(),
        body_html: None,
    }
}

#[tokio::test]
async fn rejects_empty_recipients() {
    let err = send_mail(&PROVIDERS[0], "a@qq.com", "secret", &draft("a@qq.com", &[]))
        .await
        .expect_err("空收件人应被拒");
    assert!(
        matches!(err, MailError::NoRecipients),
        "错误类型不符：{err:?}"
    );
}

#[tokio::test]
async fn rejects_non_implicit_smtp_tls() {
    let cfg = ProviderConfig {
        smtp_tls: TlsMode::StartTls,
        ..PROVIDERS[0]
    };

    let err = send_mail(
        &cfg,
        "a@qq.com",
        "secret",
        &draft("a@qq.com", &["b@qq.com"]),
    )
    .await
    .expect_err("STARTTLS 尚未实现，应返回错误");
    assert!(
        matches!(err, MailError::UnsupportedTlsMode("SMTP STARTTLS")),
        "错误类型不符：{err:?}"
    );
}

#[tokio::test]
async fn rejects_invalid_from_before_touching_the_network() {
    // 这条用例同时验证**校验顺序**：若地址解析被挪到建连之后，
    // 本测试会因为真去连 smtp.qq.com 而变慢甚至超时。
    let err = send_mail(
        &PROVIDERS[0],
        "a@qq.com",
        "secret",
        &draft("这不是一个邮箱地址", &["b@qq.com"]),
    )
    .await
    .expect_err("非法发件人应被拒");
    assert!(
        matches!(err, MailError::BadAddress(ref raw) if raw == "这不是一个邮箱地址"),
        "错误类型不符：{err:?}"
    );
}

#[tokio::test]
async fn rejects_invalid_recipient_before_touching_the_network() {
    let err = send_mail(
        &PROVIDERS[0],
        "a@qq.com",
        "secret",
        &draft("a@qq.com", &["good@qq.com", "@@@"]),
    )
    .await
    .expect_err("非法收件人应被拒");
    assert!(
        matches!(err, MailError::BadAddress(ref raw) if raw == "@@@"),
        "错误类型不符：{err:?}"
    );
}

#[tokio::test]
async fn accepts_display_name_form() {
    // 「张三 <a@qq.com>」是合法写法；这里验证它不会在解析阶段被拒。
    // 用非法收件人截断，避免真的发起投递。
    let mail = OutgoingMail {
        from: "张三 <zhangsan@qq.com>".to_owned(),
        to: vec!["@@@".to_owned()],
        subject: "x".to_owned(),
        body_text: "y".to_owned(),
        body_html: None,
    };

    let err = send_mail(&PROVIDERS[0], "zhangsan@qq.com", "secret", &mail)
        .await
        .expect_err("收件人非法，应在解析阶段被拒");

    // 若 from 解析失败，错误里的地址会是 from 而不是 to —— 据此反证 from 通过了
    assert!(
        matches!(err, MailError::BadAddress(ref raw) if raw == "@@@"),
        "带显示名的发件人应能通过解析：{err:?}"
    );
}

#[test]
fn smtp_timeout_matches_imap() {
    use dp_gmail::mail::imap_client::IMAP_TIMEOUT_SECS;
    use dp_gmail::mail::smtp::SMTP_TIMEOUT_SECS;

    assert_eq!(
        SMTP_TIMEOUT_SECS, IMAP_TIMEOUT_SECS,
        "发信与收信的对外超时应当一致，避免一处改了另一处忘"
    );
}

// ── 报文组装（不涉网络）────────────────────────────────────────
//
// 断言一律用 ASCII 标记：中文主题会被 RFC2047 编码成 =?utf-8?…?=，
// 直接在原始报文里找不到原字，用中文断言只会得到假失败。

/// 用 ASCII 标记构造草稿，让对原始报文的断言稳定。
fn ascii_draft(html: Option<&str>) -> OutgoingMail {
    OutgoingMail {
        from: "sender@qq.com".to_owned(),
        to: vec!["rcpt@qq.com".to_owned()],
        subject: "PLAIN-SUBJECT".to_owned(),
        body_text: "PLAIN-BODY-MARKER".to_owned(),
        body_html: html.map(str::to_owned),
    }
}

fn raw_of(mail: &OutgoingMail) -> String {
    let message = build_message(mail).expect("应能组装报文");
    String::from_utf8_lossy(&message.formatted()).into_owned()
}

#[test]
fn plain_only_yields_single_part_message() {
    let raw = raw_of(&ascii_draft(None));

    assert!(
        !raw.contains("multipart/alternative"),
        "只有纯文本时不应套 multipart —— 单部分报文兼容性最好：\n{raw}"
    );
    assert!(raw.contains("PLAIN-BODY-MARKER"), "正文应在报文里");
    assert!(raw.contains("PLAIN-SUBJECT"), "主题应在报文里");
    assert!(raw.contains("rcpt@qq.com"), "收件人应在报文里");
}

#[test]
fn html_yields_multipart_alternative_carrying_both_parts() {
    let raw = raw_of(&ascii_draft(Some("<p>HTML-BODY-MARKER</p>")));

    assert!(
        raw.contains("multipart/alternative"),
        "同时有纯文本与 HTML 时应产出 multipart/alternative：\n{raw}"
    );
    assert!(
        raw.contains("PLAIN-BODY-MARKER"),
        "纯文本那一份不可省 —— 部分客户端与无障碍工具只读纯文本"
    );
    assert!(raw.contains("HTML-BODY-MARKER"), "HTML 那一份应在报文里");
    assert!(
        raw.contains("text/plain") && raw.contains("text/html"),
        "两个部分都应声明各自的 Content-Type"
    );
}

#[test]
fn multipart_declares_a_boundary() {
    let raw = raw_of(&ascii_draft(Some("<p>x</p>")));
    assert!(
        raw.to_lowercase().contains("boundary="),
        "multipart 必须声明分隔边界：\n{raw}"
    );
}

#[test]
fn build_message_rejects_bad_recipient() {
    let mut mail = ascii_draft(None);
    mail.to = vec!["@@@".to_owned()];

    let err = build_message(&mail).expect_err("非法收件人应在组装阶段被拒");
    assert!(
        matches!(err, MailError::BadAddress(ref raw) if raw == "@@@"),
        "错误类型不符：{err:?}"
    );
}

#[test]
fn build_message_rejects_bad_sender() {
    let mut mail = ascii_draft(None);
    mail.from = "not-an-address".to_owned();

    let err = build_message(&mail).expect_err("非法发件人应在组装阶段被拒");
    assert!(
        matches!(err, MailError::BadAddress(_)),
        "错误类型不符：{err:?}"
    );
}
