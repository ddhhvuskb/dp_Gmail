/** English dictionary. `Record<Key, string>` forces one-to-one coverage with the Chinese one. */

import type { Key } from "./zh";

export const en: Record<Key, string> = {
  "app.compose": "✏️ Compose",
  "app.accountsHeading": "Accounts",
  "app.syncHeading": "Sync",
  "app.foldersHeading": "Folders",
  "app.emptyListHint": 'Sync a folder, or click "Unified inbox".',
  "app.loading": "Loading messages…",
  "app.selectMessage": "Select a message to view its details.",
  "app.reply": "Reply",
  "app.forward": "Forward",

  "list.titleUnified": "Unified inbox",
  "list.titleSearch": "Search: {query}",
  "list.titleDefault": "Messages",
  "list.emptySearch": 'No messages matching "{query}".',
  "list.emptyUnified": "The unified inbox is empty. Sync a folder first.",
  "list.emptyFolder": "This folder has no messages yet. Sync it first.",
  "list.noMessages": "No messages here.",
  "list.loadingMore": "Loading more…",
  "list.listLabel": "Message list",
  "list.noSubject": "(no subject)",
  "list.hasAttachment": "Has attachments",

  "search.placeholder": "Search messages…",
  "search.label": "Search messages",
  "search.submit": "Search",
  "search.clear": "Clear",

  "toolbar.account": "Account",
  "toolbar.mailbox": "Folder",
  "toolbar.selectPlaceholder": "— Select —",
  "toolbar.sync": "Sync",
  "toolbar.syncing": "Syncing…",
  "toolbar.syncReport": "Fetched {fetched}, stored {stored}",
  "toolbar.noAccounts": 'Add a mailbox in "Add account" on the left first.',

  "folderTree.unified": "📥 Unified inbox",
  "folderTree.empty": "No folders synced yet.",
  "folderTree.accountFallback": "Account {id}",

  "accountForm.heading": "Add account",
  "accountForm.email": "Email address",
  "accountForm.secret": "App password / password",
  "accountForm.displayName": "Display name (optional)",
  "accountForm.submit": "Add account",
  "accountForm.saving": "Saving…",

  "accountList.empty": "No accounts yet. Add one with the form above.",
  "accountList.hasSecret": "Credentials stored",
  "accountList.noSecret": "⚠ No credentials",
  "accountList.edit": "Edit",
  "accountList.delete": "Delete",
  "accountList.deleteTitle": "Delete this account?",
  "accountList.deleteWarn":
    "Its downloaded messages and attachments are deleted too. This cannot be undone.",
  "accountList.confirmDelete": "Delete",
  "accountList.cancel": "Cancel",
  "accountList.deleting": "Deleting…",

  "accountEdit.heading": "Edit account",
  "accountEdit.secret": "New app password (leave blank to keep the current one)",
  "accountEdit.save": "Save",
  "accountEdit.saving": "Saving…",
  "accountEdit.cancel": "Cancel",

  "provider.detected": "Detected: {name} ({host}:{port})",
  "provider.notDetected":
    "Could not auto-detect a provider for this mailbox. Please choose one manually.",
  "provider.label": "Provider",
  "provider.loading": "Loading…",
  "provider.placeholder": "— Select —",

  "detail.from": "From",
  "detail.to": "To",
  "detail.date": "Date",
  "detail.noBody": "(no body)",

  "compose.title": "Compose",
  "compose.close": "Close",
  "compose.noAccount": "Select an account on the left first",
  "compose.to": "To (separate multiple addresses with commas)",
  "compose.subject": "Subject",
  "compose.bodyText": "Body (plain text)",
  "compose.bodyHtml": "HTML body (optional; sent as multipart/alternative)",
  "compose.bodyHtmlPlaceholder": "<p>Rich text body</p>",
  "compose.send": "Send",
  "compose.sending": "Sending…",
  "compose.sent": "Sent",

  "attachment.label": "Attachments",
  "attachment.add": "📎 Add attachment",
  "attachment.pickerTitle": "Choose attachments",
  "attachment.remove": "Remove {filename}",

  "drafts.replyAttribution": "On {date}, {sender} wrote:",
  "drafts.forwardHeader": "---------- Forwarded message ----------",
  "drafts.forwardFrom": "From: {value}",
  "drafts.forwardTo": "To: {value}",
  "drafts.forwardDate": "Date: {value}",
  "drafts.forwardSubject": "Subject: {value}",

  "theme.light": "Light mode",
  "theme.dark": "Dark mode",
  "theme.switchToLight": "Switch to light mode",
  "theme.switchToDark": "Switch to dark mode (easier on the eyes at night)",
  "language.label": "Language",

  "appearance.theme": "Theme",
  "appearance.accent": "Accent",
  "themeName.light": "Light",
  "themeName.dark": "Dark",
  "themeName.sepia": "Sepia",
  "themeName.midnight": "Midnight",
  "accentName.azure": "Blue",
  "accentName.emerald": "Green",
  "accentName.violet": "Violet",
  "accentName.amber": "Amber",
  "accentName.rose": "Rose",

  "format.noDate": "No date",
  "format.unknownSender": "Unknown sender",

  "settings.open": "Settings",
  "settings.title": "Settings",
  "settings.close": "Close settings",
  "settings.appearance": "Appearance",
  "settings.language": "Language",
  "settings.autoSync": "Auto-check mail",
  "settings.about": "About",
  "settings.contact": "Contact",
  "settings.privacy": "Privacy policy",

  "autoSync.label": "Auto-check mail",
  "autoSync.hint":
    "Fetches the current folder every 15 seconds. Pauses while the window is minimised, and never stacks overlapping requests.",
  "autoSync.on": "On",
  "autoSync.off": "Off",
  "autoSync.toolbarHint": "Auto-check: every 15s",

  "about.intro":
    "A local-first desktop mail client. Messages and credentials live only on this computer; no relay server is involved.",
  "about.version": "Version",
  "about.stack": "Stack",
  "about.stackValue": "Tauri v2 · Rust · React · SQLite (FTS5 full-text search)",
  "about.providers": "Providers",
  "about.providersValue": "11 entries: 8 personal + 3 enterprise",
  "contact.hint": "Questions or suggestions — email me:",

  "privacy.effective": "Effective 2026-10-17 · applies to dp_Gmail 0.1.0",
  "privacy.p1":
    "In one sentence: dp_Gmail is a local-first client. Your mail and credentials live only on this computer. We run no servers and collect no data.",
  "privacy.p2":
    "What we collect: nothing. No accounts, no analytics, no crash reporting, no advertising identifiers. Because we have no servers, there is no path by which your data could reach us.",
  "privacy.p3":
    "Where data lives: entirely in %APPDATA%\\com.dpgmail.desktop\\ on this machine. dp_gmail.sqlite holds messages, attachment metadata, accounts and folders; key.json holds the encryption key; the -wal / -shm files beside it are SQLite's temporary logs.",
  "privacy.p4":
    "How credentials are protected: app passwords are encrypted with AES-256-GCM before being written to the database. The key is key.json in that same directory, and plaintext exists in memory only during login. To be explicit: anyone who can read both files on this machine can recover the password. The encryption protects against the database file being copied on its own — not against the whole machine being compromised.",
  "privacy.p5":
    "When it goes online: only to the IMAP / SMTP servers of the provider you configured, for receiving, sending and auto-checking, always over TLS. Nothing else is requested — no update checks, no remote configuration, no telemetry.",
  "privacy.p6":
    "Message bodies: sanitised before display. Scripts, styles, event attributes and javascript: links are stripped, so code inside a message cannot run on your computer.",
  "privacy.p7":
    "Third parties: the only party that sees your mail is your own mail provider — a property of the mail protocols, not of this client. We share nothing with third parties, because we hold nothing to share.",
  "privacy.p8":
    "Deletion and backup: quit the app and delete the directory above; messages, accounts and keys disappear with no residue and cannot be recovered. There is no cloud backup — back up that directory yourself before changing machines or reinstalling.",
  "privacy.p9":
    "One caveat: the rendering engine is provided by the OS (WebView2 on Windows), and its own behaviour is outside this app's control. This policy is updated with each release; changes are recorded under docs/privacy/ in the repository.",
};
