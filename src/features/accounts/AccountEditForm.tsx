/**
 * 编辑已有账户：邮箱、显示名、授权码。
 *
 * 授权码**留空＝不修改**（「没填」与「清空」必须区分开，否则一次保存就把凭证写坏了）。
 * 改邮箱后由后端重新解析服务商；识别不出来（企业邮箱）时用 `ProviderField` 手动选 ——
 * 与「添加账户」走同一条路，不另造一套。
 */

import { useState } from "react";

import type { AccountView } from "../../lib/api";
import { useT } from "../../lib/i18n/useT";
import LabeledInput from "./LabeledInput";
import ProviderField from "./ProviderField";
import { useProviderLookup } from "./useProviderLookup";
import { useUpdateAccount } from "./useUpdateAccount";

interface Props {
  account: AccountView;
  onDone: () => void;
}

export default function AccountEditForm({ account, onDone }: Props) {
  const [email, setEmail] = useState(account.email);
  const [displayName, setDisplayName] = useState(account.display_name ?? "");
  const [secret, setSecret] = useState("");
  const [manualProviderId, setManualProviderId] = useState("");

  const t = useT();
  const lookup = useProviderLookup(email);
  const detected = lookup.data ?? null;
  const save = useUpdateAccount(onDone);

  const hasProvider = detected !== null || manualProviderId !== "";
  const canSave = email.trim() !== "" && hasProvider && !save.isPending;

  function submit() {
    save.mutate({
      accountId: account.id,
      email: email.trim(),
      displayName: displayName.trim() || undefined,
      // 空串 → undefined，表示「不改凭证」；后端对空串会直接拒绝
      secret: secret === "" ? undefined : secret,
      providerId: detected ? undefined : manualProviderId || undefined,
    });
  }

  return (
    <form
      className="account-edit"
      onSubmit={(event) => {
        event.preventDefault();
        if (canSave) {
          submit();
        }
      }}
    >
      <h3 className="account-edit__heading">{t("accountEdit.heading")}</h3>

      <LabeledInput label={t("accountForm.email")} value={email} onChange={setEmail} />

      <ProviderField
        detected={detected}
        settled={lookup.isSuccess}
        manualId={manualProviderId}
        onManualChange={setManualProviderId}
      />

      <LabeledInput label={t("accountEdit.secret")} value={secret} onChange={setSecret} password />

      <LabeledInput
        label={t("accountForm.displayName")}
        value={displayName}
        onChange={setDisplayName}
      />

      <div className="account-edit__actions">
        <button type="submit" className="button" disabled={!canSave}>
          {save.isPending ? t("accountEdit.saving") : t("accountEdit.save")}
        </button>
        <button type="button" className="account-row__action" onClick={onDone}>
          {t("accountEdit.cancel")}
        </button>
      </div>

      {save.isError && <p className="field__error">{String(save.error)}</p>}
    </form>
  );
}
