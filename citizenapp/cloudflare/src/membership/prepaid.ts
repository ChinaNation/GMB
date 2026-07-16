import type { Env } from '../types';
import { HttpError, jsonResponse, readJson } from '../shared/http';
import { ownerPubkeyHex } from '../shared/ids';
import { nowMs } from '../shared/time';
import {
  consumeActionSignature,
  issueActionChallenge,
  releaseActionChallenge
} from '../account/action_challenge';
import { applyPrepaidTierChange, getMembership } from './service';
import { assertCheckoutMembershipLevel, ownerAccountFromRequest } from './subscribe';
import { membershipPlan, type MembershipLevel } from './plans';

const DAY_MS = 86_400_000;
/// 升档补差价的日费年基：按实际日历年 365 天折算（ADR-034 段2；闰年差可忽略）。
const ACTUAL_YEAR_DAYS = 365;

/// 剩余实际日历天数（向下取整，不为负）。
function remainingDays(expiresAt: number, now: number): number {
  return Math.max(0, Math.floor((expiresAt - now) / DAY_MS));
}

/// 降档折算天数 = 剩余天数 × 旧月费 ÷ 新月费（比值，与"每月几天"无关，按实际剩余天数）。
function downgradeDays(remaining: number, oldMonthlyCents: number, newMonthlyCents: number): number {
  if (newMonthlyCents <= 0) {
    return remaining;
  }
  return Math.round((remaining * oldMonthlyCents) / newMonthlyCents);
}

/// 升档补差价（分）= 剩余天数 ×（新−旧）月费 × 12 ÷ 365（实际年折日费）。
function upgradeDiffCents(remaining: number, oldMonthlyCents: number, newMonthlyCents: number): number {
  return Math.max(
    0,
    Math.round((remaining * (newMonthlyCents - oldMonthlyCents) * 12) / ACTUAL_YEAR_DAYS)
  );
}

/// 换档签名 context：绑定目标档，防"签一档换另一档"。
function changeContext(targetLevel: MembershipLevel): string {
  return `change|${targetLevel}`;
}

const stripeCheckoutUrl = 'https://api.stripe.com/v1/checkout/sessions';

/// USDC 预付档期（ADR-034）：季=3 月、年=12 月。加密钱包无法 off-session 自动扣款，
/// 故按固定时长一次性购买、不自动续、不设取消。
export type PrepaidDuration = 'quarter' | 'year';

export function monthsForPrepaidDuration(duration: PrepaidDuration): number {
  return duration === 'year' ? 12 : 3;
}

function assertPrepaidDuration(value: unknown): PrepaidDuration {
  if (value === 'quarter' || value === 'year') {
    return value;
  }
  throw new HttpError(400, 'invalid_prepaid_duration', '预付时长不合法（季 / 年）');
}

/// 签名 context 绑定「档位|时长」，防「签一档换另一档 / 换时长」重放。
function prepaidContext(level: MembershipLevel, duration: PrepaidDuration): string {
  return `${level}|${duration}`;
}

/// 购买/续费守卫（ADR-034 段4）：已有活跃 USDC 且选了**异档** → 拒，引导走换档入口。
/// 否则 upsertPrepaidMembership 会把旧档剩余时长贴成新档（用户占便宜、平台漏收），换档
/// 必须走 /prepaid/change 的补钱/补时长折算。新购、同档续费、跨支付切换（原为卡）均放行。
async function assertPrepaidPurchaseTier(
  env: Env,
  ownerAccount: string,
  membershipLevel: MembershipLevel
): Promise<void> {
  const existing = await getMembership(env, ownerAccount);
  if (
    existing &&
    existing.subscription_source === 'usdc_prepaid' &&
    existing.expires_at > nowMs() &&
    existing.membership_level !== membershipLevel
  ) {
    throw new HttpError(
      409,
      'prepaid_tier_change_required',
      '已有其它档 USDC 会员，换档请走换档入口'
    );
  }
}

interface PrepaidRequestBody {
  owner_account?: unknown;
  membership_level?: unknown;
  duration?: unknown;
}

interface PrepaidConfirmBody extends PrepaidRequestBody {
  challenge_id?: unknown;
  signature?: unknown;
}

