import type { Env } from '../types';
import { HttpError, jsonResponse, requireSession } from '../shared/http';
import { nowMs } from '../shared/time';
import { assertMembershipLevel } from './plans';

/// 平台会员公民币轨 BFF：App 侧已用热钱包 extrinsic 把订阅/取消上链（subscribe/cancel，
/// pallet=square-post idx34，收款方=技术公司费用账户），本模块只做**上链后确认镜像**——
/// 把订阅态镜像进 D1 `square_memberships` 供发帖门禁与徽章读取，`tx_hash` 幂等。
/// 价格与按月扣款以链上 `PlatformPrice` / billing keeper 为唯一真源，BFF 不涉计价、不涉扣款。
///
/// 门禁口径与创作者一致（`creator.ts`）：只看镜像 `subscription_status='active'`，不按
/// `expires_at` 判过期——按月自动续扣发生在链上，客户端确认之间镜像不应假过期误拦。
/// TODO(硬化，与 creator 统一)：链读 `Subscriptions[(owner, Platform)]` 核实后再镜像；
/// 当前信任 App 已上链的 tx（与 `creatorSubscriptionConfirmRoute` 同一约束）。

const MONTH_MS_FALLBACK = 31 * 24 * 60 * 60 * 1000;

/// 给 [baseMs] 加 [months] 个日历月（跨月按自然日历，非固定 30 天）。
function addMonths(baseMs: number, months: number): number {
  const d = new Date(baseMs);
  d.setUTCMonth(d.getUTCMonth() + months);
  const shifted = d.getTime();
  // 极端情况下 setUTCMonth 回绕失败时回退固定周期，保证 period_end 恒大于 now。
  return shifted > baseMs ? shifted : baseMs + MONTH_MS_FALLBACK;
}

/// POST /v1/square/membership/confirm —— 平台会员订阅/取消上链后镜像（幂等）。
/// owner 由 session 派生（不采信 body）；带合法 `level`=订阅→active，缺 `level`=取消→cancelled。
export async function platformSubscriptionConfirmRoute(
  request: Request,
  env: Env
): Promise<Response> {
  const session = await requireSession(request, env);
  const body = (await request.json()) as { tx_hash?: unknown; level?: unknown };
  const txHash = typeof body.tx_hash === 'string' ? body.tx_hash : '';
  if (!/^0x[0-9a-f]{64}$/.test(txHash)) {
    throw new HttpError(400, 'invalid_request', '确认参数不完整');
  }
  const owner = session.owner_account;
  const now = nowMs();
  const hasLevel = body.level !== undefined && body.level !== null && body.level !== '';

  if (!hasLevel) {
    // 取消：翻 cancelled 并记权益失效时刻（视频冷归档时钟起点）；保留行用于展示与归档。
    await env.DB.prepare(
      `UPDATE square_memberships
        SET subscription_status = 'cancelled',
            entitlement_lapsed_at = COALESCE(entitlement_lapsed_at, ?),
            last_tx_hash = ?, updated_at = ?
        WHERE owner_account = ?`
    )
      .bind(now, txHash, now, owner)
      .run();
    return jsonResponse({ ok: true, status: 'cancelled' });
  }

  const level = assertMembershipLevel(body.level);
  // 计费周期镜像（用量额度窗口）：起点=现在、终点=下次扣款（近似 +1 日历月）。
  // 链上 billing keeper 为实际扣款唯一真源，这里只给 usage 周期与徽章一个稳定窗口。
  const periodStart = now;
  const periodEnd = addMonths(now, 1);
  await env.DB.prepare(
    `INSERT INTO square_memberships
      (owner_account, membership_level, expires_at, updated_at, subscription_status,
       current_period_start, current_period_end, entitlement_lapsed_at, last_tx_hash)
      VALUES (?, ?, ?, ?, 'active', ?, ?, NULL, ?)
      ON CONFLICT(owner_account) DO UPDATE SET
        membership_level = excluded.membership_level,
        expires_at = excluded.expires_at,
        updated_at = excluded.updated_at,
        subscription_status = 'active',
        current_period_start = excluded.current_period_start,
        current_period_end = excluded.current_period_end,
        entitlement_lapsed_at = NULL,
        last_tx_hash = excluded.last_tx_hash`
  )
    .bind(owner, level, periodEnd, now, periodStart, periodEnd, txHash)
    .run();
  return jsonResponse({ ok: true, status: 'active', membership_level: level });
}
