/**
 * 写信面板的字段区：收件人 / 主题 / 纯文本正文 / HTML 正文 / 附件。
 *
 * 从 `ComposeForm.tsx` 抽出来：那个组件还要管发送状态与反馈，
 * 两件事合起来会顶过 100 行上限（`AGENTS.md` 粒度限制）。
 *
 * 这里**只管渲染与回填**，不持有状态 —— 草稿仍由 `stores/selection.ts` 单一持有，
 * 否则面板关闭再打开会丢内容。
 */

import { useT } from "../../lib/i18n/useT";
import type { ComposeDraft } from "../../stores/selection";
import AttachmentField from "./AttachmentField";

interface Props {
  draft: ComposeDraft;
  /** 只回填改动的字段，由 store 负责合并。 */
  onChange: (patch: Partial<ComposeDraft>) => void;
}

export default function ComposeFields({ draft, onChange }: Props) {
  const t = useT();

  return (
    <>
      <label className="field">
        <span>{t("compose.to")}</span>
        <input
          value={draft.to}
          onChange={(event) => onChange({ to: event.target.value })}
          placeholder="someone@qq.com"
          autoComplete="off"
        />
      </label>

      <label className="field">
        <span>{t("compose.subject")}</span>
        <input
          value={draft.subject}
          onChange={(event) => onChange({ subject: event.target.value })}
          autoComplete="off"
        />
      </label>

      <label className="field">
        <span>{t("compose.bodyText")}</span>
        <textarea
          rows={6}
          value={draft.text}
          onChange={(event) => onChange({ text: event.target.value })}
        />
      </label>

      <label className="field">
        <span>{t("compose.bodyHtml")}</span>
        <textarea
          rows={3}
          value={draft.html}
          onChange={(event) => onChange({ html: event.target.value })}
          placeholder={t("compose.bodyHtmlPlaceholder")}
        />
      </label>

      <AttachmentField
        attachments={draft.attachments}
        onChange={(attachments) => onChange({ attachments })}
      />
    </>
  );
}