/// POST /v1/square/membership/prepaid/challenge —— 下发 USDC 预付签名挑战
/// （复用 subscribe_membership 动作，context=level|duration，不改 0x1D 布局）。
export async function prepaidChallengeRoute(request: Request, env: Env): Promise<Response> {
  const body = await readJson<PrepaidRequestBody>(request);
  const ownerAccount = ownerAccountFromRequest(body);
  const membershipLevel = assertCheckoutMembershipLevel(body.membership_level);
  const duration = assertPrepaidDuration(body.duration);
  // 会员与身份解耦（ADR-036）：无身份资格预检。异档 USDC 仍需走换档入口，签名前先挡。
  await assertPrepaidPurchaseTier(env, ownerAccount, membershipLevel);
  const challenge = await issueActionChallenge(
    env,
    ownerAccount,
    'subscribe_membership',
    prepaidContext(membershipLevel, duration)
  );
  return jsonResponse({
    ok: true,
    owner_account: ownerAccount,
    challenge_id: challenge.challengeId,
    op_tag: challenge.opTag,
    signing_payload_hex: challenge.signingPayloadHex,
    owner_pubkey_hex: ownerPubkeyHex(ownerAccount),
    membership_level: membershipLevel,
    duration,
    months: monthsForPrepaidDuration(duration),
    expires_at: challenge.expiresAt
  });
}

/// POST /v1/square/membership/prepaid —— 验签后建一次性 Checkout（mode=payment）收 N 月费；
/// 授权益走 webhook checkout.session.completed（metadata.route=usdc_prepaid）。
export async function prepaidConfirmRoute(request: Request, env: Env): Promise<Response> {
  const body = await readJson<PrepaidConfirmBody>(request);
  const ownerAccount = ownerAccountFromRequest(body);
  const membershipLevel = assertCheckoutMembershipLevel(body.membership_level);
  const duration = assertPrepaidDuration(body.duration);
  if (typeof body.challenge_id !== 'string' || typeof body.signature !== 'string') {
    throw new HttpError(400, 'invalid_action_request', '预付请求缺少挑战或签名');
  }
  await consumeActionSignature(env, {
    ownerAccount,
    action: 'subscribe_membership',
    challengeId: body.challenge_id,
    signature: body.signature,
    context: prepaidContext(membershipLevel, duration)
  });
  try {
    // 防御性再挡异档（挑战与确认之间状态可能变化）；抛错走 catch 释放挑战。
    await assertPrepaidPurchaseTier(env, ownerAccount, membershipLevel);
    const session = await createPrepaidCheckoutSession(
      env,
      ownerAccount,
      membershipLevel,
      duration
    );
    return jsonResponse({
      ok: true,
      checkout_session_id: session.id,
      checkout_url: session.url,
      membership_level: membershipLevel,
      duration
    });
  } catch (error) {
    // 资格 / 建单失败：挑战未真正兑现，释放回未用，可原地重试不必重签。
    await releaseActionChallenge(env, body.challenge_id);
    throw error;
  }
}

/// POST /v1/square/membership/prepaid/change/challenge —— USDC 换档签名挑战
/// （context=change|targetLevel）。仅对有效 usdc_prepaid 会员。
export async function prepaidChangeChallengeRoute(request: Request, env: Env): Promise<Response> {
  const body = await readJson<PrepaidRequestBody>(request);
  const ownerAccount = ownerAccountFromRequest(body);
  const targetLevel = assertCheckoutMembershipLevel(body.membership_level);
  const existing = await requireActivePrepaid(env, ownerAccount);
  if (existing.membership_level === targetLevel) {
    throw new HttpError(409, 'same_membership_level', '已是该会员档，无需换档');
  }
  const challenge = await issueActionChallenge(
    env,
    ownerAccount,
    'subscribe_membership',
    changeContext(targetLevel)
  );
  return jsonResponse({
    ok: true,
    owner_account: ownerAccount,
    challenge_id: challenge.challengeId,
    op_tag: challenge.opTag,
    signing_payload_hex: challenge.signingPayloadHex,
    owner_pubkey_hex: ownerPubkeyHex(ownerAccount),
    membership_level: targetLevel,
    preview: prepaidChangePreview(existing.expires_at, existing.membership_level, targetLevel),
    expires_at: challenge.expiresAt
  });
}

