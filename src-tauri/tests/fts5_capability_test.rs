//! FTS5 能力守卫。
//!
//! 检索方案依赖两个前提：SQLite 编译时启用了 FTS5，且支持 `trigram` 分词器。
//! 本机 rusqlite 用的是 `bundled` 特性自带的 SQLite，换版本或换 feature
//! 都可能悄悄丢掉其中一项，故用测试钉死。

use rusqlite::Connection;

fn memory_db() -> Connection {
    Connection::open_in_memory().expect("内存库")
}

#[test]
fn fts5_is_available() {
    let conn = memory_db();
    conn.execute_batch("CREATE VIRTUAL TABLE t USING fts5(body);")
        .expect("SQLite 必须启用 FTS5 —— 否则 P1 的检索方案不成立");
}

#[test]
fn trigram_tokenizer_is_available() {
    let conn = memory_db();
    conn.execute_batch("CREATE VIRTUAL TABLE t USING fts5(body, tokenize='trigram');")
        .expect("必须支持 trigram 分词器 —— 中文检索依赖它");
}

#[test]
fn unicode61_cannot_search_chinese_substrings() {
    // 钉死「为什么不用默认分词器」这个结论。若哪天这条断言失败，
    // 说明 SQLite 改了 unicode61 的中文行为，可以重新评估方案。
    let conn = memory_db();
    conn.execute_batch(
        "CREATE VIRTUAL TABLE t USING fts5(body);
         INSERT INTO t (rowid, body) VALUES (1, '项目进度汇报');",
    )
    .expect("应能建表");

    let exact: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM t WHERE t MATCH '项目进度汇报'",
            [],
            |r| r.get(0),
        )
        .expect("整串可命中");
    assert_eq!(exact, 1);

    let substring: i64 = conn
        .query_row("SELECT COUNT(*) FROM t WHERE t MATCH '项目'", [], |r| {
            r.get(0)
        })
        .expect("查询本身不应报错");
    assert_eq!(
        substring, 0,
        "unicode61 把整串汉字当单个 token，子串必然搜不到 —— 这正是弃用它的原因"
    );
}

#[test]
fn trigram_can_search_chinese_substrings_of_three_or_more() {
    let conn = memory_db();
    conn.execute_batch(
        "CREATE VIRTUAL TABLE t USING fts5(body, tokenize='trigram');
         INSERT INTO t (rowid, body) VALUES (1, '项目进度汇报');",
    )
    .expect("应能建表");

    for query in ["项目进度", "项目进度汇报", "进度汇报"] {
        let hits: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM t WHERE t MATCH ?1",
                [format!("\"{query}\"")],
                |r| r.get(0),
            )
            .expect("查询不应报错");
        assert_eq!(hits, 1, "trigram 应能命中 [{query}]");
    }
}

#[test]
fn trigram_cannot_search_two_character_queries() {
    // 这条限制是 search_messages 必须带 LIKE 回退的直接原因
    let conn = memory_db();
    conn.execute_batch(
        "CREATE VIRTUAL TABLE t USING fts5(body, tokenize='trigram');
         INSERT INTO t (rowid, body) VALUES (1, '项目进度汇报');",
    )
    .expect("应能建表");

    let hits: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM t WHERE t MATCH '\"项目\"'",
            [],
            |r| r.get(0),
        )
        .expect("查询本身不应报错");
    assert_eq!(hits, 0, "两字查询在 trigram 下必然零命中");
}
