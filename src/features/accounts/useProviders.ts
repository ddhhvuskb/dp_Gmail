import { useQuery } from "@tanstack/react-query";

import { api } from "../../lib/api";

/**
 * 全部服务商，供「自动识别不出来时手动选择」用。
 *
 * 服务商表在后端 `mail/provider.rs`，前端**不维护副本**。
 * 那张表在一次会话里不会变，故 `staleTime: Infinity`。
 */
export function useProviders() {
  return useQuery({
    queryKey: ["providers"],
    queryFn: () => api.listProviders(),
    staleTime: Infinity,
    retry: false,
  });
}