/// POST /v1/square/membership/prepaid/change —— 验签后换档：降档 / 同价即时切档 + 折算到期；
/// 升档建差价 Checkout（付成功由 webhook usdc_prepaid_upgrade 分支切档）。
export async function prepaidChangeConfirmRoute(request: Request, env: Env): Promise<Response> {
  const body = await readJson<PrepaidConfirmBody>(request);
  const ownerAccount = ownerAccountFromRequest(body);
  const targetLevel = assertCheckoutMembershipLevel(body.membership_level);
  if (typeof body.challenge_id !== 'string' || typeof body.signature !== 'string') {
    throw new HttpError(400, 'invalid_action_request', '换档请求缺少挑战或签名');
  }
  await consumeActionSignature(env, {
    ownerAccount,
    action: 'subscribe_membership',
    challengeId: body.challenge_id,
    signature: body.signature,
    context: changeContext(targetLevel)
  });
  try {
    const existing = await requireActivePrepaid(env, ownerAccount);
    const now = nowMs();
    const remaining = remainingDays(existing.expires_at, now);
    const oldCents = membershipPlan(existing.membership_level).price_usd_cents;
    const newCents = membershipPlan(targetLevel).price_usd_cents;

    if (newCents > oldCents) {
      // 升档：补差价。剩余为 0（差价 0）直接切；否则建一次性 Checkout，付成功由 webhook 切档。
      const diff = upgradeDiffCents(remaining, oldCents, newCents);
      if (diff <= 0) {
        await applyPrepaidTierChange(env, {
          ownerAccount,
          membershipLevel: targetLevel,
          expiresAt: existing.expires_at
        });
        return jsonResponse({ ok: true, action: 'upgraded', membership_level: targetLevel });
      }
      const session = await createPrepaidUpgradeCheckout(env, ownerAccount, targetLevel, diff);
      return jsonResponse({
        ok: true,
        action: 'upgrade_pending',
        checkout_url: session.url,
        amount_cents: diff,
        membership_level: targetLevel
      });
    }

    // 降档 / 同价：本地折算 → 即时切档 + 新到期（从现在起算）。
    const newDays = downgradeDays(remaining, oldCents, newCents);
    const newExpires = now + newDays * DAY_MS;
    await applyPrepaidTierChange(env, {
      ownerAccount,
      membershipLevel: targetLevel,
      expiresAt: newExpires
    });
    return jsonResponse({
      ok: true,
      action: newCents < oldCents ? 'downgraded' : 'switched',
      membership_level: targetLevel,
      expires_at: newExpires
    });
  } catch (error) {
    await releaseActionChallenge(env, body.challenge_id);
    throw error;
  }
}

async function requireActivePrepaid(env: Env, ownerAccount: string) {
  const existing = await getMembership(env, ownerAccount);
  if (
    !existing ||
    existing.subscription_source !== 'usdc_prepaid' ||
    existing.expires_at <= nowMs()
  ) {
    throw new HttpError(409, 'no_active_prepaid', '没有可换档的有效 USDC 预付会员');
  }
  return existing;
}

function prepaidChangePreview(
  expiresAt: number,
  currentLevel: string,
  targetLevel: MembershipLevel
): { kind: 'upgrade' | 'downgrade' | 'switch'; amount_cents?: number; new_days?: number } {
  const remaining = remainingDays(expiresAt, nowMs());
  const oldCents = membershipPlan(currentLevel).price_usd_cents;
  const newCents = membershipPlan(targetLevel).price_usd_cents;
  if (newCents > oldCents) {
    return { kind: 'upgrade', amount_cents: upgradeDiffCents(remaining, oldCents, newCents) };
  }
  if (newCents < oldCents) {
    return { kind: 'downgrade', new_days: downgradeDays(remaining, oldCents, newCents) };
  }
  return { kind: 'switch', new_days: remaining };
}

