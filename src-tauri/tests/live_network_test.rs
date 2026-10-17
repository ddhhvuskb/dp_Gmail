//! 真实网络验证 —— 把「零真实执行」的网络代码真跑一遍。
//!
//! 本文件的用例会**真的连到 QQ / 网易的线上服务器**，故全部标 `#[ignore]`，
//! 默认不参与 `cargo test`。运行方式：
//!
//! ```text
//! cargo test --manifest-path src-tauri/Cargo.toml --test live_network_test -- --ignored --nocapture
//! ```
//!
//! ## 没有授权码为什么也能验证
//!
//! 用**伪造的账号密码**登录，断言错误落在 `Imap` / `Smtp` 变体上。
//! 判别力在于：TLS 握手、证书校验、IMAP 问候语读取、`CAPABILITY`、
//! `LOGIN` 往返、SMTP `EHLO` / `AUTH` 协商 —— **只要有一环没跑通**，
//! 错误就会落在 `Tls` / `Io` / `Timeout` 上或丢失 SMTP 状态码，
//! 而不是拿到服务器明确回出的拒绝。
//!
//! 换言之：**拿到协议层拒绝 == 整条链路真的走到服务器并收到了回话。**
//!
//! ## 尚未覆盖的边界
//!
//! **成功的登录**与**真实的投递**仍需用户提供授权码。本文件不触碰该边界，
//! 也不会有任何邮件被投递 —— 认证在 `DATA` 之前就必然失败。
//! 完整的验证债清单见 `docs/tasks/STATUS.md`。

use dp_gmail::mail::error::MailError;
use dp_gmail::mail::imap_client::connect_and_login;
use dp_gmail::mail::provider::{ProviderConfig, TlsMode, PROVIDERS};
use dp_gmail::mail::smtp::{send_mail, OutgoingMail};

/// 伪造凭证。**不得**替换成真实账号 —— 本文件会被反复执行。
const FAKE_USER: &str = "dpgmail_selftest_account_not_exist";
const FAKE_SECRET: &str = "dpgmail_bogus_password_for_link_test_only";

/// 探测用配置。地址与 `provider.rs` 中已核实的条目一致，端口按被测路径取。
const QQ_IMAP_993: ProviderConfig = ProviderConfig {
    id: "qq-live-probe",
    display_name: "QQ邮箱",
    domains: &["qq.com"],
    imap_host: "imap.qq.com",
    imap_port: 993,
    imap_tls: TlsMode::Implicit,
    smtp_host: "smtp.qq.com",
    smtp_port: 465,
    smtp_tls: TlsMode::Implicit,
};

/// 163 的 IMAP STARTTLS 走 143：已实测 `a1 OK Begin TLS negotiation`。
/// （QQ 的 143 不支持 STARTTLS，实测回 `* BAD Command!`，故不能拿来测。）
const NETEASE_IMAP_143: ProviderConfig = ProviderConfig {
    id: "netease163-live-probe",
    display_name: "网易163邮箱",
    domains: &["163.com"],
    imap_host: "imap.163.com",
    imap_port: 143,
    imap_tls: TlsMode::StartTls,
    smtp_host: "smtp.163.com",
    smtp_port: 465,
    smtp_tls: TlsMode::Implicit,
};

const QQ_SMTP_587: ProviderConfig = ProviderConfig {
    id: "qq-live-probe-starttls",
    display_name: "QQ邮箱",
    domains: &["qq.com"],
    imap_host: "imap.qq.com",
    imap_port: 993,
    imap_tls: TlsMode::Implicit,
    smtp_host: "smtp.qq.com",
    smtp_port: 587,
    smtp_tls: TlsMode::StartTls,
};

fn probe_mail() -> OutgoingMail {
    OutgoingMail {
        from: "dpgmail_selftest@qq.com".to_owned(),
        to: vec!["dpgmail_selftest_recipient@example.com".to_owned()],
        subject: "dp_Gmail 网络自检".to_owned(),
        body_text: "本邮件不会被投递：认证在 DATA 之前必然失败。".to_owned(),
        body_html: None,
        attachments: Vec::new(),
    }
}

/// 断言错误发生在**服务器回过话之后**的协议层。
fn expect_protocol_rejection(err: &MailError, stage: &str) {
    match err {
        MailError::Imap(e) => {
            println!("[{stage}] 链路走通，服务器拒绝登录: {e}");
        }
        MailError::Smtp(e) => {
            let status = e.status();
            assert!(
                status.is_some(),
                "[{stage}] 期望服务器回出 SMTP 状态码，实际没有 —— 错误是 {e:?}"
            );
            assert!(!e.is_tls(), "[{stage}] TLS 握手就没过: {e:?}");
            println!("[{stage}] 链路走通，服务器拒绝认证: {status:?} {e}");
        }
        other => panic!(
            "[{stage}] 期望在协议层被服务器拒绝，实际是 {other:?} —— \
             说明 TLS 握手 / 问候语 / 命令往返至少有一环没走到"
        ),
    }
}

#[tokio::test]
#[ignore = "真实网络：连 imap.qq.com:993"]
async fn live_imap_implicit_tls_reaches_server_rejection() {
    let err = match connect_and_login(&QQ_IMAP_993, FAKE_USER, FAKE_SECRET).await {
        Ok(_) => panic!("伪造凭证不应登录成功"),
        Err(e) => e,
    };
    expect_protocol_rejection(&err, "IMAP 993 隐式 TLS");
}

