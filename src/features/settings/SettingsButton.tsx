/** 侧栏的「设置」入口。设置项都在 `SettingsDialog` 里，侧栏不再平铺。 */

import { useT } from "../../lib/i18n/useT";
import { useSettingsDialog } from "../../stores/settingsDialog";

export default function SettingsButton() {
  const openSettings = useSettingsDialog((state) => state.openSettings);
  const t = useT();

  return (
    <button type="button" className="button button--ghost settings-open" onClick={openSettings}>
      <span aria-hidden="true">⚙️</span>
      <span>{t("settings.open")}</span>
    </button>
  );
}
