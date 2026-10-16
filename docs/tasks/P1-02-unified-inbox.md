# P1-02 多账户 + 统一收件箱

- **阶段**：P1
- **依赖**：P0-03（表结构）、P1-01（FTS5，无直接依赖但同属 P1）
- **状态**：🟨 后端完成（前端待 P1-03）
- **关键约束**：跨模块调用走已定义接口

---

## 1. 交付物（后端）

| 层 | 内容 |
| --- | --- |
| `storage/folder_store.rs` | `FolderRecord` + `list_folders(conn, account_id)` |
| `storage/message_store.rs` | `UnifiedMessageSummary` + `list_unified_messages(conn, account_id, mailbox, limit, offset)` |
| `commands/folders.rs` | `list_folders` 命令 |
| `commands/messages.rs` | `list_unified_messages` 命令 |

前端（文件夹树、统一收件箱视图）留到 P1-03。

---

## 2. 设计决定

### 2.1 为什么新建 `UnifiedMessageSummary` 而不改 `MessageSummary`

跨文件夹聚合时，「这封信属于哪个账户 / 哪个文件夹」是**必需信息** ——
列表里要能一眼看出「这是 B 账户 INBOX 的信」。

`MessageSummary` 里没有这些字段。加进去会污染单文件夹视图的语义，
也会让既有的 `list_messages` 白读一堆 JOIN。

故单独一个 DTO。**不改既有结构**是 `AGENTS.md`「改前先读当前内容、不覆盖已有代码」的直接要求。

### 2.2 两个可选过滤条件，一条 SQL 覆盖四种组合

```sql
WHERE (?1 IS NULL OR f.account_id = ?1)
  AND (?2 IS NULL OR f.name = ?2 COLLATE NOCASE)
```

| `account_id` | `mailbox` | 语义 |
| --- | --- | --- |
| `None` | `None` | 全部账户的全部邮件 |
| `Some(id)` | `None` | 某账户的全部邮件 |
| `None` | `Some("INBOX")` | **统一收件箱**：各账户 INBOX 合并 |
| `Some(id)` | `Some("INBOX")` | 某账户的收件箱 |

比维护两份 SQL（带/不带过滤）更不容易出错 —— 那种写法迟早会改了一边忘了另一边。

### 2.3 `COLLATE NOCASE`

IMAP 里邮箱名大小写不敏感（`INBOX` 是保留名，必须大小写不敏感匹配）。
不同服务商返回的可能是 `INBOX` / `Inbox` / `inbox`，不加 `NOCASE` 会导致
「统一收件箱」在某些账户上静默漏掉整个文件夹。

---

## 3. 验收结果

| 检查项 | 实测 |
| --- | --- |
| `cargo check --all-targets` | exit 0，**零警告** ✅ |
| `cargo fmt --check` | exit 0 ✅ |
| `cargo clippy --all-targets -- -D warnings` | exit 0 ✅ |
| `cargo test --test unified_inbox_test` | **13/13 通过** ✅ |
| 全量测试 | **145/145 通过** ✅ |

### 3.1 关键性质已被测试固化

| 性质 | 用例 |
| --- | --- |
| 跨账户跨文件夹按时间倒序合并 | `unifies_messages_across_accounts` |
| 结果携带来源文件夹与账户 | `carries_source_folder_and_account` |
| 四种过滤组合各自正确 | `scopes_to_one_account`、`scopes_to_inbox_across_accounts`、`combines_account_and_mailbox_filters` |
| **大小写不影响邮箱名匹配** | `mailbox_filter_is_case_insensitive` |
| 分页不重不漏 | `paginates_unified_view_without_gaps` |
| 文件夹的邮件数与未读数正确、且随已读状态变化 | `counts_messages_and_unread_per_folder` |
| 空库 / 无文件夹账户返回空而非报错 | `returns_empty_on_empty_database`、`returns_empty_for_account_without_folders` |

---

## 4. 尚未验证

- **前端未接入**。这两个命令目前没有界面调用，只到 `cargo test` 层。
- 真实多账户数据下的表现未实测（需至少两个真实邮箱）。
