/**
 * 隐私政策（应用内展示）。
 *
 * 正文在词典里（`privacy.p1`…`privacy.p9`），三种语言各一份 ——
 * 中文版是准据，日/英为译文。
 *
 * 内容必须**与代码事实一致**，否则就是欺骗用户。每条都能对上：
 * 无服务器 / 无遥测（没有 HTTP 客户端）、AES-256-GCM（`security/credentials.rs`）、
 * ammonia 净化（`security/sanitize.rs`）、只连服务商（`mail/provider.rs`）。
 */

import { useT } from "../../lib/i18n/useT";
import type { Key } from "../../lib/i18n";

/** 顺序即阅读顺序，不要只按 key 排序。 */
const PARAGRAPHS: Key[] = [
  "privacy.p1",
  "privacy.p2",
  "privacy.p3",
  "privacy.p4",
  "privacy.p5",
  "privacy.p6",
  "privacy.p7",
  "privacy.p8",
  "privacy.p9",
];

export default function PrivacySection() {
  const t = useT();

  return (
    <section className="settings__section">
      <h3 className="settings__heading">{t("settings.privacy")}</h3>
      <p className="settings__meta">{t("privacy.effective")}</p>
      {PARAGRAPHS.map((key) => (
        <p key={key} className="settings__text settings__policy">
          {t(key)}
        </p>
      ))}
    </section>
  );
}
