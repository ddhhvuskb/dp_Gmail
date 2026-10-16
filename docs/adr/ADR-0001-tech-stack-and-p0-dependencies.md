# ADR-0001：技术栈锁定与 P0 依赖基线

- **状态**：Accepted（用户 2026-10-16 授权出本 ADR；带 ⚠️ 的增删项见 §5，需用户过目）
- **日期**：2026-10-16
- **决策人**：用户（批准）/ AI 工程师（起草）
- **依据**：`DESIGN.md` 第二章（技术栈锁定）、第六章（新增依赖必须人工批准）

---

## 1. 背景

`DESIGN.md` 第二章锁定技术栈，但未给出 crate 级清单与 feature 标志。要求同时规定：

- 新依赖必须在 `Cargo.toml` 已声明（防"架构漂移"）；
- 新依赖必须人工批准并写入 `docs/adr/`；
- 不得自行做技术选型。

因此需要一份可执行的依赖基线，作为后续所有任务的唯一依赖来源。

---

## 2. 决策：技术栈确认（不变更）

| 层级 | 选型 | 版本 |
| --- | --- | --- |
| 框架 | Tauri | v2 |
| 后端 | Rust stable | MSVC toolchain |
| 前端 | React + TypeScript | React 19+ |
| 状态 | Zustand + TanStack Query | ≥5.0 |
| 数据库 | SQLite (rusqlite) + FTS5 | bundled |
| 构建 | GitHub Actions 矩阵 | Windows x64 / ARM64 |

---

## 3. 决策：TLS 路线 = native-tls（Windows 走 SChannel）

用户 2026-10-16 选定 **native-tls**，不选 rustls。

理由：复用系统证书链，国内邮箱（QQ/163/阿里）兼容性最稳，无需随包分发根证书。

对应实施：`async-native-tls` 作为 `async-imap` 的 TLS 适配器 —— 这是 `async-imap` 官方文档示例使用的写法。

---

## 4. 决策：P0 依赖基线（后端）

| crate | 版本 | features | 用途 | 对应任务 |
| --- | --- | --- | --- | --- |
| `tauri` | 2 | — | 框架 / 命令桥 | P0-04 |
| `tauri-build` | 2 | — | `build.rs` 构建脚本 | 脚手架 |
| `serde` | 1 | `derive` | DTO 序列化 | P0-04 |
| `serde_json` | 1 | — | JSON 载荷 | P0-04 |
| `thiserror` | 2 | — | 模块级错误枚举 | 全局 |
| `anyhow` | 1 | — | 应用边界错误 | 全局 |
| `tracing` | 0.1 | — | 结构化日志 | 全局 |
| `tokio` | 1 | `rt-multi-thread`, `macros`, `net`, `time`, `io-util` | 异步运行时 | P0-01 |
| `async-imap` | 0.12 | `runtime-tokio`，**`default-features = false`** | IMAP 协议 | P0-01 |
| `async-native-tls` | 0.6 | `runtime-tokio`，**`default-features = false`** | TLS 流 | P0-01 |
| `futures` | 0.3 | — | `StreamExt`（消费 `fetch`/`list`） | P0-01 |
| `mail-parser` | 0.11 | `full_encoding` | MIME / RFC2047 解析 | P0-02 |
| `rusqlite` | 0.40 | `bundled` | SQLite + FTS5 | P0-03 |
| `ammonia` | 4 | — | HTML 净化 | P0-02 / P0-06 |
| `chrono` | 0.4 | — | 邮件日期 → Unix 整数 | P0-02 / P0-03 |
| `aes-gcm` | 0.11 | — | AES-256-GCM | P0-07 |
| `zeroize` | 1 | — | 密钥内存擦除 | P0-07 |
| `rand` | 0.10 | — | nonce 生成 | P0-07 |
| `base64` | 0.23 | — | 密钥文件编解码 | P0-07 |

### 4.1 关键陷阱：两个 crate 的默认 feature 是错的运行时

实测 crates.io 元数据：

- `async-imap` → `default = ["runtime-async-std"]`
- `async-native-tls` → `default = ["runtime-smol"]`

本项目用 tokio。若漏写 `default-features = false`，会同时拉入 async-std/smol 运行时并导致类型不匹配。**两者都必须显式关闭默认 feature 并开启 `runtime-tokio`。**

### 4.2 FTS5 说明

`rusqlite` 的 `bundled` feature 会在本地编译 SQLite，其 FTS5 扩展默认已启用，不需要额外 crate。P1 建 FTS5 虚表时直接写 SQL 即可。

### 4.3 P0-01 实施中核实到的 API 事实（推翻网上资料）

以下三条不是推测，是读 `~/.cargo/registry/src/` 下真实源码 + 编译器实测得出的结论。**docs.rs 上的示例与事实不符，不得照抄。**

