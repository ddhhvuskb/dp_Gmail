/**
 * 右栏：邮件详情 + 回复/转发入口。
 *
 * 从 `App.tsx` 抽出来：外壳要拼三栏，再塞进详情分支就会顶过 100 行上限
 * （`AGENTS.md` 粒度限制）。本组件自己从 store 取选中邮件，不接 props ——
 * 它需要的状态**只有**「选中了哪封」，没必要让外壳转发一遍。
 */

import { buildForward, buildReply } from "../compose/drafts";
import { useT } from "../../lib/i18n/useT";
import { useSelection } from "../../stores/selection";
import MessageDetail from "./MessageDetail";
import { useMessage } from "./useInbox";

export default function DetailPane() {
  const messageId = useSelection((state) => state.messageId);
  const openCompose = useSelection((state) => state.openCompose);
  const setNav = useSelection((state) => state.setNav);
  const t = useT();
  const message = useMessage(messageId);
  const data = message.data;

  return (
    <section className="app__detail">
      {/* 只在窄屏显示：桌面端三栏并排，不需要返回 */}
      <button type="button" className="mobile-back" onClick={() => setNav("list")}>
        <span aria-hidden="true">←</span>
        <span>{t("nav.backToList")}</span>
      </button>

      {messageId === null && <p className="app__hint">{t("app.selectMessage")}</p>}
      {messageId !== null && message.isPending && <p className="app__hint">{t("app.loading")}</p>}
      {message.isError && <p className="app__error">{String(message.error)}</p>}
      {data && (
        <>
          <div className="app__detail-actions">
            <button
              type="button"
              className="button"
              onClick={() => openCompose(buildReply(data, t))}
            >
              {t("app.reply")}
            </button>
            <button
              type="button"
              className="button"
              onClick={() => openCompose(buildForward(data, t))}
            >
              {t("app.forward")}
            </button>
          </div>
          <MessageDetail message={data} />
        </>
      )}
    </section>
  );
}
