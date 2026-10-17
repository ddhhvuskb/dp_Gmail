# P3-01 国内邮箱兼容层（调研与设计）

- **阶段**：P3
- **状态**：🟨 调研完成，**实现未开始**
- **来源**：`DESIGN.md` P3「国内邮箱兼容层 + OAuth」

---

## 1. 本轮只做调研，不写实现 —— 以及为什么

原计划是把 `provider.rs` 的服务商表从「仅 QQ」扩到国内主流邮箱。
核实服务器地址的过程中发现一个**会改变设计的前提**，必须先记下来、由用户确认后再动手。

在没有解决这个前提之前写实现，只会产出一份跑不通的表。

---

## 2. 已核实的服务器地址

### 2.1 个人邮箱（可用域名自动识别）

来源分两类：**厂商官方页**，或 **2026-10-17 的 TLS 实测**。后者更强 ——
证书主机名匹配直接证明该地址属于这家服务商，不是转述。

| 服务商 | IMAP | SMTP | 核实方式 |
| --- | --- | --- | --- |
| QQ 邮箱 | `imap.qq.com:993` | `smtp.qq.com:465` | 官方帮助页（P0-01）+ TLS 实测 |
| 网易 163 | `imap.163.com:993` | `smtp.163.com:465` | Zoho 汇总表 + TLS 实测 |
| 网易 126 | `imap.126.com:993` | `smtp.126.com:465` | **TLS 实测**，证书 `*.126.com` |
| 网易 yeah.net | `imap.yeah.net:993` | `smtp.yeah.net:465` | **TLS 实测**，证书 `*.yeah.net` |
| 新浪邮箱 | `imap.sina.com:993` | `smtp.sina.com:465` | **TLS 实测**，证书 `*.sina.com` |
| 中国移动 139 | `imap.139.com:993` | `smtp.139.com:465` | **TLS 实测**，证书 SAN 含 `imap.139.com` |
| 中国电信 189 | `imap.189.cn:993` | `smtp.189.cn:465` | **TLS 实测**，证书 `*.189.cn` |
| 阿里云（个人） | `imap.aliyun.com:993` | `smtp.aliyun.com:465` | **TLS 实测**，证书 SAN 含 `aliyun.com` |

**这六家的 `CAPABILITY` 都广告 `ID`**（126 / yeah.net 是 Coremail，与 163 同源），
故 `send_client_id_if_supported` 自动覆盖，**无需按服务商特判**。

### 2.2 企业邮箱（**无法**用域名自动识别，见 §5）

来源：<https://www.west.cn/faq/listagent.asp?unid=2132> **+ 2026-10-17 TLS 实测**

| 服务商 | IMAP | SMTP |
| --- | --- | --- |
| 腾讯企业邮 | `imap.exmail.qq.com:993` | `smtp.exmail.qq.com:465` |
| 网易企业邮 | `imap.qiye.163.com:993` | **`smtp.qiye.163.com:994`** ← 实测确认，不是常见的 465 |
| 阿里云企业邮箱（原万网） | `imap.mxhichina.com:993` | `smtp.mxhichina.com:465` |

### 2.3 仍未核实（**不得凭记忆填入**）

本节原先列的六个域名，**2026-10-17 已全部用 TLS 实测核实**（见 §2.1）。名单缩小到：

| 域名 | 为什么仍留空 |
| --- | --- |
| `sina.cn` | 新浪的另一个域名。同属一家，但**没有实测过** |
| `10086.cn` | 中国移动的另一个域名，未实测 |
| `gmail.com` / `outlook.com` | 需要 OAuth（P3-02），密码登录会被服务商拒绝 |

**同属一家不等于可以顺手套用。** 这正是
`genuinely_unverified_domains_still_resolve_to_nothing` 这条测试守着的东西 ——
「未核实就留空」的规矩没变，只是名单变短了。

### 2.4 STARTTLS 支持情况（2026-10-17 实测，非文档转述）

加新服务商时**必须实测**这一项 —— 端口号相同的两个服务商，STARTTLS 支持可能相反：

| 服务商 | IMAP 143 STARTTLS | 实测证据 |
| --- | --- | --- |
| QQ | ❌ **不支持** | 发 `STARTTLS` 回 `* BAD Command!` |
| 网易 163 | ✅ 支持 | 回 `a1 OK Begin TLS negotiation`，升级后 TLSv1.3 |

