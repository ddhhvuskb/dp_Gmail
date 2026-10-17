# 交接说明

> **给下一个会话/开发者。** 本文件是接手入口，读完它再读 `STATUS.md`。
> 最后更新：**2026-10-17**（P0–P2 收口；P3-01 完成；P4-03 八项用户需求做完七项，手机版已决定暂缓）

---

## 1. 一句话现状

**这是一个能真收发邮件的桌面邮件客户端了。**

- 后端：P0-01…P0-08、P1、P2 全部完成；**`cargo test` 202 通过 / 0 失败 / 11 个真机用例默认 `#[ignore]`**
- 真机验证：**QQ / 163 / 新浪**三家都做过真登录 → `SELECT`/`FETCH` → 解析体检 → 自寄 HTML+附件闭环
- 前端：主题（4 套基调 × 5 种重点色）、中/日/英多语言、设置面板、每 15 秒自动收信
- 未完成：P3-02（OAuth，需你注册应用）、P4-02（前端单测，需你批准 vitest）
- **已决定不做**：手机版移植（用户 2026-10-17 决定）；正文翻译（用户撤回）

---

## 2. 先读什么

| 顺序 | 文件 | 作用 |
| --- | --- | --- |
| 1 | 本文件 | 现状、阻塞、从哪继续 |
| 2 | `docs/tasks/STATUS.md` | 进度总览、未验证清单、实跑记录、变更日志 |
| 3 | `AGENTS.md` | **行为总纲**：硬性规则、粒度限制、验证债、文档审计、Windows 工具规范 |
| 4 | `DESIGN.md` | 设计契约（权威，冲突时以它为准） |
| 5 | `docs/adr/` | 五个技术决策的记录与理由 |
| 6 | `docs/tasks/P*.md` | 各任务的接口契约、设计决定、**尚未验证清单** |

---

## 3. 已完成

| 阶段 | 内容 |
| --- | --- |
| P0-01~08 | 服务商识别、IMAP 连接、MIME 解析、HTML 净化、SQLite 表、Tauri 命令、账户界面、邮件列表+详情、凭证加密、CI 工作流 |
| P1 | FTS5 全文检索（含 LIKE 回退）、多账户 + 统一收件箱、搜索界面 |
| P2 | 发信全链路：纯文本 / HTML / 附件 / STARTTLS + 写信界面 + 回复转发 |
| P3-01 | 服务商表 **11 条**（8 家个人邮箱 + 3 家企业邮箱）；网易 `ID` 命令；企业邮箱手动选择（ADR-0004） |
| P4-03（用户追加需求） | 见下表 |

### P4-03 七项状态（第八项已决定不做）

| 顺序 | 项 | 状态 |
| --- | --- | --- |
| 1 | 主题切换（明/暗） | ✅ |
| 2 | UI 现代化（平色面板配配套色 + 小高光） | ✅ 真机 11/11 |
| 3 | 多语言（中/日/英） | ✅ 真机 23/23，决策见 ADR-0005 |
| 4 | 多套主题 + 重点色（4×5） | ✅ 对比度 20/20 |
| 5 | ~~邮件正文翻译~~ | ⛔ **用户已撤回，不得开工** |
| 6 | 手机版移植 | ⏸ **用户已决定暂缓**，不是待办 |
| 7 | 设置面板 + 关于我们 + 联系方式 + 隐私政策 | ✅ 真机 19/19 + 3/3 + 6/6 |
| 8 | 每 15 秒自动收信 | ✅ 间隔实测 14.2–15.7 秒 |

### 额外做的

IMAP 侧 STARTTLS、网易 `ID`、性能实测（1000 封滚动零掉帧、搜索最慢 2.11 ms）、
真实数据实跑、XSS 端到端验证、真实网络连接层验证、真机收发验证、
隐私政策（`docs/privacy/PRIVACY.md`）。

---

## 4. 未完成 / 阻塞

| 项 | 状态 | 卡在哪 |
| --- | --- | --- |
| **P3-02 OAuth** | ⬜ 未开始 | 需**你**去服务商侧注册应用拿 client id/secret；且要新增 HTTP 客户端依赖 + 改 DB schema。设计已写在 `P3-02-oauth-design.md` |
| **P4-02 前端单测** | ⬜ 未开始 | 仓库里**一个前端测试都没有**，也没有测试框架。补＝新增 vitest 依赖＝**须你批准**。见 `P4-02-frontend-tests.md` |
| **ARM64 构建** | ⛔ 本地做不了 | MSVC 的 ARM64 库装不上（`setup.exe modify --quiet` 直接退出）；仓库**没有 git 远端**，CI 从未跑过。要么交互式装 VS 组件，要么配远端让 CI 跑 |
| P4-01 AI 摘要/待办 | ⬜ 未开始 | 需你先定三件事：用哪个模型、正文是否允许发给第三方、摘要是否入库 |
| 企业邮箱真实登录 | 🟨 | 地址已 TLS 实测核实，但**没有真实企业邮箱账号**，从未登进去过 |
| 126 / yeah / 139 / 189 / 阿里云 | 🟨 | 只到连接层（走到协议层被拒），**没有成功登录过** |
| 大信箱同步耗时 | ⬜ | 没测过上千封文件夹的完整同步时长 |

