//! 真机收发验证 —— 用用户自己的邮箱跑通「收信 / 发信 / 闭环」。
//!
//! ## 凭证从哪来
//!
//! **一律从环境变量读，本文件不含任何账号或授权码**
//! （`DESIGN.md` 验收总则：「无硬编码密码」）。
//!
//! 缺变量时**直接失败**，不静默跳过 —— 静默跳过会让「没跑」看起来像「跑过了」，
//! 正是本项目反复防的那种假阳性。
//!
//! ```text
//! $env:DP_GMAIL_TEST_USER   = "you@qq.com"
//! $env:DP_GMAIL_TEST_SECRET = "16 位授权码"
//! cargo test --manifest-path src-tauri/Cargo.toml --test live_account_test -- --ignored --nocapture --test-threads=1
//! ```
//!
//! 全部标 `#[ignore]`：默认不跑（CI 无凭证），但 **「N ignored」会显式出现在
//! 每次 `cargo test` 的输出里**，而不是悄悄消失。
//!
//! ## 会往真实邮箱里发信
//!
//! `live_send_then_receive_roundtrip` 会**真的发一封邮件**给账号自己，
//! 用来证明 `DATA` 阶段与投递都成立。不想发信就别跑那一条。

use std::env;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use dp_gmail::mail::parser::parse_message;
use dp_gmail::mail::provider::resolve_by_email;
use dp_gmail::mail::smtp::{send_mail, OutgoingAttachment, OutgoingMail};
use dp_gmail::mail::sync::fetch_recent;

fn credentials() -> (String, String) {
    let user = env::var("DP_GMAIL_TEST_USER").unwrap_or_default();
    let secret = env::var("DP_GMAIL_TEST_SECRET").unwrap_or_default();
    assert!(
        !user.is_empty() && !secret.is_empty(),
        "缺少 DP_GMAIL_TEST_USER / DP_GMAIL_TEST_SECRET。\n\
         本文件的用例只在显式 --ignored 时运行，所以缺凭证是**失败**，不是跳过。"
    );
    (user, secret)
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("系统时间早于 1970")
        .as_secs()
}

/// 收信：`SELECT` + `FETCH BODY.PEEK[]` + MIME 解析。
///
/// 顺带真实执行了 `send_client_id_if_supported` —— 它在 `login` 成功之后才调用，
/// 所以此前的连接层验证（用伪造凭证）**没有覆盖到它**。
#[tokio::test]
#[ignore = "真机：需要真实授权码，会连线上服务器"]
async fn live_fetch_recent_from_real_inbox() {
    let (user, secret) = credentials();
    let cfg = resolve_by_email(&user).expect("该邮箱域名不在 provider 表里，解析不出服务器地址");

    let fetched = fetch_recent(cfg, &user, &secret, "INBOX", 3)
        .await
        .expect("fetch_recent 失败");

    println!("[收信] {} → 拉到 {} 封", user, fetched.len());
    assert!(!fetched.is_empty(), "INBOX 应当至少有 1 封邮件");

    for item in &fetched {
        let parsed = parse_message(&item.raw).expect("parse_message 失败");
        println!(
            "[收信] UID {} | 主题 {:?} | 发件人 {:?} | 纯文本 {} 字 | 有 HTML {} | 附件 {}",
            item.uid,
            parsed.subject,
            parsed.from.as_ref().map(|a| a.address.as_str()),
            parsed
                .text_body
                .as_ref()
                .map(|s| s.chars().count())
                .unwrap_or(0),
            parsed.html_body.is_some(),
            parsed.attachments.len(),
        );
    }
}

/// 从原始报文里取出指定名字的头部行，折行已拼回一行（原样，不解码）。
fn raw_header_lines(raw: &[u8], names: &[&str]) -> Vec<String> {
    let text = String::from_utf8_lossy(raw).replace("\r\n", "\n");
    let head = text.split("\n\n").next().unwrap_or("");

    let mut joined: Vec<String> = Vec::new();
    for line in head.split('\n') {
        if (line.starts_with(' ') || line.starts_with('\t')) && !joined.is_empty() {
            joined.last_mut().expect("非空").push_str(line);
        } else {
            joined.push(line.to_string());
        }
    }

    joined
        .into_iter()
        .filter(|line| {
            names.iter().any(|name| {
                line.get(..name.len())
                    .is_some_and(|p| p.eq_ignore_ascii_case(name))
                    && line.get(name.len()..).is_some_and(|r| r.starts_with(':'))
            })
        })
        .collect()
}

