# P0-07 凭证加密（AES-256-GCM，密钥文件管理）

- **阶段**：P0
- **依赖**：ADR-0002（方案，用户已确认）
- **状态**：🟨 进行中（加密核心完成；**落地位置待 schema 决策**）
- **关键约束**：AES-256-GCM
- **禁止**：硬编码密钥，ECB 模式

---

## 1. 目标

让邮箱密码/授权码**不以明文形式**出现在任何持久化介质里。

方案已由用户确认并记录在 `docs/adr/ADR-0002-credential-encryption.md`：
**版本化 JSON 密钥文件，无口令保护**。

## 2. 任务分解

| 子任务 | 交付物 | 状态 |
| --- | --- | --- |
| P0-07a | `security/keyfile.rs` 密钥文件的生成 / 读取 / 校验 | ✅ |
| P0-07b | `security/credentials.rs` AES-256-GCM 加解密 | ✅ |
| P0-07c | `accounts.secret_encrypted` 列 + schema 版本迁移 + 账户读写 | ✅ |
| P0-07d | `commands::add_account` 命令（P0-04 并入） | ✅ |
| P0-07e | `commands::sync_folder` 命令（P0-04d） | ✅ |

---

## 3. 接口契约

```rust
// security/keyfile.rs
pub const KEY_FILE_VERSION: u8 = 1;
pub const KEY_LEN: usize = 32;

/// 读取密钥文件；不存在则生成 32 字节随机密钥并落盘。
/// 版本或算法不匹配一律报错，不猜测。
pub fn load_or_create_key(path: &Path) -> Result<[u8; KEY_LEN], SecurityError>;

// security/credentials.rs
/// 密文布局：`nonce(12) || ciphertext || GCM tag(16)`。
pub fn encrypt_secret(plaintext: &str, key: &[u8; KEY_LEN]) -> Result<Vec<u8>, SecurityError>;
pub fn decrypt_secret(blob: &[u8], key: &[u8; KEY_LEN]) -> Result<String, SecurityError>;
```

---

## 4. 已核实的 API 事实（读 crate 源码得出）

| 事实 | 依据 |
| --- | --- |
| `aes-gcm` 0.11 **不再**用 0.10 的 `Aead` 作为主 trait，改为 `AeadInOut`（就地操作）；分配式的 `encrypt`/`decrypt` 仍在 `aead::Aead` 上 | `aes-gcm-0.11.1/src/lib.rs:85`、`aead-0.6.1/src/lib.rs:131,220` |
| 0.11 的 `default` feature 已含 `getrandom`，故 `Key::<Aes256Gcm>::generate()` / `Nonce::generate()` 开箱可用 | `aes-gcm-0.11.1/Cargo.toml` `[features]` |
| `aes-gcm` 有独立 `zeroize` feature（默认关），开启后内部密钥材料用后清零 | 同上 |
| `Nonce::from_slice` 在 0.11 已**弃用**，须改用 `TryFrom` | 编译器 deprecation 警告 |
| `base64` 0.23 的标准引擎是 `engine::general_purpose::STANDARD`，需 `use base64::Engine as _` 才有点法 | `base64-0.23.1/src/engine/general_purpose/mod.rs:436` |

---

## 5. 验收结果（实测）

| 检查项 | 实测 |
| --- | --- |
| `cargo check --all-targets` | exit 0，**零警告** ✅ |
| `cargo fmt --check` | exit 0 ✅ |
| `cargo test --test credentials_test` | **8/8 通过** ✅ |
| `cargo test --test keyfile_test` | **8/8 通过** ✅ |

### 5.1 安全性质已被测试固化

| 性质 | 用例 |
| --- | --- |
| 加解密往返正确（含空串、中文、emoji、换行制表符） | `round_trips_a_secret`、`round_trips_empty_and_unicode` |
| **nonce 每次不同** | `uses_a_fresh_nonce_every_time` |
| 密文不含明文 | `ciphertext_does_not_contain_plaintext` |
| 换密钥解不开 | `fails_with_wrong_key` |
| **篡改认证标签被拦下** | `detects_tampering` |
| **篡改正文被拦下** | `detects_tampering_in_ciphertext_body` |
| 长度不足被拦下 | `rejects_truncated_blob` |
| 密钥文件格式/版本/算法/长度不符一律拒绝 | `rejects_unsupported_version` 等 4 个用例 |
| 同一文件反复读取得到同一密钥、不同文件得到不同密钥 | `returns_same_key_on_second_load`、`generates_distinct_keys_for_distinct_files` |

