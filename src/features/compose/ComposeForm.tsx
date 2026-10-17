/**
 * 写信面板外壳：标题栏 + 字段区 + 发送按钮与反馈。
 *
 * 字段区见 `ComposeFields.tsx`，发送逻辑与字段映射见 `useSendMail.ts`，
 * 本文件只做组装。
 */

import { useT } from "../../lib/i18n/useT";
import { useSelection } from "../../stores/selection";
import ComposeFields from "./ComposeFields";
import { useSendMail } from "./useSendMail";

export default function ComposeForm() {
  const accountId = useSelection((state) => state.accountId);
  const draft = useSelection((state) => state.compose);
  const update = useSelection((state) => state.updateCompose);
  const close = useSelection((state) => state.closeCompose);
  const t = useT();

  const send = useSendMail(accountId, close);

  if (!draft) {
    return null;
  }

  const canSend = accountId !== null && draft.to.trim() !== "" && !send.isPending;

  return (
    <div className="compose" role="dialog" aria-label={t("compose.title")}>
      <header className="compose__header">
        <h2>{t("compose.title")}</h2>
        <button
          type="button"
          className="compose__close"
          onClick={close}
          aria-label={t("compose.close")}
        >
          ×
        </button>
      </header>

      {accountId === null && <p className="field__error">{t("compose.noAccount")}</p>}

      <ComposeFields draft={draft} onChange={update} />

      <div className="compose__actions">
        <button
          type="button"
          className="button"
          disabled={!canSend}
          onClick={() => send.mutate(draft)}
        >
          {send.isPending ? t("compose.sending") : t("compose.send")}
        </button>
        {send.isError && <p className="field__error">{String(send.error)}</p>}
        {send.isSuccess && <p className="field__ok">{t("compose.sent")}</p>}
      </div>
    </div>
  );
}
