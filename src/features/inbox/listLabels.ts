/**
 * 中栏的文案。
 *
 * 从 `App.tsx` 抽出来：它们是**纯函数**，与组件组装无关，
 * 留在壳子里会把 `App` 顶过 100 行上限（`AGENTS.md` 粒度限制）。
 */

import type { ListMode } from "../../stores/selection";

/** 中栏标题：让人一眼知道当前看的是什么。 */
export function listTitle(mode: ListMode): string {
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
export function emptyHint(mode: ListMode): string {
  switch (mode.kind) {
    case "search":
      return `没有匹配「${mode.query}」的邮件。`;
    case "unified":
      return "统一收件箱里还没有邮件，先同步一个文件夹。";
    default:
      return "这个文件夹里还没有邮件，先同步一次。";
  }
}
