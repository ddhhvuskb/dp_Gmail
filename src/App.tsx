/**
 * 应用外壳：侧栏（账户 + 同步）+ 邮件列表 + 详情。
 *
 * 真正的账户配置界面（含新增账户表单）属 P0-05，本文件只做 P0-06 需要的骨架。
 */

import InboxToolbar from "./features/inbox/InboxToolbar";
import MessageDetail from "./features/inbox/MessageDetail";
import MessageList from "./features/inbox/MessageList";
import { useAccounts, useMessage, useMessages } from "./features/inbox/useInbox";
import { useSelection } from "./stores/selection";

export default function App() {
  const accounts = useAccounts();

  const folderId = useSelection((state) => state.folderId);
  const messageId = useSelection((state) => state.messageId);
  const selectMessage = useSelection((state) => state.selectMessage);

  const messages = useMessages(folderId);
  const message = useMessage(messageId);

  return (
    <div className="app">
      <aside className="app__sidebar">
        <h1 className="app__title">dp_Gmail</h1>
        <InboxToolbar accounts={accounts.data ?? []} />
        {accounts.isError && <p className="app__error">{String(accounts.error)}</p>}
      </aside>

      <section className="app__list">
        {folderId === null && <p className="app__hint">选一个账户，然后点「同步」。</p>}
        {folderId !== null && messages.isPending && <p className="app__hint">读取本地邮件…</p>}
        {messages.isError && <p className="app__error">{String(messages.error)}</p>}
        {folderId !== null && messages.isSuccess && (
          <MessageList
            messages={messages.data}
            selectedId={messageId}
            onSelect={selectMessage}
          />
        )}
      </section>

      <section className="app__detail">
        {messageId === null && <p className="app__hint">选一封邮件查看详情。</p>}
        {messageId !== null && message.isPending && <p className="app__hint">读取邮件…</p>}
        {message.isError && <p className="app__error">{String(message.error)}</p>}
        {message.data && <MessageDetail message={message.data} />}
      </section>
    </div>
  );
}
