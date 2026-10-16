# P0-03 SQLite 表（accounts / folders / messages / attachments，UID 去重）

- **阶段**：P0
- **依赖**：ADR-0001（`rusqlite` bundled）、P0-02（提供 `ParsedMessage`）
- **状态**：✅ 已完成并验证（2026-10-16）
- **关键约束**：时间用 Unix 整数，联合唯一索引
- **禁止**：用 ORM，省略事务

---

## 1. 目标

建立 P0 所需的四张表与索引，并保证建表动作在事务内完成。
本任务**不**做 Tauri 命令（P0-04）、**不**建 FTS5 虚表（P1）。

## 2. 任务分解

| 子任务 | 交付物 | 验证 |
| --- | --- | --- |
| P0-03a | `src-tauri/src/storage/schema.rs` 建表 | `cargo test --test storage_test` |
| P0-03b | `src-tauri/src/storage/mod.rs` 打开数据库与 PRAGMA | `cargo test --test storage_test` |
| P0-03c | `src-tauri/src/storage/message_store.rs` UID 去重写入 | `cargo test --test storage_test` |

## 3. 表结构（接口契约）

时间字段一律 `INTEGER`，存 **Unix 秒**。跨表引用一律带 `ON DELETE CASCADE`。

| 表 | 关键列 | 约束 |
| --- | --- | --- |
| `accounts` | `email`, `display_name`, `provider_id`, `created_at` | `email` 唯一 |
| `folders` | `account_id`, `name`, `uid_validity`, `uid_next`, `last_sync_at` | **`UNIQUE(account_id, name)`** |
| `messages` | `folder_id`, `uid`, `message_id`, `subject`, `from_name`, `from_address`, `to_addresses`, `date_unix`, `text_body`, `html_body`, `is_seen`, `fetched_at` | **`UNIQUE(folder_id, uid)`**（UID 去重） |
| `attachments` | `message_id`, `filename`, `content_type`, `size` | — |

索引：`idx_messages_folder_date (folder_id, date_unix DESC)`、`idx_attachments_message (message_id)`。

### 3.1 接口

```rust
// storage/error.rs
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("数据库错误：{0}")]
    Sqlite(#[from] rusqlite::Error),
}

// storage/schema.rs
/// 在**事务内**建表建索引。可重复调用（全部 IF NOT EXISTS）。
pub fn init_schema(conn: &mut rusqlite::Connection) -> Result<(), StorageError>;
```

---

## 4. 已核实的 API 事实

| 事实 | 依据 |
| --- | --- |
| `Connection::open(path)` / `Connection::open_in_memory()` | `rusqlite-0.40.2/src/lib.rs:437,448` |
| `Connection::execute_batch(&str)` | `lib.rs:548` |
| `conn.transaction()` 返回 `Transaction`，其 `Deref<Target = Connection>` | `transaction.rs:233`；`lib.rs:1361` 测试用例 |
| `Transaction::commit()` | `transaction.rs:187` |
| `PRAGMA` 在事务内对 `foreign_keys` 无效，必须放在事务外 | SQLite 语义 |

---

## 5. 约束块（要求 5.2）

```
【角色】Rust开发工程师
【任务】实现 schema.rs 的 init_schema
【输入】conn: &mut rusqlite::Connection（调用方负责打开）
【输出】fn init_schema(conn: &mut Connection) -> Result<(), StorageError>
【约束】
  - 只生成这一个函数 + 错误类型 + SQL 常量
  - crate：rusqlite 0.40（bundled）
  - 全部 DDL 必须在**一个事务**内执行
  - 错误处理返回 Result，不 unwrap
  - 不改已有文件
【验收】cargo test 通过：四张表齐备、UID 联合唯一、可重复调用
【禁止】使用 ORM；使用 `CREATE TABLE` 但不带 `IF NOT EXISTS`；把 PRAGMA foreign_keys 放进事务
```

---

## 6. 验收标准

| 检查项 | 标准 |
| --- | --- |
| 编译 | `cargo check` 无错误；`cargo fmt --check` 干净 |
| 单测 | 建表、幂等、UID 去重、跨文件夹同 UID 允许、email 唯一，全过 |
| 事务 | DDL 在事务内（代码可读性检查） |
| 时间 | 全部时间列为 `INTEGER`（无 `TEXT`/`DATETIME`） |

---

## 7. 验收结果（实测）

| 检查项 | 实测 |
| --- | --- |
| `cargo check --all-targets` | exit 0 ✅ |
| `cargo fmt --check` | exit 0 ✅ |
| `cargo test --test storage_test` | **8/8 通过**（P0-03a） ✅ |
| `cargo test --test storage_open_test` | **5/5 通过**（P0-03b） ✅ |
| `cargo test --test message_store_test` | **7/7 通过**（P0-03c） ✅ |

### 两处设计决定

1. **`PRAGMA foreign_keys` 不放在 `init_schema` 里。** 它在事务内是**静默失效**的空操作，放进去会导致级联删除完全不生效却毫无报错。已放在 `open_database` 的事务之外，并补了 `enables_foreign_key_enforcement` 与 `cascades_delete_from_account_down_to_attachments` 两个测试锁死该行为。
2. **`storage` 不直接吃 `mail::parser::ParsedMessage`。** `DESIGN.md` 第三章的调用方向是 `mail → storage`，反向依赖会形成环。故在 `storage::message_store` 内定义了入参 `NewMessage` / `NewAttachment`，由 mail 层做映射。

### 行为约定（已由测试固化）

| 场景 | 行为 |
| --- | --- |
| 同一 `(folder_id, uid)` 重复同步 | 更新原行，不产生重复；返回同一 `messages.id` |
| 重复同步时的 `is_seen` | **不被覆盖**，保留本地已读状态 |
| 重复同步时的附件 | 全量替换（先删后插） |
| 写入中途失败 | 整事务回滚，附件不残留 |
