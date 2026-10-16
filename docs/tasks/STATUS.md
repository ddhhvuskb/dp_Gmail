# 任务状态板

> 每完成一个任务，更新本文件。规则：`DESIGN.md` 第五章 5.1（一次一个函数/组件）。

**图例**：⬜ 未开始 · 🟨 进行中 · ✅ 已完成并通过验证 · ⛔ 阻塞

---

## 环境

| 项 | 状态 | 备注 |
| --- | --- | --- |
| Node / npm | ✅ | v24.16.0 / 11.13.0；前端依赖已装 |
| WebView2 | ✅ | 155.0.4283.45 |
| Rust stable (MSVC) | ✅ | rustc / cargo **1.99.1** |
| VS 2022 Build Tools (VCTools) | ✅ | MSVC 14.44.35207，`link.exe` 就位 |
| git | ✅ | **2.55.0**，仓库已 `git init`（branch `main`） |
| `tsc --noEmit` | ✅ | TS 5.9.3，已通过 |
| `cargo check` / `test` | ✅ | 已验证 |

**环境备注**：`static.rust-lang.org` 在国内拉不动（卡 20 分钟 0 字节），已改用 `rsproxy.cn` 镜像，并以用户级环境变量持久化：

```
RUSTUP_DIST_SERVER=https://rsproxy.cn
RUSTUP_UPDATE_ROOT=https://rsproxy.cn/rustup
```

---

## P0 任务

| 任务 | 内容 | 状态 | 验证方式 | 验证结果 |
| --- | --- | --- | --- | --- |
| P0-01a | `mail/provider.rs` 服务商识别 | ✅ | `cargo test --test provider_test` | **7/7** |
| P0-01b | `mail/imap_client.rs` TLS + 30s 超时 | ✅ | `cargo test --test imap_client_test` | **2/2**，超时实测 30.02s |
| P0-02a | `mail/parser.rs` MIME 解析 | ✅ | `cargo test --test parser_test` | **9/9** |
| P0-02b | `security/sanitize.rs` HTML 净化 | ✅ | `cargo test --test sanitize_test` | **5/5** |
| P0-03a | `storage/schema.rs` 建表（事务内） | ✅ | `cargo test --test storage_test` | **8/8** |
| P0-03b | `storage::open_database` | ✅ | `cargo test --test storage_open_test` | **5/5** |
| P0-03c | `storage::upsert_message` UID 去重 | ✅ | `cargo test --test message_store_test` | **7/7** |
| P0-04a | Tauri 脚手架 + 前端壳子 | ✅ | `cargo check` + `npm run build` | exit 0 |
| P0-04b | `storage::list_messages` + DTO | ✅ | `cargo test --test list_messages_test` | **6/6** |
| P0-04c | `commands::get_messages` + 分页校验 | ✅ | `cargo test --test commands_test` | **4/4** |
| P0-04d | `commands::sync_folder` | ✅ | `cargo test --test sync_support_test` | **已改编号为 P0-07e**，见下方 |
| P0-05 | 账户配置界面 | ✅ | `tsc --noEmit` + 实跑 | 见下方同名行（原先因依赖 P0-07 阻塞） |
| P0-06 | 邮件列表 + 详情（虚拟滚动，HTML 净化） | ✅ | `tsc --noEmit` + 实跑 | `get_message`（P0-06e）已实现；见 `P0-06-message-list-detail.md` |
| P0-07a | `security/keyfile.rs` 密钥文件管理 | ✅ | `cargo test --test keyfile_test` | **8/8** |
| P0-07b | `security/credentials.rs` AES-256-GCM | ✅ | `cargo test --test credentials_test` | **8/8** |
| P0-07c | `accounts.secret_encrypted` + schema 迁移 + 账户读写 | ✅ | `cargo test --test schema_migration_test`、`account_store_test` | **5/5 + 6/6** |
| P0-07d | `commands::add_account`（原 P0-04） | ✅ | `cargo test --test add_account_test` | **7/7** |
| P0-07e | `commands::sync_folder`（原 P0-04d） | ✅ | `cargo test --test sync_support_test` | **11/11**；IMAP 段待真机联调 |
| P0-05 | 账户配置界面（服务商自动识别，不存明文密码） | ✅ | `tsc --noEmit` + `cargo test` | **5/5**；UI 未在真实窗口跑过 |
| P0-06 | 邮件列表 + 详情（虚拟滚动，HTML 净化） | ✅ | `tsc --noEmit` + `cargo test` | **5/5 + 5/5**；UI 未在真实窗口跑过 |
| P0-08 | CI 构建 Windows x64 + ARM64 | ✅ | YAML 校验 + 本地实跑 | 工作流落地；x64 release 构建已验证，**ARM64 仅在 CI 首次运行时验证** |