---

## 5. 三个「已经验证到什么程度」的分界

这三条最容易被误读，单列：

1. **连接层 ≠ 登录成功。** 11 家服务商的地址都验过能连上并收到协议层拒绝，
   但**只有 QQ / 163 / 新浪三家真的登进去过**。
2. **企业邮箱的证书是实打实验的**（主机名匹配），但「连得上」和「登得进」是两件事。
3. **英文/日文界面下的后端报错仍是中文**（ADR-0005 的取舍）。这条的**表现**没实测过，
   只有代码事实。要验需一次故意失败的同步。

---

## 6. 别忘了这些坑

### 6.1 Windows / PowerShell

- **不要用 PowerShell 改 UTF-8 源码或文档** —— `Set-Content -Encoding UTF8` 会把中文写成乱码。
- **不要用 PowerShell 的 grep 输出判断文件是否损坏** —— PS 5.1 控制台按 GBK 解码，读 UTF-8 必然乱码，哪怕文件完全正常。用 `read` 工具确认。
- **提交信息含引号时用 `git commit -F <文件>`**。
- `pwsh` / `git` / `cargo` 不在默认 PATH 上：shell 是 Windows PowerShell 5.1，
  git 在 `C:\Program Files\Git\cmd\git.exe`，cargo 在 `%USERPROFILE%\.cargo\bin`。
- **别对应用用 `Stop-Process -Force`**：会留下 `msedgewebview2.exe` 孤儿占住
  `%LOCALAPPDATA%\com.dpgmail.desktop\EBWebView`，之后**任何**启动都静默 panic
  （exit 101，HRESULT `0x8007139F`），窗口根本不出现。
  用 `$p.CloseMainWindow()` 正常关闭（实测零残留）。详见 `AGENTS.md` 工具注意事项第 5 条。

### 6.2 磁盘占用：22 GB 是构建缓存，不是邮件

2026-10-17 用户报「占 22.8 GB」。实测定位：

| 位置 | 大小 | 是什么 |
| --- | --- | --- |
| `src-tauri/target/debug` | **20.49 GB** | Rust 调试构建缓存（可随时删，删了下次编译慢几分钟） |
| `src-tauri/target/release` | 1.98 GB | 正式构建缓存（当前桌面 exe 的来源） |
| `src-tauri/target/x86_64-pc-windows-msvc` | 1.10 GB | `tauri build` 旧产物，已过期 |
| `%APPDATA%\com.dpgmail.desktop` | **0.01 GB** | **真正属于你的东西：邮件 + 加密凭证 + 密钥** |

**为什么一句代码的项目能占 20 GB**：`src-tauri/tests/` 下有 28 个集成测试二进制，
debug 构建里每一个都独立完整链接整棵依赖树，并各自生成 PDB 调试符号 ——
光 `deps` 里就有 **80 个 >100 MB 的 .pdb**。每次 `cargo test` 还会积累不同哈希的旧产物
（实测攒了 **110 个测试 .exe**、3566 个文件）。

清理（已做过一次，可重复执行）：

```powershell
Remove-Item "D:\dp_Gmail\src-tauri\target\debug" -Recurse -Force
```

**想永久瘦身**（需用户同意再改，属 `Cargo.toml` 变更）：

```toml
[profile.dev]
debug = 1   # 只要行号表，不要完整调试符号；体积通常降一个数量级
```

代价：debug 构建的 panic 回溯里没有变量与类型信息。`debug = 0` 更小但回溯基本没用。

### 6.3 两个 exe 别搞混：`target/debug` 那个**不能双击**

| 文件 | 双击能不能用 | 为什么 |
| --- | --- | --- |
| 桌面的 `dp_Gmail.exe` | ✅ 能 | 正式构建，前端已打包进 exe，不依赖任何本地服务 |
| `src-tauri/target/debug/dp-gmail.exe` | ❌ **不能** | dev 构建，前端从 `http://localhost:1420` 加载（`devUrl`） |

双击 dev 构建时的**具体症状**（2026-10-17 用户实际看到的）：
窗口里是**浏览器的错误页，显示「拒绝访问」** —— 因为它去访问
`http://localhost:1420`，而 Vite dev 服务没启动。

