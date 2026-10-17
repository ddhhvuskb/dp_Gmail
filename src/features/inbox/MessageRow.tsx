/**
 * 列表中的一行。
 *
 * 从 `MessageList.tsx` 抽出来：那个组件要做虚拟化编排（滚动容器、overscan、触底信号），
 * 再塞进一行的 DOM 结构就会顶过 100 行上限（`AGENTS.md` 粒度限制）。
 *
 * 位置与高度由虚拟化器算好后**通过 props 传入**，本组件不做任何计算 ——
 * 它必须是纯展示的，否则会破坏 rowHeight 恒定这个前提。
 */

import { useFormat, useT } from "../../lib/i18n/useT";
import type { ListRow } from "./useInbox";

interface Props {
  row: ListRow;
  selected: boolean;
  /** 虚拟化器给的像素高度。 */
  height: number;
  /** 虚拟化器给的顶部偏移。 */
  start: number;
  onSelect: (id: number) => void;
}

export default function MessageRow({ row, selected, height, start, onSelect }: Props) {
  const t = useT();
  const format = useFormat();

  return (
    <button
      type="button"
      role="option"
      aria-selected={selected}
      className={`message-row${selected ? " message-row--selected" : ""}${
        row.is_seen ? "" : " message-row--unread"
      }`}
      style={{ height, transform: `translateY(${start}px)` }}
      onClick={() => onSelect(row.id)}
    >
      <span className="message-row__line">
        <span className="message-row__sender">
          {format.sender(row.from_name, row.from_address)}
        </span>
        <span className="message-row__date">{format.dateTime(row.date_unix)}</span>
      </span>
      <span className="message-row__line">
        <span className="message-row__subject">{row.subject ?? t("list.noSubject")}</span>
        {row.attachment_count > 0 && (
          <span className="message-row__badge" title={t("list.hasAttachment")}>
            📎 {row.attachment_count}
          </span>
        )}
      </span>
      {row.source && <span className="message-row__source">{row.source}</span>}
    </button>
  );
}
