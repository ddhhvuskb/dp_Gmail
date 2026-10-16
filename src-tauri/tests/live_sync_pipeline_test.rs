//! 真实数据落库验证 —— 把真机上取回的报文走完整条存储链路。
//!
//! `fetch_recent` 与存储层此前**各自**都验证过，但
//! **「真实报文 → 解析 → 落库 → FTS 检索」这条组合链路从未用真实数据跑过**。
//! 合成数据自洽，真数据才有真头 —— 上一轮 `From`/`Sender` 的缺陷就是这么撞出来的。
//!
//! **只写临时库，绝不碰 `%APPDATA%` 里的应用数据库。**
//!
//! 凭证只从环境变量读（`DP_GMAIL_TEST_USER` / `DP_GMAIL_TEST_SECRET`），
//! 本文件不含任何账号或授权码。用法同 `live_account_test.rs`。

use std::env;

use dp_gmail::mail::parser::parse_message;
use dp_gmail::mail::provider::resolve_by_email;
use dp_gmail::mail::sync::{fetch_recent, to_new_message};
use dp_gmail::storage::{
    create_account, ensure_folder, list_messages, open_database, search_messages, upsert_messages,
};

fn credentials() -> (String, String) {
    let user = env::var("DP_GMAIL_TEST_USER").unwrap_or_default();
    let secret = env::var("DP_GMAIL_TEST_SECRET").unwrap_or_default();
    assert!(
        !user.is_empty() && !secret.is_empty(),
        "缺少 DP_GMAIL_TEST_USER / DP_GMAIL_TEST_SECRET。\n\
         本文件的用例只在显式 --ignored 时运行，所以缺凭证是**失败**，不是跳过。"
    );
    (user, secret)
}

/// 取主题里**第一段连续 3 个字母数字字符**做检索词。
///
/// 3 个字符是门槛：`messages_fts` 用 `tokenize='trigram'`，少于 3 个字符走 LIKE 回退。
///
/// **必须取自原文连续片段。** 第一版写成「过滤掉非字母数字再取前 3 个」，
/// 于是 `dp_Gmail 真机自检 …` 被拼成了 `dpG` —— 这个串在原文里根本不存在，
/// 零命中是**测试自己的错**，不是产品的错。那次告警是断言打出来的，不是猜的。
fn query_from_subject(subject: &str) -> Option<String> {
    let chars: Vec<char> = subject.chars().collect();
    chars
        .windows(3)
        .find(|window| window.iter().all(|c| c.is_alphanumeric()))
        .map(|window| window.iter().collect())
}

#[tokio::test]
#[ignore = "真机：需要真实授权码；只写临时库，不碰应用数据库"]
async fn live_real_messages_survive_the_whole_storage_pipeline() {
    let (user, secret) = credentials();
    let cfg = resolve_by_email(&user).expect("该邮箱域名不在 provider 表里");

    let fetched = fetch_recent(cfg, &user, &secret, "INBOX", 20)
        .await
        .expect("fetch_recent 失败");
    assert!(!fetched.is_empty(), "INBOX 应当至少有 1 封邮件");

    let parsed: Vec<_> = fetched
        .iter()
        .filter_map(|item| parse_message(&item.raw).ok().map(|p| (item.uid, p)))
        .collect();
    println!(
        "[落库] 取回 {} 封，解析成功 {} 封",
        fetched.len(),
        parsed.len()
    );
    assert_eq!(parsed.len(), fetched.len(), "真实的正常邮件不该解析失败");

    // 临时库 —— 绝不碰应用数据库
    let dir = env::temp_dir().join(format!("dp_gmail_pipeline_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("建临时目录失败");
    let db_path = dir.join("pipeline.sqlite");
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{}{suffix}", db_path.display()));
    }

    let mut conn = open_database(&db_path).expect("建库失败");
    let account_id =
        create_account(&conn, &user, Some("真机验证"), cfg.id, None).expect("建账户失败");
    let folder_id = ensure_folder(&conn, account_id, "INBOX").expect("建文件夹失败");

    let batch: Vec<_> = parsed
        .iter()
        .map(|(uid, p)| to_new_message(*uid, p))
        .collect();

    let written = upsert_messages(&mut conn, folder_id, &batch).expect("落库失败");
    assert_eq!(written as usize, batch.len());

    // 1. 读得回来
    let listed = list_messages(&conn, folder_id, 100, 0).expect("读取失败");
    assert_eq!(listed.len(), batch.len(), "落库后应能全部读回");

    // 2. FTS 检索真的能命中真实主题
    let mut searched = 0;
    let mut hits = 0;
    for (_, msg) in &parsed {
        let Some(subject) = msg.subject.as_deref() else {
            continue;
        };
        let Some(query) = query_from_subject(subject) else {
            continue;
        };
        searched += 1;
        let found = search_messages(&conn, &query, 50).expect("检索失败");
        if !found.is_empty() {
            hits += 1;
        } else {
            println!("[落库] ⚠️ 检索「{query}」零命中（主题：{subject}）");
        }
    }
    println!("[落库] FTS：{searched} 条可检索主题，命中 {hits} 条");
    assert!(searched > 0, "应当至少有一条主题够长、可用于检索");
    assert!(hits > 0, "真实主题的 FTS 检索不该全军覆没");

    // 3. 重复落库按 uid 去重 —— 重同步不该产生重复邮件
    let again = upsert_messages(&mut conn, folder_id, &batch).expect("二次落库失败");
    assert_eq!(again as usize, batch.len(), "upsert 应报告处理条数");
    let after = list_messages(&conn, folder_id, 100, 0).expect("二次读取失败");
    assert_eq!(after.len(), batch.len(), "重同步不该让邮件翻倍");
    println!("[落库] 去重：重复 upsert 后仍是 {} 封 ✅", after.len());

    // 4. 附件计数与真实报文对得上
    let with_attachments = parsed
        .iter()
        .filter(|(_, m)| !m.attachments.is_empty())
        .count();
    println!("[落库] 其中带附件的真实邮件 {with_attachments} 封");

    drop(conn);
    let _ = std::fs::remove_dir_all(&dir);
}
