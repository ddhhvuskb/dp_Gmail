/**
 * 应用外壳：侧栏（账户配置 + 同步 + 文件夹树）+ 邮件列表 + 详情 + 写信面板。
 */

import AccountForm from "./features/accounts/AccountForm";
import AccountList from "./features/accounts/AccountList";
import ComposeForm from "./features/compose/ComposeForm";
import { buildForward, buildReply } from "./features/compose/drafts";
import FolderTree from "./features/inbox/FolderTree";
import InboxToolbar from "./features/inbox/InboxToolbar";
import { emptyHint, listTitle } from "./features/inbox/listLabels";
import MessageDetail from "./features/inbox/MessageDetail";
import MessageList from "./features/inbox/MessageList";
import SearchBox from "./features/inbox/SearchBox";
import { useAccounts, useFolders, usePagedRows, useMessage } from "./features/inbox/useInbox";
import ThemeToggle from "./features/settings/ThemeToggle";
import { useSelection } from "./stores/selection";

export default function App() {
  const accounts = useAccounts();
  const accountId = useSelection((state) => state.accountId);
  const mode = useSelection((state) => state.mode);
  const messageId = useSelection((state) => state.messageId);
  const selectMessage = useSelection((state) => state.selectMessage);
  const openCompose = useSelection((state) => state.openCompose);

  const folders = useFolders(accountId);
  const { query: rows, rows: rowList, loadMore } = usePagedRows(mode, accountId);
  const message = useMessage(messageId);

  const accountOptions = (accounts.data ?? []).map((account) => ({
    id: account.id,
    label: account.display_name ?? account.email,
  }));

  return (
    <div className="app">
      <aside className="app__sidebar">
        <h1 className="app__title">dp_Gmail</h1>

        <ThemeToggle />

        <button type="button" className="button" onClick={() => openCompose()}>
          ✏️ 写邮件
        </button>

        <AccountForm />

        <h2 className="sidebar__heading">已配置账户</h2>
        {accounts.isError && <p className="app__error">{String(accounts.error)}</p>}
        <AccountList accounts={accounts.data ?? []} />

        <h2 className="sidebar__heading">同步</h2>
        <InboxToolbar accounts={accounts.data ?? []} />

        <h2 className="sidebar__heading">文件夹</h2>
        {folders.isError && <p className="app__error">{String(folders.error)}</p>}
        <FolderTree folders={folders.data ?? []} accounts={accountOptions} />
      </aside>

      <section className="app__list">
        <SearchBox />
        <p className="app__list-title">{listTitle(mode)}</p>

        {mode.kind === "none" && <p className="app__hint">同步一个文件夹，或点「统一收件箱」。</p>}
        {mode.kind !== "none" && rows.isPending && <p className="app__hint">读取邮件…</p>}
        {rows.isError && <p className="app__error">{String(rows.error)}</p>}
        {mode.kind !== "none" && rows.isSuccess && rowList.length === 0 && (
          <p className="app__hint">{emptyHint(mode)}</p>
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

      <section className="app__detail">
        {messageId === null && <p className="app__hint">选一封邮件查看详情。</p>}
        {messageId !== null && message.isPending && <p className="app__hint">读取邮件…</p>}
        {message.isError && <p className="app__error">{String(message.error)}</p>}
        {message.data && (
          <>
            <div className="app__detail-actions">
              <button
                type="button"
                className="button"
                onClick={() => openCompose(buildReply(message.data))}
              >
                回复
              </button>
              <button
                type="button"
                className="button"
                onClick={() => openCompose(buildForward(message.data))}
              >
                转发
              </button>
            </div>
            <MessageDetail message={message.data} />
          </>
        )}
      </section>

      <ComposeForm />
    </div>
  );
}
