/**
 * 已配置账户列表。
 *
 * 只展示 `has_secret` 布尔 —— 凭证本身（哪怕是密文）都不出后端。
 *
 * 每行带「编辑 / 删除」：删账户是**不可逆**操作，所以删除走两步确认，
 * 并把「会连带删掉什么」讲在确认框里，而不是只问一句「确定吗」。
 */

import { useState } from "react";

import type { AccountView } from "../../lib/api";
import { useT } from "../../lib/i18n/useT";
import AccountEditForm from "./AccountEditForm";
import AccountRow from "./AccountRow";

interface Props {
  accounts: AccountView[];
}

export default function AccountList({ accounts }: Props) {
  const [editingId, setEditingId] = useState<number | null>(null);
  const t = useT();

  if (accounts.length === 0) {
    return <p className="sidebar__hint">{t("accountList.empty")}</p>;
  }

  return (
    <ul className="account-list">
      {accounts.map((account) =>
        account.id === editingId ? (
          <li key={account.id} className="account-list__item">
            <AccountEditForm account={account} onDone={() => setEditingId(null)} />
          </li>
        ) : (
          <li key={account.id} className="account-list__item">
            <AccountRow account={account} onEdit={() => setEditingId(account.id)} />
          </li>
        ),
      )}
    </ul>
  );
}