---

## 6. 密文落地：`accounts.secret_encrypted`（用户 2026-10-16 确认）

用户选定：**`accounts` 加一列 `secret_encrypted BLOB`**，表数保持 P0-03 的四张。

### 6.1 schema 版本机制

原 `init_schema` 只有 `CREATE TABLE IF NOT EXISTS`，**对已存在的库不会补列**。
因此引入 SQLite 内建的 `PRAGMA user_version`：

| 版本 | 内容 |
| --- | --- |
| 1 | P0-03 的初始四表 |
| 2 | P0-07，`accounts` 增加 `secret_encrypted` |

`init_schema` 的判定顺序：

1. **建表前**探测 `accounts` 是否已存在，据此区分"全新库"与"历史库"；
2. 执行 `SCHEMA_SQL`（全新库的 `CREATE TABLE` 已含新列）；
3. 全新库起点直接算作最新版，**跳过全部迁移** —— 否则会对刚建好的表重复 `ALTER TABLE` 而报"列已存在"；
4. 历史库按 `user_version` 逐个跑 `MIGRATIONS` 里版本号大于当前的语句；
5. 盖上 `SCHEMA_VERSION`。

### 6.2 接口

```rust
// storage/account_store.rs
/// `secret_encrypted` 必须是 AES-256-GCM 密文，不是明文。
pub fn create_account(
    conn: &Connection,
    email: &str,
    display_name: Option<&str>,
    provider_id: &str,
    secret_encrypted: Option<&[u8]>,
) -> Result<i64, StorageError>;

/// 账户不存在或未设凭证都返回 None。
pub fn load_encrypted_secret(
    conn: &Connection,
    account_id: i64,
) -> Result<Option<Vec<u8>>, StorageError>;
```

---

## 7. 验收结果（P0-07c）

| 检查项 | 实测 |
| --- | --- |
| `cargo check --all-targets` | exit 0，零警告 ✅ |
| `cargo test --test schema_migration_test` | **5/5 通过** ✅ |
| `cargo test --test account_store_test` | **6/6 通过** ✅ |

### 7.1 迁移机制已被测试固化

| 性质 | 用例 |
| --- | --- |
| 全新库被盖上当前版本号 | `fresh_database_is_stamped_with_current_version` |
| 全新库直接就有 `secret_encrypted` | `fresh_database_has_encrypted_secret_column` |
| 旧库被补上新列，且其余缺失的表也补齐 | `migrates_legacy_database_missing_the_column` |
| 迁移可重复执行（不会重复 `ALTER`） | `migration_is_idempotent` |
| **迁移不丢老数据**，老行新列为 NULL | `preserves_legacy_rows_during_migration` |

### 7.2 安全性质已被测试固化

| 性质 | 用例 |
| --- | --- |
| 加密 → 落库 → 读回 → 解密 全链路还原 | `encrypted_secret_round_trips_through_the_database` |
| **逐列扫描整行，明文密码不出现在任何列** | `stored_blob_is_not_the_plaintext` |
| 换密钥解不开库里存的密文 | `wrong_key_cannot_open_stored_secret` |
| 重复邮箱被 `UNIQUE` 拒绝 | `rejects_duplicate_email` |

> 写 `stored_blob_is_not_the_plaintext` 时踩了一个坑：一开始把每列都按 `Vec<u8>` 读，
> 撞上 `InvalidColumnType(0, "id", Integer)`。必须按 `rusqlite::types::Value` 分类型取字节，
> 否则 INTEGER/TEXT 列根本读不出来，测试会以假阳性通过。

---

## 8. 验收结果（P0-07d `add_account`）

| 检查项 | 实测 |
| --- | --- |
| `cargo check --all-targets` | exit 0，零警告 ✅ |
| `cargo test --test add_account_test` | **7/7 通过** ✅ |

### 8.1 命令链路

```
add_account(email, secret, display_name)
  ├─ validate_new_account()          参数校验 + 服务商自动识别
  ├─ security::encrypt_secret()      AES-256-GCM 加密
  └─ storage::create_account()       密文落库，返回 id
```

