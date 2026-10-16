# 真实网络验证（live network verification）

> 2026-10-16。补上本项目最大的一笔验证债：IMAP / SMTP 的**连接层**此前零真实执行。

---

## 1. 这一笔债为什么此前记成「阻塞」

此前的记录是：

> `fetch_recent` 与 `send_mail` 合计约 150 行网络代码**零真实执行**。
> TLS 握手、AUTH、DATA、IMAP SELECT/FETCH 全都没跑过。

推理链是「要跑网络代码 → 要登录 → 要授权码 → 用户没给 → 所以跑不了」。
**这条链在第二环就断了**：验证「连接层是否真的能连上」并不需要成功的登录。

`connect_and_login` 与 `send_mail` 的签名都收 `&ProviderConfig`（字段全 `pub`），
**服务器地址是可注入的**，不存在"只能连生产配置"的限制。

于是：**用伪造凭证登录，断言错误落在协议层**，就能在零凭证的条件下把连接层真跑一遍。

判别力来自错误变体本身：

| 若哪一环没走通 | 错误会落在 |
| --- | --- |
| DNS / TCP / TLS 握手 / 证书校验 | `MailError::Tls` / `MailError::Io` |
| 问候语读取 | `MailError::Imap`（无问候语分支）或 `Timeout` |
| IMAP `LOGIN` 往返 | ✅ `MailError::Imap`（服务器回 NO） |
| SMTP `EHLO` / `AUTH` 往返 | ✅ `MailError::Smtp` 且 `status()` 为 `Some(535)` |

拿到 `Imap` / `Smtp` 且带服务器状态码 ⇒ **整条链路真的走到服务器并收到了回话**。

用例：`src-tauri/tests/live_network_test.rs`（4 个，全部 `#[ignore]`）。
运行：

```text
cargo test --manifest-path src-tauri/Cargo.toml --test live_network_test -- --ignored --nocapture
```

**不会有任何邮件被投递** —— 认证在 `DATA` 之前必然失败。

---

## 2. 实测结果

4/4 通过，用时 4.25 s。

| 用例 | 路径 | 服务器实际回话 |
| --- | --- | --- |
| `live_imap_implicit_tls_reaches_server_rejection` | `imap.qq.com:993` 隐式 TLS → `LOGIN` | `Login fail. Please input email address` |
| `live_imap_starttls_upgrade_reaches_server_rejection` | `imap.163.com:143` 明文 → `STARTTLS` → TLS → `LOGIN` | `LOGIN Login error or password error` |
| `live_smtp_implicit_tls_reaches_server_rejection` | `smtp.qq.com:465` → `EHLO` → `AUTH` | `535 Login fail. …` |
| `live_smtp_starttls_reaches_server_rejection` | `smtp.qq.com:587` → `EHLO` → `STARTTLS` → TLS → `AUTH` | `535 Login fail. …` |

### 2.1 因此**已被真实执行**的代码

- **`async-native-tls` 的 `runtime-tokio` 接线** —— 即此前 35 个 E0277 后移除 `tokio_util::Compat` 的那次改动。真实 TLSv1.3 握手成功即证明该接线正确。
- **真实证书链校验** —— QQ 证书 `CN=*.mail.qq.com`（签发者 DigiCert Secure Site OV G2），163 证书 `CN=*.163.com`（NetEase）。默认校验器没有放行错误证书。
- **`read_greeting`** —— 对真实服务器读并解析了问候语。
- **`CAPABILITY` 往返** —— 实测确认 QQ 与 163 的 CAPABILITY **都含 `ID`**。
- **`async-imap` 的 `login()` 往返** —— 命令发出、服务器应答、错误正确映射为 `MailError::Imap`。
- **IMAP STARTTLS 升级序列** —— 明文建连 → 读问候语 → `STARTTLS` → `plain.into_inner()` → 重新套 TLS → 新建 `Client` → `login`。这是整条 IMAP 路径里最容易写错的一段，现已在真实服务器上跑通。
- **lettre 的 `relay()`（465 隐式）与 `starttls_relay()`（587）两条建连路径**，以及 `credentials()` 之后的 `AUTH` 协商。
- **错误映射**：`MailError::Imap` / `MailError::Smtp` 两个变体在真实失败下确实被命中。

### 2.2 顺带实测到的服务商事实

| 事实 | 证据 |
| --- | --- |
| **QQ 的 IMAP 143 不支持 STARTTLS** | 发 `a2 STARTTLS` 回 `* BAD Command!` |
| 163 的 IMAP 143 支持 STARTTLS | 回 `a1 OK Begin TLS negotiation`，升级后 TLSv1.3 |
| QQ 问候语直接带 CAPABILITY | `* OK [CAPABILITY IMAP4 IMAP4rev1 ID AUTH=PLAIN AUTH=LOGIN AUTH=XOAUTH2 NAMESPACE] QQMail XMIMAP4Server ready` |
| QQ 支持 `AUTH=PLAIN / AUTH=LOGIN / AUTH=XOAUTH2` | 同上 |
| QQ SMTP 465 的 EHLO 能力 | `PIPELINING` / `SIZE 73400320` / `AUTH LOGIN PLAIN XOAUTH XOAUTH2` / `SMTPUTF8` / `8BITMIME` |
| QQ SMTP 587 广告 `STARTTLS`，升级后 TLSv1.3 | `220 Ready to start TLS …` |

**结论：`provider.rs` 中 QQ 的两条端口配置（IMAP 993 / SMTP 465）与 163 的 993 均正确。**
**不要给 QQ 配 `TlsMode::StartTls`** —— 服务器不认。

---

## 3. 仍然未验证的边界（**没有缩小到零**）

本次只证明了**连接层**。以下仍然一个字节都没跑过：

| 项 | 为什么还没验证 |
| --- | --- |
| **成功的登录** | 需要真实授权码 |
| **`send_client_id_if_supported` 发 `ID` 命令** | 它在 `login` **成功之后**才调用 ⇒ 本次完全没执行到。163 要求这步，**风险原样保留** |
| **`SELECT` / `FETCH` / `fetch_recent` 取信与解析** | 未建立会话，邮箱层零执行 |
| **SMTP 的 `DATA` 阶段** | 认证先失败，报文从未上线 |
| **真实投递与在别的客户端里的呈现** | 同上 |

也就是说：**「客户端从未收发过一封真实邮件」这个事实没有改变**。
改变的是——现在知道它连得上、握得手、走得通认证协商，
而不是"连这一层都没跑过"。

判据照旧：**没有真实跑通过的功能，不得只写「完成」。**

---

## 4. 复现

```powershell
$cargo = "$env:USERPROFILE\.cargo\bin\cargo.exe"
& $cargo test --manifest-path D:\dp_Gmail\src-tauri\Cargo.toml --test live_network_test -- --ignored --nocapture
```

用例标了 `#[ignore]`，默认不参与 `cargo test`（CI 无网络时的稳定性考虑）。
**拿掉 `--ignored` 不会运行它们** —— 这是有意为之，不是漏配。
