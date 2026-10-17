/**
 * 外观状态：主题基调 + 重点色。
 *
 * 两个维度分开存，因为它们是**正交**的：用户可能想要「深空蓝 + 绿色重点色」。
 * 若把 20 种组合做成 20 个主题 ID，词典、CSS、选择器都要跟着翻倍。
 *
 * 持久化到 `localStorage`：存的是外观标识，**不是凭证**，符合 `DESIGN.md` 的限制。
 */

import { useEffect } from "react";
import { create } from "zustand";

/** 主题基调。每套在 `App.css` 里有一整组颜色变量。 */
export type ThemeId = "light" | "dark" | "sepia" | "midnight";

/** 重点色。只改色相，明度与饱和度跟基调走。 */
export type AccentId = "azure" | "emerald" | "violet" | "amber" | "rose";

export const THEME_IDS: ThemeId[] = ["light", "dark", "sepia", "midnight"];
export const ACCENT_IDS: AccentId[] = ["azure", "emerald", "violet", "amber", "rose"];

const THEME_KEY = "dpgmail.theme";
const ACCENT_KEY = "dpgmail.accent";

/** 基调是「深色系」还是「浅色系」。切换按钮据此决定往哪边翻。 */
export function isDarkFamily(theme: ThemeId): boolean {
  return theme === "dark" || theme === "midnight";
}

export function isThemeId(value: unknown): value is ThemeId {
  return value === "light" || value === "dark" || value === "sepia" || value === "midnight";
}

export function isAccentId(value: unknown): value is AccentId {
  return (
    value === "azure" ||
    value === "emerald" ||
    value === "violet" ||
    value === "amber" ||
    value === "rose"
  );
}

/**
 * 初始主题。
 *
 * 旧版本存的就是 `light` / `dark`，而它们仍是合法的 `ThemeId`，
 * 所以**老用户的选择能直接沿用**，不需要迁移逻辑。
 *
 * 没有记录时跟随系统的 `prefers-color-scheme` —— 用户的原始诉求是「晚上太亮」，
 * 系统级深色偏好正是这个意图最直接的表达。
 */
function initialTheme(): ThemeId {
  const saved = localStorage.getItem(THEME_KEY);
  if (isThemeId(saved)) {
    return saved;
  }
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function initialAccent(): AccentId {
  const saved = localStorage.getItem(ACCENT_KEY);
  return isAccentId(saved) ? saved : "azure";
}

interface AppearanceState {
  theme: ThemeId;
  accent: AccentId;
  /** 上一次用过的浅色主题 / 深色主题，供 `toggle()` 在两侧之间来回翻。 */
  lastLight: ThemeId;
  lastDark: ThemeId;

  setTheme: (theme: ThemeId) => void;
  setAccent: (accent: AccentId) => void;
  /** 明/暗快捷切换：在「上次的浅色」与「上次的深色」之间翻，不会重置成默认主题。 */
  toggle: () => void;
}

const initial = initialTheme();

export const useTheme = create<AppearanceState>((set) => ({
  theme: initial,
  accent: initialAccent(),
  lastLight: isDarkFamily(initial) ? "light" : initial,
  lastDark: isDarkFamily(initial) ? initial : "dark",

  setTheme: (theme) =>
    set(() => {
      localStorage.setItem(THEME_KEY, theme);
      return isDarkFamily(theme) ? { theme, lastDark: theme } : { theme, lastLight: theme };
    }),

  setAccent: (accent) =>
    set(() => {
      localStorage.setItem(ACCENT_KEY, accent);
      return { accent };
    }),
  toggle: () =>
    set((state) => {
      const next = isDarkFamily(state.theme) ? state.lastLight : state.lastDark;
      localStorage.setItem(THEME_KEY, next);
      return { theme: next };
    }),
}));

/**
 * 把当前外观写到 `<html>` 上，并返回它。
 *
 * CSS 靠 `[data-theme=…]` 与 `[data-accent=…]` 两组属性选择器切色，
 * 所以这个副作用**必须挂上**，否则组件里拿到的改了、界面永远不变色。
 *
 * 只挂一处（`App`），不要每个用到主题的组件都挂一遍 —— 那会重复写 DOM。
 */
export function useAppliedAppearance(): { theme: ThemeId; accent: AccentId } {
  const theme = useTheme((state) => state.theme);
  const accent = useTheme((state) => state.accent);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    document.documentElement.dataset.accent = accent;
  }, [theme, accent]);

  return { theme, accent };
}
