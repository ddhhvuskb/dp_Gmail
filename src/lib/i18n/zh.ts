/**
 * 中文词典 —— **所有 key 的唯一来源**。
 *
 * 另两门语言被约束成 `Record<Key, string>`（见 `ja.ts` / `en.ts`），
 * 所以少写一个 key、或多写一个不存在的 key，都会在 `tsc --noEmit` 阶段报错，
 * 而不是运行时静默回退成中文。这是本项目选「手写字典」而非引入 i18n 库的**唯一理由**：
 * 类型系统替我们做完整性检查。
 *
 * 命名空间前缀与目录对应：`app` / `list` / `search` / `toolbar` / `folderTree` /
 * `accountForm` / `accountList` / `provider` / `detail` / `compose` / `attachment` /
 * `drafts` / `theme` / `language` / `format`。
 *
 * ⚠️ **Rust 侧错误消息不在本词典内**。`commands/` 返回的 `String` 保持中文，
 * 见 `docs/adr/ADR-0005-frontend-i18n-backend-errors-zh.md`。
 */

export const zh = {
  // ── 侧栏 / 外壳 ─────────────────────────────────────────────
  "app.compose": "✏️ 写邮件",
  "app.accountsHeading": "已配置账户",
  "app.syncHeading": "同步",
  "app.foldersHeading": "文件夹",
  "app.emptyListHint": "同步一个文件夹，或点「统一收件箱」。",
  "app.loading": "读取邮件…",
  "app.selectMessage": "选一封邮件查看详情。",
  "app.reply": "回复",
  "app.forward": "转发",

  // ── 中栏标题 / 空态 ─────────────────────────────────────────
  "list.titleUnified": "统一收件箱",
  "list.titleSearch": "搜索：{query}",
  "list.titleDefault": "邮件",
  "list.emptySearch": "没有匹配「{query}」的邮件。",
  "list.emptyUnified": "统一收件箱里还没有邮件，先同步一个文件夹。",
  "list.emptyFolder": "这个文件夹里还没有邮件，先同步一次。",
  "list.noMessages": "这里没有邮件。",
  "list.loadingMore": "正在加载更多…",
  "list.listLabel": "邮件列表",
  "list.noSubject": "(无主题)",
  "list.hasAttachment": "含附件",

  // ── 搜索框 ─────────────────────────────────────────────────
  "search.placeholder": "搜索邮件…",
  "search.label": "搜索邮件",
  "search.submit": "搜索",
  "search.clear": "清除",

  // ── 同步工具条 ──────────────────────────────────────────────
  "toolbar.account": "账户",
  "toolbar.mailbox": "文件夹",
  "toolbar.selectPlaceholder": "— 请选择 —",
  "toolbar.sync": "同步",
  "toolbar.syncing": "同步中…",
  "toolbar.syncReport": "拉取 {fetched} 封，写入 {stored} 封",
  "toolbar.noAccounts": "先在左侧「添加账户」里配置一个邮箱。",

  // ── 文件夹树 ────────────────────────────────────────────────
  "folderTree.unified": "📥 统一收件箱",
  "folderTree.empty": "还没有同步过文件夹。",
  "folderTree.accountFallback": "账户 {id}",

  // ── 添加账户表单 ────────────────────────────────────────────
  "accountForm.heading": "添加账户",
  "accountForm.email": "邮箱地址",
  "accountForm.secret": "授权码 / 密码",
  "accountForm.displayName": "显示名（可选）",
  "accountForm.submit": "添加账户",
  "accountForm.saving": "保存中…",

  // ── 已配置账户列表 ──────────────────────────────────────────
  "accountList.empty": "还没有账户。用上面的表单添加一个。",
  "accountList.hasSecret": "已存凭证",
  "accountList.noSecret": "⚠ 无凭证",
  "accountList.edit": "编辑",
  "accountList.delete": "删除",
  "accountList.deleteTitle": "删除账户？",
  "accountList.deleteWarn": "会同时删除该账户已下载的邮件与附件，不可恢复。",
  "accountList.confirmDelete": "确认删除",
  "accountList.cancel": "取消",
  "accountList.deleting": "删除中…",

  "accountEdit.heading": "编辑账户",
  "accountEdit.secret": "新授权码（留空＝不修改）",
  "accountEdit.save": "保存",
  "accountEdit.saving": "保存中…",
  "accountEdit.cancel": "取消",

  // ── 服务商识别 / 选择 ───────────────────────────────────────
  "provider.detected": "已识别：{name}（{host}:{port}）",
  "provider.notDetected": "未能自动识别该邮箱的服务商，请手动选择",
  "provider.label": "服务商",
  "provider.loading": "读取中…",
  "provider.placeholder": "— 请选择 —",

  // ── 详情 ────────────────────────────────────────────────────
  "detail.from": "发件人",
  "detail.to": "收件人",
  "detail.date": "时间",
  "detail.noBody": "(无正文)",

  // ── 写信 ────────────────────────────────────────────────────
  "compose.title": "写邮件",
  "compose.close": "关闭",
  "compose.noAccount": "请先在左侧选择一个账户",
  "compose.to": "收件人（多个用逗号分隔）",
  "compose.subject": "主题",
  "compose.bodyText": "正文（纯文本）",
  "compose.bodyHtml": "HTML 正文（可选，填写后以 multipart/alternative 发送）",
  "compose.bodyHtmlPlaceholder": "<p>富文本正文</p>",
  "compose.send": "发送",
  "compose.sending": "发送中…",
  "compose.sent": "已发送",

  // ── 附件 ────────────────────────────────────────────────────
  "attachment.label": "附件",
  "attachment.add": "📎 添加附件",
  "attachment.pickerTitle": "选择附件",
  "attachment.remove": "移除 {filename}",

  // ── 回复 / 转发草稿 ─────────────────────────────────────────
  "drafts.replyAttribution": "在 {date}，{sender} 写道：",
  "drafts.forwardHeader": "---------- 转发邮件 ----------",
  "drafts.forwardFrom": "发件人：{value}",
  "drafts.forwardTo": "收件人：{value}",
  "drafts.forwardDate": "时间：{value}",
  "drafts.forwardSubject": "主题：{value}",

  // ── 主题 / 语言开关 ─────────────────────────────────────────
  // 图标由组件里的独立 `<span aria-hidden>` 负责，词典里**不带** emoji，否则会重复。
  "theme.light": "白天模式",
  "theme.dark": "黑夜模式",
  "theme.switchToLight": "切换到白天模式",
  "theme.switchToDark": "切换到黑夜模式（夜间护眼）",
  "language.label": "语言",

  // ── 外观：主题基调 + 重点色 ─────────────────────────────────
  "appearance.theme": "主题",
  "appearance.accent": "重点色",
  "themeName.light": "明",
  "themeName.dark": "暗",
  "themeName.sepia": "护眼米色",
  "themeName.midnight": "深空蓝",
  "accentName.azure": "蓝",
  "accentName.emerald": "绿",
  "accentName.violet": "紫",
  "accentName.amber": "橙",
  "accentName.rose": "玫红",

  // ── 格式化 ──────────────────────────────────────────────────
  "format.noDate": "无日期",
  "format.unknownSender": "未知发件人",

  // ── 设置 ────────────────────────────────────────────────────
  "settings.open": "设置",
  "settings.title": "设置",
  "settings.close": "关闭设置",
  "settings.appearance": "外观",
  "settings.language": "语言",
  "settings.autoSync": "自动收信",
  "settings.about": "关于我们",
  "settings.contact": "联系方式",
  "settings.privacy": "隐私政策",

  // ── 自动收信 ────────────────────────────────────────────────
  "autoSync.label": "自动收信",
  "autoSync.hint": "每 15 秒拉取一次当前信箱。窗口最小化时暂停，不叠加并发请求。",
  "autoSync.on": "已开启",
  "autoSync.off": "已关闭",
  "autoSync.toolbarHint": "自动收信：每 15 秒",

  // ── 关于我们 / 联系方式 ─────────────────────────────────────
  "about.intro": "本地优先的桌面邮件客户端。邮件与授权码只存在这台电脑上，不经任何中转服务器。",
  "about.version": "版本",
  "about.stack": "技术栈",
  "about.stackValue": "Tauri v2 · Rust · React · SQLite（FTS5 全文检索）",
  "about.providers": "支持服务商",
  "about.providersValue": "11 条：8 家个人邮箱 + 3 家企业邮箱",
  "contact.hint": "有问题或建议，欢迎邮件联系：",

  // ── 隐私政策 ────────────────────────────────────────────────
  "privacy.effective": "生效日期 2026-10-17 · 适用于 dp_Gmail 0.1.0",
  "privacy.p1":
    "一句话：dp_Gmail 是本地优先的客户端。你的邮件与授权码只存在这台电脑上，我们没有任何服务器，也不收集任何数据。",
  "privacy.p2":
    "我们收集什么：什么都不收集。没有账号体系，没有统计埋点，没有崩溃上报，没有广告标识。我们没有服务器，所以不存在「把数据传给我们」这条路径。",
  "privacy.p3":
    "数据存在哪里：全部在本机的 %APPDATA%\\com.dpgmail.desktop\\ 目录。dp_gmail.sqlite 存邮件、附件信息、账户与文件夹；key.json 存加密密钥；同目录下的 -wal / -shm 是 SQLite 的临时日志。",
  "privacy.p4":
    "授权码怎么保护：授权码/密码用 AES-256-GCM 加密后才写入数据库，密钥是同目录下的 key.json，明文只在登录那一刻存在于内存。需要说明的是：能读到这台电脑上这两个文件的人就能解出授权码 —— 加密防的是「数据库文件被单独拷走」，挡不住「整台机器被控制」。",
  "privacy.p5":
    "什么时候联网：只在收信、发信与自动收信时，连接你自己配置的邮件服务商的 IMAP / SMTP 服务器，且始终走 TLS。除此之外不发起任何请求：没有自动更新检查，没有远程配置，没有统计上报。",
  "privacy.p6":
    "邮件正文：显示前会被净化 —— 脚本、样式、事件属性与 javascript: 链接都会被剥掉，避免邮件里的代码在你的电脑上执行。",
  "privacy.p7":
    "第三方：唯一会看到你邮件的是你的邮件服务商本身，这是邮件协议决定的，与使用哪个客户端无关。我们不向任何第三方共享数据，因为我们根本没有这些数据。",
  "privacy.p8":
    "删除与备份：退出应用后删除上面那个目录，邮件、账户与密钥会一并消失，无残留、不可恢复。本机数据没有云端备份，换机或重装前请自行备份。",
  "privacy.p9":
    "补充说明：界面渲染引擎由系统提供（Windows 上为 WebView2），它自身的行为不在本应用控制范围内。本政策随版本更新，改动会记录在仓库的 docs/privacy/ 下。",
} as const;

/** 词典的 key 集合。另两门语言必须一一覆盖。 */
export type Key = keyof typeof zh;
