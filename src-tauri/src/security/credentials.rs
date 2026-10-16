//! 凭证加解密。
//!
//! 密文布局：`nonce(12) || ciphertext || GCM tag(16)`。
//!
//! - 算法固定 **AES-256-GCM**，**禁止 ECB**（`AGENTS.md` 禁止项）。
//! - nonce 每次加密独立随机生成，绝不复用。
//! - 密钥一律由调用方从 [`super::keyfile`] 取，**不得硬编码**。

use aes_gcm::aead::{Aead, Generate, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};

use super::error::{SecurityError, NONCE_LEN};
use super::keyfile::KEY_LEN;

/// 加密一条凭证。返回 `nonce || ciphertext || tag`。
pub fn encrypt_secret(plaintext: &str, key: &[u8; KEY_LEN]) -> Result<Vec<u8>, SecurityError> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| SecurityError::KeyFileFormat(format!("密钥应为 {KEY_LEN} 字节")))?;

    let nonce = Nonce::generate();
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|_| SecurityError::Encrypt)?;

    let mut blob = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    blob.extend_from_slice(nonce.as_slice());
    blob.extend_from_slice(&ciphertext);
    Ok(blob)
}

/// 解密 [`encrypt_secret`] 产出的密文。
///
/// 密文被改动一个字节、密钥不对、长度不足，都会返回 [`SecurityError::Decrypt`] 一类错误，
/// **不会**返回被篡改的明文 —— 这正是 GCM 认证标签的意义。
pub fn decrypt_secret(blob: &[u8], key: &[u8; KEY_LEN]) -> Result<String, SecurityError> {
    if blob.len() <= NONCE_LEN {
        return Err(SecurityError::CiphertextTooShort);
    }

    let (nonce_bytes, ciphertext) = blob.split_at(NONCE_LEN);
    // 上面已保证长度 > NONCE_LEN，这里不会失败；仍返回错误而非 unwrap。
    let nonce = Nonce::try_from(nonce_bytes).map_err(|_| SecurityError::CiphertextTooShort)?;

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| SecurityError::KeyFileFormat(format!("密钥应为 {KEY_LEN} 字节")))?;

    let plaintext = cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|_| SecurityError::Decrypt)?;

    String::from_utf8(plaintext).map_err(|_| SecurityError::NotUtf8)
}
