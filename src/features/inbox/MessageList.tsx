/**
 * 邮件列表（虚拟滚动）。
 *
 * 用 `@tanstack/react-virtual` 做 windowing：只挂载可视区附近的行，
 * 1000 封邮件时 DOM 节点数保持在几十个（`DESIGN.md` 性能验收标准）。
 */

import { useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";

import type { MessageSummary } from "../../lib/api";
import { formatDateTime, formatSender } from "../../lib/format";

/** 行高固定，虚拟化才能用常量估算。 */
const ROW_HEIGHT = 64;

interface Props {
  messages: MessageSummary[];
  selectedId: number | null;
  onSelect: (id: number) => void;
}

export default function MessageList({ messages, selectedId, onSelect }: Props) {
  const scrollRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: messages.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 8,
  });

  return (
    <div ref={scrollRef} className="message-list" role="listbox" aria-label="邮件列表">
      <div className="message-list__inner" style={{ height: virtualizer.getTotalSize() }}>
        {virtualizer.getVirtualItems().map((row) => {
          const message = messages[row.index];
          if (!message) {
            return null;
          }
          const isSelected = message.id === selectedId;

          return (
            <button
              key={message.id}
              type="button"
              role="option"
              aria-selected={isSelected}
              className={`message-row${isSelected ? " message-row--selected" : ""}${
                message.is_seen ? "" : " message-row--unread"
              }`}
              style={{ height: row.size, transform: `translateY(${row.start}px)` }}
              onClick={() => onSelect(message.id)}
            >
              <span className="message-row__line">
                <span className="message-row__sender">
                  {formatSender(message.from_name, message.from_address)}
                </span>
                <span className="message-row__date">{formatDateTime(message.date_unix)}</span>
              </span>
              <span className="message-row__line">
                <span className="message-row__subject">{message.subject ?? "(无主题)"}</span>
                {message.attachment_count > 0 && (
                  <span className="message-row__badge" title="含附件">
                    📎 {message.attachment_count}
                  </span>
                )}
              </span>
            </button>
          );
        })}
      </div>
    </div>
  );
}
