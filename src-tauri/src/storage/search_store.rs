//! 全文检索。
//!
//! 索引是 FTS5 外部内容虚表 `messages_fts`（见 `storage::schema`），
//! 由触发器与 `messages` 保持同步。

use rusqlite::{params, Connection, Row};

use super::error::StorageError;
use super::message_store::MessageSummary;

/// 单次搜索返回的最大条数。
pub const MAX_SEARCH_LIMIT: u32 = 200;

/// FTS5 的 trigram 分词器只索引**三字符序列**，
/// 短于此长度的 MATCH 查询必然零命中，必须换一条路。
const MIN_FTS_QUERY_CHARS: usize = 3;

/// LIKE 回退时用的转义符。
const LIKE_ESCAPE: char = '\\';

/// 全文检索邮件，按收件时间倒序返回摘要。
///
/// 查询短于 3 字符时走 LIKE 回退。这不是优化取舍，而是**正确性要求**：
/// FTS5 的 trigram 索引对两字查询无能为力，而中文里「项目」「进度」「会议」
/// 这类双字词恰恰最常见 —— 硬走 MATCH 会静默返回空结果，
/// 用户只会以为「搜不到」而不知道是分词器限制。
pub fn search_messages(
    conn: &Connection,
    query: &str,
    limit: u32,
) -> Result<Vec<MessageSummary>, StorageError> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }
    let limit = limit.clamp(1, MAX_SEARCH_LIMIT);

    if query.chars().count() < MIN_FTS_QUERY_CHARS {
        return search_with_like(conn, query, limit);
    }

    let mut stmt = conn.prepare(
        "SELECT m.id, m.uid, m.subject, m.from_name, m.from_address, m.date_unix, m.is_seen,
                (SELECT COUNT(*) FROM attachments a WHERE a.message_id = m.id)
         FROM messages_fts
         JOIN messages m ON m.id = messages_fts.rowid
         WHERE messages_fts MATCH ?1
         ORDER BY m.date_unix DESC, m.id DESC
         LIMIT ?2",
    )?;

    let rows = stmt.query_map(params![as_fts_phrase(query), limit], map_summary)?;
    collect(rows)
}

/// 短查询回退：对正文与地址做子串匹配。
///
/// 走不了索引，但本地库规模下（万级邮件）仍是毫秒级。
fn search_with_like(
    conn: &Connection,
    query: &str,
    limit: u32,
) -> Result<Vec<MessageSummary>, StorageError> {
    let pattern = format!("%{}%", escape_like(query));

    let mut stmt = conn.prepare(
        "SELECT m.id, m.uid, m.subject, m.from_name, m.from_address, m.date_unix, m.is_seen,
                (SELECT COUNT(*) FROM attachments a WHERE a.message_id = m.id)
         FROM messages m
         WHERE m.subject      LIKE ?1 ESCAPE '\\'
            OR m.from_name    LIKE ?1 ESCAPE '\\'
            OR m.from_address LIKE ?1 ESCAPE '\\'
            OR m.to_addresses LIKE ?1 ESCAPE '\\'
            OR m.text_body    LIKE ?1 ESCAPE '\\'
         ORDER BY m.date_unix DESC, m.id DESC
         LIMIT ?2",
    )?;

    let rows = stmt.query_map(params![pattern, limit], map_summary)?;
    collect(rows)
}

/// 把用户输入包成 FTS5 短语查询，并转义内部双引号。
///
/// 不包引号的话，`-`、`*`、`(`、`:` 等字符会被 FTS5 当成语法：
/// 轻则报 "fts5: syntax error"，重则语义完全变样。
fn as_fts_phrase(raw: &str) -> String {
    format!("\"{}\"", raw.replace('"', "\"\""))
}

/// 转义 LIKE 模式里的通配符，否则用户输入 `%` 会匹配到所有邮件。
fn escape_like(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch == LIKE_ESCAPE || ch == '%' || ch == '_' {
            out.push(LIKE_ESCAPE);
        }
        out.push(ch);
    }
    out
}

/// 两条查询路径共用同一套列顺序，避免映射写两份。
fn map_summary(row: &Row<'_>) -> rusqlite::Result<MessageSummary> {
    Ok(MessageSummary {
        id: row.get(0)?,
        uid: row.get(1)?,
        subject: row.get(2)?,
        from_name: row.get(3)?,
        from_address: row.get(4)?,
        date_unix: row.get(5)?,
        is_seen: row.get::<_, i64>(6)? != 0,
        attachment_count: row.get(7)?,
    })
}

fn collect(
    rows: rusqlite::MappedRows<'_, impl FnMut(&Row<'_>) -> rusqlite::Result<MessageSummary>>,
) -> Result<Vec<MessageSummary>, StorageError> {
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}