/// 通用一次性 Checkout（mode=payment）：金额 + 商品名 + metadata（同步写 checkout 与
/// payment_intent，webhook 两处都可读）；dev 短路返回合成 url。
async function createOneTimeCheckout(
  env: Env,
  params: {
    ownerAccount: string;
    amountCents: number;
    productName: string;
    metadata: Record<string, string>;
    devId: string;
  }
): Promise<{ id: string; url: string }> {
  const successUrl = env.CHECKOUT_SUCCESS_URL;
  const cancelUrl = env.CHECKOUT_CANCEL_URL;
  if (!successUrl || !cancelUrl) {
    throw new HttpError(503, 'stripe_checkout_urls_not_configured', 'Stripe Checkout 回跳地址未配置');
  }
  if (env.STRIPE_DEV_PROXY === '1') {
    return {
      id: params.devId,
      url: `${successUrl}${successUrl.includes('?') ? '&' : '?'}session_id=${params.devId}`
    };
  }
  const secretKey = env.STRIPE_API_KEY;
  if (!secretKey) {
    throw new HttpError(503, 'stripe_secret_not_configured', 'Stripe secret key 未配置');
  }

  const form = new URLSearchParams();
  form.set('mode', 'payment');
  // 强制 Stripe Crypto：USDC 入口不得回落到银行卡或其它动态支付方式。
  form.set('payment_method_types[0]', 'crypto');
  form.set('line_items[0][price_data][currency]', 'usd');
  form.set('line_items[0][price_data][unit_amount]', String(params.amountCents));
  form.set('line_items[0][price_data][product_data][name]', params.productName);
  form.set('line_items[0][quantity]', '1');
  form.set('client_reference_id', params.ownerAccount);
  form.set('success_url', successUrl);
  form.set('cancel_url', cancelUrl);
  for (const [key, value] of Object.entries(params.metadata)) {
    form.set(`metadata[${key}]`, value);
    form.set(`payment_intent_data[metadata][${key}]`, value);
  }

  const startedAt = Date.now();
  const response = await fetch(stripeCheckoutUrl, {
    method: 'POST',
    headers: {
      authorization: `Bearer ${secretKey}`,
      'content-type': 'application/x-www-form-urlencoded'
    },
    body: form
  });
  const elapsedMs = Date.now() - startedAt;
  // 读原始文本再解析：不再把 Stripe 的真实错误吞成通用文案（含非 JSON 网关响应）。
  const rawBody = await response.text();
  let data: { id?: string; url?: string | null; error?: { message?: string } } = {};
  try {
    data = JSON.parse(rawBody) as typeof data;
  } catch {
    // 非 JSON 响应：保留原始片段，交由下方带状态回报。
  }
  if (!response.ok) {
    // 出口诊断（仅失败时）：打印 CF-Ray/耗时/响应头，供追出口 POP 与 TLS 失败链路。
    const cfRay = response.headers.get('cf-ray');
    console.error(
      '[stripe-egress-fail]',
      JSON.stringify({
        endpoint: 'prepaid_checkout',
        status: response.status,
        ms: elapsedMs,
        cf_ray: cfRay,
        server: response.headers.get('server'),
        cf_cache: response.headers.get('cf-cache-status'),
        body: rawBody.slice(0, 200)
      })
    );
    throw new HttpError(
      502,
      'stripe_checkout_failed',
      data.error?.message ??
        `Stripe ${response.status} cf-ray=${cfRay ?? '无'} ${elapsedMs}ms：${rawBody.slice(0, 120) || '(空响应体)'}`
    );
  }
  if (!data.id || !data.url) {
    throw new HttpError(502, 'stripe_checkout_invalid_response', 'Stripe Checkout 响应不完整');
  }
  return { id: data.id, url: data.url };
}

/// 预付购买 Checkout：金额 = 月数 × 月费（无折扣）。
async function createPrepaidCheckoutSession(
  env: Env,
  ownerAccount: string,
  level: MembershipLevel,
  duration: PrepaidDuration
): Promise<{ id: string; url: string }> {
  const plan = membershipPlan(level);
  return createOneTimeCheckout(env, {
    ownerAccount,
    amountCents: plan.price_usd_cents * monthsForPrepaidDuration(duration),
    productName: `${plan.display_name} · ${duration === 'year' ? '年付' : '季付'}`,
    metadata: {
      route: 'usdc_prepaid',
      owner_account: ownerAccount,
      membership_level: level,
      duration
    },
    devId: `cs_prepaid_${level}_${duration}`
  });
}

/// 升档补差价 Checkout：只收差价（分）；付成功由 webhook usdc_prepaid_upgrade 分支切档。
async function createPrepaidUpgradeCheckout(
  env: Env,
  ownerAccount: string,
  targetLevel: MembershipLevel,
  diffCents: number
): Promise<{ id: string; url: string }> {
  const plan = membershipPlan(targetLevel);
  return createOneTimeCheckout(env, {
    ownerAccount,
    amountCents: diffCents,
    productName: `${plan.display_name} · 升档补差价`,
    metadata: {
      route: 'usdc_prepaid_upgrade',
      owner_account: ownerAccount,
      membership_level: targetLevel
    },
    devId: `cs_prepaid_upgrade_${targetLevel}`
  });
}
