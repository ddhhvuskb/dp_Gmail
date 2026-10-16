import { useEffect, useState } from "react";

/**
 * 把频繁变化的值延迟到稳定后再输出。
 *
 * 用于输入框驱动的查询：不做去抖的话，每敲一个字符就会打一次 Tauri 命令。
 */
export function useDebounced<T>(value: T, delayMs: number): T {
  const [settled, setSettled] = useState(value);

  useEffect(() => {
    const timer = setTimeout(() => setSettled(value), delayMs);
    return () => clearTimeout(timer);
  }, [value, delayMs]);

  return settled;
}
