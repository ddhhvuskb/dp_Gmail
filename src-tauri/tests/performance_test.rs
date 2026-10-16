//! 性能验收测试。
//!
//! `DESIGN.md` 验收总则写着「1000 封列表滚动不卡，搜索 <200ms」。
//! 这条标准此前只有设计上的论证，没有实测 —— 本文件把它钉成回归测试。
//!
//! **注意运行模式**：`cargo test` 默认是 debug 构建，比 release 慢数倍。
//! 这里的断言按 debug 取值，是为了在开发过程中就能发现数量级退化；
//! release 下的真实数字会明显更好（见测试输出的实测打印）。

use std::time::{Duration, Instant};

use dp_gmail::storage::{
    create_account, ensure_folder, init_schema, list_messages, list_unified_messages,
    search_messages, upsert_messages, NewMessage,
};
use rusqlite::Connection;

/// `DESIGN.md` 的搜索耗时上限。
const SEARCH_BUDGET: Duration = Duration::from_millis(200);

/// 验收标准里的邮件规模。
const MESSAGE_COUNT: usize = 1000;

fn seeded_db() -> (Connection, i64) {
    let mut conn = Connection::open_in_memory().expect("内存库");
    init_schema(&mut conn).expect("建表");

    create_account(&conn, "a@qq.com", Some("甲"), "qq", None).expect("账户 A");
    create_account(&conn, "b@qq.com", Some("乙"), "qq", None).expect("账户 B");
    let a_inbox = ensure_folder(&conn, 1, "INBOX").expect("A/INBOX");
    let b_inbox = ensure_folder(&conn, 2, "INBOX").expect("B/INBOX");

    // 一半一半，两边都塞满，统一收件箱才有意义
    let mut batch_a = Vec::with_capacity(MESSAGE_COUNT / 2);
    let mut batch_b = Vec::with_capacity(MESSAGE_COUNT / 2);

    for index in 0..MESSAGE_COUNT {
        let subject = if index % 7 == 0 {
            format!("项目进度汇报 第{index}期 — 里程碑与风险")
        } else if index % 5 == 0 {
            format!("会议纪要 {index}：排期与人力")
        } else if index % 3 == 0 {
            format!("发票 {index} 差旅报销单")
        } else {
            format!("Invoice #{index} 报销凭证")
        };

        // 正文里刻意放入高频字「的」，供最坏情况用例使用
        let body = format!(
            "本期待办共 {} 项。项目排期需要 review 的环节涉及跨团队协作与资源协调。\n\
             相关附件见邮件，如有疑问请回复。这是第 {} 封测试邮件。\n编号 {}",
            index % 13,
            index,
            index
        );

        let message = NewMessage {
            uid: index as i64 + 1,
            message_id: Some(format!("<m{index}@qq.com>")),
            subject: Some(subject),
            from_name: Some(format!("发件人{}", index % 20)),
            from_address: Some(format!("sender{}@qq.com", index % 20)),
            to_addresses: Some("me@qq.com".to_owned()),
            date_unix: 1_700_000_000 + index as i64 * 60,
            text_body: Some(body.clone()),
            html_body: Some(format!(
                "<p>{}</p><script>alert(1)</script>",
                body.replace('\n', "<br>")
            )),
            attachments: if index % 11 == 0 {
                vec![dp_gmail::storage::NewAttachment {
                    filename: format!("report-{index}.pdf"),
                    content_type: "application/pdf".to_owned(),
                    size: 4096 + index as i64,
                }]
            } else {
                vec![]
            },
        };

        if index % 2 == 0 {
            batch_a.push(message);
        } else {
            batch_b.push(message);
        }
    }

    upsert_messages(&mut conn, a_inbox, &batch_a).expect("批量写入 A");
    upsert_messages(&mut conn, b_inbox, &batch_b).expect("批量写入 B");

    (conn, a_inbox)
}

fn time<T>(label: &str, f: impl FnOnce() -> T) -> (T, Duration) {
    let started = Instant::now();
    let value = f();
    let elapsed = started.elapsed();
    println!("  {label}: {elapsed:?}");
    (value, elapsed)
}

