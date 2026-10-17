/**
 * 「添加账户」的提交逻辑。
 *
 * 从 `AccountForm.tsx` 抽出来：那个组件本来就在 100 行上限边缘
 * （`AGENTS.md` 粒度限制），多语言化之后每处文案还变长了一点。
 * 这里只有 mutation 与缓存失效，不碰表单状态。
 */

import { useMutation, useQueryClient } from "@tanstack/react-query";

import { api } from "../../lib/api";

export interface NewAccountInput {
  email: string;
  secret: string;
  /**
   * 显示名 / 服务商标识无值时**必须显式写 `undefined`**，不能省略这个字段。
   * 原因：`tsconfig` 开了 `exactOptionalPropertyTypes`，`displayName?: string`
   * 不接受值为 `undefined` —— 而表单的空值恰好就是 `undefined`。
   */
  displayName: string | undefined;
  /** 自动识别失败时用户手选的服务商；识别成功时**不要**传值，否则会覆盖识别结果。 */
  providerId: string | undefined;
}

/**
 * @param onSuccess 提交成功后的清理动作（清空表单）。由调用方决定要清哪些字段。
 */
export function useAddAccount(onSuccess: () => void) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: NewAccountInput) =>
      api.addAccount(input.email, input.secret, input.displayName, input.providerId),
    onSuccess: async () => {
      onSuccess();
      await queryClient.invalidateQueries({ queryKey: ["accounts"] });
    },
  });
}
