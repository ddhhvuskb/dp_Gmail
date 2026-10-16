//! `storage::search_messages` 的单元测试。

use dp_gmail::storage::{
    create_account, ensure_folder, init_schema, search_messages, upsert_message, upsert_messages,
    NewMessage,
};
use rusqlite::Connection;

fn memory_db() -> Connection {
    let mut conn = Connection::open_in_memory().expect("内存库");
    init_schema(&mut conn).expect("建表");
    create_account(&conn, "a@qq.com", None, "qq", None).expect("建账户");
    ensure_folder(&conn, 1, "INBOX").expect("建文件夹");
    conn
}

fn message(uid: i64, subject: &str, body: &str) -> NewMessage {
    NewMessage {
        uid,
        message_id: None,
        subject: Some(subject.to_owned()),
        from_name: Some("张三".to_owned()),
        from_address: Some("zhangsan@qq.com".to_owned()),
        to_addresses: Some("bob@qq.com".to_owned()),
        date_unix: 1_700_000_000 + uid,
        text_body: Some(body.to_owned()),
        html_body: None,
        attachments: vec![],
    }
}

fn subjects(hits: &[dp_gmail::storage::MessageSummary]) -> Vec<&str> {
    hits.iter()
        .map(|m| m.subject.as_deref().unwrap_or(""))
        .collect()
}

#[test]
fn empty_query_returns_nothing() {
    let mut conn = memory_db();
    upsert_message(&mut conn, 1, &message(1, "项目进度汇报", "内容")).expect("写入");

    assert!(search_messages(&conn, "", 20).expect("查询").is_empty());
    assert!(search_messages(&conn, "   ", 20).expect("查询").is_empty());
}

#[test]
fn triggers_index_newly_inserted_messages() {
    let mut conn = memory_db();
    upsert_message(&mut conn, 1, &message(1, "项目进度汇报", "本周完成三项")).expect("写入");

    let hits = search_messages(&conn, "项目进度", 20).expect("查询");
    assert_eq!(subjects(&hits), vec!["项目进度汇报"]);
}

#[test]
fn searches_chinese_substring_through_fts() {
    let mut conn = memory_db();
    upsert_message(&mut conn, 1, &message(1, "项目进度汇报", "无关正文")).expect("写入");
    upsert_message(&mut conn, 1, &message(2, "周会纪要", "讨论了项目排期")).expect("写入");
    upsert_message(&mut conn, 1, &message(3, "发票", "差旅报销")).expect("写入");

    // 「项目排期」在第二封的正文里
    let hits = search_messages(&conn, "项目排期", 20).expect("查询");
    assert_eq!(subjects(&hits), vec!["周会纪要"]);

    // 三字子串也能命中第一封的主题
    let hits = search_messages(&conn, "进度汇报", 20).expect("查询");
    assert_eq!(subjects(&hits), vec!["项目进度汇报"]);
}

#[test]
fn falls_back_to_like_for_two_character_queries() {
    let mut conn = memory_db();
    upsert_message(&mut conn, 1, &message(1, "项目进度汇报", "正文")).expect("写入");
    upsert_message(&mut conn, 1, &message(2, "无关邮件", "正文")).expect("写入");

    // 两字查询：FTS trigram 必然零命中，必须靠 LIKE 回退
    let hits = search_messages(&conn, "项目", 20).expect("查询");
    assert_eq!(
        subjects(&hits),
        vec!["项目进度汇报"],
        "两字中文词必须能搜到，否则用户只会以为「搜不到」"
    );
}

#[test]
fn falls_back_to_like_for_single_character_queries() {
    let mut conn = memory_db();
    upsert_message(&mut conn, 1, &message(1, "会议纪要", "正文")).expect("写入");
    upsert_message(&mut conn, 1, &message(2, "发票", "正文")).expect("写入");

    let hits = search_messages(&conn, "会", 20).expect("查询");
    assert_eq!(subjects(&hits), vec!["会议纪要"]);
}

