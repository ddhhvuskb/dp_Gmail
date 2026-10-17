/** 日本語辞書。`Record<Key, string>` 保证与中文词典逐 key 对齐。 */

import type { Key } from "./zh";

export const ja: Record<Key, string> = {
  "app.compose": "✏️ メール作成",
  "app.accountsHeading": "設定済みアカウント",
  "app.syncHeading": "同期",
  "app.foldersHeading": "フォルダー",
  "app.emptyListHint": "フォルダーを同期するか、「統合受信トレイ」をクリックしてください。",
  "app.loading": "メールを読み込み中…",
  "app.selectMessage": "メールを選択すると詳細が表示されます。",
  "app.reply": "返信",
  "app.forward": "転送",

  "list.titleUnified": "統合受信トレイ",
  "list.titleSearch": "検索：{query}",
  "list.titleDefault": "メール",
  "list.emptySearch": "「{query}」に一致するメールはありません。",
  "list.emptyUnified": "統合受信トレイにまだメールがありません。まずフォルダーを同期してください。",
  "list.emptyFolder": "このフォルダーにまだメールがありません。まず同期してください。",
  "list.noMessages": "メールがありません。",
  "list.loadingMore": "さらに読み込み中…",
  "list.listLabel": "メール一覧",
  "list.noSubject": "(件名なし)",
  "list.hasAttachment": "添付ファイルあり",

  "search.placeholder": "メールを検索…",
  "search.label": "メールを検索",
  "search.submit": "検索",
  "search.clear": "クリア",

  "toolbar.account": "アカウント",
  "toolbar.mailbox": "フォルダー",
  "toolbar.selectPlaceholder": "— 選択してください —",
  "toolbar.sync": "同期",
  "toolbar.syncing": "同期中…",
  "toolbar.syncReport": "{fetched} 件取得、{stored} 件保存",
  "toolbar.noAccounts": "左側の「アカウント追加」でメールボックスを設定してください。",

  "folderTree.unified": "📥 統合受信トレイ",
  "folderTree.empty": "まだフォルダーを同期していません。",
  "folderTree.accountFallback": "アカウント {id}",

  "accountForm.heading": "アカウント追加",
  "accountForm.email": "メールアドレス",
  "accountForm.secret": "認証コード / パスワード",
  "accountForm.displayName": "表示名（任意）",
  "accountForm.submit": "アカウント追加",
  "accountForm.saving": "保存中…",

  "accountList.empty": "アカウントがありません。上のフォームで追加してください。",
  "accountList.hasSecret": "認証情報あり",
  "accountList.noSecret": "⚠ 認証情報なし",

  "provider.detected": "識別済み：{name}（{host}:{port}）",
  "provider.notDetected":
    "このメールボックスのプロバイダーを自動識別できませんでした。手動で選択してください。",
  "provider.label": "プロバイダー",
  "provider.loading": "読み込み中…",
  "provider.placeholder": "— 選択してください —",

  "detail.from": "差出人",
  "detail.to": "宛先",
  "detail.date": "日時",
  "detail.noBody": "(本文なし)",

  "compose.title": "メール作成",
  "compose.close": "閉じる",
  "compose.noAccount": "左側でアカウントを選択してください",
  "compose.to": "宛先（複数はカンマ区切り）",
  "compose.subject": "件名",
  "compose.bodyText": "本文（プレーンテキスト）",
  "compose.bodyHtml": "HTML 本文（任意。入力すると multipart/alternative で送信されます）",
  "compose.bodyHtmlPlaceholder": "<p>リッチテキスト本文</p>",
  "compose.send": "送信",
  "compose.sending": "送信中…",
  "compose.sent": "送信しました",

  "attachment.label": "添付ファイル",
  "attachment.add": "📎 添付を追加",
  "attachment.pickerTitle": "添付ファイルを選択",
  "attachment.remove": "{filename} を削除",

  "drafts.replyAttribution": "{date} に {sender} が書きました：",
  "drafts.forwardHeader": "---------- 転送メール ----------",
  "drafts.forwardFrom": "差出人：{value}",
  "drafts.forwardTo": "宛先：{value}",
  "drafts.forwardDate": "日時：{value}",
  "drafts.forwardSubject": "件名：{value}",

  "theme.light": "ライトモード",
  "theme.dark": "ダークモード",
  "theme.switchToLight": "ライトモードに切り替え",
  "theme.switchToDark": "ダークモードに切り替え（夜間の目に優しい）",
  "language.label": "言語",

  "format.noDate": "日付なし",
  "format.unknownSender": "不明な差出人",
};