**结论：`ProviderConfig.imap_tls` 不能按「143 就是 STARTTLS」推断。**
QQ 若误配 `TlsMode::StartTls` 会直接失败。

SMTP 侧：QQ 465 隐式 ✅ / 587 STARTTLS ✅，两条均实测可用。

实测脚本与结果见 `live-network-verification.md` §2.2。

---

## 3. 第二个前提：网易 163/126 需要 IMAP `ID` 命令

把 163 加进服务商表**不是加一行数据那么简单**。

网易（163 / 126）在 `LOGIN` 之后、`SELECT` 之前要求客户端发送 IMAP `ID` 命令
（RFC 2971）。不发的话 `SELECT` 会被拒：

```
SELECT Unsafe Login. Please contact kefu@188.com for help
```

多个独立来源证实了这一点：

- [mcp-email-server#85](https://github.com/Wh1isper/mcp-email-server/issues/85)：
  「SELECT Unsafe Login due to IMAP ID command format」
- [hermes-agent#22271](https://github.com/NousResearch/hermes-agent/issues/22271)：
  「Missing IMAP ID extension support」
- [zeroclaw#2235](https://github.com/zeroclaw-labs/zeroclaw/issues/2235)：
  明确要求为 NetEase 邮箱增加 IMAP ID 支持

### 3.1 影响

| 项 | 说明 |
| --- | --- |
| 症状 | 连得上、登录成功，**但一读邮件就报 `Unsafe Login`** |
| 误判风险 | 极易被当成"密码错了"或"163 不支持 IMAP"而放弃 |
| 范围 | 163.com、126.com、yeah.net（网易系全部） |
| 不适用 | QQ 邮箱不需要（现有实现正确） |

### 3.2 实现（已完成）

已在 `mail/imap_client.rs` 的 `send_client_id_if_supported` 实现：

```rust
let capabilities = session.capabilities().await?;
if !capabilities.has_str("ID") {
    return Ok(());              // 先探测再发送
}
session.run_command_and_check_ok(r#"ID ("name" "dp_Gmail" …)"#).await?;
```

在 `login` 成功之后、返回 `Session` 之前调用。QQ 邮箱不声明 `ID` 能力，
故走 `return Ok(())` 提前返回，**行为与之前完全一致**。

#### 实现时踩到的 API 细节

`run_command_and_check_ok` 在 `async-imap` 里有**两个签名**：

| 调用载体 | 解析到 | 参数 |
| --- | --- | --- |
| `Client<TcpStream>`（STARTTLS 分支） | `Connection` 版本 | **2 个**（第二参 `None`） |
| `Session`（本函数） | `Client` 版本 | **1 个** |

同一个方法名，因载体不同而参数个数不同。两次都是编译器直接指出来 ——
**这正是「先核实 API 再写」的价值：错了能立刻知道错在哪，而不是靠猜反复试。**

#### 仍未验证

`ID` 命令的**参数格式**本身有讲究（[mcp-email-server#85](https://github.com/Wh1isper/mcp-email-server/issues/85)
的标题就是 "ID command format"）。当前用的是常见写法，
**必须在真实 163 账户上验证**，否则可能仍然被拒。

---

## 4. 关键发现：企业邮箱用客户自己的域名

上表三项**全部是企业邮箱**，它们的特点是：

> 服务商提供的是**服务器地址**，而**邮箱域名是客户自己的**（`@yourcompany.com`）。

而现有 `ProviderConfig` 的自动识别靠的是 `domains` 字段做域名匹配：

```rust
pub struct ProviderConfig {
    pub domains: &'static [&'static str],   // ["qq.com", "vip.qq.com", "foxmail.com"]
    …
}

pub fn resolve_by_email(email: &str) -> Option<&'static ProviderConfig> {
    // 按 @ 后面的域名查表
}
```

**企业邮箱没有任何可以预先写死的域名。** 无论 `imap.exmail.qq.com` 还是
`imap.mxhichina.com`，用户的地址都长成 `someone@任意公司域名`。

所以：

| 场景 | 域名自动识别是否可行 |
| --- | --- |
| 个人邮箱（qq.com / 163.com / sina.com） | ✅ 可行 |
| 企业邮箱（自有域名） | ❌ **不可行** |

---

## 5. 由此产生的设计问题（✅ 已决策：方案 A，见 ADR-0004）

`DESIGN.md` P0-05 定的是「服务商自动识别」。企业邮箱无法自动识别，
当初看起来有两条路：

### 方案 A：手动选择服务商 ← **已采纳**

`ProviderConfig` 增加企业邮箱条目，`domains` 留空；自动识别失败时
前端提示并给一个下拉列表。

**2026-10-17 已实现**。决策、地址核实方法与接口变更清单见
`docs/adr/ADR-0004-enterprise-mailbox-manual-provider.md`。

### 方案 B：域名探测（猜）← **已拒绝**

对未知域名尝试 `imap.<domain>` / `mail.<domain>` 等常见命名并连接探测。

**拒绝理由不是体验，是安全**：这是拿用户的密码去戳一个未知服务器。
猜错意味着**把凭证发给错误的服务器**，而这类错误不可撤销。

### 更正：方案 A 并不是「降级」

本节原先把方案 A 描述为「把自动识别降级为手动选择」，**这个判断不准确**。
企业邮箱在此**从来没有**自动识别可用，所谓降级是拿一个从未存在过的能力做基准。

| 域名 | 改动前 | 改动后 |
| --- | --- | --- |
| `qq.com` / `163.com`（已收录） | 自动识别 ✅ | 自动识别 ✅ **未变** |
| 企业邮箱自有域名 | **无路可走** ⛔ | 手动选择 ✅ **新增一条路** |

自动识别路径一个字节都没改。真实窗口实测印证了这点：
输入 `someone@qq.com` 不出现选择器、仍显示「已识别：QQ邮箱（imap.qq.com:993）」。

---

## 6. 尚未完成

| 项 | 状态 |
| --- | --- |
| 免费个人邮箱地址核实 | ✅ 2026-10-17 **126 / yeah.net / sina / 139 / 189 / aliyun 六项已用 TLS 实测核实并加入服务商表**；`sina.cn` / `10086.cn` 等仍未实测，留空 |
| 网易 IMAP `ID` 命令支持 | ✅ 已实现（见 §3），**真机未验证** |
| 企业邮箱的手动选择 UI | ✅ 已完成（ADR-0004）：`list_providers` + `ProviderField`，真实窗口已验证 |
| 企业邮箱地址核实 | ✅ 2026-10-17 改用 **TLS 实测**核实（证书主机名匹配），见 ADR-0004 §4 |
| OAuth（P3 后半） | ⬜ 未开始 |
| IMAP 侧 STARTTLS | ✅ 已完成（见 `P2-01-smtp-send.md` 与 `imap_client.rs`） |

---

## 7. 尚未验证

### 7.1 `ID` 参数格式：已实测（2026-10-17）

**关键前提：RFC 2971 允许 `ID` 在连接的任何状态下发送，所以不需要登录就能验证语法。**
拿代码里发的**原字符串**直接打三家网易系服务器：

```text
a1 ID ("name" "dp_Gmail" "version" "0.1.0" "vendor" "dp_Gmail")

163 / 126 / yeah.net 三家一致：
  * ID ("name" "Coremail Imap" "vendor" "Mailtech" "TransID" "IcalGQAAAJa90mqHZLwA")
  a1 OK ID completed
```

**带阴性对照**：故意把括号写坏（`a2 ID ("name"`）→ 三家一致回
`a2 BAD Parse command error`。

对照能被区分 ⇒ **那个 `OK` 是有判别力的**，不是「服务器对什么都点头」。
这条测试本身因此站得住。

顺带还学到：服务器的 `* ID` 回应自报 `"name" "Coremail Imap" "vendor" "Mailtech"`，
**从协议层面证实了 163/126/yeah.net 是同一套 Coremail 系统** ——
与 §3 里「网易系需要 `ID`」的判断互相印证。

> 边界：本次验证是**登录前**做的。代码在 `login` 成功之后才发，
> 但发的是**同一个字符串**，服务端的语法解析与认证状态无关。
> 剩余风险从「格式可能不对」缩小到「**命令被接受 ≠ 限制被解除**」。

### 7.2 其余未验证项

| 项 | 说明 |
| --- | --- |
| **`ID` 是否真能解除 `Unsafe Login`** | ⛔ 「发了 `ID` 之后 `SELECT` 是否真的不再被拒」**仍需真实账号**。命令被接受 ≠ 限制被解除 |
| 表里 8 家个人邮箱的真实登录 | 🟨 2026-10-17 **连接层 8/8 走通**（见 §7.3）；⛔ 但**成功登录只验证过 QQ 一家** |

### 7.3 连接层：表里 8 家全部走通（2026-10-17）

加服务商时最容易犯的错，是只验证「**地址**是对的」（用别的工具探一下主机通不通），
却漏掉「**我们的代码**连得上」。这两件事不是一回事 —— TLS 栈、超时、
问候语解析、`CAPABILITY` 任何一环出问题，都只在真跑时才暴露。

`live_every_personal_provider_reaches_login_rejection` **从 `PROVIDERS` 取地址**，
所以以后再加服务商会**自动**被覆盖，不需要有人记得补测试。实测 8/8：

| 服务商 | 服务器回什么 |
| --- | --- |
| qq | `Login fail. Please input email address` |
| netease163 / netease126 / netease_yeah | `LOGIN Login error or password error` |
| sina | `LOGIN authentication error` |
| china_mobile_139 | `LOGIN errno:1424, account not exist` |
| china_telecom_189 | `LOGIN failure, domain is disable.` |
| aliyun_personal | `LOGIN failed.` |

**八家回话各不相同** —— 这本身旁证了确实在和八台不同的服务器说话，
而不是某个代理返回同样的东西。耗时 14.4 秒。

判据仍是「错误落在 `MailError::Imap` 变体上」：任何一环没走通，
错误会落在 `Tls` / `Io` / `Timeout` 上，这条用例就会 panic 并指名是哪一家。

### 7.4 SMTP 连接层：同样 8/8（2026-10-17）

与 §7.3 对称。发信路径此前只有 QQ 验过，另外 7 家从没走过我们的代码。

**这条用例的断言必须比 IMAP 那条更细**：lettre 把**连接失败和 TLS 失败都**包进
它自己的 `smtp::Error`，所以「错误是 `MailError::Smtp`」**并不能证明握手成功** ——
主机名写错、证书不匹配，同样落进这个分支。因此额外要求：

1. `status().is_some()` —— 服务器**真的回出了 SMTP 状态码**，连不上是不会有状态码的；
2. `!is_tls()` —— 明确排除 TLS 层失败。

> **少了这两条，这条用例会在「地址全写错」的情况下照样通过。** 这是本项目
> 反复防的那类假阳性。

实测 8/8，4.9 秒，状态码**互不相同**：

| 服务商 | 状态码 | 服务器原话 |
| --- | --- | --- |
| qq | 535 | `Login fail. Account is abnormal, …` |
| 163 / 126 / yeah.net | 535 | `Error: authentication failed` |
| sina | 535 | `5.7.8 authentication failed` |
| **china_mobile_139** | **454**（transient） | `Authentication failed(USER_NOTFOUND_ERR)` |
| **china_telecom_189** | **550** | `Error: the invalide password` |
| **aliyun_personal** | **526** | `Authentication failure[0]` |

**状态码互不相同**同样旁证了在和八台不同服务器说话。

> ⚠️ **139 对错误密码返回的是 `454`（transient，可重试类）而不是 535。**
> 这是个反常行为：把它当「临时故障、稍后重试」的调用方会**反复重试错误密码**。
> 当前 `send_mail` 不做重试，故无影响；但将来若加重试逻辑，必须先排除这一条。

| 项 | 说明 |
| --- | --- |
| ~~163 的 `ID` 命令参数格式~~ | ✅ **2026-10-17 已实测**，见 §3.1 |
| **`ID` 是否真能解除 `Unsafe Login`** | ⛔ 格式已验证被接受，但「发了 `ID` 之后 `SELECT` 是否真的不再被拒」**仍需真实账号**。命令被接受 ≠ 限制被解除 |
| 163 的实际可用性 | 地址来自公开汇总表，未经真机连通性验证。 |
| 企业邮箱条目 | 尚未加入服务商表（方案未定）。 |

## 8. 本轮为何不写代码

按 `AGENTS.md` 硬性规则第 6 条「不写占位代码」：
在方案未定前写 `provider.rs` 的扩展，只能写出一个**无法通过自动识别命中**的表 ——
那不是实现，是占位。

宁可交一份说清前提的调研，也不交一份看着完整、实际跑不通的代码。
