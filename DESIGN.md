# DESIGN.md — 本地优先跨平台桌面邮件客户端

> 本文件为项目唯一设计契约。任何实现不得偏离本文件；如需变更，先走 `docs/adr/`。
> 来源：桌面 `要求.txt`（总纲）+ `要求1.txt`（设计契约）。原文快照见 `docs/source/`。
> **权威顺序：`DESIGN.md`（实现契约）> `docs/source/`（原始快照）。** 桌面原件仅供用户修改，不作为实现依据。

---

## 一、目标与边界

**做**：本地优先跨平台桌面邮件客户端，统一多邮箱，国内邮箱开箱即用。

**不做**：Web / 移动端 / 服务端 / 日历 / 联系人 / PGP。

**本地优先的含义**：

- 邮件正文、索引、附件元数据全部落本地 SQLite，离线可读。
- 网络只用于 IMAP/SMTP 同步，不经任何自建服务端中转。
- 凭证本地加密存储，密钥文件在用户目录，不随仓库分发。

---

## 二、技术栈锁定（不得变更）

| 层级 | 选型 | 版本约束 |
| --- | --- | --- |
| 框架 | Tauri | v2，`>=2.0` |
| 后端 | Rust | stable（MSVC toolchain） |
| 前端 | React + TypeScript | React 19+ |
| 状态 | Zustand + TanStack Query | `>=5.0` |
| 数据库 | SQLite (rusqlite) + FTS5 | bundled |
| 构建 | GitHub Actions 矩阵 | Windows x64 + ARM64 |

**新增依赖必须人工批准，写入 `docs/adr/`。** 已批准的 P0 基线见 `docs/adr/ADR-0001-tech-stack-and-p0-dependencies.md`。

---

## 三、模块边界与接口

### 3.1 后端目录

```
src-tauri/src/
├── mail/        IMAP/SMTP/解析/provider
├── storage/     表结构/CRUD/FTS5
├── security/    加密/HTML净化
├── ai/          只读，可选
└── commands/    Tauri桥，只做参数校验和转发
```

### 3.2 前端目录

```
src/
├── components/  通用UI
├── features/    accounts/inbox/compose/search/settings
├── stores/      Zustand
└── lib/         工具
```

### 3.3 接口规则

- 跨模块调用必须走已定义函数签名，**禁止访问内部数据结构**。
- `commands/` 不写业务逻辑，只做转发。
- `ai/` 只能读，不能发信/删除/修改；输出需人工确认才可写。

### 3.4 调用契约表

| 调用方 | 被调用方 | 接口方式 | 定义位置 |
| --- | --- | --- | --- |
| 前端 → Rust | Tauri Commands | `#[tauri::command]` | `commands/` |
| commands → mail | Rust 函数 | 同步/异步 | `mail/mod.rs` |
| mail → storage | Rust 函数 | 同步 | `storage/mod.rs` |
| ai → storage | 只读查询 | 只读函数 | `storage/message_store.rs` |

---

## 四、任务优先级

### P0 — 单 QQ 邮箱同步展示

| 任务 | 内容 | 关键约束 | 禁止 |
| --- | --- | --- | --- |
| P0-01 | IMAP 连接（async-imap，TLS，超时 30s） | 必须支持 TLS，超时 30s | 硬编码服务器地址 |
| P0-02 | MIME 解析（mail-parser，RFC2047 解码，unknown-8bit 处理） | 主题 RFC2047 解码，HTML 净化 | 手动实现编码转换 |
| P0-03 | SQLite 表（accounts/folders/messages/attachments，UID 去重） | 时间用 Unix 整数，联合唯一索引 | 用 ORM，省略事务 |
| P0-04 | Tauri 命令（add_account / sync_folder / get_messages） | 只做参数校验和转发 | 写业务逻辑，返回原始 DB 行 |
| P0-05 | 账户配置界面（服务商自动识别，不存明文密码） | 自动识别服务商 | 在 localStorage 存密码 |
| P0-06 | 邮件列表 + 详情（虚拟滚动，HTML 净化后渲染） | 虚拟滚动，HTML 净化 | 直接使用 `dangerouslySetInnerHTML` |
| P0-07 | 凭证加密（AES-256-GCM，密钥文件管理） | AES-256-GCM | 硬编码密钥，ECB 模式 |
| P0-08 | CI 构建（Windows x64 + ARM64） | 矩阵构建 x64/ARM64 | — |

