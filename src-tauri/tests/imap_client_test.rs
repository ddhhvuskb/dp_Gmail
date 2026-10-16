//! `mail::imap_client` 的单元测试。
//!
//! 覆盖 `docs/tasks/P0-01-imap-connect.md` §4.2 约定的两个分支。

use std::time::{Duration, Instant};

use dp_gmail::mail::error::MailError;
use dp_gmail::mail::imap_client::{connect_and_login, IMAP_TIMEOUT_SECS};
use dp_gmail::mail::provider::{ProviderConfig, TlsMode, PROVIDERS};

#[tokio::test]
async fn starttls_mode_attempts_a_connection_instead_of_bailing_out() {
    // STARTTLS 现已实现，原先「返回 UnsupportedTlsMode」的断言不再成立。
    //
    // 这条用例替换掉它而不是删掉：用一个沉默的本地假服务器，
    // 断言它**确实去建连了**（否则会立刻返回错误而不是等超时）。
    // 若哪天有人把 STARTTLS 分支改回早退，本用例会因为"秒回"而失败。
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        tokio::time::sleep(Duration::from_secs(120)).await;
        drop(stream);
    });

    // ProviderConfig 的 host 是 &'static str，测试里用运行时端口，故泄漏一个短字符串
    let host: &'static str = Box::leak("127.0.0.1".to_owned().into_boxed_str());
    let cfg = ProviderConfig {
        imap_host: host,
        imap_port: port,
        imap_tls: TlsMode::StartTls,
        ..PROVIDERS[0]
    };

    let started = Instant::now();
    let err = connect_and_login(&cfg, "user@qq.com", "secret")
        .await
        .expect_err("服务端沉默，STARTTLS 路径应超时");

    assert!(
        matches!(err, MailError::Timeout(_)),
        "应返回超时错误而非早退，实际：{err:?}"
    );
    assert!(
        started.elapsed() >= Duration::from_secs(IMAP_TIMEOUT_SECS),
        "STARTTLS 路径应真的去建连并等待，耗时 {:?}",
        started.elapsed()
    );
}

#[tokio::test]
async fn times_out_when_server_never_speaks_tls() {
    // 假 IMAP 服务：接受 TCP 连接后保持沉默，不发 ServerHello 也不关闭。
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        tokio::time::sleep(Duration::from_secs(120)).await;
        drop(stream);
    });

    // ProviderConfig 的 host 是 &'static str，测试里用运行时端口，故泄漏一个短字符串。
    let host: &'static str = Box::leak("127.0.0.1".to_owned().into_boxed_str());
    let cfg = ProviderConfig {
        imap_host: host,
        imap_port: port,
        ..PROVIDERS[0]
    };

    let started = Instant::now();
    let err = connect_and_login(&cfg, "user@qq.com", "secret")
        .await
        .expect_err("服务端沉默，应触发超时");

    assert!(
        matches!(err, MailError::Timeout(_)),
        "应返回超时错误，实际：{err:?}"
    );
    assert!(
        started.elapsed() >= Duration::from_secs(IMAP_TIMEOUT_SECS),
        "超时提前触发，耗时 {:?}",
        started.elapsed()
    );
}