`AppState` 增加 `crypto_key`，由 `run()` 的 `setup` 阶段从密钥文件读取或生成，
**只存在于进程内存**，不进日志、不进 `Debug`。

### 8.2 安全性质已被测试固化

| 性质 | 用例 |
| --- | --- |
| 未收录的服务商被拒（不猜） | `rejects_unknown_provider` |
| 空邮箱 / 空授权码被拒 | `rejects_empty_email`、`rejects_empty_secret` |
| **错误信息绝不回显授权码**（错误会打到前端甚至日志） | `error_messages_never_echo_the_secret` |
| 本地部分大小写不被改写 | `preserves_local_part_case` |

### 8.3 P0-07e `sync_folder` 的实施结果

| 层 | 新增 | 状态 |
| --- | --- | --- |
| `mail/provider.rs` | `find_by_id(id)` | ✅ 同步时手里只有 `provider_id`，没有邮箱可反查 |
| `mail/sync.rs` | `fetch_recent(cfg, user, secret, mailbox, limit)` | ✅ 异步段：SELECT + FETCH `BODY.PEEK[]` |
| `mail/sync.rs` | `to_new_message(uid, parsed)` | ✅ mail → storage 的入参映射 |
| `storage/folder_store.rs` | `ensure_folder(conn, account_id, name)` | ✅ `ON CONFLICT DO NOTHING` + 回查 |
| `storage/message_store.rs` | `upsert_messages(conn, folder_id, &[..])` | ✅ **整批一个事务** |
| `storage/account_store.rs` | `load_account_credentials(conn, id)` | ✅ 一次查询取齐身份 + 密文 |
| `commands/sync.rs` | `sync_folder(...)` | ✅ 三段式编排 |

**`MutexGuard` 不跨 `.await`** 这条约束落实为三段式：

```
第一段（同步）  加锁：读身份 + 解密凭证 → 出块即释放
第二段（异步）  全程不持锁：IMAP 拉取 + MIME 解析
第三段（同步）  重新加锁：ensure_folder + upsert_messages
```

实现要点：

- 用 `BODY.PEEK[]` 而非 `BODY[]` —— 后者会隐式把邮件标记为 `\Seen`，**同步不该改变服务端状态**。
- 只取最后 `limit` 封：IMAP 序号区间是闭区间 `[exists-limit+1, exists]`。
- 抽出了私有 `write_one(tx, ...)`，让单封写入与批量写入共用同一段 SQL，避免复制一份 20 行的 UPSERT。
- `upsert_messages` **整批共用一个事务与一个时间戳**；批次中途失败整批回滚。

**`Mailbox.exists: u32`、`Fetch.uid`、`Fetch::body()`、`Uid = u32` 均已读源码核实**
（`async-imap-0.12.0/src/types/mailbox.rs`、`fetch.rs:28,110`、`types/mod.rs:80`）。

### 8.4 验收结果（P0-07e）

| 检查项 | 实测 |
| --- | --- |
| `cargo check --all-targets` | exit 0，零警告 ✅ |
| `cargo test --test sync_support_test` | **11/11 通过** ✅ |

| 性质 | 用例 |
| --- | --- |
| 同名文件夹复用同一行、跨账户互不干扰 | `ensure_folder_creates_then_reuses`、`ensure_folder_isolates_by_account` |
| 批量写入整批落库 | `upsert_messages_stores_whole_batch` |
| 同 UID 重同步是更新而非新增 | `upsert_messages_is_idempotent_per_uid` |
| **批次失败整批回滚** | `upsert_messages_rolls_back_whole_batch_on_failure` |
| 解析结果 → 存储入参的字段映射（含附件大小、多收件人拼接） | `to_new_message_*` 3 个用例 |
| 缺 Date 头时记 0（列表排到最后，不会被误当新邮件） | `to_new_message_defaults_missing_date_to_zero` |
| 批大小钳制 | `normalize_batch_clamps_to_bounds` |

> **尚未验证**：`fetch_recent` 需要真实 IMAP 服务器才能真正跑通。
> 目前只到 `cargo check` + 上游/下游单测。等 QQ 授权码到位后做真机联调。
