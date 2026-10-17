/**
 * Tauri 命令的类型化封装。
 *
 * 这些接口与后端 Rust 结构体一一对应，**改动任何一侧都必须同步另一侧**：
 * - `MessageSummary` ← `storage::message_store::MessageSummary`
 * - `MessageView`    ← `commands::messages::MessageView`
 * - `AccountView`    ← `commands::accounts::AccountView`
 * - `SyncReport`     ← `commands::sync::SyncReport`
 *
 * 参数名用 camelCase：Tauri v2 的 `#[tauri::command]` 默认按 camelCase
 * 解析前端载荷（见 `tauri-macros` 的 `rename_all` 文档）。
 */

import { invoke } from "@tauri-apps/api/core";

export interface AttachmentView {
  filename: string;
  content_type: string;
  size: number;
}

export interface MessageSummary {
  id: number;
  uid: number;
  subject: string | null;
  from_name: string | null;
  from_address: string | null;
  /** Unix 秒。 */
  date_unix: number;
  is_seen: boolean;
  attachment_count: number;
}

export interface MessageView {
  id: number;
  uid: number;
  subject: string | null;
  from_name: string | null;
  from_address: string | null;
  to_addresses: string | null;
  date_unix: number;
  is_seen: boolean;
  text_body: string | null;
  /** 已由后端 `security::sanitize_html` 净化。 */
  html_body: string | null;
  attachments: AttachmentView[];
}

export interface AccountView {
  id: number;
  email: string;
  display_name: string | null;
  provider_id: string;
  provider_name: string;
  has_secret: boolean;
}

export interface SyncReport {
  folder_id: number;
  fetched: number;
  stored: number;
}

/** 服务商识别结果。未收录的域名，命令返回 `null`。 */
export interface ProviderView {
  provider_id: string;
  provider_name: string;
  imap_host: string;
  imap_port: number;
  smtp_host: string;
  smtp_port: number;
}

export interface FolderRecord {
  id: number;
  account_id: number;
  name: string;
  uid_validity: number | null;
  last_sync_at: number | null;
  message_count: number;
  unread_count: number;
}

/** 统一收件箱的行：在 `MessageSummary` 之上补出来源账户与文件夹。 */
export interface UnifiedMessageSummary {
  id: number;
  uid: number;
  folder_id: number;
  folder_name: string;
  account_id: number;
  account_email: string;
  subject: string | null;
  from_name: string | null;
  from_address: string | null;
  date_unix: number;
  is_seen: boolean;
  attachment_count: number;
}

/** 发信附件入参：给**路径**而不是字节。 */
export interface AttachmentInput {
  path: string;
  /** 显示用的文件名；缺省取路径的文件名。 */
  filename?: string | null;
}

export const api = {
  listAccounts: () => invoke<AccountView[]>("list_accounts"),

  /** 由邮箱地址识别服务商，供表单实时提示。未收录返回 `null`。 */
  resolveProvider: (email: string) => invoke<ProviderView | null>("resolve_provider", { email }),

  /** 全部服务商，供「自动识别不出来时手动选择」用。 */
  listProviders: () => invoke<ProviderView[]>("list_providers"),

  /**
   * 新增账户。
   *
   * `providerId` 可选：企业邮箱用客户自己的域名，自动识别对它无解，
   * 只能由用户显式指定。不传则交给后端按域名自动识别。
   */
  addAccount: (email: string, secret: string, displayName?: string, providerId?: string) =>
    invoke<AccountView>("add_account", {
      email,
      secret,
      displayName: displayName ?? null,
      providerId: providerId ?? null,
    }),

  /**
   * 编辑已有账户。
   *
   * `secret` 传 `undefined` 表示**不改凭证** —— 只改显示名或邮箱时不必重填授权码。
   * 改邮箱会由后端重新解析服务商（地址变了，连接参数必须跟着变）。
   */
  updateAccount: (
    accountId: number,
    email: string,
    displayName?: string,
    secret?: string,
    providerId?: string,
  ) =>
    invoke<AccountView>("update_account", {
      accountId,
      email,
      displayName: displayName ?? null,
      secret: secret ?? null,
      providerId: providerId ?? null,
    }),

  /**
   * 删除账户，**连带删除它已下载的文件夹、邮件与附件**。
   *
   * 不可逆。调用方必须先让用户确认。
   */
  deleteAccount: (accountId: number) =>
    invoke<{ messages: number; attachments: number }>("delete_account", { accountId }),

  /** `accountId` 传 `null` 即列出全部账户的文件夹。 */
  listFolders: (accountId: number | null) => invoke<FolderRecord[]>("list_folders", { accountId }),
  getMessages: (folderId: number, limit: number, offset: number) =>
    invoke<MessageSummary[]>("get_messages", { folderId, limit, offset }),

  /** 统一收件箱。`mailbox` 传 `"INBOX"` 即各账户收件箱合并。 */
  listUnifiedMessages: (
    accountId: number | null,
    mailbox: string | null,
    limit: number,
    offset: number,
  ) =>
    invoke<UnifiedMessageSummary[]>("list_unified_messages", {
      accountId,
      mailbox,
      limit,
      offset,
    }),

  searchMessages: (query: string, limit: number) =>
    invoke<MessageSummary[]>("search_messages", { query, limit }),

  getMessage: (messageId: number) => invoke<MessageView>("get_message", { messageId }),

  syncFolder: (accountId: number, mailbox: string, limit: number) =>
    invoke<SyncReport>("sync_folder", { accountId, mailbox, limit }),

  /**
   * 发送一封邮件。
   *
   * 附件传路径而非字节 —— 前端传 `Uint8Array` 会被序列化成 JSON 数字数组，
   * 几 MB 就能把 IPC 打爆（见 `commands/compose.rs` 的说明）。
   */
  sendMail: (input: {
    accountId: number;
    to: string[];
    subject: string;
    bodyText: string;
    bodyHtml: string | null;
    attachments: AttachmentInput[];
  }) => invoke<void>("send_mail", input),

  /**
   * 把刚发出的一封信记进本地「已发送」。
   *
   * 只在**发送成功后**调用；投递失败的信不该出现在已发送里。
   * 返回的 `folder_id` 供前端直接跳到该文件夹。
   */
  recordSent: (
    accountId: number,
    to: string[],
    subject: string,
    bodyText: string,
    bodyHtml: string | null,
  ) =>
    invoke<{ folder_id: number; message_id: number }>("record_sent", {
      accountId,
      to,
      subject,
      bodyText,
      bodyHtml,
    }),
};
