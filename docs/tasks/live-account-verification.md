# 真机收发验证（live account verification）

> 2026-10-17。用户提供了真实 QQ 邮箱与授权码，
> 本项目**最大的一笔验证债**（「客户端从未收发过一封真实邮件」）就此结清。

---

## 1. 这一笔债此前为什么一直挂着

见 `live-network-verification.md`：那一轮用伪造凭证把**连接层**跑通了，
但邮箱层（`SELECT` / `FETCH`）、投递层（SMTP `DATA`）、
以及网易 `ID` 命令都还零执行 —— 因为它们全部位于「登录成功」之后。

判据没变：**没有真实跑通过的功能，不得只写「完成」。**
所以那句话一直挂着，直到真的收发成功。

---

## 2. 凭证怎么处理

**测试文件里不含任何账号或授权码。** 一律从环境变量读：

```powershell
$env:DP_GMAIL_TEST_USER   = "you@qq.com"
$env:DP_GMAIL_TEST_SECRET = "16 位授权码"
cargo test --manifest-path src-tauri/Cargo.toml --test live_account_test -- --ignored --nocapture --test-threads=1
```

三条设计约束：

| 约束 | 理由 |
| --- | --- |
| 只读环境变量，不写文件 | `DESIGN.md` 验收总则「无硬编码密码」 |
| 全部标 `#[ignore]` | CI 无凭证；但 **「N ignored」每次都会显式打印**，不是消失 |
| 缺变量时**直接失败**，不静默跳过 | 静默跳过会让「没跑」看起来像「跑过了」—— 正是本项目反复防的假阳性 |

提交后已核查：**工作区文件与提交历史中均无授权码**。

---

## 3. 实测结果

### 3.1 收信：`live_fetch_recent_from_real_inbox`

真登录 → `SELECT INBOX` → `FETCH BODY.PEEK[]` → MIME 解析。

```
[收信] aabb1029384756@qq.com → 拉到 4 封
[收信] UID 333 | 主题 Some("你的一次性代码") | 发件人 Some("account-security-noreply@accountprotection.microsoft.com") | 纯文本 269 字 | 有 HTML true | 附件 0
[收信] UID 334 | 主题 Some("【NOI报名系统】退费申请结果通知") | 发件人 None | 纯文本 94 字 | 有 HTML true | 附件 0
[收信] UID 335 | 主题 Some("Pricing Update for Your .COM Domain(s)") | 发件人 Some("abnous@namesilo.email") | 纯文本 2380 字 | 有 HTML true | 附件 0
[收信] UID 336 | 主题 Some("收藏本站，避免迷路 10.13更新") | 发件人 Some("aabb1029384756@qq.com") | 纯文本 60 字 | 有 HTML true | 附件 0
```

**顺带真实执行了 `send_client_id_if_supported`** —— 它跑在 `login` 成功之后，
所以此前用伪造凭证做的连接层验证**没有覆盖到它**。QQ 的 `CAPABILITY` 确实含 `ID`，
这条分支现在真的走过了。

### 3.2 闭环：`live_send_then_receive_roundtrip`

真发一封带 HTML 与附件的邮件给账号自己：

```
[发信] 已投递：dp_Gmail 真机自检 1792194123
[闭环] 收到了自己发的邮件：UID 337 | 有 HTML true | 附件 1 个 ["dp_gmail_selftest.txt"]
```

约 40 秒到达。**SMTP `DATA` 阶段、多部分 MIME 组装、投递后解析全部真实跑通。**

### 3.3 解析体检：`live_parse_audit`

拿 30 封真实邮件（实得 31 封）逐个检查 `from` / `subject` / `date` 是否解析得出。

**第一次运行：31 封里 3 封有缺字段。** 合成数据永远撞不出这些：

| UID | 现象 | 根因 |
| --- | --- | --- |
| 309 / 334 | `from = None` | `From` 头**没有地址**，只有 RFC2047 显示名（解码为「NOI竞赛办公室」），真实地址在 **`Sender`** 头里 |
| 316 | `date = None` | 整封邮件**没有 `Date` 头** |

### 3.4 由体检驱动的一处真实修复

`parser.rs` 新增 `resolve_from()`：`From` 里取不到地址时回退到 `Sender`，
**同时保留 `From` 的显示名**（「NOI竞赛办公室」比裸地址对用户有用）。

修完复验（同一条体检、同一批真实邮件）：

```
[体检] 检查 31 封，1 封有缺字段      ← 修之前是 3 封
```

剩下那 1 封是 UID 316 的「无 `Date` 头」。`parser.rs` 里
`date_unix.unwrap_or(0)` 是**已记录的有意选择**（排到列表末尾，不会被误当成新邮件），
不算缺陷。

单测两条守门用例（`parser_test.rs`）：

- `falls_back_to_sender_when_from_has_no_address` —— 用真实那封的头部原样构造
- `prefers_from_when_it_has_an_address` —— 防止回退把正常邮件的发件人顶掉

---

## 4. 现在可以说「完成」了吗

| 项 | 状态 |
| --- | --- |
| `fetch_recent` 真实收信 | ✅ **已真实跑通**（真登录 / `SELECT` / `FETCH` / 解析） |
| `send_mail` 真实发信 | ✅ **已真实跑通**（含 `DATA` 与投递） |
| `send_client_id_if_supported` | ✅ 已真实执行（QQ 走的是「有 `ID`」分支） |
| 端到端「发出去 → 收回来」 | ✅ 已真实跑通 |
| 网易 163 的 `ID` 命令 | ⛔ **仍未在真实 163 账户上试过** —— 本次用的是 QQ，QQ 对 `ID` 不敏感 |
| 大附件 / 被服务器拒收 | ⛔ 未测 |
| 中文主题在别的客户端里的呈现 | ⛔ 只验证了自己收自己，没在第三方客户端里看过 |

**「客户端从未收发过一封真实邮件」这句话，自 2026-10-17 起作废。**

---

## 5. 顺带暴露的两处与本次无关的缺口

体检和读数过程中发现两件不属于「收发」范畴、但都真实存在的问题，
已分别记录，不在本文档内解决：

1. **列表硬顶 200 条且无翻页** —— 900 封的文件夹只能看到最新 200 封。
   见 `perf-verification.md` §6.3。
2. **网易 `ID` 命令仍未真机验证** —— 本次用户给的是 QQ 账号。