### 累计测试：**116 全绿**

### P0 完成度：**8/8 任务全部落地**

| 阶段 | 状态 |
| --- | --- |
| P0-01 ~ P0-08 | ✅ 全部完成 |
| **已解除的验证缺口** | ~~界面从未在真实窗口跑过~~ → 2026-10-16 已实跑，见下 |
| **仍未验证** | `fetch_recent` 未真机联调（需 QQ 授权码）；滚动帧率未实测；搜索/统一收件箱界面未实操；ARM64 构建仅在 CI 首次运行时验证 |

### 已关掉的欠账

| 项 | 状态 |
| --- | --- |
| ~~搜索 <200ms~~ | ✅ **已实测**：最慢 2.11 ms |
| ~~LIKE 回退全表扫描的性能风险~~ | ✅ **已实测**：2.12 ms |
| ~~1000 封规模下的列表/统一收件箱查询~~ | ✅ **已实测**：1.57 / 6.14 ms |
| ~~界面只在空库上跑过~~ | ✅ **已用 1260 封仿真数据实跑**，见 `real-data-verification.md` |
| ~~HTML 净化是否真的拦得住 XSS~~ | ✅ **端到端验证**：`<script>` 被剥除、`onerror` 未执行 |
| 滚动帧率（前端侧） | ⛔ 只验证了渲染正确，未测帧率 |
| ~~搜索 / 统一收件箱的界面操作~~ | ✅ **均已实跑验证**：搜索链路见下；统一收件箱见 `p1-unified-inbox.png` |
| ~~空结果提示~~ | ✅ **已实跑验证**：搜索无结果时显示「没有匹配「…」的邮件。」，不再是空白 |

> ⚠️ **未验证项仍在**：虚拟滚动帧率、详情 HTML 渲染的真实观感、
> 「1000 封不卡」的帧率侧 —— 都还只有单测或渲染正确性撑着。

#### 统一收件箱实跑记录（2026-10-16）

截图：`docs/verification/p1-unified-inbox.png`

点击文件夹树中的「统一收件箱」后观察到：

| 观察 | 说明 |
| --- | --- |
| 该按钮高亮为当前视图 | 选中态正常 |
| 中栏标题为「统一收件箱」 | `listTitle` 的 unified 分支正常 |
| **每行标注「zhangsan@qq.com · INBOX」** | **账户归属生效** —— P1-02 的核心 |

账户归属是这次验证里最要紧的一条：统一收件箱把多账户邮件聚合在一起，
不标明来源用户就分不清哪封属于哪个邮箱。

**操作提示**：800px 高的窗口下「统一收件箱」被挤在侧栏可视区最底部、只露一半，
需先把窗口调高（本轮用 990px）才能稳定点中。
>
> ⚠️ **教训**：P0-05 的残留过期文案、FolderTree 提示指向不存在的按钮，
> 这两类问题都是**只有实跑才能发现**的。任何纯前端的改动，
> 只要没在真实窗口里看过，就应视为未验证。

#### 搜索链路实跑记录（2026-10-16）

截图：`docs/verification/p1-empty-search-hint.png`

在真实窗口中输入 `zzznotfoundqqq` 并点「搜索」，观察到：

| 观察 | 说明 |
| --- | --- |
| 输入框显示 `zzznotfoundqqq` | 受控输入正常 |
| 中栏标题变为 `搜索：zzznotfoundqqq` | `listTitle` 的 search 分支正常 |
| 中栏显示「没有匹配「zzznotfoundqqq」的邮件。」 | **本轮新增的空结果提示生效** |

这一次操作**顺带验证了整条搜索链路**：输入 → 按钮 → IPC 参数映射 →
`commands::search_messages` → 返回空数组 → 前端按空结果渲染。
此前这条链路只有单测覆盖（单测直接调函数，绕过 serde 与 IPC 层）。

**踩到的两个坑**（供后续 UI 自动化参考）：

1. **中文输入法会截走 `SendKeys` 的字符**，弹出的候选框还会盖住按钮。
   解法：改用剪贴板 `^v` 粘贴，并先 `{ESC}` 关掉候选框。
