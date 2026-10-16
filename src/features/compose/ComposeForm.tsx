/**
 * 写信面板：收件人 / 主题 / 纯文本正文 / HTML 正文 / 附件 + 发送。
 *
 * 附件选取见 `AttachmentField.tsx`（需 dialog 插件，webview 拿不到真实路径）。
 */

import { useMutation } from "@tanstack/react-query";

import { api } from "../../lib/api";
import { useSelection } from "../../stores/selection";
import AttachmentField from "./AttachmentField";

export default function ComposeForm() {
  const accountId = useSelection((state) => state.accountId);
  const draft = useSelection((state) => state.compose);
  const update = useSelection((state) => state.updateCompose);
  const close = useSelection((state) => state.closeCompose);

  const send = useMutation({
    mutationFn: () => {
      if (!draft || accountId === null) {
        throw new Error("未选择账户");
      }
      return api.sendMail({
        accountId,
        to: draft.to
          .split(",")
          .map((item) => item.trim())
          .filter((item) => item !== ""),
        subject: draft.subject,
        bodyText: draft.text,
        bodyHtml: draft.html.trim() === "" ? null : draft.html,
        attachments: draft.attachments.map((item) => ({
          path: item.path,
          filename: item.filename,
        })),
      });
    },
    onSuccess: () => close(),
  });

  if (!draft) {
    return null;
  }

  const canSend = accountId !== null && draft.to.trim() !== "" && !send.isPending;

  return (
    <div className="compose" role="dialog" aria-label="写邮件">
      <header className="compose__header">
        <h2>写邮件</h2>
        <button type="button" className="compose__close" onClick={close} aria-label="关闭">
          ×
        </button>
      </header>

      {accountId === null && <p className="field__error">请先在左侧选择一个账户</p>}

      <label className="field">
        <span>收件人（多个用逗号分隔）</span>
        <input
          value={draft.to}
          onChange={(event) => update({ to: event.target.value })}
          placeholder="someone@qq.com"
          autoComplete="off"
        />
      </label>

      <label className="field">
        <span>主题</span>
        <input
          value={draft.subject}
          onChange={(event) => update({ subject: event.target.value })}
          autoComplete="off"
        />
      </label>

      <label className="field">
        <span>正文（纯文本）</span>
        <textarea
          rows={6}
          value={draft.text}
          onChange={(event) => update({ text: event.target.value })}
        />
      </label>

      <label className="field">
        <span>HTML 正文（可选，填写后以 multipart/alternative 发送）</span>
        <textarea
          rows={3}
          value={draft.html}
          onChange={(event) => update({ html: event.target.value })}
          placeholder="<p>富文本正文</p>"
        />
      </label>

      <AttachmentField
        attachments={draft.attachments}
        onChange={(attachments) => update({ attachments })}
      />

      <div className="compose__actions">
        <button type="button" className="button" disabled={!canSend} onClick={() => send.mutate()}>
          {send.isPending ? "发送中…" : "发送"}
        </button>
        {send.isError && <p className="field__error">{String(send.error)}</p>}
        {send.isSuccess && <p className="field__ok">已发送</p>}
      </div>
    </div>
  );
}
