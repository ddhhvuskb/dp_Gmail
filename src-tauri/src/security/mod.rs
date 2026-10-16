//! 安全模块：加密 / HTML 净化。
//!
//! 加密方案见 `docs/adr/ADR-0002-credential-encryption.md`（P0-07，用户已确认）。

pub mod credentials;
pub mod error;
pub mod keyfile;
pub mod sanitize;

pub use credentials::{decrypt_secret, encrypt_secret};
pub use error::SecurityError;
pub use keyfile::{load_or_create_key, KEY_FILE_VERSION, KEY_LEN};
pub use sanitize::sanitize_html;
