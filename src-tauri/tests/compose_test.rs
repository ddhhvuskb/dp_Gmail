//! `commands::compose` 纯函数的单元测试。
//!
//! 命令本体需要 Tauri 运行时，但收件人归一、MIME 猜测、附件读取都是可独立验证的。

use std::path::PathBuf;

use dp_gmail::commands::compose::{
    guess_content_type, load_attachments, normalize_recipients, AttachmentInput,
};

// ── normalize_recipients ────────────────────────────────────────

#[test]
fn trims_and_drops_blank_recipients() {
    let out = normalize_recipients(vec![
        "  a@qq.com ".to_owned(),
        String::new(),
        "   ".to_owned(),
        "b@qq.com".to_owned(),
    ]);
    assert_eq!(out, vec!["a@qq.com", "b@qq.com"]);
}

#[test]
fn keeps_display_name_recipients_intact() {
    let out = normalize_recipients(vec!["  张三 <a@qq.com>  ".to_owned()]);
    assert_eq!(out, vec!["张三 <a@qq.com>"]);
}

#[test]
fn handles_empty_input() {
    assert!(normalize_recipients(vec![]).is_empty());
    assert!(normalize_recipients(vec!["".to_owned()]).is_empty());
}

// ── guess_content_type ──────────────────────────────────────────

#[test]
fn guesses_common_extensions() {
    assert_eq!(guess_content_type("a.pdf"), "application/pdf");
    assert_eq!(guess_content_type("a.png"), "image/png");
    assert_eq!(
        guess_content_type("a.docx"),
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
    );
    assert_eq!(
        guess_content_type("a.xlsx"),
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
    );
}

#[test]
fn guess_is_case_insensitive_and_path_aware() {
    assert_eq!(guess_content_type("C:/tmp/REPORT.PDF"), "application/pdf");
    assert_eq!(guess_content_type("/home/u/photo.JPEG"), "image/jpeg");
    assert_eq!(guess_content_type("dir.with.dots/a.txt"), "text/plain");
}

#[test]
fn unknown_extension_falls_back_to_octet_stream() {
    // 猜不到就别猜 —— 猜错（例如把 exe 当 image/png）比 application/octet-stream 更糟
    assert_eq!(guess_content_type("a.qqq"), "application/octet-stream");
    assert_eq!(
        guess_content_type("noextension"),
        "application/octet-stream"
    );
    assert_eq!(guess_content_type(""), "application/octet-stream");
}

// ── load_attachments ────────────────────────────────────────────

fn temp_file(tag: &str, content: &[u8]) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("dp_gmail_att_{tag}_{}.bin", std::process::id()));
    std::fs::write(&path, content).expect("应能写临时文件");
    path
}

#[test]
fn reads_file_and_derives_filename_from_path() {
    let path = temp_file("derive", b"HELLO");
    let input = AttachmentInput {
        path: path.to_string_lossy().into_owned(),
        filename: None,
    };

    let loaded = load_attachments(std::slice::from_ref(&input)).expect("应能读取");
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].data, b"HELLO");
    assert!(
        loaded[0].filename.starts_with("dp_gmail_att_derive_"),
        "缺省文件名应取自路径，实际：{}",
        loaded[0].filename
    );

    let _ = std::fs::remove_file(&path);
}

#[test]
fn explicit_filename_wins_over_path() {
    let path = temp_file("explicit", b"X");
    let input = AttachmentInput {
        path: path.to_string_lossy().into_owned(),
        filename: Some("季度报告.pdf".to_owned()),
    };

    let loaded = load_attachments(std::slice::from_ref(&input)).expect("应能读取");
    assert_eq!(loaded[0].filename, "季度报告.pdf");
    assert_eq!(
        loaded[0].content_type, "application/pdf",
        "Content-Type 应据显示名的扩展名推断"
    );

    let _ = std::fs::remove_file(&path);
}

#[test]
fn reports_missing_file_with_the_path() {
    let missing = if cfg!(windows) {
        "C:/definitely/not/here/nope.pdf"
    } else {
        "/definitely/not/here/nope.pdf"
    };
    let input = AttachmentInput {
        path: missing.to_owned(),
        filename: None,
    };

    let err = load_attachments(std::slice::from_ref(&input)).expect_err("不存在的文件应报错");
    assert!(
        err.contains("nope.pdf"),
        "错误信息应带上出问题的路径，实际：{err}"
    );
}

#[test]
fn accepts_multiple_attachments() {
    let first = temp_file("multi_a", b"AAA");
    let second = temp_file("multi_b", b"BBBB");

    let inputs = vec![
        AttachmentInput {
            path: first.to_string_lossy().into_owned(),
            filename: None,
        },
        AttachmentInput {
            path: second.to_string_lossy().into_owned(),
            filename: None,
        },
    ];

    let loaded = load_attachments(&inputs).expect("应能读取");
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].data, b"AAA");
    assert_eq!(loaded[1].data, b"BBBB");

    let _ = std::fs::remove_file(&first);
    let _ = std::fs::remove_file(&second);
}

#[test]
fn empty_input_yields_no_attachments() {
    assert!(load_attachments(&[]).expect("空输入不应报错").is_empty());
}
