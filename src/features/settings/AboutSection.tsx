/**
 * 「关于我们」与「联系方式」。
 *
 * 邮箱地址**不做成 `<a href="mailto:…">`**：本项目没有装 opener 插件，
 * webview 里点站外链接要么被 CSP 挡掉、要么把整个界面导航走。
 * 给成可选中的纯文本，用户自己复制到邮件客户端，比一个点了没反应的链接好。
 */

import { APP_VERSION, CONTACT_EMAIL } from "../../lib/appInfo";
import { useT } from "../../lib/i18n/useT";

export default function AboutSection() {
  const t = useT();

  return (
    <section className="settings__section">
      <h3 className="settings__heading">{t("settings.about")}</h3>
      <p className="settings__text">{t("about.intro")}</p>

      <dl className="settings__facts">
        <dt>{t("about.version")}</dt>
        <dd>{APP_VERSION}</dd>
        <dt>{t("about.stack")}</dt>
        <dd>{t("about.stackValue")}</dd>
        <dt>{t("about.providers")}</dt>
        <dd>{t("about.providersValue")}</dd>
      </dl>

      <h3 className="settings__heading">{t("settings.contact")}</h3>
      <p className="settings__text">{t("contact.hint")}</p>
      <p className="settings__contact">
        <code className="settings__code">{CONTACT_EMAIL}</code>
      </p>
    </section>
  );
}
