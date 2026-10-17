# P3-02 OAuth 接入：决策记录与实测基线

> 2026-10-17。**尚未开工。** 本文只记录事实、路线与待决策项。
> 开工前需要用户批准「新增依赖」与「令牌存储」两项 —— 见 §6。

---

## 1. 实测基线（2026-10-17，未使用任何凭证）

| 端点 | 结果 |
| --- | --- |
| `imap-mail.outlook.com:993` | `* CAPABILITY IMAP4 IMAP4rev1 AUTH=XOAUTH2 LOGINDISABLED SASL-IR …` |
| `outlook.office365.com:993` | 同上 |
| 对 IMAP 发 `LOGIN <假凭证>` | `a2 NO Basic authentication is disabled.` |
| `smtp-mail.outlook.com:587`（STARTTLS **之后**的 EHLO） | `250-AUTH LOGIN XOAUTH2` |
| `smtp.office365.com:587`（同上） | `250-AUTH LOGIN XOAUTH2` |

**两条结论：**

1. **基础认证在 IMAP 端已被明确关闭。** `LOGINDISABLED` 是服务器告诉客户端
   「不要发 `LOGIN`」。任何密码/应用密码方案对 Outlook **一律无效** ——
   所以 `outlook.com` / `hotmail.com` **不得**按密码登录加进 `PROVIDERS`。
2. **`XOAUTH2` 在 IMAP 与 SMTP 两端都被广告。**

> ⚠️ **第 2 条的边界**：只观察到「服务器广告了该机制」，**没有用真实 token 走通过**。
> 广告 ≠ 放行。不要把它当作「OAuth 一定可用」的证明。

---

## 2. 与一处外部说法的分歧

有说法称「个人账户不支持 SMTP OAuth，唯一出路是 Graph API」。

**我们实测到的 SMTP EHLO 里明确有 `XOAUTH2`**，与该说法不一致。

两种可能，目前无法区分：

- (a) 该说法针对的是更早的时间点或特定账户状态；
- (b) 服务器广告但实际拒绝（即上面说的「广告 ≠ 放行」）。

**在拿到真实 token 之前，不要基于任何一种说法做架构决定。**
这一点必须先探测清楚，否则会选错路线。

---

## 3. 两种 OAuth 流程（决定技术路线）

| | 授权码流程（delegated） | 客户端凭据流程（client credentials） |
| --- | --- | --- |
| 令牌代表谁 | **具体用户** | 应用自身，无用户上下文 |
| 需要什么 | 用户在浏览器登录并授权 | 租户管理员授予应用权限 |
| IMAP/SMTP XOAUTH2 | ✅ 可用 | ❌ 不可用（协议要求代表具体用户） |
| Graph `/me/sendMail` | ✅ | ❌（用 `/users/{id}/sendMail`） |
| 个人账户（outlook.com） | ✅ | ❌ **没有租户管理员** |

**对个人 Outlook.com 账户，只有授权码流程一条路。** 这一点没有变通余地。

---

## 4. 两条技术路线

### 路线 A：IMAP/SMTP + XOAUTH2

沿用现有 `ImapSession` / lettre 栈，只是把「密码」换成「Access Token + SASL XOAUTH2」。

- 优点：**改动局限在认证层**，`fetch_recent` / `send_mail` / 存储 / UI 全部不动；
  与现有 8 家密码型服务商共用同一套代码路径。
- 缺点：需自己处理 token 过期与刷新；微软长期方向是 Graph。

### 路线 B：Microsoft Graph API

发信 `POST /me/sendMail`，读信走 Graph 的消息接口。

- 优点：微软主推方向；不受传统协议策略影响。
- 缺点：**是一条平行的第二条收发链路** —— 需要新的 HTTP 客户端、新的
  DTO、新的映射到 `NewMessage` 的逻辑、新的错误类型。
  等于在 `mail/` 里再造一个 provider 家族，与 `async-imap` + `lettre` 并存。

> **这不是「加个认证」的工作量，是「加一套收发实现」的工作量。**
> 而且它绕开了 `DESIGN.md` 已锁定的 `async-imap` + `lettre` 栈，
> 属于需要新 ADR 的重大变更。

---

## 5. 无论哪条路线都缺的东西

| 缺什么 | 说明 |
| --- | --- |
| **HTTP 客户端** | 取 token 必须发 HTTPS POST。当前 `Cargo.toml` **没有** HTTP 客户端（`reqwest` 等一律未引入） |
| **令牌存储** | Access / Refresh Token 与密码同等级敏感，必须走 `security/` 的 AES-256-GCM，且**要能刷新**。这会动 `accounts` 表结构 |
| **浏览器授权回调** | 授权码流程需要打开系统浏览器 + 本地回环接收 `redirect_uri`。当前无此能力 |
| **用户自己的应用注册** | `client_id` 只能由账号持有人在 Entra ID 注册。我做不到 |

---

## 6. 开工前必须由用户确认的三件事

1. **新增依赖**：至少一个 HTTP 客户端（如 `reqwest` 或 `ureq`）。
   `要求.txt` 第 1 条「技术栈锁定」+ `AGENTS.md` 第 1 条「新依赖必须人工批准」。
2. **数据库表结构变更**：令牌需要落库且可刷新。`AGENTS.md` 明令「变更数据库表结构」须人工确认。
3. **选择路线 A 还是 B**：A 是认证层改造，B 是第二套收发实现。
   两者的工作量与长期维护成本差一个数量级。

---

## 7. 未验证 / 不采信

| 项 | 状态 |
| --- | --- |
| 个人 Outlook.com 能否用 XOAUTH2 真正收发 | ⛔ 只看到广告，未用真 token 走通 |
| Graph API 在本项目中的可行性 | ⛔ 未做任何 PoC |
| Azure 中国（21Vianet）端点差异 | ⛔ 未验证。若支持国内用户需单独处理 —— 端点、scope、token 端点全部不同，混用会得到 `invalid_scope` 或 404 |
| 外部推荐的开源参考实现 | ⛔ **未核实**。不采信未经验证的仓库推荐 |
