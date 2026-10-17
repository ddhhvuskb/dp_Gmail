# dp_Gmail

本地优先的跨平台邮件客户端。邮件与授权码只存在你自己的电脑上，不经任何中转服务器。

## 下载

不装开发环境也想用的话，直接下现成的安装包：

| 平台 | 文件 | 大小 |
| --- | --- | --- |
| **Android 7.0+** | [`dp_Gmail-android.apk`](https://github.com/ddhhvuskb/dp_Gmail/releases/download/v0.1.0/dp_Gmail-android.apk) | 27.81 MB |
| **Windows x64**（安装版） | [`dp_Gmail-windows-setup.exe`](https://github.com/ddhhvuskb/dp_Gmail/releases/download/v0.1.0/dp_Gmail-windows-setup.exe) | 3.42 MB |
| **Windows x64**（免安装） | [`dp_Gmail-windows-portable.exe`](https://github.com/ddhhvuskb/dp_Gmail/releases/download/v0.1.0/dp_Gmail-windows-portable.exe) | 13.07 MB |

全部版本见 [**Releases**](https://github.com/ddhhvuskb/dp_Gmail/releases)。

**装之前请读 [`INTRODUCTION.md`](INTRODUCTION.md)** —— 里面有逐条列出的限制，
其中几条会直接影响你能否用起来（比如安卓版连不上新浪邮箱的 IMAP）。

> **两个包都没有代码签名证书。** Windows 首次运行会提示「未知发布者」，
> 点「更多信息」→「仍要运行」；安卓包是自签名，安装时需允许「未知来源应用」。
> 这不是程序有问题。

## 这是什么

一个能真正收发邮件的桌面 + 安卓客户端，用 Tauri v2 + Rust + React + SQLite 写成。

- **本地优先**：邮件正文存在本机 SQLite，授权码用 AES-256-GCM 加密后落库，密钥也在本机
- **没有任何服务器**：项目里连一个 HTTP 客户端依赖都没有；联网只发生在收发信时，连的是你自己配置的邮件服务商
- **服务商开箱即用**：内置 11 家国内邮箱配置（QQ / 163 / 126 / yeah.net / 新浪 / 139 / 189 / 阿里云 + 腾讯企业邮 / 网易企业邮 / 阿里云企业邮）
- **全文检索**：SQLite FTS5，1000 封邮件下搜索实测最慢 2.11 ms
- **虚拟滚动**：1200 封列表滚动 200 帧，零帧掉帧
- **多账户 + 统一收件箱**
- **HTML 正文净化**：后端用 ammonia 剥掉脚本、样式、事件属性与 `javascript:` 链接后再渲染
- **四套主题 × 五种重点色**，全部走 CSS 变量，配色对比度按 WCAG AA 实测（20 种组合最低 5.01:1）
- **中 / 日 / 英**三语界面，日期格式跟随地区
- **每 15 秒自动收信**（可关；窗口最小化时暂停）

## 状态

| 项 | 状态 |
| --- | --- |
| 单元测试 | **215 通过 / 0 失败** |
| 真机验证 | QQ / 163 / 新浪三家均完成真实登录、收信、发信闭环 |
| 平台 | Windows x64 已验证；Android（arm64-v8a / armeabi-v7a）已出包 |

### 已知限制

- **安卓版连不上新浪的 IMAP**：该服务器只提供 TLS1.2 的有限域 DHE / 纯 RSA 密钥交换，
  而安卓用的 rustls 只支持 ECDHE + TLS1.3。桌面版走系统 TLS（SChannel），不受影响。
- **iOS 不支持**：编译 iOS 必须有 macOS + Xcode。
- 企业邮箱只做过地址的 TLS 实测核实，**没有真实企业邮箱账号验证过登录**。

## 技术栈

Tauri v2 · Rust（stable）· React 19 · TypeScript · Zustand · TanStack Query · SQLite（rusqlite + FTS5）

依赖清单与技术决策全部记在 `docs/adr/`：
- `ADR-0001` 技术栈与 P0 依赖
- `ADR-0002` 凭证加密方案
- `ADR-0003` SMTP 发信选型
- `ADR-0004` 企业邮箱手动选择服务商
- `ADR-0005` 前端多语言（后端错误保持中文）
- `ADR-0006` Android 移植

## 本地构建

```bash
npm install
npm run tauri dev          # 开发（会同时拉起前端与 Rust）
npm run tauri build        # 正式版（Windows）
npx tauri android build --apk   # 安卓 APK
```

后端验证：

```bash
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
npx tsc --noEmit
```

## 数据存在哪

Windows：`%APPDATA%\com.dpgmail.desktop\`

| 文件 | 内容 |
| --- | --- |
| `dp_gmail.sqlite` | 邮件、附件信息、账户、文件夹 |
| `key.json` | 加密密钥 |
| `-wal` / `-shm` | SQLite 临时日志 |

退出应用后删掉这个目录，所有邮件与凭证一并消失，无残留。
本机数据没有云端备份，换机前请自行备份。

## 隐私

见 [`docs/privacy/PRIVACY.md`](docs/privacy/PRIVACY.md)。一句话：不收集任何数据，没有服务器。

## 详细说明 / 使用方法

中英双语的完整说明（含**逐条列出的缺点与限制**、各服务商的验证程度、数据位置、
联系方式）在 [`INTRODUCTION.md`](INTRODUCTION.md)。**装之前请先看那份文档。**
