import { useQuery } from "@tanstack/react-query";

import { api } from "../../lib/api";
import { useDebounced } from "../../lib/useDebounced";

/** 输入停止多久后才发起识别。 */
const DEBOUNCE_MS = 300;

/**
 * 由邮箱地址识别服务商。
 *
 * 域名表在后端 `mail/provider.rs`，前端**不维护副本** —— 那会形成两份真相。
 */
export function useProviderLookup(email: string) {
  const settled = useDebounced(email.trim(), DEBOUNCE_MS);

  return useQuery({
    queryKey: ["provider", settled],
    queryFn: () => api.resolveProvider(settled),
    // 还没打出 @ 就不必问后端
    enabled: settled.includes("@"),
    staleTime: Infinity,
    retry: false,
  });
}