2. **坐标必须从实拍截图量，不能凭上一张截图推算** ——
   本轮加入「写邮件」按钮后整个侧栏下移，旧坐标全部失效。
   搜索框在顶栏、位置稳定，故选它做验证而不是侧栏里的文件夹。
> 一次真机联调（QQ 授权码）能同时消掉其中大部分。

---

## 端到端实跑验证（2026-10-16）

用 `npm run tauri dev` 真实启动过一次应用，证据如下。
截图存档：`docs/verification/p0-app-running.png`。

### 应用启动

- 进程 `dp-gmail.exe`，主窗口标题 **`dp_Gmail`**，占用 33 MB。
- 三栏界面完整渲染：侧栏「添加账户 / 已配置账户 / 同步」、中栏列表占位、右栏详情占位。
- 提交按钮在服务商识别成功前正确保持禁用（截图确认）。

### 数据目录与密钥文件

应用 `setup` 阶段在 `%APPDATA%\com.dpgmail.desktop\` 下创建了：

| 文件 | 说明 |
| --- | --- |
| `dp_gmail.sqlite` | 4096 B |
| `dp_gmail.sqlite-wal` / `-shm` | **WAL 生效的直接证据** |
| `key.json` | 133 B，内容 `{version:1, algorithm:"AES-256-GCM", key:"<base64 32B>", created_at}` |

### 库结构（只读查询运行中的库）

```
user_version = 2
表: accounts, attachments, folders, messages
accounts 列: id, email, display_name, provider_id, created_at, secret_encrypted
索引: idx_attachments_message, idx_messages_folder_date
自动唯一索引: sqlite_autoindex_accounts_1, sqlite_autoindex_folders_1, sqlite_autoindex_messages_1
```

三个自动唯一索引正好对应 `accounts.email`、`folders(account_id,name)`、`messages(folder_id,uid)`，
即 `DESIGN.md` 要求的「联合唯一索引 / UID 去重」。

### IPC 往返已验证

启动即触发 `useAccounts` → `list_accounts`。
界面显示的是**空列表文案**（「还没有账户。用上面的表单添加一个。」），
而不是 `App.tsx` 里 `accounts.isError` 分支的红色错误 ——
说明该命令**成功返回了空数组**。

这一次往返同时证明了：命令已注册、capabilities 未拦截自定义命令、
camelCase 参数解析正确、`AccountView` 序列化正常。

### 本次发现并修正

界面里残留了一句过期文案「账户添加界面属 P0-05，尚未实现」——
P0-05 落地后它已不成立，容易误导使用者。已改为「先在左侧「添加账户」里配置一个邮箱」。
这正是只有把界面真跑起来才会暴露的问题。

| 测试目标 | 用例数 |
| --- | --- |
| `provider_test` | 10 |
| `imap_client_test` | 3 |
| `parser_test` | 9 |
| `sanitize_test` | 5 |
| `storage_test` | 8 |
| `storage_open_test` | 5 |
| `message_store_test` | 7 |
| `list_messages_test` | 6 |
| `commands_test` | 4 |
| `keyfile_test` | 8 |
| `credentials_test` | 8 |
| `schema_migration_test` | 5 |
| `account_store_test` | 6 |
| `add_account_test` | 7 |
| `sync_support_test` | 11 |
| `fts5_capability_test` | 5 |
| `search_store_test` | 12 |
| `unified_inbox_test` | 13 |
| `performance_test` | 7 |
| `smtp_send_test` | 19 |
| `provider_163_test` | 6 |
| `resolve_provider_test` | 7 |
| `compose_test` | 11 |
| `message_detail_test` | 5 |
| `list_accounts_test` | 5 |
| `resolve_provider_test` | 5 |

**前端**：`tsc --noEmit` exit 0；`npm run build` exit 0。

---

## 后端模块现状

```
src-tauri/
├── build.rs              tauri_build::build()
├── tauri.conf.json       identifier / frontendDist / CSP
├── capabilities/default.json
├── icons/                tauri icon 生成（icon.ico 为 Windows 必需）
└── src/
    ├── lib.rs            run() → setup(建目录/开库/manage) + invoke_handler
    ├── main.rs           入口
    ├── state.rs          AppState { db: Mutex<Connection> }
    ├── commands/
    │   └── messages.rs       get_messages（仅校验 + 转发）
    ├── mail/
    │   ├── error.rs         MailError
    │   ├── provider.rs      服务商表 + resolve_by_email / find_by_id
    │   ├── imap_client.rs   connect_and_login（TLS + 30s）
    │   ├── sync.rs          fetch_recent / to_new_message
    │   └── parser.rs        parse_message
    ├── security/
    │   ├── error.rs         SecurityError
    │   ├── keyfile.rs       load_or_create_key（版本化 JSON）
    │   ├── credentials.rs   encrypt_secret / decrypt_secret（AES-256-GCM）
    │   └── sanitize.rs      sanitize_html
    └── storage/
        ├── error.rs         StorageError
        ├── schema.rs        init_schema / SCHEMA_VERSION（user_version 迁移）
        ├── account_store.rs create_account / load_account_credentials
        ├── folder_store.rs  ensure_folder
        ├── message_store.rs upsert_message(s) / list_messages
        └── mod.rs           open_database
