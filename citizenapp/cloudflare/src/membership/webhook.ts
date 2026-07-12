import type { Env } from '../types';
import { HttpError, jsonResponse, parsePositiveInt } from '../shared/http';
import { nowMs } from '../shared/time';
import { fetchChainIdentityState, type ChainIdentityState } from '../chain/identity';
import {
  assertMembershipLevel,
  identitySatisfies,
  membershipPlan,
  type MembershipPlan,
  type MembershipLevel
} from './plans';
import { markStripeMembershipInactive, upsertStripeMembership } from './service';
import { restoreOwnerVideos } from './archive';
import { readLimitedText } from '../limits/request';

interface StripeEvent {
  id: string;
  type: string;
  data: {
    object: Record<string, unknown>;
  };
}

interface StripeSubscriptionShape {
  id: string;
  customer: string | null;
  status: string;
  current_period_start: number | null;
  current_period_end: number;
  cancel_at_period_end: boolean;
  metadata: Record<string, string>;
  price_id: string | null;
  price_currency: string | null;
  price_unit_amount: number | null;
}

const activeStripeStatuses = new Set(['active', 'trialing']);

export async function stripeWebhookRoute(request: Request, env: Env): Promise<Response> {
  const secret = env.STRIPE_HOOK_SECRET;
  if (!secret) {
    throw new HttpError(503, 'stripe_webhook_not_configured', 'Stripe webhook secret 未配置');
  }
  const rawBody = await readLimitedText(request, 'stripe_webhook');
  const signature = request.headers.get('stripe-signature');
  const toleranceSeconds = parsePositiveInt(env.STRIPE_HOOK_WINDOW, 300);
  await verifyStripeSignature(rawBody, signature, secret, undefined, toleranceSeconds);

  const event = parseStripeEvent(rawBody);
  const result = await handleStripeEvent(env, event);
  return jsonResponse({
    ok: true,
    event_id: event.id,
    event_type: event.type,
    ...result
  });
}

export async function handleStripeEvent(
  env: Env,
  event: StripeEvent
): Promise<{ action: string; owner_account?: string; membership_level?: MembershipLevel }> {
  if (
    event.type === 'customer.subscription.created' ||
    event.type === 'customer.subscription.updated'
  ) {
    return processSubscription(env, event.data.object);
  }

  if (event.type === 'customer.subscription.deleted') {
    const subscription = normalizeSubscription(event.data.object);
    await markStripeMembershipInactive(env, subscription.id, subscription.status || 'canceled');
    return { action: 'subscription_inactivated' };
  }

  // Checkout 完成事件不直接授予权益；正式权益以 subscription 事件为准，避免误用 checkout 过期时间。
  if (event.type === 'checkout.session.completed') {
    return { action: 'checkout_session_observed' };
  }

  return { action: 'ignored' };
}

async function processSubscription(
  env: Env,
  raw: Record<string, unknown>
): Promise<{ action: string; owner_account: string; membership_level: MembershipLevel }> {
  const subscription = normalizeSubscription(raw);
  const ownerAccount = subscription.metadata.owner_account?.trim();
  if (!ownerAccount) {
    throw new HttpError(400, 'stripe_owner_account_missing', 'Stripe metadata 缺少 owner_account');
  }
  const membershipLevel = resolveMembershipLevel(env, subscription);
  const plan = membershipPlan(membershipLevel);
  assertSubscriptionUsesPlanPrice(subscription, plan);
  const identity = plan.required_identity_level === 'visitor'
    ? visitorIdentity(ownerAccount)
    : await fetchChainIdentityState(env, ownerAccount);
  const status = activeStripeStatuses.has(subscription.status) &&
    !identitySatisfies(identity.identity_level, plan.required_identity_level)
    ? 'identity_required'
    : subscription.status;

  await upsertStripeMembership(env, {
    ownerAccount,
    membershipLevel,
    stripeCustomerId: subscription.customer,
    stripeSubscriptionId: subscription.id,
    stripePriceId: subscription.price_id,
    subscriptionStatus: status,
    currentPeriodStart: subscription.current_period_start
      ? subscription.current_period_start * 1000
      : null,
    currentPeriodEnd: subscription.current_period_end * 1000,
    cancelAtPeriodEnd: subscription.cancel_at_period_end,
    identity
  });

  // 重订解冻：订阅重新生效 → 回灌该 owner 已归档的视频（幂等，失败不阻断权益落地）。
  if (status === 'active' || status === 'trialing') {
    try {
      await restoreOwnerVideos(env, ownerAccount);
    } catch (error) {
      console.error(
        `[video-archive] restore on resubscribe failed: ${error instanceof Error ? error.message : error}`
      );
    }
  }

  return {
    action: status === 'identity_required' ? 'identity_rejected' : 'subscription_upserted',
    owner_account: ownerAccount,
    membership_level: membershipLevel
  };
}

