/**
 * 邮件详情。
 *
 * 正文渲染的信任边界：`message.html_body` **已由后端 `security::sanitize_html`
 * 净化**（剥掉 `<script>`/`<style>` 内容与事件属性、丢弃 `javascript:` scheme），
 * 是唯一可以进入 `dangerouslySetInnerHTML` 的数据。
 *
 * `DESIGN.md` P0-06 禁止的是**直接**把未净化 HTML 交给它渲染；
 * 若日后需要双保险，应在此处再套一层前端净化器，而不是取消后端净化。
 */

import type { MessageView } from "../../lib/api";
import { formatBytes, formatDateTime, formatSender } from "../../lib/format";

interface Props {
  message: MessageView;
}

export default function MessageDetail({ message }: Props) {
  return (
    <article className="message-detail">
      <header className="message-detail__header">
        <h2>{message.subject ?? "(无主题)"}</h2>
        <dl>
          <dt>发件人</dt>
          <dd>{formatSender(message.from_name, message.from_address)}</dd>
          <dt>收件人</dt>
          <dd>{message.to_addresses ?? "—"}</dd>
          <dt>时间</dt>
          <dd>{formatDateTime(message.date_unix)}</dd>
        </dl>
      </header>

      {message.attachments.length > 0 && (
        <ul className="message-detail__attachments">
          {message.attachments.map((att) => (
            <li key={att.filename}>
              📎 {att.filename}
              <span className="message-detail__size">
                {att.content_type} · {formatBytes(att.size)}
              </span>
            </li>
          ))}
        </ul>
      )}

      {message.html_body ? (
        // 后端已净化；见文件头注释。
        <div className="message-detail__body" dangerouslySetInnerHTML={{ __html: message.html_body }} />
      ) : (
        <pre className="message-detail__body message-detail__body--text">
          {message.text_body ?? "(无正文)"}
        </pre>
      )}
    </article>
  );
}
