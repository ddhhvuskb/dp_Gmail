/**
 * 设置面板。
 *
 * 侧栏原本平铺了主题按钮、外观选择、语言选择三块，视觉上太杂 ——
 * 现在只剩一个「设置」入口，其余全收进来。
 *
 * 实现成一个覆盖层（不是新窗口）：`DESIGN.md` 锁的是单窗口应用，
 * 多开一个 WebView 窗口要改 tauri.conf 与 capabilities，收益不值。
 */

import { useEffect } from "react";

import { useT } from "../../lib/i18n/useT";
import { useSettingsDialog } from "../../stores/settingsDialog";
import AboutSection from "./AboutSection";
import AppearancePicker from "./AppearancePicker";
import AutoSyncToggle from "./AutoSyncToggle";
import LanguageSelect from "./LanguageSelect";
import PrivacySection from "./PrivacySection";
import ThemeToggle from "./ThemeToggle";

export default function SettingsDialog() {
  const open = useSettingsDialog((state) => state.open);
  const closeSettings = useSettingsDialog((state) => state.closeSettings);
  const t = useT();

  // Esc 关闭：覆盖层挡住主界面时，键盘是唯一不经过鼠标的退出路径
  useEffect(() => {
    if (!open) {
      return;
    }
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        closeSettings();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, closeSettings]);

  if (!open) {
    return null;
  }

  return (
    <div className="settings" role="dialog" aria-modal="true" aria-label={t("settings.title")}>
      <div className="settings__backdrop" onClick={closeSettings} />

      <div className="settings__panel">
        <header className="settings__header">
          <h2 className="settings__title">{t("settings.title")}</h2>
          <button
            type="button"
            className="settings__close"
            onClick={closeSettings}
            aria-label={t("settings.close")}
          >
            ×
          </button>
        </header>

        <div className="settings__body">
          <section className="settings__section">
            <h3 className="settings__heading">{t("settings.appearance")}</h3>
            <ThemeToggle />
            <AppearancePicker />
          </section>

          <section className="settings__section">
            <h3 className="settings__heading">{t("settings.language")}</h3>
            <LanguageSelect />
          </section>

          <section className="settings__section">
            <h3 className="settings__heading">{t("settings.autoSync")}</h3>
            <AutoSyncToggle />
          </section>

          <AboutSection />
          <PrivacySection />
        </div>
      </div>
    </div>
  );
}
