# dp_Gmail 说明 / About dp_Gmail

> 中文在前，English below.
> 本文档只陈述事实与限制，不做宣传。

---

# 中文

## 这是什么

一个邮件客户端，能收发邮件、能读 HTML 正文、能搜索。有两个版本：

- Windows 桌面版（x64）
- Android 手机版（arm64-v8a / x86_64）

数据全部存在本机，没有任何服务器参与。

## 怎么用

1. 打开程序，在侧栏「添加账户」里填邮箱地址和**授权码**
   - 注意是授权码，不是你登录邮箱用的密码
   - QQ 邮箱：设置 → 账户 → 开启 IMAP/SMTP → 生成授权码
   - 163 邮箱：设置 → POP3/SMTP/IMAP → 开启 → 新增授权码
2. 选中账户，文件夹名保持 `INBOX`，点「同步」
3. 想发信就点「写邮件」，在面板里选「发件账户」

手机上侧栏是抽屉：点左上角 ☰ 打开，点空白处或再点一次 × 关闭。

## 能做哪些事

| 功能 | 说明 |
| --- | --- |
| 多账户 | 可以加多个邮箱，切换查看 |
| 统一收件箱 | 把所有账户的收件箱合并成一个列表，每封标注来自哪个账户 |
| 全文检索 | 搜标题、发件人、收件人、正文 |
| 收信 | IMAP，支持 993 隐式 TLS 与 143 STARTTLS |
| 发信 | SMTP，支持纯文本、HTML 正文、附件（附件合计上限 25 MiB） |
| 回复 / 转发 | 自动带原文引用 |
| 列表滚动 | 只渲染可视区附近的行，邮件多了也不卡 |
| 自动收信 | 每 15 秒拉一次，可在设置里关掉 |
| 界面 | 四套主题（明 / 暗 / 护眼米色 / 深空蓝）× 五种重点色；中 / 日 / 英三语 |
| 编辑与删除账户 | 邮箱填错了可以改，也可以连它的邮件一起删掉 |

## 缺点与限制（这部分请认真看）

**1. 不是所有邮箱都能用**

内置 11 条服务商配置，但这不等于"都验证过能登录"。实际情况：

| 服务商 | 验证程度 |
| --- | --- |
| QQ、163、新浪 | 有真实账号，登录、收信、发信都跑通过 |
| 126、yeah.net、139、189、阿里云个人邮 | **只验证过能连上服务器**，没有成功登录过 |
| 腾讯企业邮、网易企业邮、阿里云企业邮 | **只验证过服务器地址和证书**，没有真实企业账号登录过 |

也就是说：上表后两行，能不能真的收发，我不知道。

**2. 安卓版连不上新浪邮箱的 IMAP**

新浪的邮件服务器只提供 TLS 1.2 的 DHE / 纯 RSA 密钥交换，没有 ECDHE 也没有 TLS 1.3。
Windows 版走系统自带的 TLS（SChannel），支持这些老式加密，所以**电脑上新浪正常**。
安卓版用的是 rustls，它**只支持 ECDHE + TLS 1.3**，握手必然失败。

这是加密库的能力边界，不是配置问题。安卓版上新浪发信正常，只有收信不行。

**3. 不支持 iOS**

编译 iOS 程序必须有 macOS + Xcode，Windows 上做不到。

**4. 收不了服务器上的「已发送」文件夹**

各家服务商对发件箱的命名不同（`Sent Messages`、`已发送`、还有一堆看不懂的编码），
逐个适配工作量很大，所以暂时没做。

目前的替代方案是：**发信成功后，在本地记一份到「已发送」**。
所以你在应用里看得到自己发的信，但它和服务器上的发件箱是两套，删除不会互相同步。

**5. 同步只能一个一个文件夹来**

没有"自动发现所有文件夹并全部同步"，需要自己填文件夹名点同步。
服务器上的中文文件夹名可能是一串奇怪的编码（如 `&XfJT0ZAB-`），遇到就得自己试。

**6. 已读状态不会同步回服务器**

在应用里读过一封信，只记在本地，服务器那边还是未读。
换一个客户端看，仍然是未读。

**7. 邮件正文里的图片多数显示不出来**

因为加载远程图片会把你"打开过这封信"这件事暴露给发件人，所以没有自动加载。
目前也没有做"询问是否加载"的开关。

**8. 删账户 = 删数据，不可逆**

删掉一个账户会**连带删掉它已经下载的所有邮件和附件**，没有回收站。
删之前请自己确认。

**9. 数据没有云端备份**

全部在本机。硬盘坏了、系统重装没备份，邮件就没了。
数据位置见下面的「数据存在哪」。

**10. 没有做的东西**

- 邮件规则 / 过滤器
- 通讯录
- 日历
- 邮件模板
- 多语言之外的界面自定义
- 后台常驻收信（安卓上应用切到后台就不会再拉信了）

## 数据存在哪

Windows：

```
%APPDATA%\com.dpgmail.desktop\
    dp_gmail.sqlite      邮件、账户、文件夹
    key.json             加密密钥
    -wal / -shm          SQLite 临时日志
```

想彻底清空：先正常退出程序（点右上角 ×，别用任务管理器强杀），
再把上面这个文件夹整个删掉。删完不可恢复。

授权码用 AES-256-GCM 加密后才落库，明文只在登录那一刻存在于内存。
**但要说清楚**：能读到这台电脑上这两个文件的人，就能解出授权码。
加密防的是"数据库文件被单独拷走"，挡不住"整台机器被控制"。

## 会不会偷偷联网

