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
} as const;

/** 词典的 key 集合。另两门语言必须一一覆盖。 */
export type Key = keyof typeof zh;
