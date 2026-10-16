/**
 * 客户端选择状态。
 *
 * 用 Zustand 承载**纯客户端**状态（当前看哪个文件夹 / 是否统一收件箱 / 搜索词）；
 * 服务器数据一律交给 TanStack Query（见 `features/inbox/useInbox.ts`）。
 */

import { create } from "zustand";

/** 中栏列表当前展示什么。用判别联合而不是几个并列布尔 —— 后者能表达出非法状态。 */
export type ListMode =
  | { kind: "none" }
  | { kind: "folder"; folderId: number; label: string }
  | { kind: "unified" }
  | { kind: "search"; query: string };

interface SelectionState {
  accountId: number | null;
  mode: ListMode;
  messageId: number | null;

  setAccount: (accountId: number | null) => void;
  showFolder: (folderId: number, label: string) => void;
  showUnified: () => void;
  showSearch: (query: string) => void;
  clearList: () => void;
  selectMessage: (messageId: number | null) => void;
}

export const useSelection = create<SelectionState>((set) => ({
  accountId: null,
  mode: { kind: "none" },
  messageId: null,

  // 换账户会连带清空列表与选中邮件，避免跨账户的残留选中态
  setAccount: (accountId) => set({ accountId, mode: { kind: "none" }, messageId: null }),
  showFolder: (folderId, label) =>
    set({ mode: { kind: "folder", folderId, label }, messageId: null }),
  showUnified: () => set({ mode: { kind: "unified" }, messageId: null }),
  showSearch: (query) => set({ mode: { kind: "search", query }, messageId: null }),
  clearList: () => set({ mode: { kind: "none" }, messageId: null }),
  selectMessage: (messageId) => set({ messageId }),
}));
