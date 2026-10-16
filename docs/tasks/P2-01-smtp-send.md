# P2-01 SMTP 发信核心

- **阶段**：P2（写信/回复/转发 + SMTP + 附件）
- **依赖**：P0-01（`ProviderConfig` 的 SMTP 字段）、P0-07（凭证解密）、ADR-0003
- **状态**：🟨 P2-01a 完成（纯文本发信）；HTML 正文、附件、界面待做
- **关键约束**：`DESIGN.md` P2 范围

---

## 1. 任务分解

| 子任务 | 交付物 | 状态 |
| --- | --- | --- |
| P2-01a | `mail/smtp.rs`：`send_mail`（纯文本、隐式 TLS、30s 超时） | ✅ |
| P2-01b | HTML 正文（multipart/alternative） | ⬜ |
| P2-01c | 附件（`MultiPart::mixed` + RFC2231 长文件名） | ⬜ |
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
}

/// 凭证只在内存中传递，不得被日志记录。
pub async fn send_mail(
    cfg: &ProviderConfig,
    user: &str,
    secret: &str,
    mail: &OutgoingMail,
) -> Result<(), MailError>;
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
| `cargo fmt --check` | exit 0 ✅ |
| `cargo test --test smtp_test` | **6/6 通过**（耗时 0.00s，未触网）✅ |
| 全量测试 | 待本轮收尾统一跑 |

### 4.1 测试为何只覆盖"网络之前"

真正的投递需要真实 SMTP 服务器。在单测里连 `smtp.qq.com` 既慢又没意义，
而且会把测试变成对外部服务的依赖。故本轮只测参数校验路径，
**真实投递必须等 QQ 授权码到位后做端到端联调**。

这一限制已明确记录，不含糊过去。

---

## 5. 尚未验证

- **`send_mail` 从未真的发出去过一封信。** 全部路径都止步于建连之前。
  真实投递（TLS 握手、AUTH、DATA）**完全未验证**。
- HTML 正文与附件尚未实现。
