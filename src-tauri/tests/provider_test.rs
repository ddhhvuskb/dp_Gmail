//! `mail::provider` 的单元测试。
//!
//! 覆盖 `docs/tasks/P0-01-imap-connect.md` §4.1 约定的 7 个用例。

use dp_gmail::mail::provider::{find_by_id, resolve_by_email, TlsMode};

#[test]
fn resolves_plain_qq_domain() {
    let cfg = resolve_by_email("12345678@qq.com").expect("qq.com 应命中");
    assert_eq!(cfg.id, "qq");
    assert_eq!(cfg.display_name, "QQ邮箱");
    assert_eq!(cfg.imap_host, "imap.qq.com");
    assert_eq!(cfg.imap_port, 993);
    assert_eq!(cfg.imap_tls, TlsMode::Implicit);
    assert_eq!(cfg.smtp_host, "smtp.qq.com");
    assert_eq!(cfg.smtp_port, 465);
}

#[test]
fn resolves_all_known_domains() {
    for addr in ["user@qq.com", "user@vip.qq.com", "user@foxmail.com"] {
        assert!(resolve_by_email(addr).is_some(), "{addr} 应命中");
    }
}

#[test]
fn is_case_insensitive() {
    let cfg = resolve_by_email("User@QQ.COM").expect("大写域名应命中");
    assert_eq!(cfg.id, "qq");
}

#[test]
fn trims_surrounding_whitespace() {
    assert!(resolve_by_email("  user@qq.com  ").is_some());
    assert!(resolve_by_email("\tuser@qq.com\n").is_some());
}

#[test]
fn rejects_unknown_domain() {
    assert!(resolve_by_email("user@unknown.tld").is_none());
    assert!(resolve_by_email("user@qq.com.evil.tld").is_none());
}

#[test]
fn rejects_missing_at_sign() {
    assert!(resolve_by_email("noatsign").is_none());
    assert!(resolve_by_email("user.qq.com").is_none());
}

#[test]
fn rejects_empty_input() {
    assert!(resolve_by_email("").is_none());
    assert!(resolve_by_email("   ").is_none());
    assert!(resolve_by_email("@qq.com").is_none());
    assert!(resolve_by_email("user@").is_none());
}

#[test]
fn finds_provider_by_id() {
    let cfg = find_by_id("qq").expect("qq 应命中");
    assert_eq!(cfg.display_name, "QQ邮箱");
    assert_eq!(cfg.imap_host, "imap.qq.com");
}

#[test]
fn find_by_id_rejects_unknown() {
    assert!(find_by_id("gmail").is_none());
    assert!(find_by_id("").is_none());
}

#[test]
fn resolve_and_find_by_id_agree() {
    // 同步时只有 provider_id、没有邮箱，两条入口必须指向同一份静态配置
    let by_email = resolve_by_email("12345678@qq.com").expect("应命中");
    let by_id = find_by_id(by_email.id).expect("应命中");
    assert_eq!(by_email.id, by_id.id);
    assert_eq!(by_email.imap_host, by_id.imap_host);
    assert_eq!(by_email.imap_port, by_id.imap_port);
}