/// 真实报文的解析体检。
///
/// 合成数据撞不出真实世界的头部写法 —— 这条用例存在的唯一理由就是拿真邮件去撞。
/// 首次运行时它立刻抓到一封 `From` 解析不出来的真实邮件。
#[tokio::test]
#[ignore = "真机：需要真实授权码"]
async fn live_parse_audit() {
    let (user, secret) = credentials();
    let cfg = resolve_by_email(&user).expect("该邮箱域名不在 provider 表里");

    let fetched = fetch_recent(cfg, &user, &secret, "INBOX", 30)
        .await
        .expect("fetch_recent 失败");

    let mut gaps = 0;
    for item in &fetched {
        let Ok(parsed) = parse_message(&item.raw) else {
            println!("[体检] UID {} 解析直接失败", item.uid);
            gaps += 1;
            continue;
        };
        if parsed.from.is_some() && parsed.subject.is_some() && parsed.date_unix.is_some() {
            continue;
        }
        gaps += 1;
        println!(
            "[体检] UID {} 缺字段 from={} subject={} date={}",
            item.uid,
            parsed.from.is_some(),
            parsed.subject.is_some(),
            parsed.date_unix.is_some()
        );
        for line in raw_header_lines(&item.raw, &["From", "Sender", "Date", "Subject"]) {
            println!("        {}", line.trim_end());
        }
    }
    println!("[体检] 检查 {} 封，{} 封有缺字段", fetched.len(), gaps);
}
/// 闭环：发一封带 HTML 与附件的邮件给账号自己，再等它出现在自己的 INBOX 里。
///
/// 这一条同时覆盖 `DATA` 阶段、多部分 MIME 组装、以及投递后的真实解析。
#[tokio::test]
#[ignore = "真机：会真的发一封邮件给账号自己"]
async fn live_send_then_receive_roundtrip() {
    let (user, secret) = credentials();
    let cfg = resolve_by_email(&user).expect("该邮箱域名不在 provider 表里");

    let stamp = now_secs();
    let subject = format!("dp_Gmail 真机自检 {stamp}");

    let mail = OutgoingMail {
        from: user.clone(),
        to: vec![user.clone()],
        subject: subject.clone(),
        body_text: format!(
            "这是 dp_Gmail 的真机自检邮件，时间戳 {stamp}。收到即证明发信链路成立。"
        ),
        body_html: Some(format!(
            "<p>这是 <b>dp_Gmail</b> 的真机自检邮件，时间戳 {stamp}。</p>"
        )),
        attachments: vec![OutgoingAttachment {
            filename: "dp_gmail_selftest.txt".to_owned(),
            content_type: "text/plain".to_owned(),
            data: format!("dp_Gmail 附件自检 {stamp}\n").into_bytes(),
        }],
    };

    send_mail(cfg, &user, &secret, &mail)
        .await
        .expect("send_mail 失败（DATA 阶段）");
    println!("[发信] 已投递：{subject}");

    let deadline = SystemTime::now() + Duration::from_secs(150);
    loop {
        let fetched = fetch_recent(cfg, &user, &secret, "INBOX", 10)
            .await
            .expect("拉取失败");

        for item in &fetched {
            let Ok(parsed) = parse_message(&item.raw) else {
                continue;
            };
            if parsed.subject.as_deref() == Some(subject.as_str()) {
                println!(
                    "[闭环] 收到了自己发的邮件：UID {} | 有 HTML {} | 附件 {} 个 {:?}",
                    item.uid,
                    parsed.html_body.is_some(),
                    parsed.attachments.len(),
                    parsed
                        .attachments
                        .iter()
                        .map(|a| a.filename.as_str())
                        .collect::<Vec<_>>(),
                );
                assert!(parsed.html_body.is_some(), "多部分邮件应当保留 HTML 正文");
                assert_eq!(parsed.attachments.len(), 1, "应当带回 1 个附件");
                return;
            }
        }

        assert!(
            SystemTime::now() < deadline,
            "150 秒内没在 INBOX 看到自己发的邮件：{subject}"
        );
        println!("[闭环] 还没到，10 秒后再试…");
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
