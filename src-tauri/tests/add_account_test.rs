//! `commands::accounts` 参数校验的测试。
//!
//! 命令本体需要 Tauri 运行时，但参数校验是纯函数，可独立验证。

use dp_gmail::commands::accounts::validate_new_account;

#[test]
fn accepts_supported_provider() {
    let (email, provider) = validate_new_account("12345678@qq.com", "auth-code").expect("应通过");
    assert_eq!(email, "12345678@qq.com");
    assert_eq!(provider.id, "qq");
    assert_eq!(provider.display_name, "QQ邮箱");
}

#[test]
fn trims_surrounding_whitespace() {
    let (email, _) = validate_new_account("  user@qq.com \n", "auth-code").expect("应通过");
    assert_eq!(email, "user@qq.com", "首尾空白应被去掉");
}

#[test]
fn preserves_local_part_case() {
    // 邮箱本地部分按 RFC 是大小写敏感的，不做小写化，避免改掉用户的地址
    let (email, _) = validate_new_account("User.Name@QQ.COM", "code").expect("应通过");
    assert_eq!(email, "User.Name@QQ.COM");
}

#[test]
fn rejects_empty_email() {
    let err = validate_new_account("", "code").expect_err("空邮箱应被拒");
    assert!(err.contains("邮箱地址"), "错误信息不符：{err}");

    let err = validate_new_account("   ", "code").expect_err("纯空白邮箱应被拒");
    assert!(err.contains("邮箱地址"), "错误信息不符：{err}");
}

#[test]
fn rejects_empty_secret() {
    let err = validate_new_account("user@qq.com", "").expect_err("空授权码应被拒");
    assert!(err.contains("授权码"), "错误信息不符：{err}");
}

#[test]
fn rejects_unknown_provider() {
    let err = validate_new_account("user@gmail.com", "code").expect_err("未收录的服务商应被拒");
    assert!(err.contains("暂不支持"), "错误信息不符：{err}");

    let err = validate_new_account("no-at-sign", "code").expect_err("缺 @ 应被拒");
    assert!(err.contains("暂不支持"), "错误信息不符：{err}");
}

#[test]
fn error_messages_never_echo_the_secret() {
    // 校验失败时不能把密码回显到错误信息里（错误会被打到前端甚至日志）
    let err = validate_new_account("user@gmail.com", "SUPER-SECRET-CODE").expect_err("应被拒");
    assert!(
        !err.contains("SUPER-SECRET-CODE"),
        "错误信息里出现了授权码：{err}"
    );
}
