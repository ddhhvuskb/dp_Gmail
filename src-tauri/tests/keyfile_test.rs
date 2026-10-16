//! `security::keyfile` 的单元测试。
//!
//! 覆盖密钥文件的生成、复用、以及各类损坏/不兼容情形的拒绝行为。

use std::path::PathBuf;

use dp_gmail::security::{load_or_create_key, SecurityError, KEY_FILE_VERSION, KEY_LEN};

/// 每个用例独立的临时密钥文件路径。
fn temp_key_path(tag: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("dp_gmail_key_{tag}_{}.json", std::process::id()));
    let _ = std::fs::remove_file(&path);
    path
}

fn cleanup(path: &PathBuf) {
    let _ = std::fs::remove_file(path);
}

#[test]
fn creates_key_file_when_missing() {
    let path = temp_key_path("create");
    assert!(!path.exists());

    let key = load_or_create_key(&path).expect("应能生成密钥");
    assert_eq!(key.len(), KEY_LEN);
    assert!(path.exists(), "应把密钥写入文件");

    cleanup(&path);
}

#[test]
fn returns_same_key_on_second_load() {
    let path = temp_key_path("stable");
    let first = load_or_create_key(&path).expect("首次");
    let second = load_or_create_key(&path).expect("二次");
    assert_eq!(first, second, "同一文件必须解出同一把密钥");

    cleanup(&path);
}

#[test]
fn generates_distinct_keys_for_distinct_files() {
    let path_a = temp_key_path("distinct_a");
    let path_b = temp_key_path("distinct_b");

    let a = load_or_create_key(&path_a).expect("A");
    let b = load_or_create_key(&path_b).expect("B");
    assert_ne!(a, b, "不同文件不应产出同一把密钥");

    cleanup(&path_a);
    cleanup(&path_b);
}

#[test]
fn writes_versioned_json_with_expected_fields() {
    let path = temp_key_path("shape");
    load_or_create_key(&path).expect("应能生成");

    let raw = std::fs::read_to_string(&path).expect("应能读取");
    let parsed: serde_json::Value = serde_json::from_str(&raw).expect("应是合法 JSON");

    assert_eq!(parsed["version"], KEY_FILE_VERSION);
    assert_eq!(parsed["algorithm"], "AES-256-GCM");
    assert!(parsed["created_at"].is_i64(), "created_at 应是 Unix 整数");

    let key_b64 = parsed["key"].as_str().expect("key 应是字符串");
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    let decoded = STANDARD.decode(key_b64).expect("key 应是合法 base64");
    assert_eq!(decoded.len(), KEY_LEN);

    cleanup(&path);
}

#[test]
fn rejects_unsupported_version() {
    let path = temp_key_path("badver");
    std::fs::write(
        &path,
        r#"{"version":99,"algorithm":"AES-256-GCM","key":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=","created_at":0}"#,
    )
    .expect("应能写测试文件");

    let err = load_or_create_key(&path).expect_err("版本不符必须报错而非猜测");
    assert!(
        matches!(err, SecurityError::KeyFileFormat(ref m) if m.contains("版本")),
        "错误类型不符：{err:?}"
    );

    cleanup(&path);
}

#[test]
fn rejects_unsupported_algorithm() {
    let path = temp_key_path("badalg");
    std::fs::write(
        &path,
        r#"{"version":1,"algorithm":"AES-128-ECB","key":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=","created_at":0}"#,
    )
    .expect("应能写测试文件");

    let err = load_or_create_key(&path).expect_err("ECB 之类必须被拒");
    assert!(
        matches!(err, SecurityError::KeyFileFormat(ref m) if m.contains("算法")),
        "错误类型不符：{err:?}"
    );

    cleanup(&path);
}

#[test]
fn rejects_wrong_key_length() {
    let path = temp_key_path("badlen");
    // 16 字节（AES-128 长度），不是 32
    std::fs::write(
        &path,
        r#"{"version":1,"algorithm":"AES-256-GCM","key":"AAAAAAAAAAAAAAAAAAAAAA==","created_at":0}"#,
    )
    .expect("应能写测试文件");

    let err = load_or_create_key(&path).expect_err("密钥长度不对必须报错");
    assert!(
        matches!(err, SecurityError::KeyFileFormat(ref m) if m.contains("32 字节")),
        "错误类型不符：{err:?}"
    );

    cleanup(&path);
}

#[test]
fn rejects_malformed_json() {
    let path = temp_key_path("badjson");
    std::fs::write(&path, "not json at all").expect("应能写测试文件");

    let err = load_or_create_key(&path).expect_err("非法 JSON 应报错");
    assert!(
        matches!(err, SecurityError::Json(_)),
        "错误类型不符：{err:?}"
    );

    cleanup(&path);
}
