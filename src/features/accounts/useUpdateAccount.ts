/**
 * 编辑账户的提交逻辑。
 *
 * 与 `useAddAccount` 分开而不是合成一个：两者的**必填项不同** ——
 * 新增必须给授权码，编辑不给就表示「不改凭证」。合在一起会让「空字符串」
 * 同时承担「没填」和「清空」两种含义，那是 bug 的温床。
 */

import { useMutation, useQueryClient } from "@tanstack/react-query";

import { api } from "../../lib/api";

export interface EditAccountInput {
  accountId: number;
  email: string;
  displayName: string | undefined;
  /** `undefined` ＝ 不改凭证。空字符串会在后端被拒。 */
  secret: string | undefined;
  providerId: string | undefined;
}

export function useUpdateAccount(onSaved: () => void) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: EditAccountInput) =>
      api.updateAccount(
        input.accountId,
        input.email,
        input.displayName,
        input.secret,
        input.providerId,
      ),
    onSuccess: async () => {
      onSaved();
      await queryClient.invalidateQueries({ queryKey: ["accounts"] });
      // 邮箱/显示名会显示在文件夹树与统一收件箱里，改名后它们也要刷新
      await queryClient.invalidateQueries({ queryKey: ["folders"] });
      await queryClient.invalidateQueries({ queryKey: ["list"] });
    },
  });
}
