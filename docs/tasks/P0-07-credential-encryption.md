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
| P0-07c | 把密文接进存储（`accounts` 加列 or 新建表） | ⛔ **待 schema 决策** |
| P0-07d | `commands::add_account` 命令（P0-04 并入） | ⛔ 待 P0-07c |
| P0-07e | `commands::sync_folder` 命令（P0-04d） | ⛔ 待 P0-07c |

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

## 6. 待决策：密文落到哪里

`DESIGN.md` P0-03 的表清单是 `accounts / folders / messages / attachments`，
**既没有凭证列也没有凭证表**。

把密文加进 `accounts`（如 `secret_encrypted BLOB`）属于**变更数据库表结构**，
按第六章与 ADR-0002 §7，同样必须先经用户确认。

**已提交用户，未获答复前不动 schema。** 这也是 P0-07c/d/e 与 P0-05 的共同前置。
