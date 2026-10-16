//! Tauri 命令桥。
//!
//! 本模块**只做参数校验和转发**，不写业务逻辑、不写 SQL、不调 IMAP
//! （见 `DESIGN.md` P0-04 约束与禁用项）。真正的实现分别在 `storage` / `mail` / `security`。

pub mod accounts;
pub mod folders;
pub mod messages;
pub mod sync;
