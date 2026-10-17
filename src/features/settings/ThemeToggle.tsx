/**
 * 白天 / 黑夜模式切换。
 *
 * 放在侧栏顶部，一眼可见 —— 用户夜里觉得刺眼时不必去翻设置页。
 *
 * 按钮文字显示的是**当前**主题；`title` 说明点下去会切到哪个，
 * 两者不是一回事，别合并。
 */

import { useT } from "../../lib/i18n/useT";
import { useAppliedTheme, useTheme } from "../../stores/theme";

export default function ThemeToggle() {
  const theme = useAppliedTheme();
  const toggle = useTheme((state) => state.toggle);
  const t = useT();
  const isDark = theme === "dark";

  return (
    <button
      type="button"
      className="theme-toggle"
      onClick={toggle}
      title={isDark ? t("theme.switchToLight") : t("theme.switchToDark")}
    >
      <span aria-hidden="true">{isDark ? "🌙" : "☀️"}</span>
      <span>{isDark ? t("theme.dark") : t("theme.light")}</span>
    </button>
  );
}
