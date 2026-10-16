//! `security::credentials` 的单元测试。
//!
//! 覆盖 `docs/tasks/P0-07-credential-encryption.md` 的加密/解密用例。

use dp_gmail::security::{decrypt_secret, encrypt_secret, SecurityError, KEY_LEN};

fn key(seed: u8) -> [u8; KEY_LEN] {
    [seed; KEY_LEN]
}

#[test]
fn round_trips_a_secret() {
    let k = key(0x11);
    let blob = encrypt_secret("qq-authorization-code", &k).expect("应能加密");
    let back = decrypt_secret(&blob, &k).expect("应能解密");
    assert_eq!(back, "qq-authorization-code");
}

#[test]
fn round_trips_empty_and_unicode() {
    let k = key(0x22);
    for plaintext in ["", "口令", "🔐 emoji 混排 abc", "line\nbreak\ttab"] {
        let blob = encrypt_secret(plaintext, &k).expect("应能加密");
        assert_eq!(decrypt_secret(&blob, &k).expect("应能解密"), plaintext);
    }
}

#[test]
fn uses_a_fresh_nonce_every_time() {
    let k = key(0x33);
    let first = encrypt_secret("same plaintext", &k).expect("应能加密");
    let second = encrypt_secret("same plaintext", &k).expect("应能加密");

    assert_ne!(
        first, second,
        "两次加密同一明文必须产出不同密文，否则 nonce 被复用了"
    );
    // nonce 在前 12 字节，前 12 字节必须不同
    assert_ne!(
        &first[..12],
        &second[..12],
        "nonce 段必须每次不同（GCM 下复用 nonce 会直接瓦解保密性）"
    );
}

#[test]
fn ciphertext_does_not_contain_plaintext() {
    let k = key(0x44);
    let blob = encrypt_secret("SUPER-SECRET-TOKEN", &k).expect("应能加密");
    let as_text = String::from_utf8_lossy(&blob);
    assert!(
        !as_text.contains("SUPER-SECRET-TOKEN"),
        "密文中不应出现明文"
    );
}

#[test]
fn fails_with_wrong_key() {
    let blob = encrypt_secret("secret", &key(0x55)).expect("应能加密");
    let err = decrypt_secret(&blob, &key(0x66)).expect_err("换把密钥必须解密失败");
    assert!(
        matches!(err, SecurityError::Decrypt),
        "错误类型不符：{err:?}"
    );
}

#[test]
fn detects_tampering() {
    let k = key(0x77);
    let blob = encrypt_secret("secret", &k).expect("应能加密");

    // 改动最后一个字节（认证标签内的一个 bit）
    let mut tampered = blob.clone();
    let last = tampered.len() - 1;
    tampered[last] ^= 0x01;

    let err = decrypt_secret(&tampered, &k).expect_err("被篡改的密文必须解密失败");
    assert!(
        matches!(err, SecurityError::Decrypt),
        "GCM 认证标签应拦下篡改，实际：{err:?}"
    );
}

#[test]
fn detects_tampering_in_ciphertext_body() {
    let k = key(0x88);
    let mut blob = encrypt_secret("secret payload", &k).expect("应能加密");

    // 改动 nonce 之后的第一个字节（正文）
    blob[12] ^= 0x01;

    let err = decrypt_secret(&blob, &k).expect_err("正文被改动必须解密失败");
    assert!(matches!(err, SecurityError::Decrypt), "实际：{err:?}");
}

#[test]
fn rejects_truncated_blob() {
    let err = decrypt_secret(&[0u8; 12], &key(0x99)).expect_err("长度不足应报错");
    assert!(
        matches!(err, SecurityError::CiphertextTooShort),
        "错误类型不符：{err:?}"
    );

    let err = decrypt_secret(&[], &key(0x99)).expect_err("空输入应报错");
    assert!(matches!(err, SecurityError::CiphertextTooShort));
}
