# P0-02 MIME 解析（mail-parser，RFC2047 解码，unknown-8bit 处理）

- **阶段**：P0
- **依赖**：ADR-0001（`mail-parser` / `ammonia`）、P0-01（提供原始报文）
- **状态**：✅ 已完成并验证（2026-10-16）
- **关键约束**：主题 RFC2047 解码，HTML 净化
- **禁止**：手动实现编码转换

---

## 1. 目标

把 IMAP 取回的原始 RFC822 字节，转成结构化、可直接落库与展示的数据。
本任务**不**落库（P0-03）、**不**渲染（P0-06）。

## 2. 任务分解

| 子任务 | 交付物 | 验证 |
| --- | --- | --- |
| P0-02a | `src-tauri/src/mail/parser.rs` 报文解析 | `cargo test --test parser_test` |
| P0-02b | `src-tauri/src/security/sanitize.rs` HTML 净化 | `cargo test --test sanitize_test` |

---

## 3. 接口契约

### 3.1 `mail/parser.rs`

```rust
/// 邮件地址（显示名 + 地址）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailAddress {
    pub name: Option<String>,
    pub address: String,
}

/// 附件元数据。P0 阶段不落附件正文。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedAttachment {
    pub filename: String,
    pub content_type: String,
    pub size: usize,
}

/// 解析结果。时间一律 Unix 整数（DESIGN.md P0-03 约束）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedMessage {
    pub subject: Option<String>,
    pub from: Option<MailAddress>,
    pub to: Vec<MailAddress>,
    pub date_unix: Option<i64>,
    pub message_id: Option<String>,
    pub text_body: Option<String>,
    pub html_body: Option<String>,
    pub attachments: Vec<ParsedAttachment>,
}

/// 解析原始 RFC822 报文。解析器返回 None 时给 `MailError::Parse`。
pub fn parse_message(raw: &[u8]) -> Result<ParsedMessage, MailError>;
```

### 3.2 `security/sanitize.rs`

```rust
/// 净化邮件 HTML，供前端渲染。**唯一**允许产出可渲染 HTML 的入口。
pub fn sanitize_html(html: &str) -> String;
```

---

## 4. 已核实的 API 事实（读 crate 源码得出，勿凭记忆）

| 事实 | 依据 |
| --- | --- |
| `MessageParser::default().parse(raw) -> Option<Message>` | `parsers/message.rs:111` |
| `Message::subject() -> Option<&str>`，取的是已按 RFC2047 解码的文本 | `core/message.rs:380` → `header.as_text()` |
| `Message::date() -> Option<&DateTime>`，`DateTime::to_timestamp() -> i64` | `core/message.rs:160`、`parsers/fields/date.rs:183` |
| **因此不需要 `chrono`** —— 时间转换由 `mail-parser` 自己完成 | 同上 |
| `Message::body_text(pos)` / `body_html(pos)` 的 `pos` 是**正文序号**，不是 part id | `core/message.rs:419,429` |
| 附件文件名取 `content_disposition().attribute("filename")`，回退 `content_type().attribute("name")` | `core/header.rs:559,577` + `ContentType::attribute` `core/header.rs:609` |
| `unknown-8bit` 等非标准编码靠 `encoding_rs` 兜底，需开 `full_encoding` feature | ADR-0001 §4 |

---

## 5. 约束块（要求 5.2）

### 5.1 P0-02a

```
【角色】Rust开发工程师
【任务】实现 parser.rs 的 parse_message
【输入】raw: &[u8]，示例：b"Subject: =?utf-8?B?...?=\r\nFrom: a@b.com\r\n\r\nhi"
【输出】fn parse_message(raw: &[u8]) -> Result<ParsedMessage, MailError>
【约束】
  - 只生成这一个函数 + 其依赖的结构体
  - crate：mail-parser 0.11（full_encoding）
  - 错误处理返回 Result，不 unwrap、不 panic
  - 函数 ≤50 行
  - 不改已有文件
【验收】cargo test 通过，覆盖：纯文本、RFC2047 主题、多收件人、附件、无 Subject、完全非法输入
【禁止】手写 base64 / quoted-printable / 编码字解码；手写日期换算
```

### 5.2 P0-02b

```
【角色】Rust开发工程师
【任务】实现 sanitize.rs 的 sanitize_html
【输入】html: &str，示例 "<p>hi</p><script>alert(1)</script>"
【输出】fn sanitize_html(html: &str) -> String
【约束】
  - 只生成这一个函数
  - crate：ammonia 4
  - 不 panic
  - 不改已有文件
【验收】cargo test 通过：script/onerror/javascript: 被剥离，正常排版标签保留
【禁止】手写正则净化；使用 dangerouslySetInnerHTML
```

---

## 6. 已知取舍（P0 范围内）

- 只取**第一个** text body 与**第一个** html body。含多个正文字段的复杂报文（少见）在 P0 阶段不做合并。
- 附件只解析元数据，不落正文。附件正文属 P2 范围。

## 7. 验收标准

| 检查项 | 标准 |
| --- | --- |
| 编译 | `cargo check` 无错误；`cargo fmt --check` 干净 |
| 单测 | 解析 6 类用例 + 净化 4 类用例全过 |
| 禁止项 | 仓库内无手写 base64/QP/编码字解码 |
| 行数 | 每个函数 ≤50 行 |

---

## 8. 验收结果（实测）

| 检查项 | 实测 |
| --- | --- |
| `cargo check --all-targets` | exit 0 ✅ |
| `cargo fmt --check` | exit 0 ✅ |
| `cargo test --test parser_test` | **9/9 通过** ✅ |
| `cargo test --test sanitize_test` | **5/5 通过** ✅ |
| 函数行数 | `parse_message` 46 行、`sanitize_html` 3 行 ✅ |

### 实施中发现并修正的两点

1. **`message_id()` 会剥掉尖括号**：`<abc@example.com>` 返回 `abc@example.com`。已更正测试预期并记录实际行为。
2. **二进制垃圾不会返回 `MailError::Parse`**：`mail-parser` 走 best-effort，仅在「未解析出任何头部 **且** 消息为空」时返回 `None`（`parsers/message.rs:553-569`）。因此 `Parse` 分支的可达输入是**空字节串**，垃圾输入会得到一条空消息。已分别补两个测试固化该行为。

### 依赖结论

`chrono` 在 P0-02 **不需要** —— `mail_parser::DateTime::to_timestamp()` 已直接给出 Unix 整数。
但 P0-03/P0-04 处理 IMAP `INTERNALDATE` 时可能仍需 `chrono`（`async-imap` 的 `Fetch::internal_date()` 返回 `chrono::DateTime`），故 ADR-0001 基线中保留该项，待实际使用时再引入。
