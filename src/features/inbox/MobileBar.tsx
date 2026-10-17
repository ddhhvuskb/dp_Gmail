/**
 * 窄屏顶部唯一的导航控件：左上角的「☰ / ×」抽屉开关。
 *
 * 桌面端三栏并排、侧栏常驻，这个控件**不显示**（由 CSS 的 `@media` 控制）。
 *
 * 遮罩也在这里渲染：它是 `position: fixed; inset: 0`，挂在哪个父节点下都能盖满屏幕，
 * 所以不必为此在 `App` 里多写一行。点遮罩即关抽屉。
 */

import { useT } from "../../lib/i18n/useT";
import { useSelection } from "../../stores/selection";

export default function MobileBar() {
  const drawerOpen = useSelection((state) => state.drawerOpen);
  const setDrawer = useSelection((state) => state.setDrawer);
  const t = useT();

  return (
    <>
      {drawerOpen && (
        <div className="drawer-backdrop" onClick={() => setDrawer(false)} aria-hidden="true" />
      )}
      <button
        type="button"
        className="mobile-bar"
        onClick={() => setDrawer(!drawerOpen)}
        aria-expanded={drawerOpen}
        aria-label={drawerOpen ? t("nav.closeSidebar") : t("nav.openSidebar")}
      >
        <span aria-hidden="true">{drawerOpen ? "×" : "☰"}</span>
      </button>
    </>
  );
}
