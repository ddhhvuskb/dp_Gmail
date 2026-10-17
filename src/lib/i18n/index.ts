/**
 * 多语言核心。**不引入任何 i18n 依赖** —— 一个字典 + 一个插值函数就够，
 * 完整性靠类型系统（见 `zh.ts` 文件头）。
 */

import { en } from "./en";
import { ja } from "./ja";
import { zh, type Key } from "./zh";

export type { Key };

/** 支持的语言。`zh` 是回退语言，且是词典的 key 来源。 */
export type Lang = "zh" | "ja" | "en";

/** 各语言的 `Intl` / `toLocaleString` 用的地区码。 */
const LOCALE_CODES: Record<Lang, string> = {
  zh: "zh-CN",
  ja: "ja-JP",
  en: "en-US",
};

export const MESSAGES: Record<Lang, Record<Key, string>> = { zh, ja, en };

/** 语言选择器里的选项。语言名用**该语言自己的写法**，不翻译。 */
export const LANG_OPTIONS: { id: Lang; label: string }[] = [
  { id: "zh", label: "中文" },
  { id: "ja", label: "日本語" },
  { id: "en", label: "English" },
];

/** 插值参数。数字会被转成字符串。 */
export type Params = Record<string, string | number>;

/**
 * 带地区码的翻译函数。
 *
 * 挂 `locale` 是为了让 `drafts.ts` 这类**纯函数**既能取文案、又能按同一语言格式化日期，
 * 不必再额外传一个 locale 参数。
 */
export interface Translator {
  (key: Key, params?: Params): string;
  /** `zh-CN` / `ja-JP` / `en-US`，可直接交给 `toLocaleString`。 */
  readonly locale: string;
}

/**
 * 把 `{name}` 占位符替换成参数。
 *
 * 缺参数时**原样保留占位符**而不是替换成空串 —— 界面上会立刻看到
 * `{fetched}` 这样的东西，比悄悄显示「拉取 0 封」更容易发现是漏传了参数。
 */
export function interpolate(template: string, params?: Params): string {
  if (!params) {
    return template;
  }
  return template.replace(/\{(\w+)\}/g, (whole, name: string) => {
    const value = params[name];
    return value === undefined ? whole : String(value);
  });
}

function lookup(lang: Lang, key: Key): string {
  // 类型上 key 一定存在；这里保留回退只为防运行时的意外（如词典被动态改坏）。
  return MESSAGES[lang][key] ?? MESSAGES.zh[key] ?? key;
}

/** 构造一个绑定了语言与地区码的 `Translator`。 */
export function createTranslator(lang: Lang): Translator {
  const locale = LOCALE_CODES[lang];
  const translate = (key: Key, params?: Params) => interpolate(lookup(lang, key), params);
  return Object.assign(translate, { locale });
}

/** 取某语言的地区码。 */
export function localeOf(lang: Lang): string {
  return LOCALE_CODES[lang];
}

/** 判断任意字符串是否是受支持的语言。用于读取 `localStorage` 后校验。 */
export function isLang(value: unknown): value is Lang {
  return value === "zh" || value === "ja" || value === "en";
}
