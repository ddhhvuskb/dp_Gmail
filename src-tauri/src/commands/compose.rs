//! 写信 / 发信命令。

use std::path::Path;

use crate::mail::provider;
use crate::mail::smtp::{self, OutgoingAttachment, OutgoingMail};
use crate::security;
use crate::state::AppState;
use crate::storage;

/// 单封邮件所有附件合计的体积上限。
///
/// 国内邮箱普遍限制在 20–50 MB，取 25 MiB 留出编码膨胀余量。
pub const MAX_TOTAL_ATTACHMENT_BYTES: u64 = 25 * 1024 * 1024;

/// 附件入参：给**路径**而不是字节。
///
/// 前端传 `Vec<u8>` 会被序列化成 JSON 数字数组（一个字节一个数字），
/// 几 MB 的附件就能把 IPC 打爆。由 Rust 侧读文件既快又省事。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct AttachmentInput {
    pub path: String,
    /// 显示用的文件名；缺省取路径的文件名。
    pub filename: Option<String>,
}

/// 归一化收件人：去空白、丢空串。
pub fn normalize_recipients(raw: Vec<String>) -> Vec<String> {
    raw.into_iter()
        .map(|item| item.trim().to_owned())
        .filter(|item| !item.is_empty())
        .collect()
}

/// 由扩展名猜 MIME 类型。猜不到一律 `application/octet-stream` —— 猜错比不猜更糟。
pub fn guess_content_type(path: &str) -> String {
    let ext = Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    let guessed = match ext.as_str() {
        "pdf" => "application/pdf",
        "txt" | "log" => "text/plain",
        "csv" => "text/csv",
        "json" => "application/json",
        "xml" => "application/xml",
        "zip" => "application/zip",
        "gz" | "tgz" => "application/gzip",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "mp4" => "video/mp4",
        "mp3" => "audio/mpeg",
        _ => "application/octet-stream",
    };

    guessed.to_owned()
}

/// 读取附件文件内容。
///
/// 公开以便单测用真实临时文件验证 —— 文件读取与大小限制不该因为
/// `send_mail` 会建连就连带失去覆盖。
pub fn load_attachments(inputs: &[AttachmentInput]) -> Result<Vec<OutgoingAttachment>, String> {
    let mut total: u64 = 0;
    let mut out = Vec::with_capacity(inputs.len());

    for input in inputs {
        let path = Path::new(&input.path);
        let data =
            std::fs::read(path).map_err(|err| format!("读取附件失败（{}）：{err}", input.path))?;

        total += data.len() as u64;
        if total > MAX_TOTAL_ATTACHMENT_BYTES {
            return Err(format!(
                "附件合计超过 {} MiB 上限",
                MAX_TOTAL_ATTACHMENT_BYTES / 1024 / 1024
            ));
        }

        // 显示名缺省取路径文件名；两者都没有就报错，不编一个假名字出来
        let filename = input
            .filename
            .clone()
            .or_else(|| {
                path.file_name()
                    .and_then(|value| value.to_str())
                    .map(str::to_owned)
            })
            .ok_or_else(|| format!("无法确定附件文件名：{}", input.path))?;

        out.push(OutgoingAttachment {
            content_type: guess_content_type(&filename),
            filename,
            data,
        });
    }

    Ok(out)
}

