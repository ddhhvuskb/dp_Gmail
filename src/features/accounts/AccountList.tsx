/**
 * 已配置账户列表。
 *
 * 只展示 `has_secret` 布尔 —— 凭证本身（哪怕是密文）都不出后端。
 */

import { useT } from "../../lib/i18n/useT";
import type { AccountView } from "../../lib/api";

interface Props {
  accounts: AccountView[];
}

export default function AccountList({ accounts }: Props) {
  const t = useT();

  if (accounts.length === 0) {
    return <p className="sidebar__hint">{t("accountList.empty")}</p>;
  }

  return (
    <ul className="account-list">
      {accounts.map((account) => (
        <li key={account.id} className="account-list__item">
          <span className="account-list__name">{account.display_name ?? account.email}</span>
          <span className="account-list__meta">
            {account.provider_name} ·{" "}
            {account.has_secret ? t("accountList.hasSecret") : t("accountList.noSecret")}
          </span>
        </li>
      ))}
    </ul>
  );
}
