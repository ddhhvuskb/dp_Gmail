/**
 * 回复 / 转发的草稿构造。
 *
 * 与 `ComposeForm` 分开：这里是**纯函数**，可单独测试；
 * 组件只负责渲染与状态。
 */

import type { MessageView } from "../../lib/api";

/** 回复前缀不会被重复叠加。 */
const REPLY_PREFIX = "Re: ";
const FORWARD_PREFIX = "Fwd: ";

/** 引用原文时每行加的标记。 */
function quote(text: string): string {
  return text
    .split("\n")
    .map((line) => `> ${line}`)
    .join("\n");
}

/** 从已选邮件构造回复草稿。 */
export function buildReply(message: MessageView): { to: string; subject: string; text: string } {
  const original = message.text_body ?? "";
  const body = [
    "",
    "",
    `在 ${new Date(message.date_unix * 1000).toLocaleString("zh-CN")}，${message.from_address ?? "未知发件人"} 写道：`,
    quote(original),
  ].join("\n");

  return {
    to: message.from_address ?? "",
    subject: message.subject ? `${REPLY_PREFIX}${message.subject}` : REPLY_PREFIX.trim(),
    text: body,
  };
}

/** 从已选邮件构造转发草稿。 */
export function buildForward(message: MessageView): { to: string; subject: string; text: string } {
  const header = [
    "---------- 转发邮件 ----------",
    `发件人：${message.from_name ?? ""} <${message.from_address ?? ""}>`,
    `收件人：${message.to_addresses ?? ""}`,
    `时间：${new Date(message.date_unix * 1000).toLocaleString("zh-CN")}`,
    `主题：${message.subject ?? ""}`,
    "",
  ].join("\n");

  return {
    to: "",
    subject: message.subject ? `${FORWARD_PREFIX}${message.subject}` : FORWARD_PREFIX.trim(),
    text: `${header}${message.text_body ?? ""}`,
  };
}
