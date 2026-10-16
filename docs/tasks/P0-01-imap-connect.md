# P0-01 IMAP 连接（async-imap，TLS，超时 30s）

- **阶段**：P0
- **依赖**：ADR-0001（依赖基线）
- **状态**：✅ 已完成并验证（2026-10-16）
- **关键约束**：必须支持 TLS，超时 30s
- **禁止**：硬编码服务器地址

---

## 1. 目标

建立一条可复用的「域名 → 服务商配置 → TLS 连接 → 登录」链路，供后续 `sync_folder` 使用。
本任务**不**做邮件拉取、**不**做 MIME 解析、**不**落库。

## 2. 任务分解

要求 5.1 规定一次只生成一个函数。P0-01 拆为两个子任务，逐个生成、逐个验证：

| 子任务 | 交付物 | 验证 |
| --- | --- | --- |
| P0-01a | `src-tauri/src/mail/provider.rs` 服务商识别 | `cargo test mail::provider` |
| P0-01b | `src-tauri/src/mail/imap_client.rs` TLS 连接 + 30s 超时 | `cargo check` + `cargo test mail::imap_client` |

---

## 3. 接口契约（跨模块调用以此为准）

### 3.1 `mail/provider.rs`

```rust
/// TLS 连接模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsMode {
    /// 993 端口，连上即 TLS
    Implicit,
    /// 143 端口，明文连接后 STARTTLS 升级
    StartTls,
}

/// 单个服务商的连接参数。所有服务器地址只允许出现在本表内。
#[derive(Debug, Clone, Copy)]
pub struct ProviderConfig {
    pub id: &'static str,                   // "qq"
    pub display_name: &'static str,         // "QQ邮箱"
    pub domains: &'static [&'static str],   // ["qq.com", "vip.qq.com", "foxmail.com"]
    pub imap_host: &'static str,
    pub imap_port: u16,
    pub imap_tls: TlsMode,
    pub smtp_host: &'static str,
    pub smtp_port: u16,
    pub smtp_tls: TlsMode,
}

/// 由邮箱地址解析服务商配置。
///
/// 输入：`user@domain` 形式的邮箱地址（大小写不敏感，允许首尾空白）
/// 输出：命中则 `Some(&'static ProviderConfig)`，未收录则 `None`（交给用户手填）
pub fn resolve_by_email(email: &str) -> Option<&'static ProviderConfig>;
```

### 3.2 `mail/imap_client.rs`

```rust
/// 已登录的 IMAP 会话别名。具体类型参数以 P0-01b 实测编译结果为准，禁止猜测。
pub type ImapSession = async_imap::Session<...>;

/// 在 30s 超时内建立 TLS 连接并登录。
///
/// 输入：
///   - `cfg`:    &ProviderConfig，服务器地址唯一来源
///   - `user`:   完整邮箱地址，如 "12345678@qq.com"
///   - `secret`: 授权码/密码，仅在内存中传递
/// 输出：Ok(ImapSession) / Err(MailError)
pub async fn connect_and_login(
    cfg: &ProviderConfig,
    user: &str,
    secret: &str,
) -> Result<ImapSession, MailError>;
```

### 3.3 `mail/error.rs`

```rust
#[derive(Debug, thiserror::Error)]
pub enum MailError {
    #[error("连接超时（{0}s）")]
    Timeout(u64),
    #[error("不支持的邮箱服务商：{0}")]
    UnknownProvider(String),
    #[error("TLS 错误：{0}")]
    Tls(#[from] async_native_tls::Error),
    #[error("IMAP 协议错误：{0}")]
    Imap(#[from] async_imap::error::Error),
    #[error("网络错误：{0}")]
    Io(#[from] std::io::Error),
}
```

---

## 4. 约束块（要求 5.2，逐函数套用）

### 4.1 P0-01a

```
【角色】Rust开发工程师
【任务】实现 provider.rs 的服务商静态表与 resolve_by_email
【输入】email: &str，示例 "12345678@qq.com"、"user@vip.qq.com"、"a@unknown.tld"
【输出】fn resolve_by_email(email: &str) -> Option<&'static ProviderConfig>
【约束】
  - 只生成这一个函数 + 其依赖的结构体/枚举
  - 使用 std，不引入新 crate
  - 函数 ≤50 行
  - 错误处理：未命中返回 None，不 panic
  - 不改已有文件
【验收】cargo test 通过，覆盖：正常域名、子域名、大小写、首尾空白、无 @、空串、未收录域名
【禁止】在 imap_client.rs 或任何其他文件出现字面量服务器地址
```

### 4.2 P0-01b

```
【角色】Rust开发工程师
【任务】实现 imap_client.rs 的 connect_and_login
【输入】cfg: &ProviderConfig, user: &str, secret: &str
【输出】async fn connect_and_login(...) -> Result<ImapSession, MailError>
【约束】
  - 只生成这一个函数
  - crate：async-imap 0.12（default-features=false, runtime-tokio）、async-native-tls 0.6（default-features=false, runtime-tokio）、tokio
  - 错误处理返回 Result，不 unwrap
  - 函数 ≤50 行
  - 超时固定 30s，来源于 DESIGN.md P0-01 约束
  - 不改已有文件
【验收】cargo check 无错误；cargo test 用本地假 IMAP 服务验证超时分支
【禁止】硬编码 host/port；把 secret 写进日志或 Debug 输出
```

---

## 5. 安全要求

- `secret` 只以 `&str` 传入，**不得**被 `tracing` 记录，**不得**派生 `Debug`。
- 连接参数全部来自 `ProviderConfig`。

## 6. 验收标准

| 检查项 | 标准 |
| --- | --- |
| 编译 | `cargo check` 无错误 |
| 单测 | 服务商识别 7 个用例全过；超时分支有测试 |
| 禁止项 | 全仓库 `grep` 不到硬编码的 `imap.` / `smtp.` 字面量（provider.rs 表内除外） |
| 行数 | 每个函数 ≤50 行 |
