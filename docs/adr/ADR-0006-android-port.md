# ADR-0006：移植到 Android

> 状态：**已采纳**（2026-10-17，用户明确要求）
> 关联：`DESIGN.md` 第一章（定位为「本地优先跨平台**桌面**邮件客户端」）、
> `docs/tasks/P4-03-user-requested-features.md` §3 与 §15

---

## 1. 背景

用户最早提出「编译成手机版」，随后在 2026-10-17 决定**暂缓**（「先不做，桌面端收尾」）。
同一天稍后又改主意：**「待会儿弄完了搞手机版吧」**。

因此项目范围从 **桌面** 扩到 **桌面 + Android**。
按 `AGENTS.md`「授权例外」第 3 条，超出 `DESIGN.md` 锁定范围的决定**必须留痕**，就是本文件。

---

## 2. 决定性约束：iOS 在这台机器上做不了

| 平台 | 结论 | 原因 |
| --- | --- | --- |
| **Android** | ✅ 做 | 工具链在 Windows 上齐全 |
| **iOS** | ⛔ 不做 | 编译 iOS 必须有 macOS + Xcode，**没有任何替代方案**。不是配置问题，是平台限制 |

所以本 ADR 的「手机版」= **仅 Android**。这一点不写清楚，日后会有人以为漏了 iOS。

---

## 3. 验证目标

用户指定：**用 Android 10 与 Android 15 两个模拟器验证**，理由是「其他肯定就都能跑了」。

| 目标 | API | 作用 |
| --- | --- | --- |
| Android 10 | 29 | 覆盖较老设备，且是本项目**支持的最低大版本**附近 |
| Android 15 | 35 | 覆盖当前主流新系统 |

> Rust/Tauri 的官方最低 Android 版本是 API 24（Android 7）。本次**只验证** 29 与 35，
> 中间版本（30/31/32/33/34）不做逐版本验证 —— 这是有意的取舍，写在这里而不是装作覆盖了。

---

## 4. 影响面盘点：比预想的小

先说好消息 —— 有两处原本以为要大改，实际**已经是对的**：

| 项 | 结论 | 依据 |
| --- | --- | --- |
| **数据目录** | ✅ **无需改动** | `src-tauri/src/lib.rs` 用的是 `app.path().app_data_dir()`，这是 Tauri 的跨平台解析器：桌面上是 `%APPDATA%\com.dpgmail.desktop`，Android 上是应用私有目录 `.../files` |
| **移动端入口** | ✅ **已就位** | `run()` 上已有 `#[cfg_attr(mobile, tauri::mobile_entry_point)]` |
| **密钥文件** | ✅ 无需改动 | 与数据库同目录（`key.json`）。Android 的应用私有目录其他 app 无权读取 |
| **IMAP / SMTP / 解析 / 加密** | ✅ 无需改动 | 纯 Rust，与平台无关 |
| **`rusqlite` bundled** | 🟨 需要 NDK 交叉编译 | 它用 `cc` 编 SQLite 的 C 源码，必须能找到 NDK 的 clang。**未验证** |
| **界面布局** | ❌ **必须改** | 现在是 `320px | minmax(280px,380px) | 1fr` 三栏。手机屏幕放不下，要变「列表 → 点进详情」单栏导航。**这是本次的主要工作量** |
| 附件选择 | 🟨 待确认 | 桌面走 `tauri-plugin-dialog` 的 `open()` 拿**文件路径**；Android 上 scoped storage 是否还给得出可用路径，**未验证** |

### 4.1 布局决策：单栏导航，不做「窄版三栏」

在手机上硬塞三栏（哪怕缩成图标栏）会导致：列表行被压到不可读、详情区几乎没宽度。
所以做**真正的单栏**：

```
列表页  ──点一封──▶  详情页（有返回按钮）
```

实现上不引入路由库（`package.json` 已锁，加路由＝新增依赖）。
用一个 `view: "list" | "detail"` 的客户端状态切换即可。

**桌面端不受影响**：宽度 ≥ 阈值时仍是三栏，`view` 状态被忽略。

---

## 5. 明确不做的事

| 不做 | 为什么 |
| --- | --- |
| iOS | 见 §2，物理上不可能 |
| 推送通知 | 需要服务端或 FCM 接入＝**引入网络服务与账号体系**，与「本地优先、无服务器」的承诺直接冲突 |
| 后台常驻收信 | Android 会杀后台；要做需前台服务 + 通知权限，属另一个量级的工作 |
| 应用商店上架 | 需签名证书、隐私政策页托管、审核流程 |
| 15 秒自动收信在后台继续跑 | 现有实现依赖 `setInterval` + 页面可见性，**app 进后台就停**。这是合理行为，但必须在文档里说明，否则用户会以为「后台也能收」 |

---

## 6. 风险