/// 发送一封邮件。
///
/// 与 `sync_folder` 同样是三段式：**`rusqlite` 的 `MutexGuard` 绝不跨 `.await`**，
/// 否则本命令的 future 不是 `Send`。
#[tauri::command]
pub async fn send_mail(
    state: tauri::State<'_, AppState>,
    account_id: i64,
    to: Vec<String>,
    subject: String,
    body_text: String,
    body_html: Option<String>,
    attachments: Vec<AttachmentInput>,
) -> Result<(), String> {
    let recipients = normalize_recipients(to);
    if recipients.is_empty() {
        return Err("至少需要一个收件人".to_owned());
    }

    // 附件读取是同步 IO，放在加锁之前，避免持锁读盘
    let attachments = load_attachments(&attachments)?;

    // ── 第一段（同步）：读身份、解密凭证
    let (email, provider_id, secret) = {
        let conn = state
            .db
            .lock()
            .map_err(|err| format!("数据库连接不可用：{err}"))?;

        let account = storage::load_account_credentials(&conn, account_id)
            .map_err(|err| err.to_string())?
            .ok_or_else(|| format!("账户不存在：{account_id}"))?;

        let blob = account
            .secret_encrypted
            .ok_or_else(|| format!("账户 {account_id} 未设置凭证"))?;

        let secret = security::decrypt_secret(&blob, &state.crypto_key)
            .map_err(|err| format!("凭证解密失败：{err}"))?;

        (account.email, account.provider_id, secret)
    };

    let cfg = provider::find_by_id(&provider_id)
        .ok_or_else(|| format!("未知的服务商标识：{provider_id}"))?;

    let mail = OutgoingMail {
        from: email.clone(),
        to: recipients,
        subject,
        body_text,
        body_html,
        attachments,
    };

    // ── 第二段（异步）：全程不持锁
    smtp::send_mail(cfg, &email, &secret, &mail)
        .await
        .map_err(|err| err.to_string())
}

/// 本地「已发送」文件夹名。
///
/// **这是本地归档，不是服务器上的发件箱。** 各服务商对发件箱的命名完全不同
/// （`Sent Messages` / `已发送` / 修改过的 UTF-7），要同步它得按服务商逐个特判。
/// 本地记一份能保证「发完立刻在应用里看得到」，与服务器行为无关。
pub const SENT_FOLDER_NAME: &str = "已发送";

/// 本地归档结果：前端据此跳到「已发送」文件夹。
#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct SentRecord {
    pub folder_id: i64,
    pub message_id: i64,
}

/// 把刚发出的一封信记进本地「已发送」。
///
/// 为什么需要它：`send_mail` 只走 SMTP，投递成功后**本地什么也没留下** ——
/// 用户看到的是「发出去了，但应用里没有这封信，同步也同步不到」。
/// （服务器上的发件箱要另外同步，且各家的名字不一样。）
///
/// 只在**发送成功之后**调用，失败的信不该出现在已发送里。
#[tauri::command]
pub fn record_sent(
    state: tauri::State<'_, AppState>,
    account_id: i64,
    to: Vec<String>,
    subject: String,
    body_text: String,
    body_html: Option<String>,
) -> Result<SentRecord, String> {
    let recipients = normalize_recipients(to);
    if recipients.is_empty() {
        return Err("至少需要一个收件人".to_owned());
    }

    let mut conn = state
        .db
        .lock()
        .map_err(|err| format!("数据库连接不可用：{err}"))?;

    // 发件人就是账户本身，从库里取，不让前端传（前端传就可能被伪造）
    let from_email: String = conn
        .query_row("SELECT email FROM accounts WHERE id = ?1", [account_id], |row| {
            row.get(0)
        })
        .map_err(|_| format!("账户不存在：{account_id}"))?;

    let folder_id = storage::ensure_folder(&conn, account_id, SENT_FOLDER_NAME)
        .map_err(|err| err.to_string())?;

    // 本地归档用的 uid：该文件夹内自增。它永远不参与 IMAP 去重，
    // 因为服务器上的发件箱是另一个 folder 行。
    let next_uid: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(uid), 0) + 1 FROM messages WHERE folder_id = ?1",
            [folder_id],
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let msg = storage::NewMessage {
        uid: next_uid,
        message_id: None,
        subject: if subject.trim().is_empty() {
            None
        } else {
            Some(subject)
        },
        from_name: None,
        from_address: Some(from_email),
        to_addresses: Some(recipients.join(", ")),
        date_unix: now,
        text_body: Some(body_text),
        html_body: body_html.filter(|html| !html.trim().is_empty()),
        attachments: Vec::new(),
    };

    let message_id =
        storage::upsert_message(&mut conn, folder_id, &msg).map_err(|err| err.to_string())?;
    // 自己发的信当然已经读过；否则发件箱会一直显示未读
    storage::mark_seen(&conn, message_id).map_err(|err| err.to_string())?;

    Ok(SentRecord {
        folder_id,
        message_id,
    })
}