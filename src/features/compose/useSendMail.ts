/**
 * 「发送」的提交逻辑。
 *
 * 从 `ComposeForm.tsx` 抽出来：那个组件要做表单渲染 + 校验状态 + 发送反馈三件事，
 * 再带上 mutation 就会顶过 100 行上限（`AGENTS.md` 粒度限制）。
 *
 * 字段映射（`draft.to` 逗号切分、`html` 空串转 `null`、附件去字节留路径）
 * 属于**契约细节**，集中在这里，组件不再关心。
 *
 * **发送成功后要落一份到本地「已发送」**：`send_mail` 只走 SMTP，
 * 投递成功后本地什么都不留，用户看到的就是「发出去了但应用里没有这封信」。
 */

import { useMutation, useQueryClient } from "@tanstack/react-query";

import { api } from "../../lib/api";
import { useT } from "../../lib/i18n/useT";
import { useSelection, type ComposeDraft } from "../../stores/selection";

/** 收件人串 → 地址数组。`send_mail` 与 `record_sent` 必须用**同一套**切分规则。 */
function splitRecipients(raw: string): string[] {
  return raw
    .split(",")
    .map((item) => item.trim())
    .filter((item) => item !== "");
}

/** `accountId` 为 `null` 时不允许发送（按钮已禁用，这里的抛错只是兜底）。 */
export function useSendMail(accountId: number | null, onSent: () => void) {
  const t = useT();
  const queryClient = useQueryClient();
  const showFolder = useSelection((state) => state.showFolder);

  return useMutation({
    mutationFn: (draft: ComposeDraft) => {
      if (accountId === null) {
        throw new Error(t("compose.noAccount"));
      }
      return api.sendMail({
        accountId,
        to: splitRecipients(draft.to),
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
    onSuccess: async (_report, draft) => {
      if (accountId !== null) {
        try {
          const record = await api.recordSent(
            accountId,
            splitRecipients(draft.to),
            draft.subject,
            draft.text,
            draft.html.trim() === "" ? null : draft.html,
          );
          await queryClient.invalidateQueries({ queryKey: ["folders"] });
          await queryClient.invalidateQueries({ queryKey: ["list"] });
          // 直接跳到「已发送」，让用户看见这封信确实记下了
          showFolder(record.folder_id, t("compose.sentFolder"));
        } catch {
          // 归档失败不能把已经成功的发送显示成失败 —— 信确实发出去了
        }
      }
      onSent();
    },
  });
}
