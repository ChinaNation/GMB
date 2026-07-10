import type { Env } from '../types';
import { HttpError } from '../shared/http';

const STRIPE_API_BASE = 'https://api.stripe.com/v1';

function requireStripeKey(env: Env): string {
  const key = env.STRIPE_SECRET_KEY;
  if (!key) {
    throw new HttpError(503, 'stripe_not_configured', 'Stripe secret key 未配置');
  }
  return key;
}

/// 立即退订（账户注销用）：Stripe DELETE /v1/subscriptions/{id}，当场终止订阅。
///
/// 本地 Miniflare 验收（STRIPE_DEV_CHECKOUT_PROXY==='1'）短路，不真打 Stripe。
export async function cancelStripeSubscriptionNow(
  env: Env,
  subscriptionId: string
): Promise<void> {
  if (env.STRIPE_DEV_CHECKOUT_PROXY === '1') {
    return;
  }
  const key = requireStripeKey(env);
  const response = await fetch(
    `${STRIPE_API_BASE}/subscriptions/${encodeURIComponent(subscriptionId)}`,
    { method: 'DELETE', headers: { authorization: `Bearer ${key}` } }
  );
  if (!response.ok) {
    throw new HttpError(502, 'stripe_cancel_failed', `Stripe 退订失败：${response.status}`);
  }
}

/// 到期取消（官网取消订阅用）：cancel_at_period_end=true，当期用完再终止。
export async function cancelStripeSubscriptionAtPeriodEnd(
  env: Env,
  subscriptionId: string
): Promise<void> {
  if (env.STRIPE_DEV_CHECKOUT_PROXY === '1') {
    return;
  }
  const key = requireStripeKey(env);
  const response = await fetch(
    `${STRIPE_API_BASE}/subscriptions/${encodeURIComponent(subscriptionId)}`,
    {
      method: 'POST',
      headers: {
        authorization: `Bearer ${key}`,
        'content-type': 'application/x-www-form-urlencoded'
      },
      body: 'cancel_at_period_end=true'
    }
  );
  if (!response.ok) {
    throw new HttpError(502, 'stripe_cancel_failed', `Stripe 到期取消失败：${response.status}`);
  }
}
