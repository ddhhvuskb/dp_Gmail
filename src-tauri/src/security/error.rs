//! `security` 模块的错误类型。

/// 安全模块统一错误。
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("密钥文件读写失败：{0}")]
    Io(#[from] std::io::Error),

    #[error("密钥文件格式非法：{0}")]
    KeyFileFormat(String),

    #[error("密钥文件解析失败：{0}")]
    Json(#[from] serde_json::Error),

    #[error("密钥 base64 解码失败：{0}")]
    Base64(#[from] base64::DecodeError),

    #[error("加密失败")]
    Encrypt,

    /// 密文被改动、密钥不匹配、或压根不是本程序产出的数据，都会落到这里。
    #[error("解密失败：密文被篡改或密钥不匹配")]
    Decrypt,

    #[error("密文长度不足：至少需要 {NONCE_LEN} 字节 nonce 加 16 字节认证标签")]
    CiphertextTooShort,

    #[error("解密结果不是合法 UTF-8")]
    NotUtf8,
}

/// 密文布局里 nonce 的长度（AES-GCM 标准值）。
pub const NONCE_LEN: usize = 12;
