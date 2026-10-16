//! 邮件读取命令。

use crate::security;
use crate::state::AppState;
use crate::storage::{self, MessageSummary};

/// 单页最大条数。
///
/// 上限是**参数校验**职责的一部分：前端如果传 `u32::MAX`，
/// SQLite 会试图把整个文件夹读进内存。
pub const MAX_PAGE_SIZE: u32 = 200;

/// 归一化分页参数：`limit` 收敛到 `1..=MAX_PAGE_SIZE`，`offset` 原样透传。
pub fn normalize_paging(limit: u32, offset: u32) -> (u32, u32) {
    (limit.clamp(1, MAX_PAGE_SIZE), offset)
}

/// 按收件时间倒序分页读取某文件夹的邮件摘要。
///
/// 返回的 `MessageSummary` 是 `storage` 定义的**摘要 DTO**，
/// 不是数据库行 —— `DESIGN.md` P0-04 明确禁止命令层返回原始 DB 行。
#[tauri::command]
pub fn get_messages(
    state: tauri::State<'_, AppState>,
    folder_id: i64,
    limit: u32,
    offset: u32,
) -> Result<Vec<MessageSummary>, String> {
    let (limit, offset) = normalize_paging(limit, offset);

    let conn = state
        .db
        .lock()
        .map_err(|err| format!("数据库连接不可用：{err}"))?;

    storage::list_messages(&conn, folder_id, limit, offset).map_err(|err| err.to_string())
}

/// 附件视图（详情面板展示用）。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct AttachmentView {
    pub filename: String,
    pub content_type: String,
    pub size: i64,
}

/// 详情面板用的邮件视图。
///
/// **`html_body` 已净化** —— 原始 HTML 不出后端，
/// 前端因此可以安全地渲染它（见 `AGENTS.md` 禁止项）。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct MessageView {
    pub id: i64,
    pub uid: i64,
    pub subject: Option<String>,
    pub from_name: Option<String>,
    pub from_address: Option<String>,
    pub to_addresses: Option<String>,
    pub date_unix: i64,
    pub is_seen: bool,
    pub text_body: Option<String>,
    /// 经 [`security::sanitize_html`] 处理后的 HTML。
    pub html_body: Option<String>,
    pub attachments: Vec<AttachmentView>,
}

/// 读取单封邮件的完整内容。
///
/// 本命令**不**改动已读状态 —— 列表与详情都没有"标记已读"的规格，
/// 擅自改服务端/本地状态属于超范围。
#[tauri::command]
pub fn get_message(
    state: tauri::State<'_, AppState>,
    message_id: i64,
) -> Result<MessageView, String> {
    let conn = state
        .db
        .lock()
        .map_err(|err| format!("数据库连接不可用：{err}"))?;

    let detail = storage::load_message_detail(&conn, message_id)
        .map_err(|err| err.to_string())?
        .ok_or_else(|| format!("邮件不存在：{message_id}"))?;

    Ok(MessageView {
        id: detail.id,
        uid: detail.uid,
        subject: detail.subject,
        from_name: detail.from_name,
        from_address: detail.from_address,
        to_addresses: detail.to_addresses,
        date_unix: detail.date_unix,
        is_seen: detail.is_seen,
        text_body: detail.text_body,
        // 全项目唯一允许产出可渲染 HTML 的地方。
        html_body: detail.html_body.as_deref().map(security::sanitize_html),
        attachments: detail
            .attachments
            .into_iter()
            .map(|att| AttachmentView {
                filename: att.filename,
                content_type: att.content_type,
                size: att.size,
            })
            .collect(),
    })
}
