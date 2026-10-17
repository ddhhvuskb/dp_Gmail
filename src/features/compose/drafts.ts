/**
 * 回复 / 转发的草稿构造。
 *
 * 与 `ComposeForm` 分开：这里是**纯函数**，可单独测试；
 * 组件只负责渲染与状态。
 *
 * 多语言化后多接一个 `t`：日期按 `t.locale` 格式化，抬头文字取自词典。
 * 纯函数性质不变 —— `t` 是参数，不是全局状态。
 */

import type { Translator } from "../../lib/i18n";
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
export function buildReply(
  message: MessageView,
  t: Translator,
): { to: string; subject: string; text: string } {
  const original = message.text_body ?? "";
  const body = [
    "",
    "",
    t("drafts.replyAttribution", {
      date: new Date(message.date_unix * 1000).toLocaleString(t.locale),
      sender: message.from_address ?? t("format.unknownSender"),
    }),
    quote(original),
  ].join("\n");

  return {
    to: message.from_address ?? "",
    subject: message.subject ? `${REPLY_PREFIX}${message.subject}` : REPLY_PREFIX.trim(),
    text: body,
  };
}

/** 从已选邮件构造转发草稿。 */
export function buildForward(
  message: MessageView,
  t: Translator,
): { to: string; subject: string; text: string } {
  const header = [
    t("drafts.forwardHeader"),
    t("drafts.forwardFrom", {
      value: `${message.from_name ?? ""} <${message.from_address ?? ""}>`,
    }),
    t("drafts.forwardTo", { value: message.to_addresses ?? "" }),
    t("drafts.forwardDate", {
      value: new Date(message.date_unix * 1000).toLocaleString(t.locale),
    }),
    t("drafts.forwardSubject", { value: message.subject ?? "" }),
    "",
  ].join("\n");

  return {
    to: "",
    subject: message.subject ? `${FORWARD_PREFIX}${message.subject}` : FORWARD_PREFIX.trim(),
    text: `${header}${message.text_body ?? ""}`,
  };
}
