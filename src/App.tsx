/**
 * 应用外壳：侧栏（账户配置 + 同步 + 文件夹树）+ 邮件列表 + 详情 + 写信面板。
 */

import AccountForm from "./features/accounts/AccountForm";
import AccountList from "./features/accounts/AccountList";
import ComposeForm from "./features/compose/ComposeForm";
import { buildForward, buildReply } from "./features/compose/drafts";
import FolderTree from "./features/inbox/FolderTree";
import InboxToolbar from "./features/inbox/InboxToolbar";
import MessageDetail from "./features/inbox/MessageDetail";
import MessageList from "./features/inbox/MessageList";
import SearchBox from "./features/inbox/SearchBox";
import { useAccounts, useFolders, useListRows, useMessage } from "./features/inbox/useInbox";
import { useSelection, type ListMode } from "./stores/selection";

/** 中栏标题：让人一眼知道当前看的是什么。 */
function listTitle(mode: ListMode): string {
  switch (mode.kind) {
    case "folder":
      return mode.label;
    case "unified":
      return "统一收件箱";
    case "search":
      return `搜索：${mode.query}`;
    default:
      return "邮件";
  }
}

/**
 * 列表为空时的提示。
 *
 * 必须区分「搜不到」与「文件夹没邮件」：两者都渲染空白中栏的话，
 * 用户分不清是没结果、还在加载、还是界面坏了。
 */
function emptyHint(mode: ListMode): string {
  switch (mode.kind) {
    case "search":
      return `没有匹配「${mode.query}」的邮件。`;
    case "unified":
      return "统一收件箱里还没有邮件，先同步一个文件夹。";
    default:
      return "这个文件夹里还没有邮件，先同步一次。";
  }
}

export default function App() {
  const accounts = useAccounts();
  const accountId = useSelection((state) => state.accountId);
  const mode = useSelection((state) => state.mode);
  const messageId = useSelection((state) => state.messageId);
  const selectMessage = useSelection((state) => state.selectMessage);
  const openCompose = useSelection((state) => state.openCompose);

  const folders = useFolders(accountId);
  const rows = useListRows(mode, accountId);
  const message = useMessage(messageId);

  const accountOptions = (accounts.data ?? []).map((account) => ({
    id: account.id,
    label: account.display_name ?? account.email,
  }));

  return (
    <div className="app">
      <aside className="app__sidebar">
        <h1 className="app__title">dp_Gmail</h1>

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
        {mode.kind !== "none" && rows.data?.length === 0 && (
          <p className="app__hint">{emptyHint(mode)}</p>
        )}
        {rows.data && rows.data.length > 0 && (
          <MessageList rows={rows.data} selectedId={messageId} onSelect={selectMessage} />
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
