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
| P2-01d | `commands::compose::send_mail`（解密凭证 → 投递） | ✅ |
| P2-01e | 写信界面 + 回复/转发 | ✅ |
| P2-01g | 附件选择器（`@tauri-apps/plugin-dialog`） | ✅ |
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
| `cargo test --test compose_test` | **11/11 通过** ✅ |
| 全量测试 | **183/183 通过** ✅ |

### 4.4 命令层：附件为什么传路径而不是字节

`commands::compose::send_mail` 的附件入参是**文件路径**，不是字节数组。

前端传 `Vec<u8>` 会被序列化成 JSON 数字数组（一个字节一个数字），
几 MB 的附件就能把 IPC 打爆。由 Rust 侧读文件既快又省事。

配套约束：

- 合计体积上限 **25 MiB**（国内邮箱普遍限 20–50 MB，留出编码膨胀余量）；
- MIME 类型由**扩展名**推断，猜不到一律 `application/octet-stream` ——
  猜错（比如把 exe 当 image/png）比不猜更糟；
- 显示名缺省取路径文件名，两者都没有就报错，**不编一个假名字出来**。

同样沿用三段式：附件读取（同步 IO）→ 解密凭证（加锁）→ SMTP 投递（不持锁）。

### 4.5 写信界面（P2-01e）

| 文件 | 作用 |
| --- | --- |
| `features/compose/ComposeForm.tsx` | 收件人 / 主题 / 纯文本正文 / HTML 正文 + 发送 |
| `features/compose/drafts.ts` | **纯函数**：`buildReply` / `buildForward` 从已选邮件构造草稿 |
| `stores/selection.ts` | 新增 `compose` 草稿状态与 `openCompose` / `updateCompose` / `closeCompose` |

`drafts.ts` 与组件分开，是为了让「回复/转发怎么拼草稿」这件事**可被单独验证**，
不必挂载 React 组件。回复会引用原文（逐行 `> `），转发会带上原始头信息。

#### 附件选择器（P2-01g）

文件选择必须走 `@tauri-apps/plugin-dialog`：webview 的 `<input type="file">`
只能拿到 `File` 对象、**拿不到真实路径**，而后端按路径读附件。

接入涉及四处，缺一不可：

| 位置 | 内容 |
| --- | --- |
| `package.json` | `@tauri-apps/plugin-dialog` 2.8.3 |
| `Cargo.toml` | `tauri-plugin-dialog` 2.8.3 |
| `src/lib.rs` | `.plugin(tauri_plugin_dialog::init())` |
| `capabilities/default.json` | `permissions` 增加 `"dialog:default"` |

**第 4 项是编译期查不出问题的那一项** —— 权限名写错 `cargo check` 照样过，
要到运行时调用才被拒。故本任务结束时实跑了一次应用确认无权限错误。

已核实：`open<T extends OpenDialogOptions>(options?: T): Promise<OpenDialogReturn<T>>`，
`multiple: true` 时返回 `string[] | null`（`plugin-dialog/dist-js/index.d.ts:309`）。

`AttachmentField.tsx` 独立成组件，让 `ComposeForm` 留在 100 行以内。

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
