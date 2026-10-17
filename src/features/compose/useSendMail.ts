/**
 * 「发送」的提交逻辑。
 *
 * 从 `ComposeForm.tsx` 抽出来：那个组件要做表单渲染 + 校验状态 + 发送反馈三件事，
 * 再带上 mutation 就会顶过 100 行上限（`AGENTS.md` 粒度限制）。
 *
 * 字段映射（`draft.to` 逗号切分、`html` 空串转 `null`、附件去字节留路径）
 * 属于**契约细节**，集中在这里，组件不再关心。
 */

import { useMutation } from "@tanstack/react-query";

import { api } from "../../lib/api";
import { useT } from "../../lib/i18n/useT";
import type { ComposeDraft } from "../../stores/selection";

/** `accountId` 为 `null` 时不允许发送（按钮已禁用，这里的抛错只是兜底）。 */
export function useSendMail(accountId: number | null, onSent: () => void) {
  const t = useT();

  return useMutation({
    mutationFn: (draft: ComposeDraft) => {
      if (accountId === null) {
        throw new Error(t("compose.noAccount"));
      }
      return api.sendMail({
        accountId,
        to: draft.to
          .split(",")
          .map((item) => item.trim())
          .filter((item) => item !== ""),
        subject: draft.subject,
        bodyText: draft.text,
        // 空串表示「只发纯文本」，必须转成 null 交给后端
        bodyHtml: draft.html.trim() === "" ? null : draft.html,
        attachments: draft.attachments.map((item) => ({
          path: item.path,
          filename: item.filename,
        })),
      });
    },
    onSuccess: () => onSent(),
  });
}
