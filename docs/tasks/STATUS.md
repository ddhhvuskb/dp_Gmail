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
| P0-04d | `commands::sync_folder` | ⛔ | — | **依赖 P0-07 凭证存储** |
| P0-05 | 账户配置界面 | ⛔ | `tsc --noEmit` | **依赖 P0-07（密码无处可去）** |
| P0-06 | 邮件列表 + 详情（虚拟滚动，HTML 净化） | ⬜ | `tsc --noEmit` | 详情缺一条取正文的命令，待决策 |
| P0-07 | 凭证加密 AES-256-GCM + 密钥文件管理 | ⬜ | `cargo test` + **人工确认** | — |
| P0-08 | CI 构建 Windows x64 + ARM64 | ⬜ | workflow 语法校验 | — |

### 累计测试：**53 全绿**

| 测试目标 | 用例数 |
| --- | --- |
| `provider_test` | 7 |
| `imap_client_test` | 2 |
| `parser_test` | 9 |
| `sanitize_test` | 5 |
| `storage_test` | 8 |
| `storage_open_test` | 5 |
| `message_store_test` | 7 |
| `list_messages_test` | 6 |
| `commands_test` | 4 |

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
    │   ├── provider.rs      服务商表 + resolve_by_email
    │   ├── imap_client.rs   connect_and_login（TLS + 30s）
    │   └── parser.rs        parse_message
    ├── security/
    │   └── sanitize.rs      sanitize_html
    └── storage/
        ├── error.rs         StorageError
        ├── schema.rs        init_schema
        ├── message_store.rs upsert_message / list_messages
        └── mod.rs           open_database
```

前端（最小壳子，P0-05 起填充）：

```
package.json  vite.config.ts  tsconfig.json  index.html
src/main.tsx  src/App.tsx  src/vite-env.d.ts
```

---

## P1–P4

| 阶段 | 内容 | 状态 |
| --- | --- | --- |
| P1 | 多账户 + 统一收件箱 + FTS5 搜索 | ⬜ |
| P2 | 写信/回复/转发 + SMTP + 附件 | ⬜ |
| P3 | 国内邮箱兼容层 + OAuth | ⬜ |
| P4 | UI 打磨 + 可选 AI 摘要/待办提取 | ⬜ |

---

## 待用户决策 / 阻塞

| 事项 | 状态 |
| --- | --- |
| **P0-07 加密方案需人工确认**（密钥文件位置、格式、是否口令保护） | ⛔ **当前关键路径**：卡住 P0-04d / P0-05 |
| **P0-06 详情缺取正文的命令**（新增接口需确认） | ⏳ 待决策 |
| ADR-0001 §5 依赖增删项（删 keyring/argon2/hkdf；增 ammonia 等） | ⏳ 待过目 |
| QQ 授权码（P0-01 真机联调） | ⏳ 用户稍后提供 |
| 应用图标 | ⚠️ 我用 System.Drawing 生成的临时信封图标，可随时替换 |

### 依赖关系图（P0）

```
P0-01 ─┐
P0-02 ─┼─→ P0-03 ─→ P0-04b/c ─→ P0-06 ─→ (需新增取正文命令)
P0-04a ┘                │
                        └─→ P0-04d sync_folder ─┐
                                                 ├─→ 都要 P0-07 的凭证存储
P0-05 账户配置界面 ──────────────────────────────┘
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
