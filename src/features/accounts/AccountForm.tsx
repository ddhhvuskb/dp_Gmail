/**
 * 新增账户表单。
 *
 * 安全约定：
 * - 密码/授权码**只存在于组件内存**，提交成功后立即清空。
 *   绝不写入 `localStorage`（`DESIGN.md` P0-05 禁止项），后端也只会存 AES-256-GCM 密文。
 * - 服务商由后端 `resolve_provider` 识别，前端不维护域名表。
 * - 识别成功前提交按钮保持禁用，避免用户对着"暂不支持"的地址白填一遍密码。
 */

import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";

import { api } from "../../lib/api";
import { useProviderLookup } from "./useProviderLookup";

export default function AccountForm() {
  const [email, setEmail] = useState("");
  const [secret, setSecret] = useState("");
  const [displayName, setDisplayName] = useState("");

  const lookup = useProviderLookup(email);
  const queryClient = useQueryClient();

  const add = useMutation({
    mutationFn: () => api.addAccount(email.trim(), secret, displayName.trim() || undefined),
    onSuccess: async () => {
      setEmail("");
      setSecret("");
      setDisplayName("");
      await queryClient.invalidateQueries({ queryKey: ["accounts"] });
    },
  });

  const detected = lookup.data ?? null;
  const canSubmit = email.trim() !== "" && secret !== "" && detected !== null && !add.isPending;

  return (
    <form
      className="account-form"
      onSubmit={(event) => {
        event.preventDefault();
        if (canSubmit) {
          add.mutate();
        }
      }}
    >
      <h2 className="sidebar__heading">添加账户</h2>

      <label className="field">
        <span>邮箱地址</span>
        <input
          value={email}
          onChange={(event) => setEmail(event.target.value)}
          placeholder="12345678@qq.com"
          autoComplete="off"
          spellCheck={false}
        />
      </label>

      {detected && (
        <p className="field__ok">
          已识别：{detected.provider_name}（{detected.imap_host}:{detected.imap_port}）
        </p>
      )}
      {lookup.isSuccess && detected === null && (
        <p className="field__error">暂不支持该邮箱服务商</p>
      )}

      <label className="field">
        <span>授权码 / 密码</span>
        <input
          type="password"
          value={secret}
          onChange={(event) => setSecret(event.target.value)}
          autoComplete="off"
        />
      </label>

      <label className="field">
        <span>显示名（可选）</span>
        <input value={displayName} onChange={(event) => setDisplayName(event.target.value)} />
      </label>

      <button type="submit" className="button" disabled={!canSubmit}>
        {add.isPending ? "保存中…" : "添加账户"}
      </button>

      {add.isError && <p className="field__error">{String(add.error)}</p>}
    </form>
  );
}
