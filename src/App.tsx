/**
 * 应用外壳。三栏细节都拆成独立组件，这里只拼装。
 * `data-nav` / `data-drawer` 供窄屏 CSS 决定显示列表还是详情、以及抽屉开合。
 */

import AccountForm from "./features/accounts/AccountForm";
import AccountList from "./features/accounts/AccountList";
import ComposeForm from "./features/compose/ComposeForm";
import DetailPane from "./features/inbox/DetailPane";
import FolderTree from "./features/inbox/FolderTree";
import InboxToolbar from "./features/inbox/InboxToolbar";
import { emptyHint, listTitle } from "./features/inbox/listLabels";
import MessageList from "./features/inbox/MessageList";
import MobileBar from "./features/inbox/MobileBar";
import SearchBox from "./features/inbox/SearchBox";
import {
  useAccounts,
  useAccountOptions,
  useFolders,
  usePagedRows,
} from "./features/inbox/useInbox";
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
  const nav = useSelection((state) => state.nav);
  const drawerOpen = useSelection((state) => state.drawerOpen);
  const selectMessage = useSelection((state) => state.selectMessage);
  const openCompose = useSelection((state) => state.openCompose);

  useAppliedLang();
  useAppliedAppearance();
  const t = useT();

  const folders = useFolders(accountId);
  const { query: rows, rows: rowList, loadMore } = usePagedRows(mode, accountId);
  const accountOptions = useAccountOptions();
  return (
    <div className="app" data-nav={nav} data-drawer={drawerOpen ? "open" : "closed"}>
      <MobileBar />
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
