/**
 * 写信面板里的「发件账户」选择。
 *
 * **这是修一个真缺陷**：原先账户选择只存在于侧栏的「同步」区里，
 * 语义上属于「同步用哪个账户」，写信的人根本不会去那里找。
 * 结果是只有一个账户时也会卡在「请先选择账户」上，且界面上无路可走。
 *
 * 两条处理：
 * 1. 写信面板里直接给选择器；
 * 2. 只有一个账户时自动选中 —— 用户点「写邮件」就是想写，不该先做一道选择题。
 */

import { useEffect } from "react";

import { useT } from "../../lib/i18n/useT";
import { useSelection } from "../../stores/selection";
import { useAccounts } from "../inbox/useInbox";

export default function FromAccountField() {
  const accounts = useAccounts();
  const accountId = useSelection((state) => state.accountId);
  const setAccount = useSelection((state) => state.setAccount);
  const t = useT();

  const list = accounts.data ?? [];

  // 只有一个账户就直接用，不必让用户点一下
  useEffect(() => {
    if (accountId === null && list.length === 1) {
      setAccount(list[0]?.id ?? null);
    }
  }, [accountId, list, setAccount]);

  if (!accounts.isPending && list.length === 0) {
    return <p className="field__error">{t("compose.noAccount")}</p>;
  }

  return (
    <label className="field">
      <span>{t("compose.from")}</span>
      <select
        value={accountId ?? ""}
        onChange={(event) => {
          const value = event.target.value;
          setAccount(value === "" ? null : Number(value));
        }}
      >
        <option value="">{t("toolbar.selectPlaceholder")}</option>
        {list.map((account) => (
          <option key={account.id} value={account.id}>
            {account.display_name ?? account.email}
          </option>
        ))}
      </select>
    </label>
  );
}
