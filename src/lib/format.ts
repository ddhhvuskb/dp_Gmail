/** 展示层格式化工具。 */

/** Unix 秒 → 本地日期时间。0 视为"无日期"。 */
export function formatDateTime(unixSeconds: number): string {
  if (!unixSeconds) {
    return "无日期";
  }
  return new Date(unixSeconds * 1000).toLocaleString("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}

/** 字节数 → 人类可读。 */
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

/** 发件人展示串。缺显示名时退回地址。 */
export function formatSender(name: string | null, address: string | null): string {
  if (name && address) {
    return `${name} <${address}>`;
  }
  return name ?? address ?? "未知发件人";
}