联网只发生在收信、发信、自动收信时，连的是你自己配置的邮件服务商。
没有自动更新检查，没有统计上报，没有崩溃上报，没有广告。
项目代码里**没有任何 HTTP 客户端依赖**，可以自己查 `src-tauri/Cargo.toml`。

## 联系方式

邮箱：`aabb1029384756@qq.com`

用出问题了、发现 bug、有想加的功能，都可以发邮件。
请尽量写清楚：哪个版本（电脑/手机）、哪个邮箱服务商、报了什么错。

---

# English

## What it is

An email client that can receive, read, and send mail, including HTML bodies, with search.
Two builds:

- Windows desktop (x64)
- Android (arm64-v8a / x86_64)

All data stays on the local machine. No server is involved.

## Getting started

1. Open the app, fill in your email address and an **app password** under "Add account"
   - An app password, not your normal login password
   - QQ Mail: Settings → Account → enable IMAP/SMTP → generate an app password
   - 163 Mail: Settings → POP3/SMTP/IMAP → enable → add an app password
2. Select the account, keep the folder name `INBOX`, click "Sync"
3. To write mail, click "Compose" and pick the "Send from" account inside the panel

On Android the sidebar is a drawer: tap ☰ at the top-left to open it, tap the dimmed
background or the × to close.

## What it does

| Feature | Notes |
| --- | --- |
| Multiple accounts | Add several mailboxes and switch between them |
| Unified inbox | All inboxes merged into one list, each row labelled with its account |
| Full-text search | Subject, sender, recipients, body |
| Receiving | IMAP, 993 implicit TLS and 143 STARTTLS |
| Sending | SMTP, plain text, HTML body, attachments (25 MiB combined cap) |
| Reply / forward | Quotes the original automatically |
| Scrolling | Only rows near the viewport are rendered, so large mailboxes stay smooth |
| Auto-check | Fetches every 15 seconds; can be turned off in Settings |
| Appearance | Four themes (light / dark / sepia / midnight) × five accent colours; Chinese, Japanese and English UI |
| Edit and delete accounts | Fix a mistyped address, or remove an account and its mail |

## Limitations — please read this part

**1. Not every provider is confirmed working**

Eleven provider profiles ship with the app, but that does not mean all of them have been
tested end to end:

| Provider | What was actually verified |
| --- | --- |
| QQ, 163, Sina | Real accounts; login, receiving and sending all exercised |
| 126, yeah.net, 139, 189, Aliyun personal | **Only that the server can be reached**; never logged in successfully |
| Tencent Exmail, NetEase enterprise, Aliyun enterprise | **Only the server address and TLS certificate**; no real enterprise account was ever used |

So for the last two rows: whether they actually work, I do not know.

**2. The Android build cannot read Sina's IMAP**

Sina's IMAP server only offers TLS 1.2 with finite-field DHE or plain-RSA key exchange —
no ECDHE, no TLS 1.3. The Windows build uses the system TLS stack (SChannel), which supports
those legacy suites, so **Sina works on desktop**. The Android build uses rustls, which
**only supports ECDHE and TLS 1.3**, so the handshake always fails.

This is a hard limit of the TLS library, not a configuration mistake. Sending mail through
Sina still works on Android; only receiving does not.

**3. No iOS build**

Building for iOS requires macOS and Xcode. It cannot be produced on Windows.

**4. The server-side "Sent" folder is not synchronised**

Mail providers name their sent folder differently (`Sent Messages`, `已发送`, and various
encoded forms), so adapting to each one is a lot of work and has not been done.

The current workaround: after a message is sent successfully, **a copy is written into a
local "Sent" folder**. You can see what you sent inside the app, but that folder is separate
from the server's, and deleting in one does not affect the other.

**5. Folders are synced one at a time**

There is no "discover all folders and sync everything". You type a folder name and sync it.
Chinese folder names on the server may appear as encoded strings (for example `&XfJT0ZAB-`),
in which case you have to guess.

**6. Read state is not written back to the server**

Opening a message marks it read locally only. The server still considers it unread, and
other clients will show it as unread.

**7. Most images inside messages do not load**

Remote images are not fetched automatically, because doing so tells the sender you opened
the message. There is currently no "load images?" prompt either.

**8. Deleting an account deletes its data, irreversibly**

Removing an account also **removes every message and attachment already downloaded for it**.
There is no recycle bin. Please be sure before you confirm.

**9. No cloud backup**

Everything is local. If the disk fails, or the system is reinstalled without a backup, the
mail is gone. See "Where the data lives" below.

**10. Not implemented**

- Mail rules / filters
- Contacts
- Calendar
- Message templates
- Any UI customisation beyond the three languages
- Background receiving (on Android the app stops fetching once it goes to the background)

## Where the data lives

Windows:

```
%APPDATA%\com.dpgmail.desktop\
    dp_gmail.sqlite      messages, accounts, folders
    key.json             encryption key
    -wal / -shm          SQLite journals
```

To wipe everything: quit the app normally (click × at the top-right — do **not** kill it
from Task Manager, that can leave WebView2 processes holding the data folder), then delete
the directory above. This cannot be undone.

App passwords are encrypted with AES-256-GCM before being written; plaintext exists in
memory only while logging in. **To be explicit:** anyone who can read both files on this
machine can recover the password. The encryption protects against the database file being
copied on its own — not against the whole machine being compromised.

## Does it phone home

Network access happens only while receiving, sending, or auto-checking, against the mail
provider you configured. There is no update check, no analytics, no crash reporting and no
advertising. The project has **no HTTP client dependency at all** — see
`src-tauri/Cargo.toml`.

## Contact

Email: `aabb1029384756@qq.com`

Bug reports, questions and feature requests are welcome. Please mention which build
(desktop or Android), which mail provider, and the exact error message.
