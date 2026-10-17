//! TLS 薄抽象 —— **按平台二选一**，见 `docs/adr/ADR-0006-android-port.md` §9.6。
//!
//! 为什么需要这一层：两端的 TLS 后端不是同一套。
//!
//! | 平台 | 后端 | 原因 |
//! | --- | --- | --- |
//! | 桌面 | `native-tls`（Windows 是 SChannel） | 已验证 8 家服务商全通，含 sina |
//! | Android | `rustls` | native-tls 在安卓会退回 OpenSSL，而 Windows 上编不了 Android 版 OpenSSL（见 ADR §9.4） |
//!
//! 后果是**安卓上 sina 的 IMAP 连不上**：它的服务器只提供 TLS1.2 的
//! 有限域 DHE / 纯 RSA 密钥交换，而 rustls 只做 ECDHE + TLS1.3。
//! 这是已知且被用户接受的行为差异，不是缺陷。
//!
//! 上层（`imap_client`）只调 [`connect`]，**不认识后端**；
//! 加平台或换后端只需改本文件，不动协议代码。

use tokio::net::TcpStream;

use super::error::MailError;

#[cfg(not(target_os = "android"))]
mod imp {
    use super::{MailError, TcpStream};

    /// `native-tls` 产出的 tokio 流。
    pub type TlsStream = async_native_tls::TlsStream<TcpStream>;

    /// 建立 TLS。`host` 同时用于 SNI 与证书主机名校验。
    pub async fn connect(host: &str, tcp: TcpStream) -> Result<TlsStream, MailError> {
        let connector = async_native_tls::TlsConnector::new();
        connector
            .connect(host, tcp)
            .await
            .map_err(|err| MailError::Tls(err.to_string()))
    }
}

#[cfg(target_os = "android")]
mod imp {
    use std::sync::Arc;

    use rustls::pki_types::ServerName;
    use rustls::RootCertStore;
    use tokio_rustls::client::TlsStream as RustlsStream;
    use tokio_rustls::TlsConnector;

    use super::{MailError, TcpStream};

    /// `tokio-rustls` 产出的 tokio 流。
    pub type TlsStream = RustlsStream<TcpStream>;

    /// 建立 TLS。
    ///
    /// 根证书用 Mozilla 的静态包（`webpki-roots`），**不读系统信任库**：
    /// 安卓上读系统库要走 JNI，为它单独写一套不值得。代价是自签名的内部 CA
    /// （部分企业邮箱）校验不过 —— 已在 ADR §4 记录。
    pub async fn connect(host: &str, tcp: TcpStream) -> Result<TlsStream, MailError> {
        let mut roots = RootCertStore::empty();
        roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        let config = rustls::ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();
        let connector = TlsConnector::from(Arc::new(config));

        // to_owned 是为了拿到 `ServerName<'static>`，避免把 &str 的生命周期
        // 拖进 `.await` 之后的代码
        let name = ServerName::try_from(host.to_owned())
            .map_err(|err| MailError::Tls(format!("非法服务器名 {host}：{err}")))?;

        connector
            .connect(name, tcp)
            .await
            .map_err(|err| MailError::Tls(err.to_string()))
    }
}

pub use imp::{connect, TlsStream};
