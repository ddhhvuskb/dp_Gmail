//! `mail::smtp` 的单元测试。
//!
//! **刻意只测网络请求之前的路径**：真正的投递需要真实 SMTP 服务器，
//! 在单测里连 smtp.qq.com 既慢又没意义。这里验证的是参数校验与校验顺序。

use dp_gmail::mail::error::MailError;
use dp_gmail::mail::provider::{ProviderConfig, TlsMode, PROVIDERS};
use dp_gmail::mail::smtp::{send_mail, OutgoingMail};

fn draft(from: &str, to: &[&str]) -> OutgoingMail {
    OutgoingMail {
        from: from.to_owned(),
        to: to.iter().map(|s| (*s).to_owned()).collect(),
        subject: "测试主题".to_owned(),
        body_text: "测试正文".to_owned(),
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
    // 这条用例同时验证**校验顺序**：若 parse_mailbox 被挪到建连之后，
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
