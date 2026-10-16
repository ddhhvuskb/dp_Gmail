# P2-01 SMTP 发信核心

- **阶段**：P2（写信/回复/转发 + SMTP + 附件）
- **依赖**：P0-01（`ProviderConfig` 的 SMTP 字段）、P0-07（凭证解密）、ADR-0003
- **状态**：🟨 P2-01a 完成（纯文本发信）；HTML 正文、附件、界面待做
- **关键约束**：`DESIGN.md` P2 范围

---

## 1. 任务分解

| 子任务 | 交付物 | 状态 |
| --- | --- | --- |
| P2-01a | `mail/smtp.rs`：`send_mail` + `build_message`（纯文本 / 隐式 TLS / 30s） | ✅ |
| P2-01b | HTML 正文（`multipart/alternative`） | ✅ |
| P2-01c | 附件（`MultiPart::mixed`） | ✅ |
| P2-01d | `commands::send_mail`（解密凭证 → 投递） | ⬜ |
| P2-01e | 写信/回复/转发界面 | ⬜ |
| P2-01f | STARTTLS（587）支持 | ⬜ |

## 2. 接口契约

```rust
// mail/smtp.rs
pub const SMTP_TIMEOUT_SECS: u64 = 30;

pub struct OutgoingMail {
    pub from: String,        // "张三 <a@qq.com>" 或裸地址
    pub to: Vec<String>,
    pub subject: String,
    pub body_text: String,
    /// 提供时以 multipart/alternative 同时发送纯文本与 HTML。
    pub body_html: Option<String>,
}

/// 凭证只在内存中传递，不得被日志记录。
pub async fn send_mail(
    cfg: &ProviderConfig,
    user: &str,
    secret: &str,
    mail: &OutgoingMail,
) -> Result<(), MailError>;

/// 单独暴露以便脱离网络验证 —— 组装结果不该因为 send_mail 会建连而失去覆盖。
pub fn build_message(mail: &OutgoingMail) -> Result<Message, MailError>;
```

新增 `MailError` 变体：`NoRecipients`、`BadAddress(String)`、`BuildMail`（lettre 组装错误）、`Smtp`（lettre 传输错误）。

---

## 3. 设计决定

### 3.1 校验一律排在建连之前

`send_mail` 的顺序是：**校验 TLS 模式 → 校验收件人非空 → 解析全部地址 → 组报文 → 才建传输**。

这不是随手写的顺序，而是有测试钉住的：若把地址解析挪到建连之后，
非法地址会先白白建立一次 TLS 连接再失败。`smtp_test` 里两条
`*_before_touching_the_network` 用例就是为此 —— 它们都在 0.00s 内返回，
一旦顺序被改就会去真连 `smtp.qq.com` 而变慢甚至超时。

### 3.2 超时常量与其他模块对齐

`SMTP_TIMEOUT_SECS` 与 `IMAP_TIMEOUT_SECS` 都是 30，且有一条测试断言两者相等 ——
避免日后一处改了另一处忘。

### 3.3 服务器地址仍只来自 `provider.rs`

`send_mail` 收 `&ProviderConfig`，host / port / TLS 模式全部从中读。
本模块**不出现任何地址字面量**，与 `AGENTS.md` 禁止项一致。

---

## 4. 验收结果

| 检查项 | 实测 |
| --- | --- |
| `cargo check --all-targets` | exit 0，零警告 ✅ |
| `cargo fmt --check` / `cargo clippy -D warnings` | exit 0 ✅ |
| `cargo test --test smtp_send_test` | **18/18 通过**（0.00s，未触网）✅ |
| 全量测试 | **172/172 通过** ✅ |

### 4.1 测试为何只覆盖"网络之前"

真正的投递需要真实 SMTP 服务器。在单测里连 `smtp.qq.com` 既慢又没意义，
而且会把测试变成对外部服务的依赖。

但**报文组装不依赖网络**，那部分必须测。故把 `build_message` 单独公开出来，
用 `Message::formatted()` 取到原始 RFC5322 字节直接断言：

| 性质 | 用例 |
| --- | --- |
| 纯文本邮件**不**套 multipart（单部分兼容性最好） | `plain_only_yields_single_part_message` |
| 有 HTML 时产出 `multipart/alternative`，**且纯文本那一份仍在** | `html_yields_multipart_alternative_carrying_both_parts` |
| multipart 必须声明 `boundary=` | `multipart_declares_a_boundary` |
| 组装阶段就拒绝非法收发件人 | `build_message_rejects_bad_*` |

> 断言一律用 **ASCII 标记**：中文主题会被 RFC2047 编码成 `=?utf-8?…?=`，
> 直接在原始报文里找不到原字 —— 用中文断言只会得到假失败。

### 4.2 附件：一个必须避开的坑

`Attachment::new(filename)` 会设 `Disposition::Attached` —— 也就是说
**拿它构造正文部分，客户端会把正文当附件显示**（源码 `message/attachment.rs:50`）。
正文必须用 `SinglePart::builder().header(ContentType::TEXT_PLAIN)`。

这一点有专门用例 `body_part_is_not_marked_as_attachment` 钉住：
断言整个报文里 `Content-Disposition` **只出现一次**（就是那个真附件）。

结构按「最小够用」选择：

| 正文 | 附件 | 结构 |
| --- | --- | --- |
| 纯文本 | 无 | 单部分 |
| 纯文本 + HTML | 无 | `multipart/alternative` |
| 任意 | 有 | `multipart/mixed` 外层，正文按上表决定内层 |

每条分支都有用例：`attachment_wraps_message_in_multipart_mixed`、
`html_plus_attachment_nests_alternative_inside_mixed`、
`plain_text_attachment_does_not_force_html`（纯文本附件不该凭空造出 alternative）、
`attachment_body_still_present_alongside_attachment`（加附件不能把正文弄丢）、
`supports_multiple_attachments`、`rejects_invalid_attachment_content_type`。

### 4.3 本轮两处工程教训

1. **不要用 PowerShell 5.1 的 `Set-Content -Encoding UTF8` 改 UTF-8 源码。**
   我用它给 `smtp_test.rs` 加一行 import，结果整个文件的中文变成乱码，编译直接失败。
   改源码一律用 `edit` / `write` 工具。文件已重写为 `smtp_send_test.rs`。
2. **`Match` 的可见性决定了可测性。** 原本 `build_message` 是私有的，
   于是 HTML 路径一行都测不到 —— 测试全绿，但新功能零覆盖。把它公开后才补上真正的验证。

---

## 5. 尚未验证

- **`send_mail` 从未真的发出去过一封信。** TLS 握手、AUTH、DATA 三段**完全未验证**。
  已覆盖的是：参数校验、校验顺序、MIME 组装。
- 附件尚未实现。
- STARTTLS（587）尚未实现。