function visitorIdentity(ownerAccount: string): ChainIdentityState {
  return {
    owner_account: ownerAccount,
    identity_level: 'visitor',
    has_voting_identity: false,
    has_candidate_identity: false,
    cid_number: null,
    checked_at: nowMs()
  };
}

function normalizeSubscription(raw: Record<string, unknown>): StripeSubscriptionShape {
  const id = stringValue(raw.id);
  const status = stringValue(raw.status);
  const currentPeriodEnd = numberValue(raw.current_period_end);
  if (!id || !status || !currentPeriodEnd) {
    throw new HttpError(400, 'invalid_stripe_subscription', 'Stripe subscription 事件字段不完整');
  }
  return {
    id,
    customer: stripeId(raw.customer),
    status,
    current_period_start: numberValue(raw.current_period_start),
    current_period_end: currentPeriodEnd,
    cancel_at_period_end: raw.cancel_at_period_end === true,
    metadata: metadataValue(raw.metadata),
    ...priceValue(raw)
  };
}

function assertSubscriptionUsesPlanPrice(
  subscription: StripeSubscriptionShape,
  plan: MembershipPlan
): void {
  const currency = subscription.price_currency?.toLowerCase() ?? null;
  if (!currency || subscription.price_unit_amount === null) {
    throw new HttpError(400, 'stripe_price_missing', 'Stripe subscription 缺少价格币种或金额');
  }
  // 用户可以用本地法币或 USDC 支付；这里校验的是 Stripe Price 的会员业务计价真源。
  if (currency !== plan.price_currency) {
    throw new HttpError(400, 'stripe_price_currency_mismatch', 'Stripe 会员套餐必须使用 USD 计价');
  }
  if (subscription.price_unit_amount !== plan.price_usd_cents) {
    throw new HttpError(400, 'stripe_price_amount_mismatch', 'Stripe 会员套餐金额与会员等级不一致');
  }
}

function resolveMembershipLevel(env: Env, subscription: StripeSubscriptionShape): MembershipLevel {
  const metadataLevel = subscription.metadata.membership_level;
  if (metadataLevel) {
    try {
      return assertMembershipLevel(metadataLevel);
    } catch {
      throw new HttpError(400, 'invalid_membership_level', '会员等级 metadata 不合法');
    }
  }

  if (subscription.price_id && subscription.price_id === env.FREEDOM_PRICE_ID) {
    return 'freedom';
  }
  if (subscription.price_id && subscription.price_id === env.DEMOCRACY_PRICE_ID) {
    return 'democracy';
  }
  if (subscription.price_id && subscription.price_id === env.VOTING_PRICE_ID) {
    return 'voting';
  }
  if (subscription.price_id && subscription.price_id === env.CANDIDATE_PRICE_ID) {
    return 'candidate';
  }

  throw new HttpError(400, 'membership_level_missing', '无法从 Stripe 事件识别会员等级');
}

