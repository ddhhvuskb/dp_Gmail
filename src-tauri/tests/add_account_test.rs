//! `commands::accounts` 参数校验的测试。
//!
//! 命令本体需要 Tauri 运行时，但参数校验是纯函数，可独立验证。

use dp_gmail::commands::accounts::validate_new_account;

#[test]
fn accepts_supported_provider() {
    let (email, provider) =
        validate_new_account("12345678@qq.com", "auth-code", None).expect("应通过");
    assert_eq!(email, "12345678@qq.com");
    assert_eq!(provider.id, "qq");
    assert_eq!(provider.display_name, "QQ邮箱");
}

#[test]
fn trims_surrounding_whitespace() {
    let (email, _) = validate_new_account("  user@qq.com \n", "auth-code", None).expect("应通过");
    assert_eq!(email, "user@qq.com", "首尾空白应被去掉");
}

#[test]
fn preserves_local_part_case() {
    // 邮箱本地部分按 RFC 是大小写敏感的，不做小写化，避免改掉用户的地址
    let (email, _) = validate_new_account("User.Name@QQ.COM", "code", None).expect("应通过");
    assert_eq!(email, "User.Name@QQ.COM");
}

#[test]
fn rejects_empty_email() {
    let err = validate_new_account("", "code", None).expect_err("空邮箱应被拒");
    assert!(err.contains("邮箱地址"), "错误信息不符：{err}");

    let err = validate_new_account("   ", "code", None).expect_err("纯空白邮箱应被拒");
    assert!(err.contains("邮箱地址"), "错误信息不符：{err}");
}

#[test]
fn rejects_empty_secret() {
    let err = validate_new_account("user@qq.com", "", None).expect_err("空授权码应被拒");
    assert!(err.contains("授权码"), "错误信息不符：{err}");
}

#[test]
fn rejects_unknown_provider() {
    let err =
        validate_new_account("user@gmail.com", "code", None).expect_err("未收录的服务商应被拒");
    assert!(err.contains("暂不支持"), "错误信息不符：{err}");

    let err = validate_new_account("no-at-sign", "code", None).expect_err("缺 @ 应被拒");
    assert!(err.contains("暂不支持"), "错误信息不符：{err}");
}

#[test]
fn error_messages_never_echo_the_secret() {
    // 校验失败时不能把密码回显到错误信息里（错误会被打到前端甚至日志）
    let err =
        validate_new_account("user@gmail.com", "SUPER-SECRET-CODE", None).expect_err("应被拒");
    assert!(
        !err.contains("SUPER-SECRET-CODE"),
        "错误信息里出现了授权码：{err}"
    );
}

// ---- 企业邮箱：手动指定服务商 ----

#[test]
fn enterprise_domain_is_unresolvable_without_a_manual_pick() {
    // 这一条是下面那条的**前提**：企业邮箱用客户自己的域名，
    // 自动识别对它永远无解 —— 不是「识别得不好」，是根本没有可用的域名线索。
    let err = validate_new_account("me@mycompany.com", "code", None).expect_err("应识别不出来");
    assert!(err.contains("暂不支持"), "错误信息不符：{err}");
}

#[test]
fn manual_provider_id_unlocks_enterprise_mailboxes() {
    let (_, provider) = validate_new_account("me@mycompany.com", "code", Some("exmail_qq"))
        .expect("手动指定后应通过");
    assert_eq!(provider.id, "exmail_qq");
    assert_eq!(provider.imap_host, "imap.exmail.qq.com");
}

#[test]
fn manual_provider_id_wins_over_auto_detection() {
    // 用户显式指定的优先级更高：即便域名能自动识别，也以用户选的为准。
    // 否则「选了却不生效」会变成一个很难查的问题。
    let (_, provider) =
        validate_new_account("user@qq.com", "code", Some("aliyun_mail")).expect("应通过");
    assert_eq!(provider.id, "aliyun_mail");
}

#[test]
fn rejects_unknown_manual_provider_id() {
    let err = validate_new_account("me@mycompany.com", "code", Some("not_a_real_provider"))
        .expect_err("未知标识应被拒");
    assert!(err.contains("未知的服务商"), "错误信息不符：{err}");
}

#[test]
fn bad_provider_id_never_echoes_the_secret() {
    // 这条路径是这轮新加的，同样不能把授权码带进错误信息
    let err = validate_new_account("me@mycompany.com", "ANOTHER-SECRET", Some("nope"))
        .expect_err("应被拒");
    assert!(
        !err.contains("ANOTHER-SECRET"),
        "错误信息里出现了授权码：{err}"
    );
}
