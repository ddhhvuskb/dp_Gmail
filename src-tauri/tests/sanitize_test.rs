//! `security::sanitize` 的单元测试。
//!
//! 覆盖 `docs/tasks/P0-02-mime-parse.md` §5.2 约定的用例。

use dp_gmail::security::sanitize::sanitize_html;

#[test]
fn strips_script_tag_and_its_content() {
    let out = sanitize_html("<p>hi</p><script>alert(1)</script>");
    assert!(!out.contains("script"), "script 标签应被删除：{out}");
    assert!(!out.contains("alert"), "script 内容应被删除：{out}");
    assert!(out.contains("hi"), "正常内容应保留：{out}");
}

#[test]
fn strips_event_handler_attributes() {
    let out = sanitize_html(r#"<img src="x.png" onerror="steal()">"#);
    assert!(!out.contains("onerror"), "事件属性应被剥离：{out}");
    assert!(!out.contains("steal"), "事件属性值应被剥离：{out}");
}

#[test]
fn strips_javascript_url_scheme() {
    let out = sanitize_html(r#"<a href="javascript:alert(1)">click</a>"#);
    assert!(
        !out.contains("javascript:"),
        "javascript: scheme 应被丢弃：{out}"
    );
    assert!(out.contains("click"), "链接文字应保留：{out}");
}

#[test]
fn keeps_ordinary_formatting() {
    let out = sanitize_html("<p><strong>bold</strong> and <em>italic</em></p>");
    assert!(out.contains("<strong>"), "strong 应保留：{out}");
    assert!(out.contains("<em>"), "em 应保留：{out}");
    assert!(
        out.contains("bold") && out.contains("italic"),
        "文字应保留：{out}"
    );
}

#[test]
fn strips_style_tag_and_its_content() {
    let out = sanitize_html("<style>body{display:none}</style><p>ok</p>");
    assert!(!out.contains("display:none"), "style 内容应被删除：{out}");
    assert!(out.contains("ok"), "正常内容应保留：{out}");
}
