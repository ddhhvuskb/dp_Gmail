/**
 * 侧栏工具条：选账户、填信箱名、触发同步。
 *
 * 同步成功后会拿到后端建好的 `folder_id`，写进 Zustand，
 * 邮件列表据此查询。
 *
 * 同时承担**自动收信**的挂载点：它本来就持有「哪个账户 + 哪个信箱」这两个参数，
 * 放到别处还得再把它们传一遍。
 */

import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";

import { api, type AccountView } from "../../lib/api";
import { useT } from "../../lib/i18n/useT";
import { useAutoSync } from "../../stores/autoSync";
import { useSelection } from "../../stores/selection";
import { useAutoSyncLoop } from "./useAutoSyncLoop";

const DEFAULT_MAILBOX = "INBOX";
const SYNC_BATCH = 50;

interface Props {
  accounts: AccountView[];
}

export default function InboxToolbar({ accounts }: Props) {
  const [mailbox, setMailbox] = useState(DEFAULT_MAILBOX);
  const t = useT();

  const accountId = useSelection((state) => state.accountId);
  const setAccount = useSelection((state) => state.setAccount);
  const showFolder = useSelection((state) => state.showFolder);
  const autoSyncEnabled = useAutoSync((state) => state.enabled);
  const queryClient = useQueryClient();

  useAutoSyncLoop(autoSyncEnabled, accountId, mailbox);

  const sync = useMutation({
    mutationFn: () => api.syncFolder(accountId as number, mailbox, SYNC_BATCH),
    onSuccess: async (report) => {
      showFolder(report.folder_id, mailbox);
      await queryClient.invalidateQueries({ queryKey: ["list"] });
      await queryClient.invalidateQueries({ queryKey: ["folders"] });
    },
  });

  return (
    <div className="toolbar">
      <label className="toolbar__field">
        <span>{t("toolbar.account")}</span>
        <select
          value={accountId ?? ""}
          onChange={(event) => {
            const value = event.target.value;
            setAccount(value === "" ? null : Number(value));
          }}
        >
          <option value="">{t("toolbar.selectPlaceholder")}</option>
          {accounts.map((account) => (
            <option key={account.id} value={account.id}>
              {account.display_name ?? account.email}（{account.provider_name}）
            </option>
          ))}
        </select>
      </label>

      <label className="toolbar__field">
        <span>{t("toolbar.mailbox")}</span>
        <input
          value={mailbox}
          onChange={(event) => setMailbox(event.target.value)}
          placeholder={DEFAULT_MAILBOX}
        />
      </label>

      <button
        type="button"
        className="toolbar__action"
        disabled={accountId === null || mailbox.trim() === "" || sync.isPending}
        onClick={() => sync.mutate()}
      >
        {sync.isPending ? t("toolbar.syncing") : t("toolbar.sync")}
      </button>

      {sync.isError && <p className="toolbar__error">{String(sync.error)}</p>}
      {sync.isSuccess && (
        <p className="toolbar__ok">
          {t("toolbar.syncReport", { fetched: sync.data.fetched, stored: sync.data.stored })}
        </p>
      )}
      {accounts.length === 0 && <p className="toolbar__hint">{t("toolbar.noAccounts")}</p>}

      {/* 说明列表会自己刷新 —— 否则用户看到邮件自动冒出来会以为是 bug */}
      {autoSyncEnabled && accountId !== null && (
        <p className="toolbar__hint">{t("autoSync.toolbarHint")}</p>
      )}
    </div>
  );
}
