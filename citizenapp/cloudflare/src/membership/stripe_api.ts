import type { Env } from '../types';
import { HttpError } from '../shared/http';

const STRIPE_API_BASE = 'https://api.stripe.com/v1';

function requireStripeKey(env: Env): string {
  const key = env.STRIPE_API_KEY;
  if (!key) {
    throw new HttpError(503, 'stripe_not_configured', 'Stripe secret key 未配置');
  }
  return key;
}

/// 识别 Stripe「客户无可扣款方式」类错误（升档扣差价时才会遇到）。
function isNoPaymentMethodError(message?: string): boolean {
  const msg = (message ?? '').toLowerCase();
  return msg.includes('payment method') || msg.includes('payment source');
}

/// 立即退订（账户注销用）：Stripe DELETE /v1/subscriptions/{id}，当场终止订阅。
///
/// 本地 Miniflare 验收（STRIPE_DEV_PROXY==='1'）短路，不真打 Stripe。
export async function cancelStripeSubscriptionNow(
  env: Env,
  subscriptionId: string
): Promise<void> {
  if (env.STRIPE_DEV_PROXY === '1') {
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
  if (env.STRIPE_DEV_PROXY === '1') {
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

/// 续订（撤销「到期取消」）：cancel_at_period_end=false，当期内反悔可恢复。
export async function resumeStripeSubscription(
  env: Env,
  subscriptionId: string
): Promise<void> {
  if (env.STRIPE_DEV_PROXY === '1') {
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
      body: 'cancel_at_period_end=false'
    }
  );
  if (!response.ok) {
    throw new HttpError(502, 'stripe_resume_failed', `Stripe 续订失败：${response.status}`);
  }
}

/// 冻结时暂停收款：pause_collection[behavior]=void，权益不可用期间不再向用户扣费。
export async function pauseStripeCollection(
  env: Env,
  subscriptionId: string
): Promise<void> {
  if (env.STRIPE_DEV_PROXY === '1') {
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
      body: 'pause_collection[behavior]=void'
    }
  );
  if (!response.ok) {
    throw new HttpError(502, 'stripe_pause_failed', `Stripe 暂停收款失败：${response.status}`);
  }
}

/// 解冻时恢复收款：清空 pause_collection（换档到匹配档后调用）。
export async function resumeStripeCollection(
  env: Env,
  subscriptionId: string
): Promise<void> {
  if (env.STRIPE_DEV_PROXY === '1') {
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
      // 空值清空 pause_collection，恢复自动收款。
      body: 'pause_collection='
    }
  );
  if (!response.ok) {
    throw new HttpError(
      502,
      'stripe_resume_collection_failed',
      `Stripe 恢复收款失败：${response.status}`
    );
  }
}

interface StripeInvoiceApi {
  status?: string;
  hosted_invoice_url?: string | null;
}

interface StripeSubscriptionApi {
  items?: { data?: Array<{ id?: string }> };
  latest_invoice?: StripeInvoiceApi | string | null;
  error?: { message?: string };
}

/// Webhook 收到 subscription.created/updated 后回读 Stripe 当前对象。
/// 迟到事件携带的旧快照不参与落库，避免乱序投递把新档位或新周期回滚。
export async function retrieveStripeSubscription(
  env: Env,
  subscriptionId: string
): Promise<Record<string, unknown>> {
  const key = requireStripeKey(env);
  const response = await fetch(
    `${STRIPE_API_BASE}/subscriptions/${encodeURIComponent(subscriptionId)}`,
    { headers: { authorization: `Bearer ${key}` } }
  );
  const raw = (await response.json().catch(() => ({}))) as Record<string, unknown>;
  if (!response.ok) {
    throw new HttpError(
      502,
      'stripe_subscription_read_failed',
      `Stripe 订阅读取失败：${response.status}`
    );
  }
  return raw;
}

/// 换档结果：applied=变更是否已即时生效；paymentUrl=升档需用户主动付差价时的托管账单地址。
export interface TierChangeResult {
  applied: boolean;
  paymentUrl: string | null;
}

/// 在既有订阅上换档（改 price item，不新建订阅），按 proration 结算：
/// - 升档：always_invoice 立即出差价账单 + pending_if_incomplete —— 差价付成功才应用；
///   卡可 off-session 扣则即时生效（applied=true），否则返回 hosted_invoice_url 引导付款
///   （applied=false）。
/// - 降档 / 同价：create_prorations —— 差额进 Stripe 信用余额抵后续账单，不收款、即时生效。
/// 权益落库仍以 subscription webhook 为唯一真源，本函数只驱动 Stripe 变更。
export async function changeStripeSubscriptionTier(
  env: Env,
  params: { subscriptionId: string; newPriceId: string; isUpgrade: boolean }
): Promise<TierChangeResult> {
  if (env.STRIPE_DEV_PROXY === '1') {
    return { applied: true, paymentUrl: null };
  }
  const key = requireStripeKey(env);
  const subId = encodeURIComponent(params.subscriptionId);

  // 1) 取当前订阅的 item id（换价须指定被替换的 item）。
  const getRes = await fetch(`${STRIPE_API_BASE}/subscriptions/${subId}`, {
    headers: { authorization: `Bearer ${key}` }
  });
  const getData = (await getRes.json().catch(() => ({}))) as StripeSubscriptionApi;
  if (!getRes.ok) {
    throw new HttpError(
      502,
      'stripe_subscription_read_failed',
      `Stripe 订阅读取失败：${getRes.status}`
    );
  }
  const itemId = getData.items?.data?.[0]?.id;
  if (!itemId) {
    throw new HttpError(502, 'stripe_subscription_item_missing', 'Stripe 订阅缺少可换档条目');
  }

  // 2) 换 price + proration。
  const body = new URLSearchParams();
  body.set('items[0][id]', itemId);
  body.set('items[0][price]', params.newPriceId);
  body.set('proration_behavior', params.isUpgrade ? 'always_invoice' : 'create_prorations');
  if (params.isUpgrade) {
    // pending_if_incomplete：差价账单付成功才应用换档，否则挂起为 pending update。
    body.set('payment_behavior', 'pending_if_incomplete');
  }
  body.set('expand[0]', 'latest_invoice');

  const postRes = await fetch(`${STRIPE_API_BASE}/subscriptions/${subId}`, {
    method: 'POST',
    headers: {
      authorization: `Bearer ${key}`,
      'content-type': 'application/x-www-form-urlencoded'
    },
    body
  });
  const postData = (await postRes.json().catch(() => ({}))) as StripeSubscriptionApi;
  if (!postRes.ok) {
    // 升档无可自动扣款方式：Stripe 直接 400（不给待付账单），返回可操作提示而非裸 502。
    if (params.isUpgrade && isNoPaymentMethodError(postData.error?.message)) {
      throw new HttpError(
        402,
        'membership_upgrade_needs_payment',
        '该订阅没有可自动扣款的支付方式，暂无法直接升档；请完成一次付款后再升档'
      );
    }
    throw new HttpError(
      502,
      'stripe_subscription_update_failed',
      postData.error?.message ?? 'Stripe 换档失败'
    );
  }

  // 降档 / 同价：即时生效，差额进信用余额。
  if (!params.isUpgrade) {
    return { applied: true, paymentUrl: null };
  }
  // 升档：看差价账单是否已付。
  const invoice =
    typeof postData.latest_invoice === 'object' && postData.latest_invoice !== null
      ? postData.latest_invoice
      : null;
  const paid = invoice?.status === 'paid';
  return {
    applied: paid,
    paymentUrl: paid ? null : (invoice?.hosted_invoice_url ?? null)
  };
}
