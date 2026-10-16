# P0-04 Tauri 命令（sync_folder / get_messages）

- **阶段**：P0
- **依赖**：P0-01（IMAP）、P0-02（解析）、P0-03（存储）、ADR-0001（tauri）
- **状态**：🟨 进行中
- **关键约束**：只做参数校验和转发
- **禁止**：写业务逻辑，返回原始 DB 行

---

## 1. 排序调整（用户 2026-10-16 确认）

原清单的 `add_account` **并入 P0-07**。

原因：`add_account` 必须收密码，而"不存明文密码"要求凭证加密，那是 P0-07 的范围。
按 5.1「不得提前实现后续任务」，P0-04 里写加密是越界；写一个不存密码的 `add_account` 是占位代码。故拆开。

### 1.1 实施中的新发现（需同样处理）

**`sync_folder` 也需要凭证** —— 它要登录 IMAP 才能拉信。所以：

| 命令 | 是否依赖 P0-07 凭证存储 |
| --- | --- |
| `get_messages` | ❌ 纯本地读库，**不依赖** |
| `sync_folder` | ✅ 需要登录密码，**依赖** |
| `add_account` | ✅ 需要落密码，**依赖** |

因此 P0-04 在 P0-07 完成前只能交付 `get_messages`。`sync_folder` 与 `add_account` 一并在 P0-07 之后完成。
这**不是**降级交付：`get_messages` + P0-05/P0-06 已足够跑通「界面显示本地邮件」这条链路。

---

## 2. 任务分解

| 子任务 | 交付物 | 状态 |
| --- | --- | --- |
| P0-04a | Tauri 脚手架（`build.rs` / `tauri.conf.json` / `main.rs` / `run()` / 图标 / capabilities） | ✅ |
| P0-04b | `storage::list_messages` 分页查询 + `MessageSummary` DTO | ✅ |
| P0-04c | `commands::messages::get_messages` 命令 + 分页参数校验 | ✅ |
| P0-04d | `commands::sync::sync_folder` 命令 | ⛔ 待 P0-07 |

---

## 3. 接口契约

### 3.1 `storage::message_store`

```rust
/// 列表用摘要。不是数据库行。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageSummary {
    pub id: i64,
    pub uid: i64,
    pub subject: Option<String>,
    pub from_name: Option<String>,
    pub from_address: Option<String>,
    pub date_unix: i64,           // Unix 秒
    pub is_seen: bool,
    pub attachment_count: i64,
}

pub fn list_messages(
    conn: &Connection,
    folder_id: i64,
    limit: u32,
    offset: u32,
) -> Result<Vec<MessageSummary>, StorageError>;
```

### 3.2 `commands/`

```rust
/// 命令层只做参数校验 + 转发，不含业务逻辑。
#[tauri::command]
pub async fn get_messages(
    state: tauri::State<'_, AppState>,
    folder_id: i64,
    limit: u32,
    offset: u32,
) -> Result<Vec<MessageSummary>, String>;
```

`limit` 需要上限保护（防止前端传 `u32::MAX` 把内存打爆），这是**参数校验**职责范围内的事。

---

## 4. 已核实的 API 事实（读 crate 源码得出）

| 事实 | 依据 |
| --- | --- |
| `tauri_build::build()` 在 `build.rs` 中调用 | `tauri-build-2.7.3/src/lib.rs:634` |
| `tauri::Builder::invoke_handler` / `run(context)` / `manage` / `setup` | `tauri-2.12.4/src/app.rs:1723,1838,2055,2616` |
| `Config` 必填字段只有 `identifier`；`app`/`build`/`bundle`/`plugins` 均有 `#[serde(default)]` | `tauri-utils-2.10.2/src/config.rs:4130-4142` |
| `BuildConfig.frontendDist` 是 `FrontendDist`（untagged：URL / 目录 / 文件数组），**编译期必须真实存在** | `config.rs:3862`、`enum FrontendDist` |
| `tauri-build` 在 Windows 上**强制要求** `icons/icon.ico` 生成资源文件，缺失即构建失败 | 实测报错：`icons/icon.ico not found` |
| `Config.identifier` 默认值 `com.tauri.dev` 会被 `tauri build` 拒绝 | `config.rs:1271` 文档 |

### 4.1 踩坑记录

1. **`[lib] name = "dp_gmail_lib"` 会改掉库 crate 名**，导致全部 `use dp_gmail::…` 的集成测试失败（7 个测试文件同时报 `E0433`）。
   且 `crate-type = ["staticlib", "cdylib"]` 是**移动端**才需要的 —— 本要求明确"不做移动端"。
   故直接删掉整个 `[lib]` 段，库名回落为 `dp_gmail`。
2. **`frontendDist` 指向的目录必须先存在**，`generate_context!` 在编译期校验。故先跑 `npm run build` 产出 `dist/`。

---

## 5. 验收标准

| 检查项 | 标准 |
| --- | --- |
| 编译 | `cargo check --all-targets` 无错误 |
| 前端 | `tsc --noEmit` 无错误 |
| 格式 | `cargo fmt --check` 干净 |
| 单测 | `list_messages` 分页不重不漏、排序稳定、附件计数正确 |
| 分层 | `commands/` 内无 SQL、无 IMAP 调用，只有校验与转发 |

---

## 6. 验收结果（截至本轮）

| 检查项 | 实测 |
| --- | --- |
| `cargo check --all-targets` | exit 0 ✅ |
| `cargo fmt --check` | exit 0 ✅ |
| `npm run build`（含 `tsc --noEmit`） | exit 0 ✅ |
| `cargo test --test list_messages_test` | **6/6 通过** ✅ |
| `cargo test --test commands_test` | **4/4 通过** ✅ |
| 累计（全部测试） | **53/53 通过** ✅ |

### 6.1 P0-04c 实施要点

- `get_messages` 是**同步**命令。SQLite 带 `LIMIT` 的本地查询是微秒级，上异步只会引入 `MutexGuard` 跨 `await` 的 `Send` 问题，没有收益。
- 参数校验收敛为一个纯函数 `normalize_paging(limit, offset) -> (u32, u32)`，把 `limit` 钳到 `1..=200`。这样命令层**唯一有逻辑的地方**可以被独立测试。
- `MessageSummary` 直接 `derive(serde::Serialize)`，避免在 `commands/` 再复制一份逐字段等同的 DTO（那是过度设计）。
- 数据库路径：`app.path().app_data_dir()?` + `dp_gmail.sqlite`，`setup` 阶段建目录、开库、`app.manage(AppState::new(conn))`。

### 6.2 已解决：P0-06「详情」缺少取正文的命令

P0-04 的命令清单是 `add_account / sync_folder / get_messages` 三个。
但 P0-06 要求「邮件列表 **+ 详情**」，而 `get_messages` 只返回摘要（不含 `text_body` / `html_body`）。

**最终结论：新增了 `get_message` 命令**（当时按第六章提交用户确认，已获批准并实现）。
没有让 `get_messages` 连带返回正文 —— 那会让列表页白白搬运大量正文，
违背摘要 DTO 的初衷。

---

## 7. 尚未验证

| 项 | 说明 |
| --- | --- |
| **IPC 往返** | 命令的**参数名映射**（Tauri v2 默认 camelCase）只在实跑中观察到过零星几次，**没有系统验证**。单测直接调函数，绕过不了 serde 与 IPC 层。 |
| 全部命令的错误分支 | 测试覆盖的是成功路径与参数校验；数据库异常、账户不存在等分支未逐一验证。 |
| 前端错误呈现 | 后端返回的 `Err(String)` 在界面上的显示效果，只在个别场景看过。 |
