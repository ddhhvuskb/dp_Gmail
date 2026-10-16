//! `mail::imap_client` 的单元测试。
//!
//! 覆盖 `docs/tasks/P0-01-imap-connect.md` §4.2 约定的两个分支。

use std::time::{Duration, Instant};

use dp_gmail::mail::error::MailError;
use dp_gmail::mail::imap_client::{connect_and_login, IMAP_TIMEOUT_SECS};
use dp_gmail::mail::provider::{ProviderConfig, TlsMode, PROVIDERS};

#[tokio::test]
async fn rejects_starttls_mode_for_now() {
    let cfg = ProviderConfig {
        imap_tls: TlsMode::StartTls,
        ..PROVIDERS[0]
    };

    let err = connect_and_login(&cfg, "user@qq.com", "secret")
        .await
        .expect_err("STARTTLS 尚未实现，应返回错误");

    assert!(
        matches!(err, MailError::UnsupportedTlsMode("STARTTLS")),
        "错误类型不符：{err:?}"
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
