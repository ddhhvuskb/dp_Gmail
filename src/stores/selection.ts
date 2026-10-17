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

/**
 * 窄屏（手机）显示「列表」还是「详情」。桌面端忽略。
 *
 * 侧栏**不在**这个枚举里：它在窄屏是一个抽屉（见 `drawerOpen`），
 * 盖在邮件区上面而不是跟它们轮流占满屏幕。
 */
export type NavPane = "list" | "detail";

/** 待发附件。给**路径**而非字节，理由见 `commands/compose.rs`。 */
export interface ComposeAttachment {
  path: string;
  filename: string;
}

/** 写信草稿。`html` 为空串表示只发纯文本。 */
export interface ComposeDraft {
  to: string;
  subject: string;
  text: string;
  html: string;
  attachments: ComposeAttachment[];
}

interface SelectionState {
  accountId: number | null;
  mode: ListMode;
  messageId: number | null;
  /** 窄屏显示列表还是详情。桌面端不看它。 */
  nav: NavPane;
  /** 窄屏侧栏抽屉是否展开。桌面端侧栏常驻，不看它。 */
  drawerOpen: boolean;
  /** `null` 表示写信面板未打开。 */
  compose: ComposeDraft | null;

  setAccount: (accountId: number | null) => void;
  showFolder: (folderId: number, label: string) => void;
  showUnified: () => void;
  showSearch: (query: string) => void;
  clearList: () => void;
  selectMessage: (messageId: number | null) => void;
  setNav: (nav: NavPane) => void;
  setDrawer: (open: boolean) => void;

  openCompose: (draft?: Partial<ComposeDraft>) => void;
  updateCompose: (patch: Partial<ComposeDraft>) => void;
  closeCompose: () => void;
}

const EMPTY_DRAFT: ComposeDraft = { to: "", subject: "", text: "", html: "", attachments: [] };

export const useSelection = create<SelectionState>((set) => ({
  accountId: null,
  mode: { kind: "none" },
  messageId: null,
  nav: "list",
  drawerOpen: false,
  compose: null,

  // 换账户会连带清空列表与选中邮件，避免跨账户的残留选中态
  setAccount: (accountId) => set({ accountId, mode: { kind: "none" }, messageId: null }),

  // 下面三个动作顺带收抽屉：用户点了文件夹，抽屉就该让开，
  // 否则窄屏上点完还得自己再关一次
  showFolder: (folderId, label) =>
    set({
      mode: { kind: "folder", folderId, label },
      messageId: null,
      nav: "list",
      drawerOpen: false,
    }),
  showUnified: () =>
    set({ mode: { kind: "unified" }, messageId: null, nav: "list", drawerOpen: false }),
  showSearch: (query) =>
    set({ mode: { kind: "search", query }, messageId: null, nav: "list", drawerOpen: false }),
  clearList: () => set({ mode: { kind: "none" }, messageId: null, nav: "list", drawerOpen: false }),

  selectMessage: (messageId) => set({ messageId, nav: messageId === null ? "list" : "detail" }),

  setNav: (nav) => set({ nav }),
  setDrawer: (open) => set({ drawerOpen: open }),

  openCompose: (draft) => set({ compose: { ...EMPTY_DRAFT, ...draft } }),
  updateCompose: (patch) =>
    set((state) => (state.compose ? { compose: { ...state.compose, ...patch } } : state)),
  closeCompose: () => set({ compose: null }),
}));
