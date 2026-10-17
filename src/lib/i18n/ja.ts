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
  "accountList.edit": "編集",
  "accountList.delete": "削除",
  "accountList.deleteTitle": "アカウントを削除しますか？",
  "accountList.deleteWarn":
    "このアカウントで取得済みのメールと添付ファイルも同時に削除されます。元に戻せません。",
  "accountList.confirmDelete": "削除する",
  "accountList.cancel": "キャンセル",
  "accountList.deleting": "削除中…",

  "accountEdit.heading": "アカウントを編集",
  "accountEdit.secret": "新しい認証コード（空欄なら変更しない）",
  "accountEdit.save": "保存",
  "accountEdit.saving": "保存中…",
  "accountEdit.cancel": "キャンセル",

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
  "compose.from": "送信元アカウント",
  "compose.close": "閉じる",
  "compose.noAccount": "アカウントが未設定です。サイドバーの「アカウント追加」で追加してください。",
  "compose.to": "宛先（複数はカンマ区切り）",
  "compose.subject": "件名",
  "compose.bodyText": "本文（プレーンテキスト）",
  "compose.bodyHtml": "HTML 本文（任意。入力すると multipart/alternative で送信されます）",
  "compose.bodyHtmlPlaceholder": "<p>リッチテキスト本文</p>",
  "compose.send": "送信",
  "compose.sentFolder": "送信済み",
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

  "nav.sidebar": "アカウントとフォルダー",
  "nav.backToList": "一覧に戻る",
  "nav.openSidebar": "アカウントとフォルダーを開く",
  "nav.closeSidebar": "サイドバーを閉じる",

  "theme.light": "ライトモード",
  "theme.dark": "ダークモード",
  "theme.switchToLight": "ライトモードに切り替え",
  "theme.switchToDark": "ダークモードに切り替え（夜間の目に優しい）",
  "language.label": "言語",

  "appearance.theme": "テーマ",
  "appearance.accent": "アクセント",
  "themeName.light": "ライト",
  "themeName.dark": "ダーク",
  "themeName.sepia": "セピア",
  "themeName.midnight": "ミッドナイト",
  "accentName.azure": "ブルー",
  "accentName.emerald": "グリーン",
  "accentName.violet": "パープル",
  "accentName.amber": "オレンジ",
  "accentName.rose": "ローズ",

  "format.noDate": "日付なし",
  "format.unknownSender": "不明な差出人",

  "settings.open": "設定",
  "settings.title": "設定",
  "settings.close": "設定を閉じる",
  "settings.appearance": "外観",
  "settings.language": "言語",
  "settings.autoSync": "自動受信",
  "settings.about": "このアプリについて",
  "settings.contact": "お問い合わせ",
  "settings.privacy": "プライバシーポリシー",

  "autoSync.label": "自動受信",
  "autoSync.hint":
    "15 秒ごとに現在のフォルダーを取得します。ウィンドウが最小化されている間は停止し、リクエストは重複しません。",
  "autoSync.on": "オン",
  "autoSync.off": "オフ",
  "autoSync.toolbarHint": "自動受信：15 秒ごと",

  "about.intro":
    "ローカルファーストのデスクトップメールクライアント。メールと認証情報はこの PC にのみ保存され、中継サーバーを経由しません。",
  "about.version": "バージョン",
  "about.stack": "技術スタック",
  "about.stackValue": "Tauri v2 · Rust · React · SQLite（FTS5 全文検索）",
  "about.providers": "対応プロバイダー",
  "about.providersValue": "11 件：個人向け 8 件 + 企業向け 3 件",
  "contact.hint": "ご質問・ご要望はメールでどうぞ：",

  "privacy.effective": "施行日 2026-10-17 · dp_Gmail 0.1.0 に適用",
  "privacy.p1":
    "ひと言で：dp_Gmail はローカルファーストのクライアントです。メールと認証情報はこの PC にのみ保存され、当方のサーバーは存在せず、いかなるデータも収集しません。",
  "privacy.p2":
    "収集するもの：何も収集しません。アカウント登録、アクセス解析、クラッシュレポート、広告 ID のいずれもありません。サーバーがないため、「データを当方へ送る」経路自体が存在しません。",
  "privacy.p3":
    "データの保存場所：すべて本機の %APPDATA%\\com.dpgmail.desktop\\ に保存されます。dp_gmail.sqlite にメール・添付情報・アカウント・フォルダー、key.json に暗号化鍵、同ディレクトリの -wal / -shm は SQLite の一時ログです。",
  "privacy.p4":
    "認証情報の保護：認証コード／パスワードは AES-256-GCM で暗号化してからデータベースに書き込みます。鍵は同じディレクトリの key.json にあり、平文はログインの瞬間にメモリ上に存在するだけです。ただし、この 2 ファイルを読める人は認証情報を復号できます。暗号化が守るのは「データベースファイルだけが持ち出された場合」であり、「端末全体が掌握された場合」ではありません。",
  "privacy.p5":
    "通信するとき：受信・送信・自動受信の際に、あなたが設定したプロバイダーの IMAP / SMTP サーバーへ接続するだけで、常に TLS を使用します。それ以外のリクエストは行いません。自動アップデート確認、リモート設定、統計送信はいずれもありません。",
  "privacy.p6":
    "メール本文：表示前にサニタイズされます。スクリプト、スタイル、イベント属性、javascript: リンクは除去され、メール内のコードがこの PC で実行されることはありません。",
  "privacy.p7":
    "第三者：あなたのメールを見るのは、プロバイダー自身だけです。これはメールプロトコル上の性質で、どのクライアントを使っても変わりません。当方は第三者とデータを共有しません — そもそも保持していないためです。",
  "privacy.p8":
    "削除とバックアップ：アプリ終了後に上記ディレクトリを削除すれば、メール・アカウント・鍵がすべて消え、残留も復元もできません。クラウドバックアップはありません。機種変更や再インストールの前にご自身でバックアップしてください。",
  "privacy.p9":
    "補足：画面描画エンジンは OS 提供（Windows では WebView2）であり、その挙動は本アプリの管理外です。本方針はバージョンとともに更新され、変更はリポジトリの docs/privacy/ に記録されます。",
};
