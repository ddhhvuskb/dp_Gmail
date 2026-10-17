/**
 * 账户列表里的一行：展示 + 编辑 / 删除入口。
 *
 * 删除用**行内二次确认**，不用 `window.confirm`：
 * 原生弹窗在 WebView2 里样式不可控、且是阻塞式，而这里需要把
 * 「会连带删掉多少封邮件」说清楚 —— 那塞不进一个系统弹窗。
 */

import { useState } from "react";

import type { AccountView } from "../../lib/api";
import { useT } from "../../lib/i18n/useT";
import { useDeleteAccount } from "./useDeleteAccount";

interface Props {
  account: AccountView;
  onEdit: () => void;
}

export default function AccountRow({ account, onEdit }: Props) {
  const [confirming, setConfirming] = useState(false);
  const remove = useDeleteAccount();
  const t = useT();

  return (
    <div className="account-row">
      <span className="account-list__name">{account.display_name ?? account.email}</span>
      <span className="account-list__meta">
        {account.provider_name} ·{" "}
        {account.has_secret ? t("accountList.hasSecret") : t("accountList.noSecret")}
      </span>

      {!confirming && (
        <span className="account-row__actions">
          <button type="button" className="account-row__action" onClick={onEdit}>
            {t("accountList.edit")}
          </button>
          <button
            type="button"
            className="account-row__action account-row__action--danger"
            onClick={() => setConfirming(true)}
          >
            {t("accountList.delete")}
          </button>
        </span>
      )}

      {confirming && (
        <div className="account-row__confirm">
          <p className="account-row__warn">{t("accountList.deleteTitle")}</p>
          <p className="account-row__warn">{t("accountList.deleteWarn")}</p>
          <span className="account-row__actions">
            <button
              type="button"
              className="account-row__action account-row__action--danger"
              disabled={remove.isPending}
              onClick={() => remove.mutate(account.id)}
            >
              {remove.isPending ? t("accountList.deleting") : t("accountList.confirmDelete")}
            </button>
            <button
              type="button"
              className="account-row__action"
              onClick={() => setConfirming(false)}
            >
              {t("accountList.cancel")}
            </button>
          </span>
        </div>
      )}

      {remove.isError && <p className="field__error">{String(remove.error)}</p>}
    </div>
  );
}