```

前端（P0-06 完成）：

```
src/
├── main.tsx                 QueryClientProvider + StrictMode
├── App.tsx                  三栏外壳
├── App.css
├── lib/
│   ├── api.ts               Tauri 命令类型化封装（camelCase 载荷）
│   └── format.ts            日期 / 字节 / 发件人格式化
├── stores/selection.ts      Zustand：账户 / 文件夹 / 邮件选择
├── features/
│   ├── accounts/
│   │   ├── AccountForm.tsx      新增账户（实时服务商识别，密码即时清空）
│   │   ├── AccountList.tsx      已配置账户
│   │   └── useProviderLookup.ts 去抖后的识别查询
│   └── inbox/
│       ├── useInbox.ts          TanStack Query hooks
│       ├── InboxToolbar.tsx     账户选择 + 信箱名 + 同步
│       ├── MessageList.tsx      虚拟滚动列表
│       └── MessageDetail.tsx    详情（净化后 HTML）
```

---

## P1–P4

| 阶段 | 内容 | 状态 |
| --- | --- | --- |
| P1-01 | **FTS5 全文检索**（schema v3 + `search_messages`） | ✅ 12/12 + 5/5 |
| P1-02 | **多账户 + 统一收件箱**（`list_folders` / `list_unified_messages`） | ✅ 后端 13/13 |
| P1-03 | **搜索界面 + 统一收件箱界面**（文件夹树 / 搜索框 / ListMode） | ✅ 已实跑验证 |
| P2-01 | **发信全链路**（后端 + 命令 + 界面 + 附件 + STARTTLS） | ✅ **P2 全部收口** |
| P3-01 | **国内邮箱兼容层**（163 已入表 + 网易 `ID` 已实现；企业邮箱方案待决策） | 🟨 见 `P3-01-provider-compat.md` |
| P3-02 | OAuth | ⬜ |
| P4 | UI 打磨 + 可选 AI 摘要/待办提取 | ⬜ |

### P1 完成度：**3/3 全部落地**

### 累计测试：**193 全绿**

### 依赖记录

| 新增 | 依据 | 说明 |
| --- | --- | --- |
| `lettre` 0.11（`tokio1-native-tls`） | **ADR-0003** | SMTP 发信；按授权边界 3 事后留痕 |
| `@tanstack/react-virtual` 3.x | ADR-0001 §5.4 | P0-06 虚拟滚动 |

---

## 待用户决策 / 阻塞

> **2026-10-16 授权**：用户当晚授权免于逐项请示（限"是否需要批准"一项）。
> 边界见 `AGENTS.md`「授权例外」。重大事项仍须请示。

| 事项 | 状态 |
| --- | --- |
| ADR-0001 §5 依赖增删项 | ✅ 按「未否决即接受」执行，已记入 ADR §5.4 |
| P0-06 取正文的 `get_message` 命令 | ✅ 已批准并实现 |
| 应用图标 | ⚠️ 临时信封图标，可随时替换 |
| **QQ 授权码** | ⛔ **唯一真正的阻塞**：`fetch_recent` 真机联调必须用它 |
| **P3 企业邮箱无法自动识别** | ⏳ 需在「手动选择服务商」与「域名探测」间决策，见 `P3-01-provider-compat.md` §4 |

### 依赖关系图（P0）

```
P0-01 ✅ ─┐
P0-02 ✅ ─┼─→ P0-03 ✅ ─→ P0-04a/b ✅ ─→ P0-04c get_messages ✅
          │                                   │
          │                                   ├─→ get_message（已实现）─→ P0-06
          │                                   │
P0-07a/b ✅ ─→ P0-07c ✅ ─┬─→ P0-07d add_account ─→ P0-05 账户配置界面
                          └─→ P0-07e sync_folder
