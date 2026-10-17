/**
 * 「标签 + 单行输入」的最小封装。
 *
 * 只为 `AccountEditForm` 服务：那个表单有三个结构完全相同的字段，
 * 展开写会把组件顶过 100 行上限（`AGENTS.md` 粒度限制）。
 *
 * 没有顺手改 `AccountForm` —— 它现在能用且已在行数边缘，为了统一而改它是拿稳定性换整洁。
 */

interface Props {
  label: string;
  value: string;
  onChange: (value: string) => void;
  /** 密码类字段用 `type="password"`，避免授权码明文留在屏幕上。 */
  password?: boolean;
}

export default function LabeledInput({ label, value, onChange, password }: Props) {
  return (
    <label className="field">
      <span>{label}</span>
      <input
        type={password ? "password" : "text"}
        value={value}
        onChange={(event) => onChange(event.target.value)}
        autoComplete="off"
        spellCheck={false}
      />
    </label>
  );
}
