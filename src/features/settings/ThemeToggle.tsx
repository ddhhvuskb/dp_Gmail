/**
 * 明 / 暗快捷切换。
 *
 * 侧栏顶部，一眼可见 —— 用户夜里觉得刺眼时不必去翻设置。
 *
 * 有了多套主题之后，这个按钮的语义是「**亮度**快捷切换」：
 * 它在最近用过的浅色主题与深色主题之间来回翻，所以从「护眼米色」点一下
 * 会回到上次的深色主题，而不是被重置成默认的「暗」。
 * 想换具体是哪一套，用下面的 `AppearancePicker`。
 *
 * 按钮文字显示的是**当前亮度**；`title` 说明点下去会切到哪边，两者不是一回事。
 */

import { useT } from "../../lib/i18n/useT";
import { isDarkFamily, useTheme } from "../../stores/theme";

export default function ThemeToggle() {
  const theme = useTheme((state) => state.theme);
  const toggle = useTheme((state) => state.toggle);
  const t = useT();
  const isDark = isDarkFamily(theme);

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
