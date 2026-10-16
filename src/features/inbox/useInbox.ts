/** 收件箱的数据访问 hooks。 */

import { useQuery } from "@tanstack/react-query";

import { api } from "../../lib/api";

/** 单页条数。后端会再钳一次上限（`MAX_PAGE_SIZE`）。 */
export const PAGE_SIZE = 200;

export function useAccounts() {
  return useQuery({
    queryKey: ["accounts"],
    queryFn: () => api.listAccounts(),
  });
}

export function useMessages(folderId: number | null) {
  return useQuery({
    queryKey: ["messages", folderId],
    queryFn: () => api.getMessages(folderId as number, PAGE_SIZE, 0),
    enabled: folderId !== null,
  });
}

export function useMessage(messageId: number | null) {
  return useQuery({
    queryKey: ["message", messageId],
    queryFn: () => api.getMessage(messageId as number),
    enabled: messageId !== null,
  });
}
