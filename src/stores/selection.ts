/**
 * 客户端选择状态。
 *
 * 用 Zustand 承载**纯客户端**状态（当前选中哪个账户 / 文件夹 / 邮件）；
 * 服务器数据一律交给 TanStack Query（见 `features/inbox/useInbox.ts`）。
 * 两者职责不重叠 —— 这是 `DESIGN.md` 技术栈里两个状态库的分工。
 */

import { create } from "zustand";

interface SelectionState {
  accountId: number | null;
  folderId: number | null;
  messageId: number | null;

  setAccount: (accountId: number | null) => void;
  setFolder: (folderId: number | null) => void;
  selectMessage: (messageId: number | null) => void;
}

export const useSelection = create<SelectionState>((set) => ({
  accountId: null,
  folderId: null,
  messageId: null,

  // 换账户会连带清掉文件夹与邮件，避免选中态跨账户残留
  setAccount: (accountId) => set({ accountId, folderId: null, messageId: null }),
  setFolder: (folderId) => set({ folderId, messageId: null }),
  selectMessage: (messageId) => set({ messageId }),
}));
