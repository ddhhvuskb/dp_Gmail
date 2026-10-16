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

| 服务商 | IMAP | SMTP | 来源 |
| --- | --- | --- | --- |
| QQ 邮箱 | `imap.qq.com:993` | `smtp.qq.com:465` | QQ 官方帮助页（P0-01 已核实） |
| 网易 163 | `imap.163.com:993` | `smtp.163.com:465` | [Zoho 汇总表](https://zdblogs.zoho.com.cn/mail/articles/pop513.html) |

### 2.2 企业邮箱（**无法**用域名自动识别，见 §3）

来源：<https://www.west.cn/faq/listagent.asp?unid=2132>（企业邮箱 POP/SMTP/IMAP 汇总表）

| 服务商 | IMAP | SMTP |
| --- | --- | --- |
| 腾讯企业邮 | `imap.exmail.qq.com:993` | `smtp.exmail.qq.com:465` |
| 网易企业邮 | `imap.qiye.163.com:993` | `smtp.qiye.163.com:994` |
| 阿里云企业邮箱（原万网） | `imap.mxhichina.com:993` | `smtp.mxhichina.com:465` |

### 2.3 仍未核实（**不得凭记忆填入**）

`126.com`、`yeah.net`（网易系其它域名）、`sina.com`、`139.com`、`189.cn`、`aliyun.com`。

两轮检索都没找到覆盖它们的权威页面。按 `AGENTS.md`「不猜」，这些留空待补，
**宁可表里少几项，也不填一个可能连不上的地址**。

---

## 3. 关键发现：企业邮箱用客户自己的域名

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

## 4. 由此产生的设计问题（需用户决策）

`DESIGN.md` P0-05 定的是「服务商自动识别」。企业邮箱无法自动识别，
只有两条路：

### 方案 A：手动选择服务商

`ProviderConfig` 增加企业邮箱条目，`domains` 留空；
`resolve_by_email` 对未知域名返回 `None`，前端提示
「未识别到服务商，请手动选择」并给一个下拉列表。

- 优点：改动小，复用现有 `ProviderConfig` 结构；
- 代价：P0-05 的「自动识别」在企业邮箱场景降级为「手动选择」。

### 方案 B：域名探测（猜）

对未知域名尝试 `imap.<domain>` / `mail.<domain>` 等常见命名并连接探测。

- **明确不建议**：这是拿用户的密码去戳一个未知服务器。
  在企业邮箱场景下，猜错意味着**把凭证发给错误的服务器** —— 这是安全问题，不是体验问题。

**我的倾向是方案 A**，但它把「自动识别」降级了，
属于改变 P0-05 的既定行为，按 `AGENTS.md`「授权例外」第 2 条
（影响用户数据安全、或超出 `DESIGN.md` 已锁定范围）应请示用户，故本轮停在这里。

---

## 5. 尚未完成

| 项 | 状态 |
| --- | --- |
| 免费个人邮箱地址核实 | 🟨 163 已核实；126 / yeah.net / sina.com / 139 / 189 未找到权威来源 |
| 企业邮箱的手动选择 UI | ⬜ 待方案确定 |
| OAuth（P3 后半） | ⬜ 未开始 |
| IMAP 侧 STARTTLS | ✅ 已完成（见 `P2-01-smtp-send.md` 与 `imap_client.rs`） |

---

## 6. 本轮为何不写代码

按 `AGENTS.md` 硬性规则第 6 条「不写占位代码」：
在方案未定前写 `provider.rs` 的扩展，只能写出一个**无法通过自动识别命中**的表 ——
那不是实现，是占位。

宁可交一份说清前提的调研，也不交一份看着完整、实际跑不通的代码。
