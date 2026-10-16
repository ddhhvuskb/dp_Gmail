/**
 * 邮件列表（虚拟滚动）。
 *
 * 用 `@tanstack/react-virtual` 做 windowing：只挂载可视区附近的行，
 * 1000 封邮件时 DOM 节点数保持在几十个（`DESIGN.md` 性能验收标准）。
 */

import { useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";

import { formatDateTime, formatSender } from "../../lib/format";
import type { ListRow } from "./useInbox";

/** 行高固定，虚拟化才能用常量估算。 */
const ROW_HEIGHT = 64;

interface Props {
  rows: ListRow[];
  selectedId: number | null;
  onSelect: (id: number) => void;
}

export default function MessageList({ rows, selectedId, onSelect }: Props) {
  const scrollRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 8,
  });

  if (rows.length === 0) {
    return <p className="app__hint">这里没有邮件。</p>;
  }

  return (
    <div ref={scrollRef} className="message-list" role="listbox" aria-label="邮件列表">
      <div className="message-list__inner" style={{ height: virtualizer.getTotalSize() }}>
        {virtualizer.getVirtualItems().map((virtualRow) => {
          const row = rows[virtualRow.index];
          if (!row) {
            return null;
          }
          const isSelected = row.id === selectedId;

          return (
            <button
              key={row.id}
              type="button"
              role="option"
              aria-selected={isSelected}
              className={`message-row${isSelected ? " message-row--selected" : ""}${
                row.is_seen ? "" : " message-row--unread"
              }`}
              style={{ height: virtualRow.size, transform: `translateY(${virtualRow.start}px)` }}
              onClick={() => onSelect(row.id)}
            >
              <span className="message-row__line">
                <span className="message-row__sender">
                  {formatSender(row.from_name, row.from_address)}
                </span>
                <span className="message-row__date">{formatDateTime(row.date_unix)}</span>
              </span>
              <span className="message-row__line">
                <span className="message-row__subject">{row.subject ?? "(无主题)"}</span>
                {row.attachment_count > 0 && (
                  <span className="message-row__badge" title="含附件">
                    📎 {row.attachment_count}
                  </span>
                )}
              </span>
              {row.source && <span className="message-row__source">{row.source}</span>}
            </button>
          );
        })}
      </div>
    </div>
  );
}
