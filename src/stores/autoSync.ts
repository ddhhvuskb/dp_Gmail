/**
 * 自动收信偏好。
 *
 * 间隔**固定 15 秒**：用户要的就是这个值，所以不做间隔选择器 ——
 * 多一个下拉框就多一种用户能配出问题的状态。
 *
 * 但**必须给开关**：每 15 秒一次意味着每小时 240 次 IMAP 连接，
 * 服务商有可能会限流。用户得能一键关掉，而不是只能卸载。
 */

import { create } from "zustand";

/** 自动收信间隔（毫秒）。改这里同时要改界面上「每 15 秒」的文案。 */
export const AUTO_SYNC_INTERVAL_MS = 15_000;

const STORAGE_KEY = "dpgmail.autosync";

/** 默认**开启**：用户的诉求就是「不用手动刷新」。 */
function initialEnabled(): boolean {
  return localStorage.getItem(STORAGE_KEY) !== "off";
}

interface AutoSyncState {
  enabled: boolean;
  setEnabled: (enabled: boolean) => void;
}

export const useAutoSync = create<AutoSyncState>((set) => ({
  enabled: initialEnabled(),
  setEnabled: (enabled) =>
    set(() => {
      localStorage.setItem(STORAGE_KEY, enabled ? "on" : "off");
      return { enabled };
    }),
}));
