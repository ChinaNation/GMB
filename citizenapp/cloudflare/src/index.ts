import type { Env } from './types';
import { errorResponse } from './shared/http';
import { routeRequest } from './routes';
import { runVideoArchiveSweep } from './membership/archive';
import { reconcileSubscriptions } from './membership/reconcile';
import { applyCors, cleanupSecurityState } from './security/request_guard';
import { cleanupExpiredUploads } from './uploads/service';
import { cleanupExpiredReservations } from './limits/usage';

export { ChatRealtimeObject } from './chat/realtime';

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    try {
      return applyCors(request, env, await routeRequest(request, env));
    } catch (error) {
      return applyCors(request, env, errorResponse(error));
    }
  },

  // Cron 触发：清理 + 会员镜像对账（每 5 分钟，限流分批、幂等可续跑，由开关控制）；
  // 退订视频冷归档扫描（每日一次，由 ARCHIVE_ENABLED 开关控制）。
  async scheduled(_controller: ScheduledController, env: Env, ctx: ExecutionContext): Promise<void> {
    const jobs: Promise<unknown>[] = [
      cleanupExpiredUploads(env),
      cleanupSecurityState(env),
      cleanupExpiredReservations(env),
      // 平台与创作者共享同一个 finalized 链锚点，只处理已经到期的有限候选。
      reconcileSubscriptions(env),
    ];
    if (_controller.cron === '0 3 * * *') {
      jobs.push(runVideoArchiveSweep(env));
    }
    ctx.waitUntil(Promise.all(jobs).catch((error) => {
        console.error(
          `[scheduled-cleanup] failed: ${error instanceof Error ? error.message : error}`
        );
      }));
  }
};
