/**
 * 写信面板。
 *
 * 附件选择器尚未接入：文件选择需要 `@tauri-apps/plugin-dialog`（JS + Rust 双侧
 * 依赖，外加 capabilities 权限声明），将在下一轮核实其 API 后补上。
 * 后端 `commands::compose::send_mail` 已经支持附件，缺的只是选取入口。
 */

import { useMutation } from "@tanstack/react-query";

import { api } from "../../lib/api";
import { useSelection } from "../../stores/selection";

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
        attachments: [],
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
          rows={8}
          value={draft.text}
          onChange={(event) => update({ text: event.target.value })}
        />
      </label>

      <label className="field">
        <span>HTML 正文（可选，填写后以 multipart/alternative 发送）</span>
        <textarea
          rows={4}
          value={draft.html}
          onChange={(event) => update({ html: event.target.value })}
          placeholder="<p>富文本正文</p>"
        />
      </label>

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