### P1 — 多账户 + 统一收件箱 + FTS5 搜索

### P2 — 写信/回复/转发 + SMTP + 附件

### P3 — 国内邮箱兼容层 + OAuth

### P4 — UI 打磨 + 可选 AI 摘要/待办提取

---

## 五、AI 协作规则（防作妖核心）

### 5.1 任务粒度

- 一次只生成**一个函数或组件**，函数 ≤50 行，组件 ≤100 行。
- 不得提前实现后续任务，不得生成占位代码，不得写 TODO。

### 5.2 每次 Prompt 必须包含的约束块

```
【角色】Rust/TS开发工程师
【任务】具体任务
【输入】字段名、类型、示例
【输出】函数签名、返回类型
【约束】
  - 只生成一个函数/组件
  - 指定crate/库
  - 错误处理返回Result
  - 不改已有文件
【验收】可测试标准
【禁止】本任务特有禁止项
```

### 5.3 防幻觉三层

1. 输入输出未明确 → 停下问，不猜。
2. 跨模块调用前先读接口定义，确认签名。
3. 生成后必须运行 `cargo check` 或 `tsc --noEmit`。

### 5.4 常见 AI 错误模式与拦截

| 错误模式 | 表现 | 拦截规则 |
| --- | --- | --- |
| API 幻觉 | 调用不存在的 crate 函数 | 先查 crate 文档确认函数存在 |
| 架构漂移 | 自行增加新模块或依赖 | 新依赖必须在 `Cargo.toml` 已声明 |
| 上下文遗忘 | 覆盖之前已完成的代码 | 改文件前先读当前内容 |
| 过度设计 | 简单任务引入复杂抽象 | 函数 ≤50 行，组件 ≤100 行 |
| 编码不一致 | 命名风格混用 | 强制 rustfmt + prettier |
| 硬编码 | 服务器地址/端口写死 | 服务商配置从 `provider.rs` 读 |

---

## 六、人工介入检查点

以下必须暂停确认：

- 新增依赖
- 修改接口签名
- 变更数据库表结构
- 安全相关代码（加密/认证）
- 任务描述有歧义

---

## 七、目录与文档

```
AGENTS.md           AI行为总纲
DESIGN.md           本文件
docs/adr/           架构决策记录
docs/tasks/         任务清单
```

**文件命名**：

| 类型 | 规范 | 示例 |
| --- | --- | --- |
| Rust | `snake_case.rs` | `provider.rs` |
| React 组件 | `PascalCase.tsx` | `MessageList.tsx` |
| Rust 测试 | `*_test.rs` | `mime_parse_test.rs` |
| TS 测试 | `*.test.tsx` | `MessageList.test.tsx` |
| 任务文档 | `P{阶段}-{序号}-{简述}.md` | `P0-01-imap-connect.md` |

---

## 八、验收总则

| 检查项 | 标准 |
| --- | --- |
| 编译 | `cargo check` + `tsc --noEmit` 无错误 |
| 测试 | 核心逻辑有单元测试（MIME/编码/搜索） |
| 安全 | 无硬编码密码，无直接 `dangerouslySetInnerHTML` |
| 性能 | 1000 封列表滚动不卡，搜索 <200ms |
| 平台 | Windows x64/ARM64 均可构建 |

---

## 九、AI 最终提醒

1. 不做架构决策，走 ADR。
2. 不做技术选型，不得新增依赖。
3. 不猜，不明确就问。
4. 不贪，一次一个任务。
5. 必须验证，不凭"看起来对"。
6. 每完成一个任务，更新状态和验证结果。
