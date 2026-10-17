/**
 * 附件选择区。
 *
 * 必须走 `@tauri-apps/plugin-dialog`：webview 的 `<input type="file">`
 * 只能拿到 `File` 对象、**拿不到真实路径**，而后端按路径读附件
 * （见 `commands/compose.rs` 关于「路径而非字节」的说明）。
 *
 * 抽成独立组件是为了让 `ComposeForm` 留在 100 行以内（`DESIGN.md` 组件上限）。
 */

import { open } from "@tauri-apps/plugin-dialog";

import { useT } from "../../lib/i18n/useT";
import type { ComposeAttachment } from "../../stores/selection";

interface Props {
  attachments: ComposeAttachment[];
  onChange: (attachments: ComposeAttachment[]) => void;
}

/** 从完整路径取文件名，兼容 Windows 反斜杠与 POSIX 斜杠。 */
function basename(path: string): string {
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] || path;
}

export default function AttachmentField({ attachments, onChange }: Props) {
  const t = useT();

  async function pick() {
    const selected = await open({
      multiple: true,
      title: t("attachment.pickerTitle"),
    });
    if (!selected) {
      return;
    }
    const paths = Array.isArray(selected) ? selected : [selected];
    const added = paths.map((path) => ({ path, filename: basename(path) }));
    onChange([...attachments, ...added]);
  }

  return (
    <div className="field">
      <span>{t("attachment.label")}</span>

      <button type="button" className="button button--ghost" onClick={() => void pick()}>
        {t("attachment.add")}
      </button>

      {attachments.length > 0 && (
        <ul className="attachment-list">
          {attachments.map((item, index) => (
            <li key={`${item.path}-${index}`} className="attachment-list__item">
              <span className="attachment-list__name" title={item.path}>
                {item.filename}
              </span>
              <button
                type="button"
                className="attachment-list__remove"
                aria-label={t("attachment.remove", { filename: item.filename })}
                onClick={() => onChange(attachments.filter((_, i) => i !== index))}
              >
                ×
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
