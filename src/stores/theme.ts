/** 主题状态：白天 / 黑夜。 */

import { useEffect } from "react";
import { create } from "zustand";

export type Theme = "light" | "dark";

const STORAGE_KEY = "dpgmail.theme";

/**
 * 初始主题。
 *
 * 优先用上次的选择；没有记录时**跟随系统**的 `prefers-color-scheme` ——
 * 用户的诉求是「晚上太亮」，而系统级深色偏好正是这个意图最直接的表达。
 */
function initialTheme(): Theme {
  const saved = localStorage.getItem(STORAGE_KEY);
  if (saved === "light" || saved === "dark") {
    return saved;
  }
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

interface ThemeState {
  theme: Theme;
  toggle: () => void;
}

export const useTheme = create<ThemeState>((set) => ({
  theme: initialTheme(),
  toggle: () =>
    set((state) => {
      const next: Theme = state.theme === "dark" ? "light" : "dark";
      localStorage.setItem(STORAGE_KEY, next);
      return { theme: next };
    }),
}));

/**
 * 把当前主题写到 `<html data-theme="…">` 上，并返回主题值。
 *
 * CSS 靠 `[data-theme="dark"]` 那组变量切色，所以这个副作用**必须挂上**，
 * 否则组件里拿到的一直是初始值、界面永远不变色。
 */
export function useAppliedTheme(): Theme {
  const theme = useTheme((state) => state.theme);
  useEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);
  return theme;
}
