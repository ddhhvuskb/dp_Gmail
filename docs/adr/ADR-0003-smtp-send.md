# ADR-0003：SMTP 发信采用 lettre

- **状态**：Accepted（2026-10-16，依据当晚用户授权自行决断，事后留痕）
- **依据**：`DESIGN.md` P2「写信/回复/转发 + SMTP + 附件」；`AGENTS.md`「授权例外」第 3 条

---

## 1. 背景

P2 需要 SMTP 发信能力。`DESIGN.md` 锁定了技术栈表，但未指定 SMTP 实现方式；
ADR-0001 的依赖基线里也没有发信相关的 crate。

按授权边界第 3 条：「任何突破 `DESIGN.md` 的决定必须新开 ADR 记录，
免的是事前请示，不是事后留痕。」故补此记录，供用户事后复核。

---

## 2. 决策

**采用 `lettre` 0.11（`tokio1-native-tls` feature），不手写 SMTP。**

```toml
lettre = { version = "0.11", features = ["tokio1-native-tls"] }
```

### 2.1 为什么是 lettre

| 维度 | 说明 |
| --- | --- |
| 生态地位 | Rust 生态事实标准的邮件发送库，被大量生产项目使用 |
| 覆盖面 | EHLO / STARTTLS / AUTH（含 CRAM-MD5、XOAUTH2 等）/ 连接池 / MIME 组装 / 附件，全部现成 |
| 与现有选型一致 | `tokio1-native-tls` 复用 ADR-0001 已定的 native-tls 路线（Windows 走 SChannel），不引入第二套 TLS 实现 |
| 与 IMAP 侧对称 | IMAP 用 `async-imap` 而非手写；SMTP 同样应交给成熟实现 |

### 2.2 为什么不手写

SMTP 看起来简单（`EHLO` → `AUTH` → `MAIL FROM` → `RCPT TO` → `DATA`），实际边界情况极多：

- 服务器扩展能力协商（`EHLO` 响应解析、`AUTH` 机制选择）；
- 隐式 TLS（465）与 STARTTLS（587）两条路径；
- 行长度限制与点填充（dot-stuffing）；
- MIME 组装：多部分边界、RFC2047 编码字、RFC2231 长文件名、Content-Transfer-Encoding 选择。

国内邮箱（QQ / 163 / 阿里）在这些细节上各有脾气。手写等于把上述每一条都变成自己的 bug 来源，
且**没有测试手段**（不可能对每个服务商都写一遍协议测试）。

这与 `AGENTS.md` 禁止项「手动实现编码转换」是同一条理由：
**协议与编码这类有成熟实现的东西，不要自己写。**

---

## 3. feature 选择

lettre 0.11 的默认 feature 为 `smtp-transport, pool, native-tls, hostname, builder`。

本 ADR **不关闭默认 feature**，只追加 `tokio1-native-tls`。理由：

- `smtp-transport` / `builder` / `hostname` 是发信必需；
- `pool` 提供连接复用，多封信连发时省一次 TLS 握手，留着无害；
- `native-tls` 是 TLS 后端，与 IMAP 侧一致；
- `tokio1-native-tls` 把 lettre 接入现有 tokio 运行时，并给出 `AsyncSmtpTransport<Tokio1Executor>`。

---

## 4. 已核实的 API（读 `lettre-0.11.23` 源码得出）

| 事实 | 依据 |
| --- | --- |
| `AsyncSmtpTransport::<Tokio1Executor>::relay(host)` 建隐式 TLS 传输 | `transport/smtp/async_transport.rs:135` |
| `starttls_relay(host)` 建 STARTTLS 传输 | 同文件 `:169` |
| builder 链：`credentials` / `port` / `timeout` / `build` | 同文件 `:381`、`:404`、`:410`、`:456` |
| `AsyncTransport::send(&self, Message)` 投递 | `transport/mod.rs:153` |
| `Message::builder() -> MessageBuilder`；`from`/`to`/`subject`/`body` | `message/mod.rs:503`、`:255`、`:295`、`:328`、`:458` |
| `Credentials::new(String, String)` | `transport/smtp/authentication.rs:22` |

**这些是读源码得到的，不是凭记忆写的。** 结果是一次编译通过、零警告。

---

## 5. 后果与边界

- 依赖树新增 lettre 及其传递依赖（`email-encoding`、`email_address`、`quoted_printable`、`httpdate` 等）。
- 本 ADR **只覆盖发信**。收信仍走 `async-imap`，两者互不替代。
- **当前实现范围**：纯文本正文、单账户、隐式 TLS（465）。
  HTML 正文（multipart/alternative）、附件、STARTTLS 属后续任务，不在本 ADR 承诺范围内。
- 凭证仍由调用方从 `security` 解密后以 `&str` 传入，**lettre 不接触密钥文件**。

---

## 6. 未采纳的方案

| 方案 | 未采纳理由 |
| --- | --- |
| 手写 SMTP over `async-native-tls` | 复用现有依赖、零新增，但见 §2.2 的边界情况清单。省一个依赖换一堆协议 bug，不划算。 |
| `mail-send` 等其他 crate | 生态活跃度与覆盖面均不如 lettre，且同样要新增依赖。 |
| 直接调外部程序（sendmail / msmtp） | 要求用户预装外部程序，与「开箱即用」的目标冲突；Windows 上也没有 sendmail。 |
