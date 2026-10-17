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
fn formerly_unverified_domains_now_resolve() {
    // 这条的前身是 `unrelated_domains_still_resolve_to_nothing` —— 它当时固化的是
    // 「126 / yeah.net / sina / 139 / 189 未核实，一律留空」这个决定（P3-01 §2.3）。
    //
    // 2026-10-17 用 TLS 实测（TCP + **证书主机名匹配** + banner）核实了它们，
    // 前提不再成立，故断言方向反转：现在**必须**解析得到。
    // 否则等于把已经拿到的核实结果又丢回去。
    for (email, expected_id) in [
        ("someone@126.com", "netease126"),
        ("someone@yeah.net", "netease_yeah"),
        ("someone@sina.com", "sina"),
        ("someone@139.com", "china_mobile_139"),
        ("someone@189.cn", "china_telecom_189"),
        ("someone@aliyun.com", "aliyun_personal"),
    ] {
        let found =
            resolve_by_email(email).unwrap_or_else(|| panic!("{email} 已核实，应当能解析出来"));
        assert_eq!(found.id, expected_id, "{email} 解析到了错误的服务商");
    }
}

#[test]
fn genuinely_unverified_domains_still_resolve_to_nothing() {
    // 「未核实就留空」这条规矩**没变**，只是名单缩小了。
    // `sina.cn` 是新浪的另一个域名，但**没有实测过**，所以仍留在名单里 ——
    // 这正是规则起作用的证据：同属一家，也不能顺手套用。
    for email in [
        "someone@sina.cn",
        "someone@gmail.com",
        "someone@outlook.com",
    ] {
        assert!(
            resolve_by_email(email).is_none(),
            "{email} 未经实测，不应出现在服务商表里"
        );
    }
}