#[tokio::test]
#[ignore = "真实网络：连 imap.163.com:143 并 STARTTLS 升级"]
async fn live_imap_starttls_upgrade_reaches_server_rejection() {
    let err = match connect_and_login(&NETEASE_IMAP_143, FAKE_USER, FAKE_SECRET).await {
        Ok(_) => panic!("伪造凭证不应登录成功"),
        Err(e) => e,
    };
    expect_protocol_rejection(&err, "IMAP 143 STARTTLS 升级");
}

#[tokio::test]
#[ignore = "真实网络：连 smtp.qq.com:465"]
async fn live_smtp_implicit_tls_reaches_server_rejection() {
    let err = match send_mail(&QQ_IMAP_993, FAKE_USER, FAKE_SECRET, &probe_mail()).await {
        Ok(()) => panic!("伪造凭证不应投递成功"),
        Err(e) => e,
    };
    expect_protocol_rejection(&err, "SMTP 465 隐式 TLS");
}

#[tokio::test]
#[ignore = "真实网络：连 smtp.qq.com:587 并 STARTTLS 升级"]
async fn live_smtp_starttls_reaches_server_rejection() {
    let err = match send_mail(&QQ_SMTP_587, FAKE_USER, FAKE_SECRET, &probe_mail()).await {
        Ok(()) => panic!("伪造凭证不应投递成功"),
        Err(e) => e,
    };
    expect_protocol_rejection(&err, "SMTP 587 STARTTLS 升级");
}

/// 表里**每一家个人邮箱**都要走得通连接层。
///
/// 地址是 `PROVIDERS` 里取的，所以**以后再加服务商会自动被这条覆盖** ——
/// 不需要有人记得补测试。
///
/// 为什么单独需要它：加服务商时容易只验证「地址是对的」（例如用别的工具
/// 探一下主机通不通），却漏掉「**我们的代码**连得上」。这两件事不是一回事 ——
/// TLS 栈、超时、问候语解析、`CAPABILITY` 任何一环出问题，都只有在真跑时才暴露。
#[tokio::test]
#[ignore = "真实网络：会依次连接表里每一家服务器"]
async fn live_every_personal_provider_reaches_login_rejection() {
    let personal: Vec<_> = PROVIDERS.iter().filter(|p| !p.domains.is_empty()).collect();
    println!("[覆盖] 表里个人邮箱 {} 家", personal.len());

    let mut covered = 0;
    for cfg in &personal {
        let err = match connect_and_login(cfg, FAKE_USER, FAKE_SECRET).await {
            Ok(_) => panic!("{} 用伪造凭证不该登录成功", cfg.id),
            Err(e) => e,
        };

        match &err {
            MailError::Imap(reason) => {
                covered += 1;
                println!(
                    "[覆盖] ✅ {}（{}）{} —— 链路走通，服务器拒绝登录: {reason}",
                    cfg.id, cfg.display_name, cfg.imap_host
                );
            }
            other => panic!(
                "[覆盖] ⛔ {} 的 {} 期望在协议层被拒，实际是 {other:?} —— \
                 地址、TLS 或问候语至少有一处不对",
                cfg.id, cfg.imap_host
            ),
        }
    }

    assert_eq!(
        covered,
        personal.len(),
        "应当每一家都走到协议层，实际只走通 {covered}/{}",
        personal.len()
    );
}

/// 表里**每一家个人邮箱的 SMTP** 也要走得通。与 IMAP 那条对称。
///
/// ## 为什么断言写得比 IMAP 那条更细
///
/// lettre 把**连接失败和 TLS 失败都**包进它自己的 `smtp::Error`，
/// 所以「错误是 `MailError::Smtp`」**并不能证明握手成功** ——
/// 主机名写错、证书不匹配，同样会走到这个分支。
///
/// 因此必须再要求两件事：
///
/// 1. `status().is_some()` —— **服务器真的回出了 SMTP 状态码**，
///    光连不上是不会有状态码的；
/// 2. `!is_tls()` —— 明确排除 TLS 层失败。
///
/// 少了这两条，这条用例会在「地址全写错」的情况下**照样通过**。
#[tokio::test]
#[ignore = "真实网络：会依次连接表里每一家的 SMTP 服务器"]
async fn live_every_personal_provider_smtp_reaches_auth_rejection() {
    let personal: Vec<_> = PROVIDERS.iter().filter(|p| !p.domains.is_empty()).collect();
    println!("[SMTP 覆盖] 表里个人邮箱 {} 家", personal.len());

    let mut covered = 0;
    for cfg in &personal {
        let err = match send_mail(cfg, FAKE_USER, FAKE_SECRET, &probe_mail()).await {
            Ok(()) => panic!("{} 用伪造凭证不该投递成功", cfg.id),
            Err(e) => e,
        };

        match &err {
            MailError::Smtp(reason) => {
                assert!(
                    reason.status().is_some(),
                    "[SMTP] ⛔ {} 的 {} 没回状态码 —— 连接或 TLS 就没过：{reason:?}",
                    cfg.id,
                    cfg.smtp_host
                );
                assert!(
                    !reason.is_tls(),
                    "[SMTP] ⛔ {} 的 {} TLS 握手失败：{reason:?}",
                    cfg.id,
                    cfg.smtp_host
                );
                covered += 1;
                println!(
                    "[SMTP 覆盖] ✅ {}（{}）{} —— 服务器回出状态码并拒绝认证：{reason}",
                    cfg.id, cfg.display_name, cfg.smtp_host
                );
            }
            other => panic!(
                "[SMTP 覆盖] ⛔ {} 的 {} 期望在 SMTP 层被拒，实际是 {other:?}",
                cfg.id, cfg.smtp_host
            ),
        }
    }

    assert_eq!(
        covered,
        personal.len(),
        "应当每一家都走到 SMTP 协议层，实际只走通 {covered}/{}",
        personal.len()
    );
}
