/** 收件箱的数据访问 hooks。 */

import { useQuery } from "@tanstack/react-query";

import { api } from "../../lib/api";
import type { ListMode } from "../../stores/selection";

/** 单页条数。后端会再钳一次上限。 */
export const PAGE_SIZE = 200;

/** 列表行的统一形状 —— 三种来源（文件夹 / 统一收件箱 / 搜索）在前端归一。 */
export interface ListRow {
  id: number;
  subject: string | null;
  from_name: string | null;
  from_address: string | null;
  date_unix: number;
  is_seen: boolean;
  attachment_count: number;
  /** 统一收件箱里显示来源；文件夹与搜索视图下为 undefined。 */
  source?: string;
}

export function useAccounts() {
  return useQuery({
    queryKey: ["accounts"],
    queryFn: () => api.listAccounts(),
  });
}

export function useFolders(accountId: number | null) {
  return useQuery({
    queryKey: ["folders", accountId],
    queryFn: () => api.listFolders(accountId),
  });
}

/**
 * 按当前列表模式取数据，并把三种来源归一成 `ListRow[]`。
 *
 * 归一放在这里而不是三个组件里各写一遍 —— 三处重复迟早会漂移。
 */
export function useListRows(mode: ListMode, accountId: number | null) {
  return useQuery({
    // mode 是判别联合，可直接进 queryKey；换了模式自然换缓存
    queryKey: ["list", mode, accountId],
    enabled: mode.kind !== "none",
    queryFn: async (): Promise<ListRow[]> => {
      switch (mode.kind) {
        case "folder": {
          const rows = await api.getMessages(mode.folderId, PAGE_SIZE, 0);
          return rows.map((row) => ({ ...row }));
        }
        case "unified": {
          const rows = await api.listUnifiedMessages(accountId, "INBOX", PAGE_SIZE, 0);
          return rows.map((row) => ({
            id: row.id,
            subject: row.subject,
            from_name: row.from_name,
            from_address: row.from_address,
            date_unix: row.date_unix,
            is_seen: row.is_seen,
            attachment_count: row.attachment_count,
            source: `${row.account_email} · ${row.folder_name}`,
          }));
        }
        case "search": {
          const rows = await api.searchMessages(mode.query, PAGE_SIZE);
          return rows.map((row) => ({ ...row }));
        }
        default:
          return [];
      }
    },
  });
}

export function useMessage(messageId: number | null) {
  return useQuery({
    queryKey: ["message", messageId],
    queryFn: () => api.getMessage(messageId as number),
    enabled: messageId !== null,
  });
}
