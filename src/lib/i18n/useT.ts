/**
 * React 侧的多语言入口。
 *
 * 词典与插值在 `index.ts`（纯函数，不依赖 React）；这里只负责
 * 「把当前语言接到组件上」，并按语言缓存 `Translator` 实例。
 */

import { useMemo } from "react";

import { formatBytes, formatDateTime, formatSender } from "../format";
import { createTranslator, type Translator } from "./index";
import { useLanguage } from "../../stores/language";

/**
 * 当前语言的翻译函数。
 *
 * 同一门语言内返回**同一个实例**，所以可以安全地放进 `useEffect` 的依赖数组
 * （例如 `MessageList` 的触底回调）。
 */
export function useT(): Translator {
  const lang = useLanguage((state) => state.lang);
  return useMemo(() => createTranslator(lang), [lang]);
}

/** 绑定了当前语言的格式化函数。 */
export interface Formatters {
  /** Unix 秒 → 本地日期时间，无日期时返回当前语言的「无日期」。 */
  dateTime: (unixSeconds: number) => string;
  /** 字节数 → `1.2 MB`。单位国际通用，不随语言变。 */
  bytes: (bytes: number) => string;
  /** 发件人展示串，两者都缺时返回当前语言的「未知发件人」。 */
  sender: (name: string | null, address: string | null) => string;
}

export function useFormat(): Formatters {
  const t = useT();
  return useMemo(
    () => ({
      dateTime: (unixSeconds: number) => formatDateTime(unixSeconds, t.locale, t("format.noDate")),
      bytes: formatBytes,
      sender: (name: string | null, address: string | null) =>
        formatSender(name, address, t("format.unknownSender")),
    }),
    [t],
  );
}