export async function verifyStripeSignature(
  rawBody: string,
  signatureHeader: string | null,
  secret: string,
  nowSeconds = Math.floor(nowMs() / 1000),
  toleranceSeconds = 300
): Promise<void> {
  if (!signatureHeader) {
    throw new HttpError(400, 'stripe_signature_missing', 'Stripe-Signature 缺失');
  }
  const parsed = parseSignatureHeader(signatureHeader);
  if (!parsed.timestamp || parsed.signatures.length === 0) {
    throw new HttpError(400, 'stripe_signature_invalid', 'Stripe-Signature 不合法');
  }
  if (Math.abs(nowSeconds - parsed.timestamp) > toleranceSeconds) {
    throw new HttpError(400, 'stripe_signature_expired', 'Stripe-Signature 已过期');
  }

  const signedPayload = `${parsed.timestamp}.${rawBody}`;
  const expected = await hmacSha256Hex(secret, signedPayload);
  if (!parsed.signatures.some((candidate) => timingSafeEqualHex(candidate, expected))) {
    throw new HttpError(400, 'stripe_signature_mismatch', 'Stripe-Signature 校验失败');
  }
}

function parseStripeEvent(rawBody: string): StripeEvent {
  try {
    const event = JSON.parse(rawBody) as StripeEvent;
    if (!event.id || !event.type || !event.data?.object) {
      throw new Error('missing fields');
    }
    return event;
  } catch {
    throw new HttpError(400, 'invalid_stripe_event', 'Stripe webhook JSON 不合法');
  }
}

function parseSignatureHeader(header: string): { timestamp: number | null; signatures: string[] } {
  let timestamp: number | null = null;
  const signatures: string[] = [];
  for (const part of header.split(',')) {
    const [key, value] = part.split('=', 2);
    if (key === 't') {
      const parsed = Number.parseInt(value ?? '', 10);
      timestamp = Number.isFinite(parsed) ? parsed : null;
    }
    if (key === 'v1' && value) {
      signatures.push(value);
    }
  }
  return { timestamp, signatures };
}

async function hmacSha256Hex(secret: string, payload: string): Promise<string> {
  const encoder = new TextEncoder();
  const key = await crypto.subtle.importKey(
    'raw',
    encoder.encode(secret),
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign']
  );
  const signature = await crypto.subtle.sign('HMAC', key, encoder.encode(payload));
  return [...new Uint8Array(signature)].map((byte) => byte.toString(16).padStart(2, '0')).join('');
}

function timingSafeEqualHex(a: string, b: string): boolean {
  if (!/^[a-f0-9]+$/i.test(a) || !/^[a-f0-9]+$/i.test(b) || a.length !== b.length) {
    return false;
  }
  let diff = 0;
  for (let index = 0; index < a.length; index += 1) {
    diff |= a.charCodeAt(index) ^ b.charCodeAt(index);
  }
  return diff === 0;
}

function metadataValue(value: unknown): Record<string, string> {
  if (!value || typeof value !== 'object') return {};
  const result: Record<string, string> = {};
  for (const [key, raw] of Object.entries(value as Record<string, unknown>)) {
    if (typeof raw === 'string') {
      result[key] = raw;
    }
  }
  return result;
}

function priceValue(raw: Record<string, unknown>): {
  price_id: string | null;
  price_currency: string | null;
  price_unit_amount: number | null;
} {
  const items = raw.items;
  if (!items || typeof items !== 'object') {
    return { price_id: null, price_currency: null, price_unit_amount: null };
  }
  const data = (items as { data?: unknown }).data;
  if (!Array.isArray(data) || data.length === 0) {
    return { price_id: null, price_currency: null, price_unit_amount: null };
  }
  const first = data[0];
  if (!first || typeof first !== 'object') {
    return { price_id: null, price_currency: null, price_unit_amount: null };
  }
  const price = (first as { price?: unknown }).price;
  if (!price || typeof price !== 'object') {
    return { price_id: null, price_currency: null, price_unit_amount: null };
  }
  const priceObject = price as { id?: unknown; currency?: unknown; unit_amount?: unknown };
  return {
    price_id: stringValue(priceObject.id) || null,
    price_currency: stringValue(priceObject.currency) || null,
    price_unit_amount: numberValue(priceObject.unit_amount)
  };
}

function stripeId(value: unknown): string | null {
  if (typeof value === 'string') return value;
  if (value && typeof value === 'object') {
    return stringValue((value as { id?: unknown }).id);
  }
  return null;
}

function stringValue(value: unknown): string {
  return typeof value === 'string' ? value : '';
}

function numberValue(value: unknown): number | null {
  return typeof value === 'number' && Number.isFinite(value) ? value : null;
}
