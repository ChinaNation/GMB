import type { Env } from "../types";
import { HttpError, jsonResponse, requireSession } from "../shared/http";
import { nowMs } from "../shared/time";
import {
  readPlatformSubscription,
  type ChainSubscriptionState,
} from "../chain/subscription";
import { assertMembershipLevel } from "./plans";

/// 平台会员公民币轨 BFF：App 侧已用热钱包 extrinsic 把订阅/取消上链（subscribe/cancel，
/// pallet=square-post idx34，收款方=技术公司费用账户），本模块只做**上链后确认镜像**——
/// 把订阅态镜像进 D1 `square_memberships` 供发帖门禁与徽章读取，`tx_hash` 幂等。
/// 价格以链上 `PlatformPrice` 为唯一真源，BFF 不涉计价、不涉扣款、不计算自然日历。
///
/// 门禁口径与创作者一致（`creator.ts`）：只看镜像 `subscription_status='active'`，不按
/// `expires_at` 判过期——按月自动续扣发生在链上，客户端确认之间镜像不应假过期误拦。
///
/// confirm 必须先读取 finalized 链上 `Subscriptions`，只有链上状态与请求动作完全一致才写
/// D1；定时对账只负责续费/欠费后的持续同步，不承担纠正伪造 confirm 的安全职责。

export interface PlatformSubscriptionConfirmDeps {
  readPlatformSubscription: (
    env: Env,
    ownerAccount: string,
  ) => Promise<ChainSubscriptionState | null>;
}

const defaultConfirmDeps: PlatformSubscriptionConfirmDeps = {
  readPlatformSubscription,
};

/// POST /v1/square/membership/confirm —— 平台会员订阅/取消上链后镜像（幂等）。
/// owner 由 session 派生（不采信 body）；带合法 `level`=订阅→active，缺 `level`=取消→cancelled。
export async function platformSubscriptionConfirmRoute(
  request: Request,
  env: Env,
  deps: PlatformSubscriptionConfirmDeps = defaultConfirmDeps,
): Promise<Response> {
  const session = await requireSession(request, env);
  const body = (await request.json()) as { tx_hash?: unknown; level?: unknown };
  const txHash = typeof body.tx_hash === "string" ? body.tx_hash : "";
  if (!/^0x[0-9a-f]{64}$/.test(txHash)) {
    throw new HttpError(400, "invalid_request", "确认参数不完整");
  }
  const owner = session.owner_account;
  const now = nowMs();
  const hasLevel =
    body.level !== undefined && body.level !== null && body.level !== "";
  const chainState = await deps.readPlatformSubscription(env, owner);

  if (!hasLevel) {
    if (chainState === null || chainState.status !== "cancelled") {
      throw new HttpError(
        409,
        "subscription_state_not_finalized",
        "链上取消状态尚未最终确认",
      );
    }
    // 取消：翻 cancelled 并记权益失效时刻（视频冷归档时钟起点）；保留行用于展示与归档。
    await env.DB.prepare(
      `UPDATE square_memberships
        SET subscription_status = 'cancelled',
            entitlement_lapsed_at = COALESCE(entitlement_lapsed_at, ?),
            last_tx_hash = ?, updated_at = ?
        WHERE owner_account = ?`,
    )
      .bind(now, txHash, now, owner)
      .run();
    return jsonResponse({ ok: true, status: "cancelled" });
  }

  const level = assertMembershipLevel(body.level);
  if (
    chainState === null ||
    chainState.status !== "active" ||
    chainState.plan.kind !== "platform" ||
    chainState.plan.membershipLevel !== level
  ) {
    throw new HttpError(
      409,
      "subscription_state_not_finalized",
      "链上订阅状态尚未最终确认",
    );
  }
  // Worker 只镜像 runtime 已计算并 finalized 的链上时间戳，不复制自然日历算法。
  const periodStart = chainState.lastChargedAt;
  const periodEnd = chainState.paidUntil;
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
        last_tx_hash = excluded.last_tx_hash`,
  )
    .bind(owner, level, periodEnd, now, periodStart, periodEnd, txHash)
    .run();
  return jsonResponse({ ok: true, status: "active", membership_level: level });
}
