/**
 * 自动收信的定时循环。
 *
 * 三条自我约束，否则「每 15 秒」会变成自伤：
 *
 * 1. **窗口不可见时不发** —— 最小化到托盘还在每 15 秒登一次 IMAP 是纯浪费，
 *    而且会平白增加被服务商限流的概率。
 * 2. **不叠加** —— 上一轮还没回来就跳过这一轮。网络慢时死等，
 *    否则会堆出一串并发连接，反而把一次正常的同步拖死。
 * 3. **失败静默** —— 每 15 秒弹一次红色报错比不报更糟。手动点「同步」
 *    仍然会如实报错，那是用户主动发起的，理应看得见。
 */

import { useEffect, useRef } from "react";
import { useQueryClient } from "@tanstack/react-query";

import { api } from "../../lib/api";
import { AUTO_SYNC_INTERVAL_MS } from "../../stores/autoSync";

/** 每轮拉多少封。与手动同步同量级，重点是**新邮件**而不是补历史。 */
const AUTO_SYNC_LIMIT = 50;

export function useAutoSyncLoop(enabled: boolean, accountId: number | null, mailbox: string): void {
  const queryClient = useQueryClient();
  const inFlight = useRef(false);

  useEffect(() => {
    if (!enabled || accountId === null) {
      return;
    }
    // 收窄成常量：下面的异步函数里 TS 不再认为它可能是 null
    const account = accountId;
    let stopped = false;

    async function tick() {
      if (stopped || inFlight.current || document.visibilityState !== "visible") {
        return;
      }
      inFlight.current = true;
      try {
        await api.syncFolder(account, mailbox, AUTO_SYNC_LIMIT);
        if (stopped) {
          return;
        }
        await queryClient.invalidateQueries({ queryKey: ["list"] });
        await queryClient.invalidateQueries({ queryKey: ["folders"] });
      } catch {
        // 见文件头第 3 条：自动收信失败不打扰用户。
      } finally {
        inFlight.current = false;
      }
    }

    const timer = window.setInterval(() => void tick(), AUTO_SYNC_INTERVAL_MS);
    return () => {
      stopped = true;
      window.clearInterval(timer);
    };
  }, [enabled, accountId, mailbox, queryClient]);
}
