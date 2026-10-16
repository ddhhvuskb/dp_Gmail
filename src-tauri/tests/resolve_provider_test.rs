//! `commands::accounts::resolve_provider` 的测试。
//!
//! 该命令不依赖 `tauri::State`，是纯函数，可以直接调用验证 ——
//! 这比只测底层的 `provider::resolve_by_email` 更贴近真实调用路径。

use dp_gmail::commands::accounts::resolve_provider;

#[test]
fn resolves_qq_mailbox() {
    let view = resolve_provider("12345678@qq.com".to_owned()).expect("应识别 QQ 邮箱");

    assert_eq!(view.provider_id, "qq");
    assert_eq!(view.provider_name, "QQ邮箱");
    assert_eq!(view.imap_host, "imap.qq.com");
    assert_eq!(view.imap_port, 993);
    assert_eq!(view.smtp_host, "smtp.qq.com");
    assert_eq!(view.smtp_port, 465);
}

#[test]
fn resolves_qq_family_domains() {
    for address in ["user@vip.qq.com", "user@foxmail.com"] {
        let view = resolve_provider(address.to_owned()).expect("应识别 QQ 系域名");
        assert_eq!(view.provider_id, "qq");
    }
}

#[test]
fn is_case_insensitive_and_trims() {
    let view = resolve_provider("  User@QQ.COM  ".to_owned()).expect("应识别");
    assert_eq!(view.provider_id, "qq");
}

#[test]
fn returns_none_for_unsupported_provider() {
    assert!(resolve_provider("user@gmail.com".to_owned()).is_none());
    assert!(resolve_provider("no-at-sign".to_owned()).is_none());
    assert!(resolve_provider(String::new()).is_none());
    assert!(
        resolve_provider("@qq.com".to_owned()).is_none(),
        "空本地部分应被拒"
    );
}

#[test]
fn never_returns_port_zero() {
    // 端口为 0 意味着配置表写漏了，会让前端显示 "imap.qq.com:0" 这种误导信息
    let view = resolve_provider("user@qq.com".to_owned()).expect("应识别");
    assert_ne!(view.imap_port, 0);
    assert_ne!(view.smtp_port, 0);
}
