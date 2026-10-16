/**
 * 侧栏工具条：选账户、填信箱名、触发同步。
 *
 * 同步成功后会拿到后端建好的 `folder_id`，写进 Zustand，
 * 邮件列表据此查询。
 */

import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";

import { api, type AccountView } from "../../lib/api";
import { useSelection } from "../../stores/selection";

const DEFAULT_MAILBOX = "INBOX";
const SYNC_BATCH = 50;

interface Props {
  accounts: AccountView[];
}

export default function InboxToolbar({ accounts }: Props) {
  const [mailbox, setMailbox] = useState(DEFAULT_MAILBOX);

  const accountId = useSelection((state) => state.accountId);
  const setAccount = useSelection((state) => state.setAccount);
  const setFolder = useSelection((state) => state.setFolder);
  const queryClient = useQueryClient();

  const sync = useMutation({
    mutationFn: () => api.syncFolder(accountId as number, mailbox, SYNC_BATCH),
    onSuccess: async (report) => {
      setFolder(report.folder_id);
      await queryClient.invalidateQueries({ queryKey: ["messages"] });
    },
  });

  return (
    <div className="toolbar">
      <label className="toolbar__field">
        <span>账户</span>
        <select
          value={accountId ?? ""}
          onChange={(event) => {
            const value = event.target.value;
            setAccount(value === "" ? null : Number(value));
          }}
        >
          <option value="">— 请选择 —</option>
          {accounts.map((account) => (
            <option key={account.id} value={account.id}>
              {account.display_name ?? account.email}（{account.provider_name}）
            </option>
          ))}
        </select>
      </label>

      <label className="toolbar__field">
        <span>文件夹</span>
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
        {sync.isPending ? "同步中…" : "同步"}
      </button>

      {sync.isError && <p className="toolbar__error">{String(sync.error)}</p>}
      {sync.isSuccess && (
        <p className="toolbar__ok">
          拉取 {sync.data.fetched} 封，写入 {sync.data.stored} 封
        </p>
      )}
      {accounts.length === 0 && (
        <p className="toolbar__hint">先在左侧「添加账户」里配置一个邮箱。</p>
      )}
    </div>
  );
}
