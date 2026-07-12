import type { Env } from './types';
import { errorResponse } from './shared/http';
import { routeRequest } from './routes';
import { runVideoArchiveSweep } from './membership/archive';
import { applyCors, cleanupSecurityState } from './security/request_guard';
import { cleanupExpiredUploads } from './uploads/service';

export { ChatRealtimeObject } from './chat/realtime';

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    try {
      return applyCors(request, env, await routeRequest(request, env));
    } catch (error) {
      return applyCors(request, env, errorResponse(error));
    }
  },

  // Cron 触发：退订视频冷归档扫描（限流分批、幂等可续跑，由 ARCHIVE_ENABLED 开关控制）。
  async scheduled(_controller: ScheduledController, env: Env, ctx: ExecutionContext): Promise<void> {
    const jobs: Promise<unknown>[] = [
      cleanupExpiredUploads(env),
      cleanupSecurityState(env)
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
