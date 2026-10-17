/**
 * 白天 / 黑夜模式切换。
 *
 * 放在侧栏顶部，一眼可见 —— 用户夜里觉得刺眼时不必去翻设置页。
 */

import { useAppliedTheme, useTheme } from "../../stores/theme";

export default function ThemeToggle() {
  const theme = useAppliedTheme();
  const toggle = useTheme((state) => state.toggle);
  const isDark = theme === "dark";

  return (
    <button
      type="button"
      className="theme-toggle"
      onClick={toggle}
      title={isDark ? "切换到白天模式" : "切换到黑夜模式（夜间护眼）"}
    >
      <span aria-hidden="true">{isDark ? "🌙" : "☀️"}</span>
      <span>{isDark ? "黑夜模式" : "白天模式"}</span>
    </button>
  );
}
