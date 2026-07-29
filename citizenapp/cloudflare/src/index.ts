import type { Env, SquareNotifyJob } from './types';
import { errorResponse } from './shared/http';
import { routeRequest } from './routes';
import { fanOutPage } from './feeds/notify_fanout';
import { runExpiredMembershipContentCleanup } from './membership/expiration_cleanup';
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

  // Cron 触发：清理 + 会员镜像对账（每 5 分钟，限流分批、幂等可续跑，由开关控制）。
  // 每日到期内容清理必须串在同一轮 finalized 对账之后，只接受该区块的链上时间戳。
  async scheduled(_controller: ScheduledController, env: Env, ctx: ExecutionContext): Promise<void> {
    const subscriptionJob = reconcileSubscriptions(env).then(async (result) => {
      if (
        _controller.cron === '0 3 * * *' &&
        result.finalized_chain_timestamp !== null
      ) {
        await runExpiredMembershipContentCleanup(
          env,
          result.finalized_chain_timestamp,
        );
      }
      return result;
    });
    const jobs: Promise<unknown>[] = [
      cleanupExpiredUploads(env),
      cleanupSecurityState(env),
      cleanupExpiredReservations(env),
      // 平台与创作者共享同一个 finalized 链锚点，只处理已经到期的有限候选。
      subscriptionJob,
    ];
    ctx.waitUntil(Promise.all(jobs).catch((error) => {
        console.error(
          `[scheduled-cleanup] failed: ${error instanceof Error ? error.message : error}`
        );
      }));
  },

  // 广场发帖通知扇出：每条消息 = 一次发帖或一页续跑；fanOutPage 满页会把下一页续跑入队。
  // 单条成功 ack、失败 retry（最多 max_retries），不因一条拖垮整批。
  async queue(batch: MessageBatch<SquareNotifyJob>, env: Env): Promise<void> {
    await Promise.all(
      batch.messages.map(async (message) => {
        try {
          await fanOutPage(env, message.body);
          message.ack();
        } catch (error) {
          console.error(
            `[square-notify] fanout failed: ${error instanceof Error ? error.message : error}`,
          );
          message.retry();
        }
      }),
    );
  }
};