| 风险 | 说明 |
| --- | --- |
| **`rusqlite` 的 C 交叉编译** | 最可能卡住的地方。NDK 版本与 `cc` 的配合问题会报很晦涩的链接错误 |
| 模拟器跑得慢 | 只能验证功能，**证明不了真机性能**。「1000 封滚动不卡」在模拟器上的数字没有参考价值 |
| 附件路径 | scoped storage 下 `open()` 返回的可能是 content:// URI 而非文件路径，而后端按路径读文件 |
| 体积 | Android 模拟器镜像与 AVD 会再占 **~5 GB**（两个 x86_64 系统镜像 + 实例）。用户对磁盘敏感，每次收尾要清 |

---

## 7. 验证标准

「能构建出 APK」**不算完成**。按本项目一贯标准，必须：

1. 在 **API 29 与 API 35 两个模拟器**上真正启动起来；
2. 界面不是白屏、三栏没挤爆；
3. 加一个账户 → 同步 → 列表出现邮件（能连真实服务商则更好）；
4. 详情能打开、能返回；
5. 主题/语言/设置在手机上可用。

任何一条没做到，就写进「尚未验证」，不得只说「已支持手机」。

---

## 8. 尚未验证（本 ADR 落笔时的状态）

- 两个 x86_64 系统镜像**正在下载**，AVD 尚未创建，模拟器**从未启动过**
- `tauri android init` 尚未执行
- `rusqlite` 的 NDK 交叉编译未试过
- 单栏布局**尚未实现**

以上全部推平之前，本项目的状态是「**桌面可用，Android 未开始**」。

---

## 9. 实测撞到的硬阻塞：TLS 后端（2026-10-17）

`tauri android init` **已成功**（生成了 `src-tauri/gen/android/dp_gmail`），
模拟器也**实测能启动**（`emulator-5554 device`，API 29）。

但 `tauri android build --apk --target aarch64` **失败**，卡在 `openssl-sys`：

```
Could not find directory of OpenSSL installation, and this `-sys` crate
cannot proceed without this knowledge.
--- stderr
$HOST = x86_64-pc-windows-msvc
$TARGET = aarch64-linux-android
openssl-sys = 0.9.117
```

### 9.1 为什么会这样

`native-tls` 在**不同平台用不同的后端**：

| 平台 | native-tls 的后端 | 是否需要额外编译 |
| --- | --- | --- |
| Windows | SChannel（系统自带） | ❌ 不需要 |
| macOS | Secure Transport | ❌ 不需要 |
| **Android / Linux** | **OpenSSL** | ✅ **必须现场有 OpenSSL** |

依赖链（`cargo tree -i openssl-sys --target aarch64-linux-android` 实测）：

```
openssl-sys
├── native-tls
│   ├── async-native-tls  ← dp-gmail（IMAP）
│   ├── lettre            ← dp-gmail（SMTP）
│   └── tokio-native-tls  ← lettre
└── openssl
```

也就是说：**桌面能编，是因为 Windows 走了 SChannel，完全没碰 OpenSSL。**
一到 Android 就绕不开。

### 9.2 本机还缺编 OpenSSL 的工具

`openssl` crate 的 `vendored` 特性会现场编 OpenSSL，但它需要 **perl** 与 **make**：

| 工具 | 状态 |
| --- | --- |
| perl | ❌ 没有 |
| nmake / make / mingw32-make | ❌ 都没有 |

### 9.3 三条路（**需要用户决定**）

| 方案 | 做法 | 代价 |
| --- | --- | --- |
| **A. 换 rustls** | `async-native-tls` → `rustls`/`tokio-rustls`；`lettre` 改 `tokio1-rustls` | **新增依赖 + 改安全代码**（TLS 校验逻辑），按 `AGENTS.md` 第 1/7 条必须批准；根证书来源要另定（`webpki-roots` 静态包 vs 系统证书）。好处是全平台**一套 TLS 栈**，不再依赖系统库 |
| **B. 硬编 OpenSSL** | 装 Strawberry Perl + make，给 `openssl` 开 `vendored` | 不动一行 TLS 逻辑，但 Windows 上给 Android 编 OpenSSL 很脆；且**每次清理缓存后都要重编**（十几分钟起） |
| **C. 放弃 Android** | —— | 桌面端已经完整可用 |

**A 是跨平台 Rust 项目的标准解法，但它动的是加密相关代码，必须用户明确批准后才做。**

### 9.4 实测结果（2026-10-17，用户选「试试 ssl」后）

用户先选了 A（rustls），实现完发现**新问题**，随后改试 B（OpenSSL）。

**A 的实测结论：rustls 用不了，因为 sina。**

逐条限制密码套件实测 sina 的 IMAP：

| 约束 | sina 结果 |
| --- | --- |
| 默认 | ✅ `DHE-RSA-AES256-GCM-SHA384` |
| **只允许 ECDHE** | ❌ 握手失败（alert 40） |
| 只允许 DHE | ✅ `DHE-RSA-AES256-GCM-SHA384` |
| 只允许纯 RSA 密钥交换 | ✅ `AES256-GCM-SHA384` |
| **仅 TLS 1.3** | ❌ 握手失败 |

即 **sina 的 IMAP 只提供 TLS1.2 + 有限域 DHE / 纯 RSA 密钥交换，没有 ECDHE，也没有 TLS1.3**。

