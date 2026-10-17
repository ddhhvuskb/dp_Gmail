/**
 * 应用外壳：侧栏（账户配置 + 同步 + 文件夹树）+ 邮件列表 + 详情 + 写信面板。
 *
 * 三栏里各自的细节都已拆成独立组件（`DetailPane` / `MessageList` / `AccountForm`…），
 * 这里只负责拼装，保持在 100 行以内（`AGENTS.md` 粒度限制）。
 */

import AccountForm from "./features/accounts/AccountForm";
import AccountList from "./features/accounts/AccountList";
import ComposeForm from "./features/compose/ComposeForm";
import DetailPane from "./features/inbox/DetailPane";
import FolderTree from "./features/inbox/FolderTree";
import InboxToolbar from "./features/inbox/InboxToolbar";
import { emptyHint, listTitle } from "./features/inbox/listLabels";
import MessageList from "./features/inbox/MessageList";
import SearchBox from "./features/inbox/SearchBox";
import { useAccounts, useFolders, usePagedRows } from "./features/inbox/useInbox";
import SettingsButton from "./features/settings/SettingsButton";
import SettingsDialog from "./features/settings/SettingsDialog";
import { useT } from "./lib/i18n/useT";
import { useAppliedLang } from "./stores/language";
import { useSelection } from "./stores/selection";
import { useAppliedAppearance } from "./stores/theme";

export default function App() {
  const accounts = useAccounts();
  const accountId = useSelection((state) => state.accountId);
  const mode = useSelection((state) => state.mode);
  const messageId = useSelection((state) => state.messageId);
  const selectMessage = useSelection((state) => state.selectMessage);
  const openCompose = useSelection((state) => state.openCompose);

  useAppliedLang();
  useAppliedAppearance();
  const t = useT();

  const folders = useFolders(accountId);
  const { query: rows, rows: rowList, loadMore } = usePagedRows(mode, accountId);

  const accountOptions = (accounts.data ?? []).map((account) => ({
    id: account.id,
    label: account.display_name ?? account.email,
  }));

  return (
    <div className="app">
      <aside className="app__sidebar">
        <h1 className="app__title">dp_Gmail</h1>

        <SettingsButton />

        <button type="button" className="button" onClick={() => openCompose()}>
          {t("app.compose")}
        </button>

        <AccountForm />

        <h2 className="sidebar__heading">{t("app.accountsHeading")}</h2>
        {accounts.isError && <p className="app__error">{String(accounts.error)}</p>}
        <AccountList accounts={accounts.data ?? []} />

        <h2 className="sidebar__heading">{t("app.syncHeading")}</h2>
        <InboxToolbar accounts={accounts.data ?? []} />

        <h2 className="sidebar__heading">{t("app.foldersHeading")}</h2>
        {folders.isError && <p className="app__error">{String(folders.error)}</p>}
        <FolderTree folders={folders.data ?? []} accounts={accountOptions} />
      </aside>

      <section className="app__list">
        <SearchBox />
        <p className="app__list-title">{listTitle(mode, t)}</p>

        {mode.kind === "none" && <p className="app__hint">{t("app.emptyListHint")}</p>}
        {mode.kind !== "none" && rows.isPending && <p className="app__hint">{t("app.loading")}</p>}
        {rows.isError && <p className="app__error">{String(rows.error)}</p>}
        {mode.kind !== "none" && rows.isSuccess && rowList.length === 0 && (
          <p className="app__hint">{emptyHint(mode, t)}</p>
        )}
        {rowList.length > 0 && (
          <MessageList
            rows={rowList}
            selectedId={messageId}
            onSelect={selectMessage}
            onLoadMore={loadMore}
            isLoadingMore={rows.isFetchingNextPage}
          />
        )}
      </section>

      <DetailPane />

      <ComposeForm />

      <SettingsDialog />
    </div>
  );
}
