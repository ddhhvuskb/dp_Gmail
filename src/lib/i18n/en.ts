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

  "format.noDate": "No date",
  "format.unknownSender": "Unknown sender",
};
