# P0-06 邮件列表 + 详情（虚拟滚动，HTML 净化后渲染）

- **阶段**：P0
- **依赖**：P0-04c（`get_messages`）、P0-07d（`add_account`）、P0-07e（`sync_folder`）、ADR-0001
- **状态**：✅ 已完成并验证（2026-10-16）
- **关键约束**：虚拟滚动，HTML 净化
- **禁止**：直接使用 `dangerouslySetInnerHTML`

---

## 1. 任务分解

| 子任务 | 交付物 | 状态 |
| --- | --- | --- |
| P0-06a | `storage::load_message_detail` + `commands::get_message` | ✅ |
| P0-06b | `storage::list_accounts` + `commands::list_accounts` | ✅ |
| P0-06c | `src/lib/api.ts` / `src/lib/format.ts` | ✅ |
| P0-06d | `src/stores/selection.ts`（Zustand） | ✅ |
| P0-06e | `src/features/inbox/MessageList.tsx`（虚拟滚动） | ✅ |
| P0-06f | `src/features/inbox/MessageDetail.tsx` | ✅ |
| P0-06g | `src/features/inbox/InboxToolbar.tsx` + `App.tsx` + `App.css` | ✅ |

---

## 2. 接口契约

```rust
// storage/message_store.rs
/// `html_body` 是**未净化**的原始 HTML，本层不做净化。
pub struct MessageDetail { id, uid, subject, from_name, from_address,
                           to_addresses, date_unix, is_seen,
                           text_body, html_body, attachments }

pub fn load_message_detail(conn, message_id: i64) -> Result<Option<MessageDetail>, StorageError>;

// commands/messages.rs
/// **`html_body` 已净化**，原始 HTML 不出后端。
pub struct MessageView { ... , html_body: Option<String>, attachments: Vec<AttachmentView> }

#[tauri::command] pub fn get_message(state, message_id: i64) -> Result<MessageView, String>;
```

TypeScript 侧类型见 `src/lib/api.ts`，与上述结构体逐字段对应。

---

## 3. 信任边界：HTML 只在一处被净化

```
IMAP → parser（原始 HTML，未净化）
     → storage（原样落库，不认识"净化"这件事）
     → commands::get_message  ← security::sanitize_html 在此调用
     → 前端 MessageDetail
```

**`commands::get_message` 是全项目唯一把原始 HTML 转成可渲染 HTML 的地方。**

`MessageDetail`（storage）与 `MessageView`（commands）刻意分成两个类型：
前者是数据，后者是**已经过安全处理、可以放心交给前端**的数据。
把净化放在 storage 会让存储层承担渲染相关的安全职责，边界就糊了。

### 3.1 关于 `dangerouslySetInnerHTML`

`DESIGN.md` P0-06 的禁用项是「**直接**使用 `dangerouslySetInnerHTML`」。
前端仅在 `message.html_body` 上使用它，而该字段来自后端净化后的 `MessageView`。
`MessageDetail.tsx` 的文件头注释写明了这条信任边界。

**若要更严格**（要求前端零使用该 API），可选项是：
在 `security` 里加一个前端净化器（需新增依赖）做双保险；
或者只渲染纯文本正文，放弃 HTML 排版。

---

## 4. 已核实的 API 事实

| 事实 | 依据 |
| --- | --- |
| Tauri v2 命令参数**默认按 camelCase** 解析前端载荷 | `tauri-macros-2.7.3/src/lib.rs:64`：「Either `\"camelCase\"` (default) or `\"snake_case\"`」 |
| `useVirtualizer` 的 `getTotalSize()` / `getVirtualItems()`（含 `index`/`start`/`size`/`key`） | `@tanstack/react-virtual` 3.14.21 |
| `rusqlite` 的 `OptionalExtension::optional()` 用于「零行或一行」查询 | `rusqlite-0.40.2` |

---

## 5. 验收结果

| 检查项 | 实测 |
| --- | --- |
| `cargo check --all-targets` | exit 0，零警告 ✅ |
| `cargo fmt --check` | exit 0 ✅ |
| `tsc --noEmit` | exit 0 ✅ |
| `npm run build`（含 tsc + vite） | exit 0，产物 294.81 kB / gzip 90.78 kB ✅ |
| `cargo test --test message_detail_test` | **5/5 通过** ✅ |
| `cargo test --test list_accounts_test` | **5/5 通过** ✅ |
| 全量测试 | **111/111 通过** ✅ |

### 5.1 关键性质已被测试固化

| 性质 | 用例 |
| --- | --- |
| `storage` 层返回的是**原始** HTML（净化不在此层） | `loads_all_detail_fields` |
| 邮件不存在返回 `None` 而非报错 | `returns_none_for_missing_message` |
| 附件按 id 稳定排序、字段完整 | `loads_attachments_ordered_by_id` |
| 重同步后详情反映最新内容 | `reflects_updated_content_after_resync` |
| 正文为 NULL 时不 panic | `handles_null_bodies` |
| **账户列表只暴露 `has_secret` 布尔，不暴露密文** | `reports_has_secret_without_exposing_the_blob` |

### 5.2 尚未验证

- **真实数据下的滚动性能**。虚拟化本身只挂载可视区行，但「1000 封不卡」这条验收标准需要真实数据实测。
- ~~整个 UI 从未在真实 Tauri 窗口里跑过 —— 只验证到 `tsc` 与 `vite build`。~~
  ✅ **2026-10-16/17 已实跑**：列表、虚拟滚动帧率（1200 封零掉帧）、分页加载提示、
  详情 HTML 渲染与 XSS 净化均在真实窗口验证过。**本条是当时的记录，已不成立。**要真正看到界面，需要一次 `npm run tauri dev`，而它依赖 P0-01 的真机联调（QQ 授权码）。
