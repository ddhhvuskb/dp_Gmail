/**
 * 外观选择：主题基调 + 重点色。
 *
 * 放在侧栏顶部，与语言选择同处「只影响显示」的一组偏好里。
 *
 * 重点色用一排小圆点而不是下拉框：颜色这种东西**看到比读到快**。
 * 圆点本身不带字面色值 —— 它靠 `data-accent` 让 CSS 里的 `--swatch` 生效，
 * 所以色相只在 `App.css` 里定义一次。
 */

import { useT } from "../../lib/i18n/useT";
import type { Key } from "../../lib/i18n";
import {
  ACCENT_IDS,
  isAccentId,
  isThemeId,
  THEME_IDS,
  useTheme,
  type AccentId,
  type ThemeId,
} from "../../stores/theme";

/** 主题 ID → 词典 key。写成显式映射，`t()` 的 key 才能被类型检查。 */
const THEME_LABEL: Record<ThemeId, Key> = {
  light: "themeName.light",
  dark: "themeName.dark",
  sepia: "themeName.sepia",
  midnight: "themeName.midnight",
};

const ACCENT_LABEL: Record<AccentId, Key> = {
  azure: "accentName.azure",
  emerald: "accentName.emerald",
  violet: "accentName.violet",
  amber: "accentName.amber",
  rose: "accentName.rose",
};

export default function AppearancePicker() {
  const theme = useTheme((state) => state.theme);
  const accent = useTheme((state) => state.accent);
  const setTheme = useTheme((state) => state.setTheme);
  const setAccent = useTheme((state) => state.setAccent);
  const t = useT();

  return (
    <div className="appearance">
      <div className="appearance__row">
        <span className="appearance__label">{t("appearance.theme")}</span>
        <select
          className="appearance__select"
          value={theme}
          onChange={(event) => {
            // 不用 `as ThemeId`：DOM 里拿到的永远是 string，校验一下更安全
            if (isThemeId(event.target.value)) {
              setTheme(event.target.value);
            }
          }}
        >
          {THEME_IDS.map((id) => (
            <option key={id} value={id}>
              {t(THEME_LABEL[id])}
            </option>
          ))}
        </select>
      </div>

      <div className="appearance__row">
        <span className="appearance__label">{t("appearance.accent")}</span>
        <div className="appearance__swatches">
          {ACCENT_IDS.map((id) => (
            <button
              key={id}
              type="button"
              // 让该元素取到自己的 --swatch，颜色不在 TSX 里写死
              data-accent={id}
              className={`appearance__swatch${accent === id ? " appearance__swatch--active" : ""}`}
              onClick={() => {
                if (isAccentId(id)) {
                  setAccent(id);
                }
              }}
              title={t(ACCENT_LABEL[id])}
              aria-label={t(ACCENT_LABEL[id])}
              aria-pressed={accent === id}
            />
          ))}
        </div>
      </div>
    </div>
  );
}
