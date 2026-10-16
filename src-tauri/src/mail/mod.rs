//! 邮件模块：IMAP / SMTP / 解析 / provider。
//!
//! 对外接口以 `DESIGN.md` 第三章调用契约表为准。

pub mod error;
pub mod imap_client;
pub mod parser;
pub mod provider;
pub mod smtp;
pub mod sync;
