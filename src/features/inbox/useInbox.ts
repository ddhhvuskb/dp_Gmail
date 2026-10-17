/** 收件箱的数据访问 hooks。 */

import { useCallback, useMemo } from "react";
import { useInfiniteQuery, useQuery } from "@tanstack/react-query";

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
 *
 * **用无限查询而非单页查询。** 以前只请求 `offset = 0` 一页，于是 200 这个
 * 「单次请求上限」事实上退化成了「产品功能上限」：900 封的文件夹只能看到
 * 最新 200 封，其余在界面上不可达。实测证明渲染能力根本不是瓶颈
 * （1200 行滚动零掉帧），卡住可达性的只有那个上限本身。
 * 见 `docs/tasks/perf-verification.md` §6.3。
 */
export function useListRows(mode: ListMode, accountId: number | null) {
  return useInfiniteQuery({
    // mode 是判别联合，可直接进 queryKey；换了模式自然换缓存
    queryKey: ["list", mode, accountId],
    enabled: mode.kind !== "none",
    initialPageParam: 0,
    queryFn: async ({ pageParam }): Promise<ListRow[]> => {
      switch (mode.kind) {
        case "folder": {
          const rows = await api.getMessages(mode.folderId, PAGE_SIZE, pageParam);
          return rows.map((row) => ({ ...row }));
        }
        case "unified": {
          const rows = await api.listUnifiedMessages(accountId, "INBOX", PAGE_SIZE, pageParam);
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
    getNextPageParam: (lastPage, _pages, lastPageParam) => {
      // 搜索结果的后端接口不收 offset，恒为单页
      if (mode.kind === "search") {
        return undefined;
      }
      // 满页才可能还有下一页；不满页说明到底了
      return lastPage.length < PAGE_SIZE ? undefined : lastPageParam + PAGE_SIZE;
    },
  });
}

/**
 * 把无限查询的原始结果整理成组件真正要用的两样东西：摊平的行、取下一页。
 *
 * 放在数据层而不是 `App` 里：外壳不该知道分页游标怎么算，
 * 也不该为了「哪些行要显示」写 `useMemo`。
 */
export function usePagedRows(mode: ListMode, accountId: number | null) {
  const query = useListRows(mode, accountId);

  const rows = useMemo(() => query.data?.pages.flat() ?? [], [query.data]);

  const loadMore = useCallback(() => {
    if (query.hasNextPage && !query.isFetchingNextPage) {
      void query.fetchNextPage();
    }
  }, [query.hasNextPage, query.isFetchingNextPage, query.fetchNextPage]);

  return { query, rows, loadMore };
}

export function useMessage(messageId: number | null) {
  return useQuery({
    queryKey: ["message", messageId],
    queryFn: () => api.getMessage(messageId as number),
    enabled: messageId !== null,
  });
}
