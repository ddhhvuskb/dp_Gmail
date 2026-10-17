/**
 * 搜索框。
 *
 * 提交时才切换列表模式 —— 不做输入即搜：`search_messages` 在短查询下走全表扫描，
 * 每敲一个字符打一次命令会白白烧 CPU。真要即时反馈，应像服务商识别那样先加去抖。
 */

import { useState } from "react";

import { useT } from "../../lib/i18n/useT";
import { useSelection } from "../../stores/selection";

export default function SearchBox() {
  const [draft, setDraft] = useState("");
  const showSearch = useSelection((state) => state.showSearch);
  const clearList = useSelection((state) => state.clearList);
  const t = useT();

  const trimmed = draft.trim();

  return (
    <form
      className="search-box"
      onSubmit={(event) => {
        event.preventDefault();
        if (trimmed !== "") {
          showSearch(trimmed);
        }
      }}
    >
      <input
        value={draft}
        onChange={(event) => setDraft(event.target.value)}
        placeholder={t("search.placeholder")}
        aria-label={t("search.label")}
      />
      <button type="submit" className="button" disabled={trimmed === ""}>
        {t("search.submit")}
      </button>
      {draft !== "" && (
        <button
          type="button"
          className="search-box__clear"
          onClick={() => {
            setDraft("");
            clearList();
          }}
        >
          {t("search.clear")}
        </button>
      )}
    </form>
  );
}
