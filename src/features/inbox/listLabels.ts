/**
 * 中栏的文案。
 *
 * 从 `App.tsx` 抽出来：它们是**纯函数**，与组件组装无关，
 * 留在壳子里会把 `App` 顶过 100 行上限（`AGENTS.md` 粒度限制）。
 *
 * 多语言化后签名多了一个 `t`：本文件**不**自己去 store 取语言，
 * 这样它仍然是可单测的纯函数。
 */

import type { Translator } from "../../lib/i18n";
import type { ListMode } from "../../stores/selection";

/**
 * 中栏标题：让人一眼知道当前看的是什么。
 *
 * 文件夹名与搜索词是**用户数据**，原样返回，不翻译。
 */
export function listTitle(mode: ListMode, t: Translator): string {
  switch (mode.kind) {
    case "folder":
      return mode.label;
    case "unified":
      return t("list.titleUnified");
    case "search":
      return t("list.titleSearch", { query: mode.query });
    default:
      return t("list.titleDefault");
  }
}

/**
 * 列表为空时的提示。
 *
 * 必须区分「搜不到」与「文件夹没邮件」：两者都渲染空白中栏的话，
 * 用户分不清是没结果、还在加载、还是界面坏了。
 */
export function emptyHint(mode: ListMode, t: Translator): string {
  switch (mode.kind) {
    case "search":
      return t("list.emptySearch", { query: mode.query });
    case "unified":
      return t("list.emptyUnified");
    default:
      return t("list.emptyFolder");
  }
}
