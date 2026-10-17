/**
 * 语言状态：中文 / 日语 / 英语。
 *
 * 与 `theme.ts` 同构：Zustand + `localStorage` + 跟随系统。
 * 存的是语言标识，**不是凭证**，符合 `DESIGN.md` 对 `localStorage` 的限制。
 */

import { useEffect } from "react";
import { create } from "zustand";

import { isLang, localeOf, type Lang } from "../lib/i18n";

const STORAGE_KEY = "dpgmail.lang";

/**
 * 初始语言。
 *
 * 优先用上次的选择；没有记录时从 `navigator.languages` 里取**第一个支持的语言**。
 * 只看 `navigator.language` 不够：中文用户常见 `zh-Hant-TW`、日语 `ja`、英语 `en-GB`，
 * 必须按前缀匹配才能落到受支持的三门语言上。
 */
function initialLang(): Lang {
  const saved = localStorage.getItem(STORAGE_KEY);
  if (isLang(saved)) {
    return saved;
  }
  for (const candidate of navigator.languages ?? [navigator.language]) {
    const prefix = candidate.toLowerCase().split("-")[0];
    if (isLang(prefix)) {
      return prefix;
    }
  }
  // 系统语言不在支持范围内：落到英语，而不是中文 —— 英语对陌生用户更可读。
  return "en";
}

interface LanguageState {
  lang: Lang;
  setLang: (lang: Lang) => void;
}

export const useLanguage = create<LanguageState>((set) => ({
  lang: initialLang(),
  setLang: (lang) =>
    set(() => {
      localStorage.setItem(STORAGE_KEY, lang);
      return { lang };
    }),
}));

/**
 * 把当前语言写到 `<html lang="…">`，并返回语言值。
 *
 * `lang` 属性影响浏览器的断词规则与字体回退，日语/中文混排时差别明显；
 * 同一份值也供 `toLocaleString` 使用（见 `lib/i18n/useFormat.ts`）。
 */
export function useAppliedLang(): Lang {
  const lang = useLanguage((state) => state.lang);
  useEffect(() => {
    document.documentElement.lang = localeOf(lang);
  }, [lang]);
  return lang;
}
