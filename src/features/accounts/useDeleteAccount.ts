/**
 * 删除账户。
 *
 * 会**连带删掉该账户已下载的邮件与附件**，不可逆；界面必须先让用户确认。
 *
 * 删掉的正好是当前选中的账户时，要把选中态一起清掉 ——
 * 否则列表会拿着一个已不存在的 `accountId` 去查，表现为「删完了列表还在转圈」。
 */

import { useMutation, useQueryClient } from "@tanstack/react-query";

import { api } from "../../lib/api";
import { useSelection } from "../../stores/selection";

export function useDeleteAccount() {
  const queryClient = useQueryClient();
  const accountId = useSelection((state) => state.accountId);
  const setAccount = useSelection((state) => state.setAccount);

  return useMutation({
    mutationFn: (id: number) => api.deleteAccount(id),
    onSuccess: async (_report, id) => {
      if (accountId === id) {
        // setAccount 内部会一并清空 mode 与 messageId
        setAccount(null);
      }
      await queryClient.invalidateQueries({ queryKey: ["accounts"] });
      await queryClient.invalidateQueries({ queryKey: ["folders"] });
      await queryClient.invalidateQueries({ queryKey: ["list"] });
    },
  });
}
