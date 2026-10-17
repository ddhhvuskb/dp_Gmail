/** 自动收信开关。间隔固定，所以这里只有一个开关加一段说明。 */

import { useT } from "../../lib/i18n/useT";
import { useAutoSync } from "../../stores/autoSync";

export default function AutoSyncToggle() {
  const enabled = useAutoSync((state) => state.enabled);
  const setEnabled = useAutoSync((state) => state.setEnabled);
  const t = useT();

  return (
    <div className="settings__auto">
      <div className="settings__auto-row">
        <span className="settings__text">{t("autoSync.label")}</span>
        <button
          type="button"
          className={`settings__switch${enabled ? " settings__switch--on" : ""}`}
          role="switch"
          aria-checked={enabled}
          aria-label={t("autoSync.label")}
          onClick={() => setEnabled(!enabled)}
        >
          <span className="settings__knob" aria-hidden="true" />
        </button>
        <span className="settings__meta">{enabled ? t("autoSync.on") : t("autoSync.off")}</span>
      </div>
      <p className="settings__hint">{t("autoSync.hint")}</p>
    </div>
  );
}
