# P4-04 账户的编辑与删除（2026-10-17，用户报的缺陷）

> 用户原话：「我突然发现一个问题，你现在这个版本没有临时编辑邮箱的能力。
> 就是比如说我不小心把我的邮箱打错了，添加进去之后删不掉也改不了。」

**这是个真缺陷，不是新需求** —— 添加账户是一条只能进不能出的单行道：
填错了没有任何补救手段，只能去删数据库文件（而那会连带丢光所有邮件）。

---

## 1. 缺的是什么

| 能力 | 之前 |
| --- | --- |
| 添加账户 | ✅ `add_account` |
| 列出账户 | ✅ `list_accounts` |
| **编辑账户** | ❌ 完全没有 |
| **删除账户** | ❌ 完全没有 |

---

## 2. 后端改动

### 2.1 删除：靠 schema 的级联，不手写删除顺序

`folders` / `messages` / `attachments` 三张表在 P0-03 建表时就写了
`ON DELETE CASCADE`，所以只需要一句：

```sql
DELETE FROM accounts WHERE id = ?1
```

就会顺着 `accounts → folders → messages → attachments` 全部清掉，
FTS 索引由 `messages_fts_ad` 触发器跟着删。

`delete_account` 先在一个事务里**数清楚**将删掉多少邮件与附件，再删 ——
删是不可逆的，得先把影响面告诉用户。

### 2.2 编辑：授权码「留空＝不改」

`update_account` 的 `new_secret_encrypted: Option<&[u8]>`：
`None` 表示不动凭证，`Some(密文)` 表示覆盖。

**没有**用「空数组表示清空」之类的约定 —— 那会让 `Option` 的两种状态变成三种，
而且「清空凭证」本身没有产品需求。

改邮箱会同时改 `provider_id`（由命令层重新解析），否则会出现
「地址是 A 家的、连接参数还是 B 家的」。

### 2.3 命令层只做校验和转发

新增 `validate_account_update`，并把服务商解析抽成 `resolve_for` ——
`validate_new_account` 与它共用，两处各写一遍必然漂移。

---

## 3. 一个被实测推翻的假设（重要）

我最初想写一条**阴性对照**：不开 `PRAGMA foreign_keys` 时级联应该失效、留下孤儿行，
以此证明上面那些「级联删干净」的测试**有判别力**。

**这条测试失败了。** 真实原因：`rusqlite` 的 `bundled` SQLite 在编译期带了
`SQLITE_DEFAULT_FOREIGN_KEYS=1`，新建连接**默认就是 1**（实测 SQLite 3.53.2）。
也就是说阴性对照根本构造不出来。

于是改成断言**真实的不变量**：`storage::open_database` 打开的连接，
`PRAGMA foreign_keys` 必须为 1。这条测试会在两种情况下变红：

- 有人把 `rusqlite` 从 `bundled` 换成系统库（默认值可能变 0，级联静默失效）
- 有人删掉 `open_database` 里那句 `PRAGMA foreign_keys = ON`

**「默认就是开的」是当前构建的偶然事实，不是 SQLite 的承诺。**
把它写成测试，而不是写成注释。

---

## 4. 前端改动

| 文件 | 作用 |
| --- | --- |
| `AccountRow.tsx` | 一行的展示 + 编辑/删除按钮 + **行内二次确认** |
| `AccountEditForm.tsx` | 就地展开的编辑表单（邮箱 / 新授权码 / 显示名） |
| `LabeledInput.tsx` | 三个结构相同的字段抽成一个，否则编辑表单会超 100 行 |
| `useUpdateAccount.ts` / `useDeleteAccount.ts` | 提交 + 缓存失效 |

### 4.1 删除为什么用行内确认而不是 `window.confirm`

因为要讲的不只是「确定吗」：

> 删除账户？
> 会同时删除该账户已下载的邮件与附件，不可恢复。

系统弹窗塞不下这段，样式也不受控。所以点「删除」→ 原位展开确认区 → 「确认删除 / 取消」。

### 4.2 删掉的正好是当前选中账户时

必须把 `accountId` 一起置空（`setAccount(null)` 会连带清 `mode` 与 `messageId`），
否则列表会拿着一个已不存在的 id 去查，表现为「删完了列表还在转圈」。

### 4.3 顺手删掉一段死代码

最初在删除成功后渲染「已删除，连带清掉 N 封邮件」。
**实测发现它永远显示不出来** —— 删除成功后该行立刻从列表消失，组件已卸载。
所以连同三个词典里的 `accountList.deleted` 一起删了：能确认删掉，就没必要再留一句
谁也看不到的文案。

---

## 5. 验证

### 5.1 后端单测 `account_admin_test.rs`：**9/9**

| 用例 | 断言要点 |
| --- | --- |
| `delete_removes_account_and_everything_below_it` | 账户/文件夹/邮件/附件/**FTS 索引**全为 0；影响面报告 2 封 2 附件 |
| `delete_unknown_account_reports_none_and_deletes_nothing` | 不存在的 id 返回 `None`，别的账户邮件不动 |
| `delete_only_touches_the_targeted_account` | 两个账户删一个，另一个的邮件必须还在 |
| `update_renames_without_touching_the_secret` | 不传授权码时，**已有密文必须原样保留** |
| `update_replaces_the_secret_when_given` | 传了就覆盖 |
| `update_changes_email_and_provider_together` | 邮箱与服务商一起改（这就是用户报的场景） |
| `update_reports_false_for_missing_account` | 不存在的 id 返回 false |
| `update_rejects_duplicate_email` | 改成已存在的邮箱被 UNIQUE 拒绝 |
| `app_connections_always_have_foreign_keys_on` | 见 §3 |

### 5.2 真机验证（WebView2 CDP）

截图：`docs/verification/account-edit-delete.png`

**第一轮 13/13**：每行都有「编辑 / 删除」→ 点编辑出现表单且邮箱预填 →
授权码框是 `type=password` → 改显示名保存 → 表单关闭、名字变了、**凭证状态仍是「已存凭证」** →
点删除先出二次确认（不直接删）→ 确认框写明了会删邮件与附件 →
点「取消」后账户仍在。

**第二轮 5/6**：用一个**一次性账户**（`deltest@qq.com`，假授权码）把删除真正走通 ——
真实账户 `mcpcnano@163.com` 与 `mcpcnano@sina.com` 全程未被波及。
唯一失败的一条正是 §4.3 那段死代码，已删除。

**没有删真实账户**：删除能力的验证用一次性账户完成，真实账户只做了「改显示名再改回」。

---

## 6. 尚未验证

| 项 | 说明 |
| --- | --- |
| **删除大量邮件时的耗时** | 一次性账户没有邮件。删一个几百封的账户要跑多少次触发器、要不要几百毫秒，**没测** |
| 改邮箱后真实登录 | 只验到「服务商被正确改判」，**没有真的用改后的地址登录过** |
| 改授权码后能否登录 | 同理：密文确实换了，但没验证过新授权码能登录成功 |
| 并发 | 删除与自动收信同时发生时（15 秒一轮）会不会撞车，**没测** —— 两者都会拿数据库锁，理论上串行，但没有实测 |

**最后一条值得注意**：自动收信是每 15 秒一次，用户点删除时很可能正好撞上一轮同步。
`AppState.db` 是 `Mutex<Connection>`，理论上会阻塞而不是出错，但**没有实测**。
