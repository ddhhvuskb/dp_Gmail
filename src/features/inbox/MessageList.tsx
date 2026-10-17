/**
 * 邮件列表（虚拟滚动）。
 *
 * 用 `@tanstack/react-virtual` 做 windowing：只挂载可视区附近的行，
 * 1000 封邮件时 DOM 节点数保持在几十个（`DESIGN.md` 性能验收标准）。
 */

import { useEffect, useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";

import { useT } from "../../lib/i18n/useT";
import MessageRow from "./MessageRow";
import type { ListRow } from "./useInbox";

/** 行高固定，虚拟化才能用常量估算。 */
const ROW_HEIGHT = 64;

/** 距列表末尾还剩这么多行时，就提前要下一页。 */
const PREFETCH_ROWS = 8;

interface Props {
  rows: ListRow[];
  selectedId: number | null;
  onSelect: (id: number) => void;
  /**
   * 滚到接近末尾时调用。
   *
   * **有没有下一页由调用方判断** —— 本组件只发出「快到底了」这个信号。
   * 否则它就得知道分页游标、请求状态等一堆与渲染无关的东西。
   */
  onLoadMore?: () => void;
  /**
   * 正在取下一页。
   *
   * 没有这个提示的话，网络慢时列表看起来就是「到头了」——
   * 用户不会去区分「只有这么多」和「还在取」。
   */
  isLoadingMore?: boolean;
}

export default function MessageList({
  rows,
  selectedId,
  onSelect,
  onLoadMore,
  isLoadingMore,
}: Props) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const t = useT();

  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 8,
  });

  const virtualRows = virtualizer.getVirtualItems();
  const lastRow = virtualRows[virtualRows.length - 1];
  const lastIndex = lastRow ? lastRow.index : -1;

  useEffect(() => {
    if (onLoadMore && lastIndex >= rows.length - 1 - PREFETCH_ROWS) {
      onLoadMore();
    }
  }, [lastIndex, rows.length, onLoadMore]);

  if (rows.length === 0) {
    return <p className="app__hint">{t("list.noMessages")}</p>;
  }

  return (
    <div ref={scrollRef} className="message-list" role="listbox" aria-label={t("list.listLabel")}>
      <div className="message-list__inner" style={{ height: virtualizer.getTotalSize() }}>
        {virtualRows.map((virtualRow) => {
          const row = rows[virtualRow.index];
          if (!row) {
            return null;
          }
          return (
            <MessageRow
              key={row.id}
              row={row}
              selected={row.id === selectedId}
              height={virtualRow.size}
              start={virtualRow.start}
              onSelect={onSelect}
            />
          );
        })}
      </div>
      {isLoadingMore && <p className="app__hint">{t("list.loadingMore")}</p>}
    </div>
  );
}
