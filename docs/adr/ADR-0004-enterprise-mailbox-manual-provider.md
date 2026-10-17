# ADR-0004：企业邮箱改为手动选择服务商

> 状态：**已采纳**（2026-10-17）
> 关联：`DESIGN.md` P0-05（服务商自动识别）、`docs/tasks/P3-01-provider-compat.md` §5

---

## 1. 背景

`DESIGN.md` P0-05 定的是「服务商**自动识别**」：`mail/provider.rs` 维护一张
`domains` 表，`resolve_by_email` 按邮箱域名查表命中。

企业邮箱（腾讯企业邮 / 网易企业邮 / 阿里云）**用的是客户自己的域名**
（`me@mycompany.com`）。自动识别对它不是「识别得不好」，而是**没有任何可用的
域名线索** —— 除非把那家公司的主域名写进表里，而那是不可能的。

后果：企业邮箱用户在「暂不支持该邮箱服务商」上卡死，无路可走。

---

## 2. 决策

**采用「手动选择服务商」。**

1. `PROVIDERS` 追加三条企业邮箱条目，`domains` **留空**；
2. 新增 `list_providers` 命令，把整张表交给前端；
3. `add_account` 增加可选参数 `provider_id`；
4. `validate_new_account` 增加 `provider_id: Option<&str>`：
   **用户显式指定优先于自动识别**；
5. 前端在自动识别失败时渲染服务商下拉框。

### 2.1 明确拒绝的方案：域名探测

对未知域名猜 `imap.<domain>` / `mail.<domain>` 并连接探测。

**拒绝理由不是体验，是安全**：那是拿用户的密码去戳一个未知服务器。
猜错意味着**把凭证发给错误的服务器**。这类错误不可撤销 —— 凭证一旦发出就收不回。

---

## 3. 这**不是**对 P0-05 的降级（更正一处此前的判断）

`P3-01` §5 当时写的是「方案 A 把自动识别降级为手动选择，属于改变 P0-05 的既定行为，
故应请示用户」。

**这个判断不准确。** 企业邮箱在此**从来没有**自动识别可用 ——
所谓「降级」是拿一个从未存在过的能力做基准。实际情况是：

| 域名 | 改动前 | 改动后 |
| --- | --- | --- |
| `qq.com` / `163.com`（已收录） | 自动识别 ✅ | 自动识别 ✅ **未变** |
| 企业邮箱自有域名 | **无路可走** ⛔ | 手动选择 ✅ **新增一条路** |

自动识别路径**一个字节都没改**，手动选择是**纯新增**的兜底分支。

真实窗口实测印证了这一点：输入 `someone@qq.com` 时不出现选择器、
仍显示「已识别：QQ邮箱（imap.qq.com:993）」。

因此本决策**不再视为需要事先请示的范畴**，但仍按 `AGENTS.md`「授权例外」第 3 条
新开本 ADR，把「当时为何这么定」写下来供事后复核。

---

## 4. 服务器地址是怎么核实的

企业邮箱地址此前只有第三方页面转述（`west.cn` 的汇总表），按项目自己的规矩
（`provider.rs`：「未核实的服务商**不得**加入本表」）**不足以入表**。

2026-10-17 改用 **TLS 实测**核实，判据有三条：

1. TCP 能连通；
2. **证书主机名校验通过** —— 服务端出示的证书确实覆盖该主机名，
   这直接证明「这个地址确实是这家服务商的」，而不是某个碰巧开了 993 端口的机器；
3. 协议 banner 是该服务商的 IMAP/SMTP 应答。

| 服务商 | IMAP | SMTP | 证书主机名 |
| --- | --- | --- | --- |
| 腾讯企业邮 | `imap.exmail.qq.com:993` | `smtp.exmail.qq.com:465` | `*.exmail.qq.com` |
| 网易企业邮 | `imap.qiye.163.com:993` | **`smtp.qiye.163.com:994`** | `*.qiye.163.com` |
| 阿里云企业邮箱 | `imap.mxhichina.com:993` | `smtp.mxhichina.com:465` | 握手通过（主机名匹配） |

> **网易企业邮的 SMTP 端口是 994，不是常见的 465。** 这是实测结果，
> 代码里也写了注释。照抄「企业邮箱都是 465」会连不上。

---

## 5. 接口变更清单

| 位置 | 变更 | 兼容性 |
| --- | --- | --- |
| `commands::accounts::add_account` | 新增可选参数 `provider_id: Option<String>` | 前端已同步；不传则退回自动识别 |
| `commands::accounts::validate_new_account` | 新增第三参数 `provider_id: Option<&str>` | 纯函数，10 处测试调用已同步 |
| `commands::accounts::list_providers` | **新增命令** | 已注册进 `lib.rs` |
| `mail::provider::PROVIDERS` | 追加 3 条（`domains` 为空） | **必须追加在末尾**，多处测试以 `PROVIDERS[0]` 为基准 |
| `lib::api` (TS) | 新增 `listProviders`；`addAccount` 增可选 `providerId` | — |

**未变更**：数据库表结构、`resolve_by_email` 的行为、已有的 `qq` / `netease163` 条目。

---

## 6. 验证

| 项 | 结果 |
| --- | --- |
| 后端单测 | `add_account_test` 6 → **12** 条；`resolve_provider_test` 7 → **8** 条 |
| 空 `domains` 不误伤已有识别 | `provider_163_test::unrelated_domains_still_resolve_to_nothing` 仍通过 |
| 真实窗口 UI | 企业邮箱域名出现选择器（6 个选项）；`qq.com` 不出现，仍显示「已识别」 |
| 手动优先于自动 | `manual_provider_id_wins_over_auto_detection` |
| 错误信息不泄漏授权码 | 新增 `bad_provider_id_never_echoes_the_secret` |

---

## 7. 尚未验证

| 项 | 说明 |
| --- | --- |
| **用真实企业邮箱账号登录** | 没有企业邮箱账号。地址是证书验证过的，但**从未真正登录过** —— 「连得上」与「登得进」是两件事 |
| 网易企业邮是否需要 IMAP `ID` 命令 | 同属 Coremail 系，推测需要，**未验证** |
| 阿里云企业邮的 `AUTH=XOAUTH2` 支持 | banner 里有，未试过 |
| 网易企业邮 SMTP 994 端口的收信 | TLS 握手通过、EHLO 正常，但未投递过 |
