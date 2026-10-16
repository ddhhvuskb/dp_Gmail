//! IMAP 连接与登录。
//!
//! 服务器地址一律由调用方从 [`crate::mail::provider`] 取，本模块不得出现地址字面量。

use std::time::Duration;

use async_native_tls::TlsStream;
use tokio::net::TcpStream;

use super::error::MailError;
use super::provider::{ProviderConfig, TlsMode};

/// 连接超时。来源：`DESIGN.md` P0-01 关键约束「超时 30s」。
pub const IMAP_TIMEOUT_SECS: u64 = 30;

/// 已登录的 IMAP 会话。
///
/// `async-imap` 在 `runtime-tokio` 下用的是 tokio 的 io trait
/// （见其 `client.rs`：`use tokio::io::{AsyncRead as Read, AsyncWrite as Write}`），
/// `async-native-tls` 在 `runtime-tokio` 下产出的正是一个 tokio 流，
/// 两者可直接对接，**不需要** futures-io 兼容层。
pub type ImapSession = async_imap::Session<TlsStream<TcpStream>>;

/// 在 30 秒内建立 TLS 连接、读取问候语并登录。
///
/// 超时覆盖「TCP 连接 + TLS 握手 + 读问候语 + 登录」全过程。
pub async fn connect_and_login(
    cfg: &ProviderConfig,
    user: &str,
    secret: &str,
) -> Result<ImapSession, MailError> {
    if cfg.imap_tls != TlsMode::Implicit {
        return Err(MailError::UnsupportedTlsMode("STARTTLS"));
    }

    let handshake = async {
        let tcp = TcpStream::connect((cfg.imap_host, cfg.imap_port)).await?;
        let tls = async_native_tls::TlsConnector::new()
            .connect(cfg.imap_host, tcp)
            .await?;
        let mut client = async_imap::Client::new(tls);

        // async-imap 0.12 起不再自动消费服务器问候语，必须显式读取。
        let greeting = client.read_response().await?;
        if greeting.is_none() {
            return Err(MailError::Io(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "服务器未发送 IMAP 问候语",
            )));
        }

        client
            .login(user, secret)
            .await
            .map_err(|(err, _client)| MailError::Imap(err))
    };

    tokio::time::timeout(Duration::from_secs(IMAP_TIMEOUT_SECS), handshake)
        .await
        .map_err(|_| MailError::Timeout(IMAP_TIMEOUT_SECS))?
}
