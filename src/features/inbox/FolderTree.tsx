/**
 * 文件夹树。
 *
 * 数据来自 `list_folders`，展示各文件夹的邮件数与未读数。
 */

import type { FolderRecord } from "../../lib/api";
import { useSelection } from "../../stores/selection";

interface Props {
  folders: FolderRecord[];
  accounts: { id: number; label: string }[];
}

export default function FolderTree({ folders, accounts }: Props) {
  const mode = useSelection((state) => state.mode);
  const showFolder = useSelection((state) => state.showFolder);
  const showUnified = useSelection((state) => state.showUnified);

  // 统一收件箱按钮**始终**渲染：中栏在空状态下也会提示「点统一收件箱」，
  // 若这里提前 return，那句提示就指向了一个不存在的按钮。
  const hasFolders = folders.length > 0;

  // 按账户分组展示；跨账户时标题才有意义
  const grouped = new Map<number, FolderRecord[]>();
  for (const folder of folders) {
    const bucket = grouped.get(folder.account_id) ?? [];
    bucket.push(folder);
    grouped.set(folder.account_id, bucket);
  }

  return (
    <div className="folder-tree">
      <button
        type="button"
        className={`folder-tree__item folder-tree__item--unified${
          mode.kind === "unified" ? " folder-tree__item--active" : ""
        }`}
        onClick={showUnified}
      >
        <span className="folder-tree__name">📥 统一收件箱</span>
      </button>

      {!hasFolders && <p className="sidebar__hint">还没有同步过文件夹。</p>}

      {[...grouped.entries()].map(([accountId, bucket]) => (
        <div key={accountId} className="folder-tree__group">
          <p className="folder-tree__account">
            {accounts.find((account) => account.id === accountId)?.label ??
              `账户 ${accountId}`}
          </p>
          {bucket.map((folder) => {
            const active = mode.kind === "folder" && mode.folderId === folder.id;
            return (
              <button
                key={folder.id}
                type="button"
                className={`folder-tree__item${active ? " folder-tree__item--active" : ""}`}
                onClick={() => showFolder(folder.id, folder.name)}
              >
                <span className="folder-tree__name">{folder.name}</span>
                <span className="folder-tree__counts">
                  {folder.unread_count > 0 && (
                    <span className="folder-tree__unread">{folder.unread_count}</span>
                  )}
                  <span className="folder-tree__total">{folder.message_count}</span>
                </span>
              </button>
            );
          })}
        </div>
      ))}
    </div>
  );
}
