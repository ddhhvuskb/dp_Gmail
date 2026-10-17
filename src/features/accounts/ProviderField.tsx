/**
 * 服务商状态区：自动识别的结果，或**手动选择**的兜底。
 *
 * 手动选择只在自动识别失败时出现 —— 企业邮箱用客户自己的域名，
 * 后端 `resolve_by_email` 对它永远返回 `null`。不给这条路，用户就卡死在
 * 「暂不支持该邮箱服务商」上。
 *
 * 单独成组件是为了让 `AccountForm` 保持在 100 行以内（`AGENTS.md` 粒度限制）。
 */

import type { ProviderView } from "../../lib/api";
import { useProviders } from "./useProviders";

interface Props {
  /** 自动识别的结果；`null` 表示识别不出来。 */
  detected: ProviderView | null;
  /** 自动识别是否**已有结论**（还没查完时不该闪「暂不支持」）。 */
  settled: boolean;
  /** 用户手动选中的服务商标识，空串表示未选。 */
  manualId: string;
  onManualChange: (providerId: string) => void;
}

export default function ProviderField({ detected, settled, manualId, onManualChange }: Props) {
  const providers = useProviders();

  if (detected) {
    return (
      <p className="field__ok">
        已识别：{detected.provider_name}（{detected.imap_host}:{detected.imap_port}）
      </p>
    );
  }

  if (!settled) {
    return null;
  }

  if (providers.isError) {
    return <p className="field__error">{String(providers.error)}</p>;
  }

  return (
    <>
      <p className="field__error">未能自动识别该邮箱的服务商，请手动选择</p>
      <label className="field">
        <span>服务商</span>
        <select
          value={manualId}
          disabled={providers.isPending}
          onChange={(event) => onManualChange(event.target.value)}
        >
          <option value="">{providers.isPending ? "读取中…" : "— 请选择 —"}</option>
          {(providers.data ?? []).map((provider) => (
            <option key={provider.provider_id} value={provider.provider_id}>
              {provider.provider_name}
            </option>
          ))}
        </select>
      </label>
    </>
  );
}