而 **rustls 从设计上不支持有限域 DHE，也不支持 RSA 密钥交换，只做 ECDHE + TLS1.3**。
BoringSSL（Chrome 用的那个）同样已移除 DHE，所以也不行。
支持 DHE 的只有 **OpenSSL / mbedTLS / Schannel**。

顺带否掉 mbedTLS：**`lettre` 只有 native-tls / rustls / boring-tls 三个后端，没有 mbedTLS**，
所以 mbedTLS 只能修 IMAP，SMTP 还得再挂一套 —— 两个 TLS 栈，比现状更差。

**B 的实测结论：Windows 上给 Android 编 OpenSSL 走不通。**

装了 Strawberry Perl 5.42.3（986 MB）+ 自带 gmake 4.4.1，然后：

```toml
[target.'cfg(target_os = "android")'.dependencies]
native-tls = { version = "0.2", features = ["vendored"] }   # = openssl/vendored
```

`cargo build --target aarch64-linux-android` 在 79 秒后失败：

```
running "perl" "./Configure" ... "no-shared" "no-module" "no-tests" "no-comp"
Configuring OpenSSL version 3.6.3 for target linux-aarch64
cargo:warning=configuring OpenSSL build: 'perl' reported failure with exit code: 255
Failure!  Makefile wasn't produced.
This perl implementation doesn't produce Unix like paths (with forward slash)
This Perl version: 5.42.3 for MSWin32-x64-multi-thread
```

**根因不是配置错误，是 OpenSSL 的硬限制**：目标为 `linux-aarch64` 时，
OpenSSL 的 `Configure` 要求宿主 perl 输出 **Unix 风格路径**；原生 Windows perl 给的是
`D:\...`，它直接拒绝生成 Makefile。这是 Windows 上交叉编译 OpenSSL 的著名坑。

配置已回退，工作区干净，桌面应用未受影响。

### 9.5 现在还剩哪些路

| 路 | 把握 | 代价 |
| --- | --- | --- |
| **C1. 装 MSYS2，用它的 perl + make** | 中 | MSYS2 的 perl 是 POSIX 风格，`Configure` 可能通过；但还要再装 ~1 GB，且**不保证**后面没有新坑 |
| **C2. 用网上预编译的 Android OpenSSL** | 高 | 往**加密路径**里塞来路不明的二进制，供应链风险，不推荐 |
| **C3. 退路：桌面 native-tls + 安卓 rustls** | 高 | 一套代码走不通，要按平台条件编译；**sina 的 IMAP 在安卓上用不了**（其余 7 家 + sina 发信正常） |

**推荐 C3**：它是唯一能把「已真机验证过的桌面端」原样保住的方案，
代价只是安卓上少一家服务商的收信能力，且这条限制可以如实写在界面上。

### 9.6 决定：走 C3（2026-10-17，用户确认）

**桌面用 native-tls（SChannel），Android 用 rustls。**

#### 落地方式（下一步照这个做）

不要在两处写 `#[cfg]` 散弹，而是抽一个极薄的**内部 TLS 抽象**，
让 `imap_client` 与 `lettre` 各自按目标选实现：

```toml
[target.'cfg(not(target_os = "android"))'.dependencies]
async-native-tls = { version = "0.6", default-features = false, features = ["runtime-tokio"] }
lettre = { version = "0.11", features = ["tokio1-native-tls"] }

[target.'cfg(target_os = "android")'.dependencies]
rustls = { version = "0.23", default-features = false, features = ["ring", "std", "tls12"] }
tokio-rustls = { version = "0.26", default-features = false, features = ["ring", "tls12"] }
webpki-roots = "0.26"
lettre = { version = "0.11", default-features = false, features = [
  "smtp-transport", "pool", "hostname", "builder", "tokio1-rustls-tls",
] }
```

Cargo 允许同一个 crate 在两个 target 段里声明不同 feature，实际只会生效一个。

代码侧新增 `src-tauri/src/mail/tls.rs`，对外只暴露两样东西：

```rust
pub type TlsStream = ...;                       // 两套实现各给一个具体类型
pub async fn connect(host: &str, tcp: TcpStream) -> Result<TlsStream, MailError>;
```

`imap_client.rs` 只调 `tls::connect`，不认后端；`MailError` 的 TLS 变体用
`String` 承载（两个后端的错误类型不同，且 `Io` 已占了 `From<std::io::Error>`）。

#### 必须接受的行为差异（写进界面，不许藏着）

| 平台 | 后果 |
| --- | --- |
| 桌面（SChannel） | 8 家全通，sina 正常 |
| **Android（rustls）** | **sina 的 IMAP 连不上**（`HandshakeFailure`）。需要在同步失败时给一句可读的提示，而不是抛一句英文 TLS 错误 |

#### 验证要求（两条都不能省）

1. **桌面**：`cargo test` 全绿 + 三家真机登录复验（163 / sina / QQ），证明没被 cfg 改动弄坏
2. **Android**：`tauri android build --apk --target aarch64` 出包，装到模拟器/真机上能启动、能登录、能收信
3. **sina 在安卓上的失败必须实测一次**，确认提示是那句可读的话，而不是白屏或崩溃


