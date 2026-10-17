//! IMAP 连接与登录。
//!
//! 服务器地址一律由调用方从 [`crate::mail::provider`] 取，本模块不得出现地址字面量。

use std::time::Duration;

use tokio::net::TcpStream;

use super::error::MailError;
use super::provider::{ProviderConfig, TlsMode};
use super::tls;

/// 连接超时。来源：`DESIGN.md` P0-01 关键约束「超时 30s」。
pub const IMAP_TIMEOUT_SECS: u64 = 30;

/// 已登录的 IMAP 会话。
///
/// `async-imap` 在 `runtime-tokio` 下用的是 tokio 的 io trait
/// （见其 `client.rs`：`use tokio::io::{AsyncRead as Read, AsyncWrite as Write}`）。
/// 流的具体类型由 [`super::tls`] 按平台给（桌面 native-tls / Android rustls），
/// 两者产出的都是 tokio 流，**不需要** futures-io 兼容层。
pub type ImapSession = async_imap::Session<tls::TlsStream>;

/// 在 30 秒内建立 TLS 连接、读取问候语并登录。
///
/// 超时覆盖「TCP 连接 + TLS 握手 + 读问候语 + STARTTLS（如需要）+ 登录」全过程。
///
/// 两种 TLS 模式的差别不只是端口：
///
/// | 模式 | 流程 |
/// | --- | --- |
/// | `Implicit`（993） | 建连即 TLS → 读问候语 → 登录 |
/// | `StartTls`（143） | 明文建连 → 读问候语 → 发 `STARTTLS` → 取出裸流 → 套 TLS → 登录 |
///
/// **`STARTTLS` 之后服务器不再发送问候语**（`async-imap` 的 crate 文档明写
/// "Note that there is no server greeting after STARTTLS."），故升级后直接登录。
pub async fn connect_and_login(
    cfg: &ProviderConfig,
    user: &str,
    secret: &str,
) -> Result<ImapSession, MailError> {
    let handshake = async {
        let tcp = TcpStream::connect((cfg.imap_host, cfg.imap_port)).await?;

        let client = match cfg.imap_tls {
            TlsMode::Implicit => {
                let stream = tls::connect(cfg.imap_host, tcp).await?;
                let mut client = async_imap::Client::new(stream);
                read_greeting(&mut client).await?;
                client
            }
            TlsMode::StartTls => {
                let mut plain = async_imap::Client::new(tcp);
                read_greeting(&mut plain).await?;

                plain.run_command_and_check_ok("STARTTLS", None).await?;

                // 升级：取出裸流重新套 TLS，再构造一个新的 Client
                let stream = tls::connect(cfg.imap_host, plain.into_inner()).await?;
                async_imap::Client::new(stream)
            }
        };

        let mut session = client
            .login(user, secret)
            .await
            .map_err(|(err, _client)| MailError::Imap(err))?;

        send_client_id_if_supported(&mut session).await?;
        Ok(session)
    };

    tokio::time::timeout(Duration::from_secs(IMAP_TIMEOUT_SECS), handshake)
        .await
        .map_err(|_| MailError::Timeout(IMAP_TIMEOUT_SECS))?
}

/// 网易（163 / 126 / yeah.net）要求在 `SELECT` 之前发送 IMAP `ID` 命令
/// （RFC 2971），否则读取邮件时服务器会拒绝：
///
/// ```text
/// SELECT Unsafe Login. Please contact kefu@188.com for help
/// ```
///
/// 这个症状极具误导性 —— 连得上、登录也成功，只有读邮件时才失败，
/// 很容易被当成"密码错了"或"163 不支持 IMAP"。
///
/// **仅当服务器声明 `ID` 能力时才发送**：无条件发送会给不支持的服务器
/// 制造无谓的失败。QQ 邮箱不发此命令，行为不变。
async fn send_client_id_if_supported(session: &mut ImapSession) -> Result<(), MailError> {
    let capabilities = session.capabilities().await?;
    if !capabilities.has_str("ID") {
        return Ok(());
    }

    session
        .run_command_and_check_ok(r#"ID ("name" "dp_Gmail" "version" "0.1.0" "vendor" "dp_Gmail")"#)
        .await?;

    Ok(())
}

/// 读取并校验服务器问候语。
///
/// `async-imap` 0.12 起不再自动消费问候语；流在问候语之前就结束属于协议异常，
/// 必须显式报错而不是当作"空邮箱"继续。
async fn read_greeting<T>(client: &mut async_imap::Client<T>) -> Result<(), MailError>
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + std::fmt::Debug + Send,
{
    let greeting = client.read_response().await?;
    if greeting.is_none() {
        return Err(MailError::Io(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "服务器未发送 IMAP 问候语",
        )));
    }
    Ok(())
}
