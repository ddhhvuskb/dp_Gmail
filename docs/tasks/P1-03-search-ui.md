# P1-03 搜索界面 + 统一收件箱界面

- **阶段**：P1
- **依赖**：P1-01（`search_messages`）、P1-02（`list_folders` / `list_unified_messages`）
- **状态**：✅ 已完成并实跑验证（2026-10-16）
- **关键约束**：`DESIGN.md` 性能验收「1000 封列表滚动不卡」

---

## 1. 交付物

| 层 | 内容 |
| --- | --- |
| `commands/search.rs` | `search_messages` 命令（P1-01 只做到 storage 层，本轮补上桥接） |
| `stores/selection.ts` | 重写为**判别联合** `ListMode` |
| `features/inbox/useInbox.ts` | `useFolders` / `useListRows`，三种来源归一成 `ListRow` |
| `features/inbox/FolderTree.tsx` | 文件夹树（含未读徽标、统一收件箱入口） |
| `features/inbox/SearchBox.tsx` | 搜索框 |
| `App.tsx` / `MessageList.tsx` / `App.css` | 接入 |

## 2. 设计决定

### 2.1 `ListMode` 用判别联合，不用几个并列布尔

```ts
type ListMode =
  | { kind: "none" }
  | { kind: "folder"; folderId: number; label: string }
  | { kind: "unified" }
  | { kind: "search"; query: string };
```

用 `isUnified: boolean` + `searchQuery: string` + `folderId: number | null` 三个字段的话，
`{ isUnified: true, folderId: 3, searchQuery: "x" }` 这种**非法状态是能写出来的**。
判别联合让它根本表达不出来，TypeScript 也会强制 switch 穷尽。

### 2.2 三种来源在前端归一成 `ListRow`

文件夹视图返回 `MessageSummary`，统一收件箱返回 `UnifiedMessageSummary`，
两者字段不全一样。归一放在 `useListRows` 里做**一次**，
而不是让 `MessageList` 认三种类型 —— 三处重复迟早漂移。

### 2.3 搜索按提交触发，不做输入即搜

`search_messages` 在短查询（<3 字符）下走 **全表扫描**（见 P1-01 §2.1）。
每敲一个字符打一次命令会白白烧 CPU。真要做即时搜索，
应当像服务商识别那样先加 300ms 去抖。

---

## 3. 实跑验证

`npm run tauri dev`，截图存档 `docs/verification/p1-search-and-folders.png`。

![运行中的界面](../verification/p1-search-and-folders.png)

### 3.1 v2 → v3 迁移在**真实数据库**上跑通

启动前 `user_version = 2`，启动后：

```
user_version = 3
FTS 相关表: messages_fts, messages_fts_config, messages_fts_data, messages_fts_docsize, messages_fts_idx
FTS 触发器: messages_fts_ad, messages_fts_ai, messages_fts_au
基表: accounts, attachments, folders, messages
FTS 查询可执行, 命中 = 0
```

四张基表原封不动（**迁移不丢数据**），虚表与三个触发器按预期建出。

### 3.2 新命令的 IPC 往返已验证

界面显示「还没有同步过文件夹。」—— 这是 `FolderTree` 收到**空数组**的渲染。
若 `list_folders` 失败，`App.tsx` 会走 `folders.isError` 分支渲染红色错误。

故这一屏同时证明了 `list_folders` 已注册、capabilities 未拦截、camelCase 参数解析正确。

### 3.3 实跑抓到的不一致

中栏在空状态下提示「同步一个文件夹，或点**「统一收件箱」**」，
但 `FolderTree` 原本在 `folders.length === 0` 时**提前 return**，
那个按钮根本没渲染 —— 提示指向了一个不存在的按钮。

已改为：统一收件箱按钮**始终**渲染，空状态提示另起一行。

> 这是第二次由「把界面真跑起来」而非类型检查/单测发现的问题。
> 上一次是 P0 阶段残留的过期文案。

---

## 4. 验收结果

| 检查项 | 实测 |
| --- | --- |
| `cargo check --all-targets` | exit 0，零警告 ✅ |
| `cargo fmt --check` / `cargo clippy -D warnings` | exit 0 ✅ |
| `tsc --noEmit` | exit 0 ✅ |
| `npm run build` | exit 0，300.88 kB / gzip 92.61 kB ✅ |
| `cargo test --all-targets` | **145/145 通过** ✅ |
| 真实窗口实跑 | 见 §3 ✅ |

## 5. 尚未验证

- **真实邮件数据下的表现**。全部界面元素都只在**空库**上跑过 ——
  邮件列表、虚拟滚动、统一收件箱聚合、搜索结果、详情渲染，
  都还没有真实数据流过。需要 QQ 授权码。
- 「1000 封列表滚动不卡」「搜索 <200ms」两条验收标准仍未实测。