#[test]
fn searches_sender_address_and_name() {
    let mut conn = memory_db();
    upsert_message(&mut conn, 1, &message(1, "主题", "正文")).expect("写入");

    assert_eq!(
        search_messages(&conn, "zhangsan@qq.com", 20)
            .expect("查询")
            .len(),
        1
    );
    assert_eq!(search_messages(&conn, "张三", 20).expect("查询").len(), 1);
}

#[test]
fn reflects_updates() {
    let mut conn = memory_db();
    upsert_message(&mut conn, 1, &message(1, "旧主题内容", "正文")).expect("写入");
    assert_eq!(search_messages(&conn, "旧主题", 20).expect("查询").len(), 1);

    // 同 UID 重同步 ⇒ UPSERT ⇒ 应触发 FTS 更新触发器
    upsert_message(&mut conn, 1, &message(1, "新主题内容", "正文")).expect("再次写入");

    assert_eq!(
        search_messages(&conn, "旧主题", 20).expect("查询").len(),
        0,
        "更新后旧词不应再命中"
    );
    assert_eq!(
        search_messages(&conn, "新主题", 20).expect("查询").len(),
        1,
        "更新后新词应命中"
    );
}

#[test]
fn reflects_deletes() {
    let mut conn = memory_db();
    let id = upsert_message(&mut conn, 1, &message(1, "会被删掉的主题", "正文")).expect("写入");
    assert_eq!(
        search_messages(&conn, "会被删掉", 20).expect("查询").len(),
        1
    );

    conn.execute("DELETE FROM messages WHERE id = ?1", [id])
        .expect("删除");

    assert_eq!(
        search_messages(&conn, "会被删掉", 20).expect("查询").len(),
        0,
        "删除后不应再命中"
    );
}

#[test]
fn escapes_like_wildcards() {
    let mut conn = memory_db();
    upsert_message(&mut conn, 1, &message(1, "普通邮件", "正文")).expect("写入");
    upsert_message(&mut conn, 1, &message(2, "含%百分号", "正文")).expect("写入");

    // 未转义的话 "%" 会匹配所有邮件
    let hits = search_messages(&conn, "%", 20).expect("查询");
    assert_eq!(subjects(&hits), vec!["含%百分号"], "通配符必须被转义");
}

#[test]
fn survives_fts_syntax_characters() {
    let mut conn = memory_db();
    upsert_message(&mut conn, 1, &message(1, "发票 - 差旅", "正文")).expect("写入");

    // 这些字符若不包进短语引号，FTS5 会报 syntax error
    for query in ["发票 - 差旅", "a*b", "(括号)", "引号\"测试", "冒号:值"] {
        let result = search_messages(&conn, query, 20);
        assert!(
            result.is_ok(),
            "查询 [{query}] 不应报错：{:?}",
            result.err()
        );
    }
}

#[test]
fn clamps_limit() {
    let mut conn = memory_db();
    let batch: Vec<NewMessage> = (1..=10)
        .map(|uid| message(uid, "项目进度汇报", "正文"))
        .collect();
    upsert_messages(&mut conn, 1, &batch).expect("批量写入");

    assert_eq!(
        search_messages(&conn, "项目进度", 3).expect("查询").len(),
        3
    );
    assert_eq!(
        search_messages(&conn, "项目进度", 0).expect("查询").len(),
        1,
        "0 应归一为 1"
    );
    assert_eq!(
        search_messages(&conn, "项目进度", u32::MAX)
            .expect("查询")
            .len(),
        10,
        "超大 limit 应被钳到上限而非报错"
    );
}

#[test]
fn orders_by_date_descending() {
    let mut conn = memory_db();
    upsert_message(&mut conn, 1, &message(1, "项目进度 早", "正文")).expect("写入");
    upsert_message(&mut conn, 1, &message(5, "项目进度 晚", "正文")).expect("写入");

    let hits = search_messages(&conn, "项目进度", 20).expect("查询");
    assert_eq!(subjects(&hits), vec!["项目进度 晚", "项目进度 早"]);
}
