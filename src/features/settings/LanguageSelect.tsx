/**
 * 语言切换。
 *
 * 与 `ThemeToggle` 并排在侧栏顶部：两者都是「只影响显示、不影响数据」的偏好，
 * 放在一起用户不必去翻第二处。
 *
 * 选项名用**该语言自己的写法**（`中文` / `日本語` / `English`），不随当前界面语言变 ——
 * 否则界面全变成看不懂的文字时，用户就找不回自己的语言了。
 */

import { isLang, LANG_OPTIONS } from "../../lib/i18n";
import { useT } from "../../lib/i18n/useT";
import { useLanguage } from "../../stores/language";

export default function LanguageSelect() {
  const lang = useLanguage((state) => state.lang);
  const setLang = useLanguage((state) => state.setLang);
  const t = useT();

  return (
    <label className="language-select">
      <span className="language-select__label">{t("language.label")}</span>
      <select
        className="language-select__input"
        value={lang}
        onChange={(event) => {
          // 不用 `as Lang` 断言：DOM 里拿到的永远是 string，
          // 万一将来加了选项却没进词典，这里会挡住而不是写进一个非法值。
          if (isLang(event.target.value)) {
            setLang(event.target.value);
          }
        }}
      >
        {LANG_OPTIONS.map((option) => (
          <option key={option.id} value={option.id}>
            {option.label}
          </option>
        ))}
      </select>
    </label>
  );
}