P0-08 CI（无依赖，可随时做）
```

---

## 变更记录

| 日期 | 事项 |
| --- | --- |
| 2026-10-16 | 建立 `DESIGN.md` / `AGENTS.md` / ADR-0001 / 状态板 |
| 2026-10-16 | Rust 1.99.1 与 VS Build Tools 装毕（改用 rsproxy.cn 镜像） |
| 2026-10-16 | **P0-01a / P0-01b 完成并验证**；记录 async-imap 0.12 三条实测 API 事实至 ADR-0001 §4.3 |
| 2026-10-16 | **P0-02a / P0-02b 完成并验证**；修正 message_id 去尖括号与「垃圾输入不报错」两处认知偏差 |
| 2026-10-16 | **P0-03a / P0-03b / P0-03c 完成并验证**；确认 `PRAGMA foreign_keys` 必须置于事务外 |
| 2026-10-16 | git 2.55.0 装毕并 `git init`；**P0-04a Tauri 脚手架 + 前端壳子完成** |
| 2026-10-16 | **P0-04b `list_messages` 完成并验证**；发现 `sync_folder` 同样依赖 P0-07 凭证存储 |
| 2026-10-16 | **P0-04c `get_messages` 命令完成并验证**；识别出 P0-05/P0-04d 均被 P0-07 卡住，P0-06 缺取正文接口 |
| 2026-10-16 | **P0-07a/b 加密核心完成**（ADR-0002，用户确认方案）；aes-gcm 0.11 的 `AeadInOut` 新 API 已核实 |
| 2026-10-16 | **P0-07c 完成**：`accounts.secret_encrypted` + `PRAGMA user_version` 迁移机制 |
| 2026-10-16 | **P0-07d `add_account` 完成**；`provider::find_by_id` 补齐 `sync_folder` 的前置 |
| 2026-10-16 | **P0-07e `sync_folder` 完成**：三段式编排规避 `MutexGuard` 跨 `.await`；`upsert_messages` 整批单事务 |
| 2026-10-16 | **P0-06 完成**：`get_message` + `list_accounts` 命令；前端三栏界面、虚拟滚动列表、净化后渲染详情 |
| 2026-10-16 | **P0-05 完成**：`resolve_provider` 命令（用户批准）+ 账户表单/列表，实时识别、密码即时清空 |
| 2026-10-16 | **P0-08 完成**：CI 工作流（矩阵 x64/ARM64）；`cargo clippy -D warnings` 本地通过；x64 release 构建实跑 |
| 2026-10-16 | **端到端实跑验证**：真实窗口启动、WAL 生效、密钥文件生成、库结构正确、`list_accounts` IPC 往返成功；修掉一处过期 UI 文案 |
| 2026-10-16 | **进入 P1**：**P1-01 FTS5 全文检索完成**（schema v3）；实测确认 unicode61 搜不到中文子串、trigram 搜不到两字查询，故配 LIKE 回退 |
| 2026-10-16 | **P1-02 后端完成**：`list_folders` + `list_unified_messages`；两个可选过滤条件用一条 SQL 覆盖四种组合 |
| 2026-10-16 | **P1-03 完成，P1 闭环**：搜索命令 + 文件夹树 + 搜索框；**实跑验证 v2→v3 迁移在真实数据库上成功**；修掉「提示指向不存在按钮」的不一致 |
| 2026-10-16 | **性能验收实测**：1000 封种子数据下最慢 6.14 ms（预算 200 ms）；搜索与 LIKE 回退的性能风险解除；修正一处「零命中也能通过」的假阳性用例 |
| 2026-10-16 | **真实数据实跑**：1260 封仿真数据灌入真实库；文件夹计数、列表渲染、未读样式、附件徽标全部正确；**端到端验证 XSS 净化拦住 `<script>` 与 `onerror`**；FTS 触发器对外部写入同样生效 |
| 2026-10-16 | **进入 P2**：**P2-01a SMTP 发信核心完成**（ADR-0003 记录 lettre 选型）；测试只覆盖建连之前的校验路径，**真实投递未验证** |
| 2026-10-16 | **P2-01b HTML 正文完成**：`multipart/alternative`；把 `build_message` 公开以脱离网络验证 MIME 组装（+5 用例）。**教训**：`build_message` 私有导致新功能零覆盖；PowerShell `Set-Content` 两次弄坏 UTF-8 源码与文档，已从 git 恢复 |
