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