| 事实 | 依据 | 影响 |
| --- | --- | --- |
| `async-imap` 0.12 **不存在 `async_imap::connect`**。 | 其 `src/lib.rs` 全文无此函数，`grep 'fn connect'` 零命中。docs.rs `Client::login` 页里那段 `async_imap::connect(...)` 示例是**过时代码**（原文标注 `ⓘ This example is not tested`）。 | 必须自己建 TCP+TLS 流，再 `Client::new(stream)`。 |
| 0.12 不再自动消费服务器问候语。 | `src/lib.rs` 顶部文档：「Then read the server greeting: `client.read_response().await?...`」 | 连接后必须显式 `read_response()` 读问候语，否则登录会错位。 |
| `runtime-tokio` 下 `async-imap` 用的是 **tokio 的 io trait**，不是 futures-io。 | `src/client.rs` 第 11 行：`use tokio::io::{AsyncRead as Read, AsyncWrite as Write, AsyncWriteExt};`（`imap_stream.rs` 同）。 | **不需要 `tokio-util` 的 `Compat` 桥接。** `async-native-tls` 在 `runtime-tokio` 下产出的正是一个 tokio 流，两者直接对接。若误加 `Compat`，会得到 35 个 `E0277`，报错文本还会指向 `futures_io`，极具误导性。 |

**因此 §4 基线中的 `tokio-util` 不是必需依赖**，实际未引入。`futures` 仍然保留，供后续消费 `fetch`/`list` 返回的 `Stream`。

---

## 5. ⚠️ 相对用户提案的增删项（需人工过目）

用户批准的原提案：`tauri / tokio / async-imap / mail-parser / rusqlite / serde / serde_json / aes-gcm / argon2 或 hkdf / keyring / thiserror / anyhow / tracing`。

### 5.1 删除项

| crate | 删除理由 |
| --- | --- |
| `keyring` | P0-07 明确写的是"**密钥文件**管理"，不是 OS 凭据库。两者是不同方案，按原文取密钥文件。 |
| `argon2` / `hkdf` | 需要 KDF 的前提是"用户口令派生密钥"。P0-07 的密钥来源是密钥文件，不存在口令派生环节，引入即多余依赖。 |

### 5.2 新增项

| crate | 新增理由 |
| --- | --- |
| `async-native-tls` | `async-imap` 自带 TLS 之外的适配层由其官方文档示例指定；用户已选 native-tls 路线，此项是该路线的必然组成。 |
| `futures` | `async-imap` 的 `fetch`/`list`/`uid_fetch` 返回 `Stream`，消费必须依赖 `StreamExt`。 |
| `ammonia` | P0-02 / P0-06 强制"HTML 净化"。净化器属安全代码，不得自行实现（见 `AGENTS.md` 禁止项）。这是 Rust 生态事实标准实现。 |
| `zeroize` | P0-07 要求密钥不残留内存。 |
| `rand` | P0-07 生成 AES-GCM nonce，不得自造随机源。 |
| `base64` | P0-07 密钥文件编解码。`async-imap` 已传递依赖此版本，不额外扩大依赖树。 |
| `chrono` | P0-03 要求"时间用 Unix 整数"。P0-02 实测：邮件 `Date` 头由 `mail_parser::DateTime::to_timestamp()` 直接给出，**不需要** chrono；但 P0-04 处理 IMAP `INTERNALDATE` 时 `async-imap` 的 `Fetch::internal_date()` 返回 `chrono::DateTime`，届时需要。故保留在本基线，实际使用到才引入。 |
| `tauri-build` | Tauri v2 脚手架的 `build.rs` 必需，非可选。 |

### 5.3 用户若否决

删除 `argon2`/`hkdf`/`keyring`、新增上述 8 项，均为 `Cargo.toml` 内的局部改动，不触及任何接口签名与表结构。用户可随时否决任意一项，届时按 `DESIGN.md` 第六章流程重走本 ADR。

---

## 6. 决策：前端依赖基线

| 包 | 版本 | 用途 |
| --- | --- | --- |
| `react` / `react-dom` | 19 | UI |
| `typescript` | 5.x | 类型检查（`tsc --noEmit`） |
| `vite` | 7.x | 构建 |
| `@vitejs/plugin-react` | latest | Vite React 插件 |
| `zustand` | ≥5.0 | 客户端状态 |
| `@tanstack/react-query` | ≥5.0 | 服务端状态 / 缓存 |
| `@tauri-apps/api` | 2 | 调用 Tauri Commands |
| `@tauri-apps/cli` | 2 | 开发/构建 CLI |

---

## 7. 后果

- 所有后续任务的 `Cargo.toml` / `package.json` 变更必须回到本 ADR 追加条目，不得直接改依赖。
- 本 ADR 未列出的 crate，一律视为"未声明依赖"，不得 `use`。
- native-tls 在 Windows 上依赖系统 SChannel，无需额外 C 库；跨平台构建时 Linux/macOS 走 OpenSSL/Secure Transport。

## 8. 变更流程

新增依赖 → 用户确认 → 本文件追加条目（新开 ADR-000N 亦可）→ 才允许写入 `Cargo.toml` / `package.json`。