#[test]
fn seeds_expected_message_count() {
    let (conn, _) = seeded_db();
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM messages", [], |row| row.get(0))
        .expect("统计");
    assert_eq!(count as usize, MESSAGE_COUNT);

    // FTS 触发器应把每一封都索引进去了
    let indexed: i64 = conn
        .query_row("SELECT COUNT(*) FROM messages_fts", [], |row| row.get(0))
        .expect("统计 FTS");
    assert_eq!(
        indexed as usize, MESSAGE_COUNT,
        "FTS 索引应与 messages 一一对应"
    );
}

#[test]
fn fts_search_stays_within_budget() {
    let (conn, _) = seeded_db();
    println!("FTS 路径（>=3 字符）:");

    for query in [
        "项目进度",
        "里程碑与风险",
        "会议纪要",
        "差旅报销单",
        "sender3@qq.com",
    ] {
        let (hits, elapsed) = time(query, || search_messages(&conn, query, 200).expect("查询"));
        assert!(!hits.is_empty(), "[{query}] 应有命中");
        assert!(
            elapsed < SEARCH_BUDGET,
            "[{query}] 超过 {SEARCH_BUDGET:?} 预算：{elapsed:?}"
        );
    }
}

#[test]
fn like_fallback_stays_within_budget() {
    // 短查询走全表扫描，是唯一有性能风险的分支（见 P1-01 §2.1）
    let (conn, _) = seeded_db();
    println!("LIKE 回退路径（<3 字符，全表扫描）:");

    for query in ["项目", "进度", "会议", "发票"] {
        let (hits, elapsed) = time(query, || search_messages(&conn, query, 200).expect("查询"));
        assert!(!hits.is_empty(), "[{query}] 应有命中");
        assert!(
            elapsed < SEARCH_BUDGET,
            "[{query}] 超预算：{elapsed:?} —— 这条正是 P1-01 里标注的性能风险分支"
        );
    }
}

#[test]
fn worst_case_query_stays_within_budget() {
    // 「的」在正文里高频出现，会命中几乎每一封，是 LIKE 回退的最坏情况
    let (conn, _) = seeded_db();
    println!("最坏情况（单字高频命中）:");

    let (hits, elapsed) = time("的", || search_messages(&conn, "的", 200).expect("查询"));
    println!("  命中 {} 条（受 limit=200 限制）", hits.len());
    assert!(
        !hits.is_empty(),
        "种子数据里应有高频字「的」—— 否则这条用例根本没测到最坏情况"
    );
    assert_eq!(hits.len(), 200, "应顶到 limit 上限");
    assert!(elapsed < SEARCH_BUDGET, "最坏情况超预算：{elapsed:?}");
}

#[test]
fn listing_a_full_folder_stays_within_budget() {
    let (conn, folder) = seeded_db();
    println!("列表路径:");

    let (rows, elapsed) = time("list_messages(1000)", || {
        list_messages(&conn, folder, 1000, 0).expect("查询")
    });
    assert_eq!(rows.len(), MESSAGE_COUNT / 2);
    assert!(elapsed < SEARCH_BUDGET, "列表查询超预算：{elapsed:?}");
}

#[test]
fn unified_inbox_stays_within_budget() {
    let (conn, _) = seeded_db();
    println!("统一收件箱路径:");

    let (rows, elapsed) = time("list_unified_messages(1000)", || {
        list_unified_messages(&conn, None, Some("INBOX"), 1000, 0).expect("查询")
    });
    assert_eq!(rows.len(), MESSAGE_COUNT, "两个账户的 INBOX 应合计 1000 条");
    assert!(elapsed < SEARCH_BUDGET, "统一收件箱查询超预算：{elapsed:?}");
}

#[test]
fn deep_pagination_does_not_degrade() {
    // 深翻页是 OFFSET 分页的经典退化点
    let (conn, _) = seeded_db();
    println!("深翻页:");

    let (_, first) = time("offset 0", || {
        list_unified_messages(&conn, None, None, 50, 0).expect("查询")
    });
    let (last_page, last) = time("offset 950", || {
        list_unified_messages(&conn, None, None, 50, 950).expect("查询")
    });

    assert_eq!(last_page.len(), 50);
    assert!(
        last < SEARCH_BUDGET,
        "深翻页超预算：{last:?}（首页 {first:?}）"
    );
}
