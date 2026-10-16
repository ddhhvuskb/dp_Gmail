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

#[test]
fn resolves_netease_163_mailbox() {
    // 163 是 P3 新增的服务商。模块层有 provider_163_test 覆盖，
    // 这里验证它**能穿过命令层** —— UI 走的是这条路径，不是模块函数。
    let view = resolve_provider("someone@163.com".to_owned()).expect("应识别 163 邮箱");

    assert_eq!(view.provider_id, "netease163");
    assert_eq!(view.provider_name, "网易163邮箱");
    assert_eq!(view.imap_host, "imap.163.com");
    assert_eq!(view.imap_port, 993);
    assert_eq!(view.smtp_host, "smtp.163.com");
    assert_eq!(view.smtp_port, 465);
}

#[test]
fn every_provider_survives_the_command_layer() {
    // 表里以后再加服务商时，这条会自动覆盖到 —— 防止出现
    // "模块表里有、命令层却拿不到" 的断层（例如忘了同步某个字段）。
    use dp_gmail::mail::provider::PROVIDERS;

    for provider in PROVIDERS {
        let address = format!("someone@{}", provider.domains[0]);
        let view =
            resolve_provider(address.clone()).unwrap_or_else(|| panic!("{address} 应能穿过命令层"));

        assert_eq!(view.provider_id, provider.id);
        assert_eq!(view.imap_host, provider.imap_host);
        assert_eq!(view.imap_port, provider.imap_port);
        assert_eq!(view.smtp_host, provider.smtp_host);
        assert_eq!(view.smtp_port, provider.smtp_port);
    }
}