dev 构建只能在项目目录里用一条命令跑起来，那条命令会同时拉起 Vite 与 Rust：

```powershell
npm run tauri dev
```

**所以：给用户交付/让他自己打开时，永远给 release 产物；dev 产物只用于开发。**

### 6.4 真机调试（WebView2）

- 用 `TAURI_CONFIG` 环境变量注入 `additionalBrowserArgs: "--remote-debugging-port=9222"`，
  再用 CDP 驱动。**`WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` 无效** —— wry 总会自己调
  `set_additional_browser_arguments()` 把它盖掉。
- `TAURI_CONFIG` 是 JSON Merge Patch，**数组是整体替换**，所以要把整个 `windows` 数组抄一遍。
- **`window.__TAURI_INTERNALS__.invoke` 不可写**，打补丁计数会静默失败（计数恒为 0）。
  要在外部观测，用 `Get-NetTCPConnection -OwningProcess <pid> -RemotePort 993` 采样。
  详见 `AGENTS.md` 工具注意事项第 4 条。
- **中文输入法会截走 `SendKeys` 的字符**，改用剪贴板粘贴。
- **坐标必须从实拍截图量**，不能凭旧截图推算。

### 6.5 开发过程本身

- **底层测通了不等于上层测通了。** 加测试前先问：这个功能从哪个入口被调用？我测的是那个入口吗？
- **规则写下来 ≠ 规则被执行。** 「每个任务文档必须写明未验证边界」加进去之后，四轮才发现 4 份最早的文档根本没写。
- **行数上限不会自己暴露。** 2026-10-17 一次查出 4 个组件超 100 行（114/112/111/102），没有一个是「一次写太长」，全是几轮各加一两行堆的。已写进 `AGENTS.md` 当固定命令。
- **假通过比失败更危险。** 同一天出现过两次：一次是选择器不唯一，「生效日期」取到的其实是「已开启」；一次是断言写反（检查切换**后**的按钮文字却按切换**前**预期）。两次都是**测试红了/绿了但结论不可信**。

---

## 7. 建议的下一步

按「需要你决定」排序，不是按工作量：

1. **Android 移植（进行中）** —— 方案已定：**桌面 native-tls，Android rustls**（C3）。
   落地细节、Cargo 配置、必须接受的行为差异、验证要求，全部写在
   `docs/adr/ADR-0006-android-port.md` **§9.6**，照它做即可。
   当前状态：工具链齐（SDK/NDK/JDK/Perl 都在），两个模拟器已建（D 盘），
   `tauri android init` 已跑过，但**还没有 APK**，单栏布局也**还没做**。
2. **P4-02 前端单测要不要做**（需批准 vitest）—— `drafts.ts`、`listLabels.ts`、`format.ts` 三处纯逻辑零覆盖。
3. **P3-02 OAuth** 要不要做（需你去 Azure/服务商侧注册应用）。调研到
   [`tauri-plugin-auth-session`](https://docs.rs/crate/tauri-plugin-auth-session/0.2.1)
   是 Tauri 生态里现成的「开浏览器 + 等回调」插件，但属新依赖 + 要改 DB schema 存 token。
4. **ARM64** 怎么办：装 VS 组件，还是配 GitHub 远端让 CI 跑。
5. 剩下的都是验证债，不是功能缺口 —— 见 §4。

---

## 8. 仓库状态

```
分支      main
追踪文件  207
测试      202 通过 / 0 失败 / 11 ignored（28 个测试二进制）
质量      cargo fmt --check / clippy -D warnings / tsc --noEmit / prettier --check 全部干净
新增依赖  lettre 0.11（ADR-0003）、@tanstack/react-virtual（ADR-0001 §5.4）
   注意    前端多语言与多主题**零新增依赖**（ADR-0005）
验证截图  docs/verification/：应用运行、真实数据、XSS 拦截、搜索、统一收件箱、
         主题四套（明/护眼米/暗/深空蓝）、中英日界面、面板配色、设置面板
git 远端  ⚠️ **没有**。所有提交只在本机，CI 从未跑过
```

### 数据目录

`%APPDATA%\com.dpgmail.desktop\`（`dp_gmail.sqlite`、`-wal`、`-shm`、`key.json`）。

当前库里是**真实的 1 个 163 账户 + 5 封真实邮件**，此前的 1260 封仿真数据已备份到
同目录 `backup\` 后清除。**不要随手清库** —— 里面有用户真实账户的密文凭证。

### 凭证

只在环境变量里：测试用 `DP_GMAIL_TEST_USER` / `DP_GMAIL_TEST_SECRET`，
CDP 调试用 `DP_USER` / `DP_SECRET`。**仓库历史已扫描确认无泄漏。**
