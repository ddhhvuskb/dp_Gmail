//! 网易 163 服务商条目的测试。
//!
//! 与 `provider_test.rs` 分开：那份覆盖的是解析逻辑，这份盯的是
//! **本表新增条目的具体参数**是否正确。
//!
//! 地址来源：`docs/tasks/P3-01-provider-compat.md` §2.1（Zoho 汇总表）。
//! **这些断言是人手抄进代码的"预期值"，不是推算出来的** —— 它们的作用是
//! 在有人改错端口或主机名时立刻报警。

use dp_gmail::mail::provider::{find_by_id, resolve_by_email, TlsMode, PROVIDERS};

#[test]
fn resolves_163_domain_to_the_netease_entry() {
    let config = resolve_by_email("someone@163.com").expect("163.com 应能识别");
    assert_eq!(config.id, "netease163");
    assert_eq!(config.display_name, "网易163邮箱");
}

#[test]
fn netease_addresses_match_the_verified_source() {
    let config = find_by_id("netease163").expect("应能找到 netease163");

    assert_eq!(config.imap_host, "imap.163.com");
    assert_eq!(config.imap_port, 993);
    assert_eq!(config.imap_tls, TlsMode::Implicit);

    assert_eq!(config.smtp_host, "smtp.163.com");
    assert_eq!(config.smtp_port, 465);
    assert_eq!(config.smtp_tls, TlsMode::Implicit);
}

#[test]
fn provider_ids_are_unique() {
    // 按 id 反查靠的是唯一性；重复 id 会让 find_by_id 静默返回错的那条
    let mut ids: Vec<&str> = PROVIDERS.iter().map(|p| p.id).collect();
    ids.sort_unstable();
    let before = ids.len();
    ids.dedup();
    assert_eq!(ids.len(), before, "服务商标识出现重复：{ids:?}");
}

#[test]
fn no_domain_is_claimed_by_two_providers() {
    // 域名重叠会让 resolve_by_email 的结果取决于表里的先后顺序 —— 那是隐式行为，
    // 不是设计。宁可在这里报错，也不要在用户那边表现为"时好时坏"。
    let mut seen: Vec<(&str, &str)> = Vec::new();
    for provider in PROVIDERS {
        for domain in provider.domains {
            if let Some((other_id, _)) = seen.iter().find(|(_, d)| d == domain) {
                panic!("域名 {domain} 同时被 {other_id} 与 {} 声明", provider.id);
            }
            seen.push((provider.id, domain));
        }
    }
}

#[test]
fn qq_entry_is_unchanged_by_the_addition() {
    // 新增条目不该动到已有的一条
    let qq = find_by_id("qq").expect("应能找到 qq");
    assert_eq!(qq.imap_host, "imap.qq.com");
    assert_eq!(qq.smtp_host, "smtp.qq.com");
    assert_eq!(qq.domains, &["qq.com", "vip.qq.com", "foxmail.com"]);
}

#[test]
fn unrelated_domains_still_resolve_to_nothing() {
    // 未核实的服务商一律留空待补（见 P3-01 §2.3），此处固化该决定：
    // 宁可识别不出、让用户手填，也不填一个可能连不上的地址。
    for email in [
        "someone@126.com",
        "someone@yeah.net",
        "someone@sina.com",
        "someone@139.com",
        "someone@189.cn",
    ] {
        assert!(
            resolve_by_email(email).is_none(),
            "{email} 尚未核实，不应出现在服务商表里"
        );
    }
}
