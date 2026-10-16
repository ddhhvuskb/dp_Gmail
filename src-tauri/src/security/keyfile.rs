//! 密钥文件管理。
//!
//! 方案见 `docs/adr/ADR-0002-credential-encryption.md`：**版本化 JSON，无口令保护**。
//!
//! 文件形如：
//!
//! ```json
//! {
//!   "version": 1,
//!   "algorithm": "AES-256-GCM",
//!   "key": "<base64 的 32 字节>",
//!   "created_at": 1760000000
//! }
//! ```
//!
//! 威胁模型：能防「数据库文件或密钥文件被单独拷走后直接读出明文密码」；
//! **防不住**本机已被完全攻破的情形 —— 那是无口令密钥文件方案的固有上限。

use std::path::Path;

use aes_gcm::aead::{Generate, Key};
use aes_gcm::Aes256Gcm;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use super::error::SecurityError;

/// 密钥文件格式版本。格式变更时递增。
pub const KEY_FILE_VERSION: u8 = 1;

/// 密钥长度（AES-256）。
pub const KEY_LEN: usize = 32;

const KEY_ALGORITHM: &str = "AES-256-GCM";

/// 版本化密钥文件的内容。
#[derive(Debug, Serialize, Deserialize)]
struct KeyFile {
    version: u8,
    algorithm: String,
    /// base64 编码的密钥。
    key: String,
    /// Unix 秒。
    created_at: i64,
}

/// 读取密钥文件；文件不存在则生成一个 32 字节随机密钥并落盘。
///
/// 版本或算法不匹配一律**报错而非猜测**，避免用错密钥静默解出垃圾数据。
pub fn load_or_create_key(path: &Path) -> Result<[u8; KEY_LEN], SecurityError> {
    if path.exists() {
        let raw = std::fs::read_to_string(path)?;
        let file: KeyFile = serde_json::from_str(&raw)?;

        if file.version != KEY_FILE_VERSION {
            return Err(SecurityError::KeyFileFormat(format!(
                "不支持的密钥文件版本 {}（本程序支持 {KEY_FILE_VERSION}）",
                file.version
            )));
        }
        if file.algorithm != KEY_ALGORITHM {
            return Err(SecurityError::KeyFileFormat(format!(
                "不支持的算法 {}（期望 {KEY_ALGORITHM}）",
                file.algorithm
            )));
        }

        // 解码结果用后清零，避免密钥残留在堆上。
        let decoded = Zeroizing::new(STANDARD.decode(file.key)?);
        let actual = decoded.len();
        return decoded.as_slice().try_into().map_err(|_| {
            SecurityError::KeyFileFormat(format!("密钥应为 {KEY_LEN} 字节，实际 {actual}"))
        });
    }

    let generated = Key::<Aes256Gcm>::generate();
    let key: [u8; KEY_LEN] = generated
        .as_slice()
        .try_into()
        .map_err(|_| SecurityError::KeyFileFormat("生成的密钥长度异常".to_owned()))?;

    let file = KeyFile {
        version: KEY_FILE_VERSION,
        algorithm: KEY_ALGORITHM.to_owned(),
        key: STANDARD.encode(key),
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0),
    };
    std::fs::write(path, serde_json::to_string_pretty(&file)?)?;

    // Windows 上 %APPDATA% 本就只有当前用户可访问；类 Unix 需要显式收紧。
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    }

    Ok(key)
}
