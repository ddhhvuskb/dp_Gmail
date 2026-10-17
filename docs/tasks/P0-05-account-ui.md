# P0-05 账户配置界面（服务商自动识别，不存明文密码）

- **阶段**：P0
- **依赖**：P0-07d（`add_account`）、P0-07a/b（凭证加密）
- **状态**：✅ 已完成并验证（2026-10-16）
- **关键约束**：自动识别服务商
- **禁止**：在 `localStorage` 存密码

---

## 1. 任务分解

| 子任务 | 交付物 | 状态 |
| --- | --- | --- |
| P0-05a | `commands::resolve_provider` + `ProviderView`（用户批准新增） | ✅ |
| P0-05b | `src/lib/useDebounced.ts` 去抖 hook | ✅ |
| P0-05c | `src/features/accounts/useProviderLookup.ts` | ✅ |
| P0-05d | `src/features/accounts/AccountForm.tsx` | ✅ |
| P0-05e | `src/features/accounts/AccountList.tsx` | ✅ |
| P0-05f | `App.tsx` 侧栏接入 | ✅ |

---

## 2. 接口契约

```rust
// commands/accounts.rs
/// 服务器地址只用于**显示**，前端不得据此硬编码。
#[derive(serde::Serialize)]
pub struct ProviderView {
    pub provider_id: String,
    pub provider_name: String,
    pub imap_host: String,
    pub imap_port: u16,
    pub smtp_host: String,
    pub smtp_port: u16,
}

/// 未收录的域名返回 None（前端据此提示"暂不支持"）。
#[tauri::command]
pub fn resolve_provider(email: String) -> Option<ProviderView>;
```

---

## 3. 为什么需要新命令（而不是前端硬编码域名表）

`DESIGN.md` §5.4 的拦截规则写着「硬编码 → 服务商配置从 `provider.rs` 读」。
如果前端自己存一份 `qq.com → QQ邮箱` 的映射：

- 会形成**两份真相**，后端加一个服务商前端就漏一个；
- 正是 `AGENTS.md` 禁止项指向的坑。

所以前端一律通过 `resolve_provider` 向后端问。前端**不认识任何域名**。

---

## 4. 安全约定

| 约定 | 落实方式 |
| --- | --- |
| 密码不落 `localStorage` | 授权码只存在于 `AccountForm` 的组件 state；提交成功后**立即清空**。全项目没有任何 `localStorage` 调用。 |
| 密码不回显 | `<input type="password">`；后端错误信息绝不回显授权码（见 `add_account_test::error_messages_never_echo_the_secret`）。 |
| 密文不出后端 | `AccountList` 只展示 `has_secret` 布尔，`AccountView` 里根本没有密文字段。 |
| 识别失败不白填密码 | 服务商识别成功前，提交按钮保持禁用。 |

---

## 5. 验收结果

| 检查项 | 实测 |
| --- | --- |
| `cargo check --all-targets` | exit 0，零警告 ✅ |
| `cargo fmt --check` | exit 0 ✅ |
| `tsc --noEmit` | exit 0 ✅ |
| `npm run build` | exit 0，产物 297.36 kB / gzip 91.54 kB ✅ |
| `cargo test --test resolve_provider_test` | **5/5 通过** ✅ |
| 全量测试 | **116/116 通过** ✅ |

### 5.1 关键性质已被测试固化

`resolve_provider` 不依赖 `tauri::State`，是纯函数，可以直接调用验证 ——
比只测底层的 `provider::resolve_by_email` 更贴近真实调用路径。

| 性质 | 用例 |
| --- | --- |
| QQ 系三个域名都识别为同一服务商 | `resolves_qq_mailbox`、`resolves_qq_family_domains` |
| 大小写与首尾空白不影响识别 | `is_case_insensitive_and_trims` |
| 未收录域名 / 缺 @ / 空串 / 空本地部分一律返回 `None` | `returns_none_for_unsupported_provider` |
| **端口不为 0**（配置表写漏会让前端显示 `imap.qq.com:0` 这种误导信息） | `never_returns_port_zero` |

---

## 6. 尚未验证

- ~~**界面从未在真实 Tauri 窗口里跑过**~~。✅ **2026-10-16/17 已实跑**：账户表单、服务商识别、
  企业邮箱手动选择（ADR-0004）均在真实窗口验证过。**本条是当时的记录，已不成立。**
- 实时识别依赖真实命令往返，去抖 300 ms 后的体感未实测。
