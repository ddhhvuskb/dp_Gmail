/** 设置面板的开关状态。纯客户端状态，不需要 Query，也不需要持久化。 */

import { create } from "zustand";

interface SettingsDialogState {
  open: boolean;
  openSettings: () => void;
  closeSettings: () => void;
}

export const useSettingsDialog = create<SettingsDialogState>((set) => ({
  open: false,
  openSettings: () => set({ open: true }),
  closeSettings: () => set({ open: false }),
}));
