/** 展示层格式化工具。文案与地区由调用方传入，本文件保持**纯函数**、不依赖 React。 */

/**
 * Unix 秒 → 本地日期时间。0 视为「无日期」。
 *
 * `locale` 与 `noDate` 由调用方传入：本模块不认识语言，
 * 由 `lib/i18n/useFormat.ts` 把当前语言的地区码和译文绑进来。
 */
export function formatDateTime(unixSeconds: number, locale: string, noDate: string): string {
  if (!unixSeconds) {
    return noDate;
  }
  return new Date(unixSeconds * 1000).toLocaleString(locale, {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}

/** 字节数 → 人类可读。单位是国际通用符号，无需翻译。 */
export function formatBytes(bytes: number): string {
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  const units = ["KB", "MB", "GB"];
  let value = bytes / 1024;
  let unitIndex = 0;
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex += 1;
  }
  return `${value.toFixed(1)} ${units[unitIndex]}`;
}

/** 发件人展示串。缺显示名时退回地址，两者都缺时用调用方给的兜底文案。 */
export function formatSender(
  name: string | null,
  address: string | null,
  unknownSender: string,
): string {
  if (name && address) {
    return `${name} <${address}>`;
  }
  return name ?? address ?? unknownSender;
}
