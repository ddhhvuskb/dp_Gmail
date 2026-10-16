//! IMAP 同步：拉取原始报文。
//!
//! 本模块只负责「从服务器取回字节」，**不碰数据库** ——
//! 落库由 `commands::sync` 在异步段结束后统一进行。
//! 这样切分是为了让 `rusqlite` 的 `MutexGuard` 不会跨 `.await` 存活。

use futures::TryStreamExt;

use super::error::MailError;
use super::imap_client::connect_and_login;
use super::parser::ParsedMessage;
use super::provider::ProviderConfig;
use crate::storage::{NewAttachment, NewMessage};

/// 从 IMAP 拉下来的一封原始报文。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchedMessage {
    /// IMAP UID，落库时的去重键。
    pub uid: i64,
    /// 原始 RFC822 报文。
    pub raw: Vec<u8>,
}

/// 拉取指定文件夹**最近** `limit` 封邮件的原始报文。
///
/// 用 `BODY.PEEK[]` 而非 `BODY[]`：后者会隐式把邮件标记为 `\Seen`，
/// 而同步**不该改变服务端状态**。
pub async fn fetch_recent(
    cfg: &ProviderConfig,
    user: &str,
    secret: &str,
    mailbox: &str,
    limit: u32,
) -> Result<Vec<FetchedMessage>, MailError> {
    let mut session = connect_and_login(cfg, user, secret).await?;

    let exists = session.select(mailbox).await?.exists;
    if exists == 0 || limit == 0 {
        session.logout().await?;
        return Ok(Vec::new());
    }

    // 只要最后 limit 封：IMAP 序号区间是闭区间 [start, exists]
    let start = exists.saturating_sub(limit).max(1);
    let sequence = format!("{start}:{exists}");

    let mut out = Vec::new();
    {
        let mut stream = session.fetch(sequence, "(UID BODY.PEEK[])").await?;
        while let Some(item) = stream.try_next().await? {
            if let (Some(uid), Some(body)) = (item.uid, item.body()) {
                out.push(FetchedMessage {
                    uid: i64::from(uid),
                    raw: body.to_vec(),
                });
            }
        }
    }

    session.logout().await?;
    Ok(out)
}

/// 把解析结果映射成存储层入参。
///
/// 放在 `mail` 侧：`DESIGN.md` 第三章的调用方向是 `mail → storage`，
/// 由 mail 负责适配 storage 的入参结构，而不是让 storage 反向依赖 mail。
pub fn to_new_message(uid: i64, parsed: &ParsedMessage) -> NewMessage {
    NewMessage {
        uid,
        message_id: parsed.message_id.clone(),
        subject: parsed.subject.clone(),
        from_name: parsed.from.as_ref().and_then(|addr| addr.name.clone()),
        from_address: parsed.from.as_ref().map(|addr| addr.address.clone()),
        to_addresses: if parsed.to.is_empty() {
            None
        } else {
            Some(
                parsed
                    .to
                    .iter()
                    .map(|addr| addr.address.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
            )
        },
        // `date_unix` 列是 NOT NULL。缺失 Date 头的邮件（极少见）记 0，
        // 在按时间倒序的列表里会排到最后，不会被误当成新邮件。
        date_unix: parsed.date_unix.unwrap_or(0),
        text_body: parsed.text_body.clone(),
        html_body: parsed.html_body.clone(),
        attachments: parsed
            .attachments
            .iter()
            .map(|att| NewAttachment {
                filename: att.filename.clone(),
                content_type: att.content_type.clone(),
                size: att.size as i64,
            })
            .collect(),
    }
}
