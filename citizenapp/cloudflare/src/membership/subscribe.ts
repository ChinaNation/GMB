import { decodeAddress } from '@polkadot/util-crypto';
import type { Env, MembershipRow } from '../types';
import { fetchChainIdentityState } from '../chain/identity';
import { HttpError, jsonResponse, readJson } from '../shared/http';
import { ownerPubkeyHex } from '../shared/ids';
import { nowMs } from '../shared/time';
import {
  consumeActionSignature,
  issueActionChallenge,
  releaseActionChallenge
} from '../account/action_challenge';
import { getMembership } from './service';
import { changeStripeSubscriptionTier, resumeStripeSubscription } from './stripe_api';
import {
  assertMembershipLevel,
  identityEligibleForPlan,
  membershipPlan,
  type MembershipLevel
} from './plans';

/// Stripe 侧「仍存活、可换档」的订阅状态集合。已彻底取消 / 未完成过期的不算，
/// 视同无活跃订阅、走全新订阅。
const LIVE_SUBSCRIPTION_STATUSES = new Set([
  'active',
  'trialing',
  'past_due',
  'unpaid',
  'paused'
]);

function hasLiveSubscription(row: MembershipRow): boolean {
  return (
    !!row.stripe_subscription_id &&
    LIVE_SUBSCRIPTION_STATUSES.has(row.subscription_status)
  );
}

interface CheckoutRequestBody {
  owner_account?: unknown;
  membership_level?: unknown;
}

interface SubscribeConfirmBody {
  owner_account?: unknown;
  membership_level?: unknown;
  challenge_id?: unknown;
  signature?: unknown;
}

interface StripeCheckoutSession {
  id?: string;
  url?: string | null;
}

const stripeCheckoutUrl = 'https://api.stripe.com/v1/checkout/sessions';

/// POST /v1/square/membership/subscribe/challenge —— 下发订阅签名挑战（0x1D，
/// 会员等级绑进 payload）。官网无私钥，凭返回的 signing_payload_hex + owner_pubkey_hex
/// 构建 QR_V1 signRequest 给 CitizenApp 扫一扫签名。
export async function subscribeChallengeRoute(request: Request, env: Env): Promise<Response> {
  const body = await readJson<CheckoutRequestBody>(request);
  const ownerAccount = ownerAccountFromRequest(body);
  const membershipLevel = assertCheckoutMembershipLevel(body.membership_level);
  // 资格预检：不满足直接拒，避免出无效签名 QR。
  await assertCheckoutEligibility(env, ownerAccount, membershipLevel);
  const challenge = await issueActionChallenge(
    env,
    ownerAccount,
    'subscribe_membership',
    membershipLevel
  );
  // 当前订阅态 + 换档金额预览：供官网判定 新订阅 / 升档 / 降档 / 续订，并在签名前展示补/转金额。
  const existing = await getMembership(env, ownerAccount);
  const live = existing && hasLiveSubscription(existing) ? existing : null;
  const current = live
    ? {
        membership_level: live.membership_level,
        cancel_at_period_end: Boolean(live.cancel_at_period_end)
      }
    : null;
  const preview = live ? tierChangePreview(live, membershipLevel) : null;
  return jsonResponse({
    ok: true,
    owner_account: ownerAccount,
    challenge_id: challenge.challengeId,
    op_tag: challenge.opTag,
    signing_payload_hex: challenge.signingPayloadHex,
    owner_pubkey_hex: ownerPubkeyHex(ownerAccount),
    membership_level: membershipLevel,
    expires_at: challenge.expiresAt,
    current,
    preview
  });
}

/// 换档金额预览（本地按当期剩余周期比例估算，规则3）：升档=补差价、降档=转权益、
/// 同价=切换。仅供官网签名前展示；实际结算以 Stripe proration（按秒）为准，故为估算值。
function tierChangePreview(
  existing: MembershipRow,
  targetLevel: MembershipLevel
): { kind: 'upgrade' | 'downgrade' | 'switch'; amount_cents: number } | null {
  if (existing.membership_level === targetLevel) {
    return null;
  }
  const currentCents = membershipPlan(existing.membership_level).price_usd_cents;
  const targetCents = membershipPlan(targetLevel).price_usd_cents;
  const start = existing.current_period_start;
  const end = existing.current_period_end;
  const now = nowMs();
  const fraction =
    start && end && end > start
      ? Math.max(0, Math.min(1, (end - now) / (end - start)))
      : 1;
  const prorated = Math.round(fraction * (targetCents - currentCents));
  if (targetCents > currentCents) {
    return { kind: 'upgrade', amount_cents: prorated };
  }
  if (targetCents < currentCents) {
    return { kind: 'downgrade', amount_cents: Math.abs(prorated) };
  }
  return { kind: 'switch', amount_cents: 0 };
}

/// POST /v1/square/membership/subscribe —— 验签（0x1D，level 一致）后创建 Stripe checkout。
/// 旧无签名 Checkout 接口已退役；官网必须先取得钱包签名再创建订阅。
export async function subscribeConfirmRoute(request: Request, env: Env): Promise<Response> {
  const body = await readJson<SubscribeConfirmBody>(request);
  const ownerAccount = ownerAccountFromRequest(body);
  const membershipLevel = assertCheckoutMembershipLevel(body.membership_level);
  if (typeof body.challenge_id !== 'string' || typeof body.signature !== 'string') {
    throw new HttpError(400, 'invalid_action_request', '订阅请求缺少挑战或签名');
  }
  await consumeActionSignature(env, {
    ownerAccount,
    action: 'subscribe_membership',
    challengeId: body.challenge_id,
    signature: body.signature,
    context: membershipLevel
  });
  try {
    await assertCheckoutEligibility(env, ownerAccount, membershipLevel);
    // 规则1 一钱包一订阅：已有活跃订阅 → 在同一订阅上换档 / 续订 / 无操作，绝不新建第二个。
    const existing = await getMembership(env, ownerAccount);
    if (existing && hasLiveSubscription(existing)) {
      const change = await applyMembershipChange(env, existing, membershipLevel);
      return jsonResponse({ ok: true, membership_level: membershipLevel, ...change });
    }
    // 无活跃卡订阅：全新订阅。若有有效 USDC 预付（切换支付 USDC→卡），卡订阅试用期
    // 到 USDC 到期日，首次扣费顺延，已付 USDC 时长不浪费（ADR-034 段3）。
    const trialEndSeconds =
      existing &&
      existing.subscription_source === 'usdc_prepaid' &&
      existing.expires_at > nowMs()
        ? Math.floor(existing.expires_at / 1000)
        : undefined;
    const session = await createStripeCheckoutSession(
      env,
      ownerAccount,
      membershipLevel,
      trialEndSeconds
    );
    return jsonResponse({
      ok: true,
      checkout_session_id: session.id,
      checkout_url: session.url,
      membership_level: membershipLevel
    });
  } catch (error) {
    // 资格校验 / 建单 / 换档失败：挑战未真正兑现，释放回未用，用户可原地重试而不必重签。
    await releaseActionChallenge(env, body.challenge_id);
    throw error;
  }
}

/// 换档动作结果（返回给官网决定下一步：跳付款页 / 提示成功）。
interface MembershipChangeResult {
  action:
    | 'resumed'
    | 'already_subscribed'
    | 'upgraded'
    | 'upgrade_pending'
    | 'downgraded'
    | 'switched';
  payment_url?: string;
}

/// 在既有活跃订阅上按目标档换档（规则 2–4）。权益落库以 subscription webhook 为准，
/// 本函数只驱动 Stripe：同档→续订/无操作；升档→补差价（付成功才生效，否则给付款页）；
/// 降档/同价→即时生效（差额进信用余额）。
async function applyMembershipChange(
  env: Env,
  existing: MembershipRow,
  targetLevel: MembershipLevel
): Promise<MembershipChangeResult> {
  const subscriptionId = existing.stripe_subscription_id;
  if (!subscriptionId) {
    throw new HttpError(409, 'subscription_id_missing', '当前订阅缺少 Stripe 订阅号，无法换档');
  }

  // 同档：撤销待取消（续订）或无操作。
  if (existing.membership_level === targetLevel) {
    if (existing.cancel_at_period_end) {
      await resumeStripeSubscription(env, subscriptionId);
      return { action: 'resumed' };
    }
    return { action: 'already_subscribed' };
  }

  const currentCents = membershipPlan(existing.membership_level).price_usd_cents;
  const targetCents = membershipPlan(targetLevel).price_usd_cents;
  const isUpgrade = targetCents > currentCents;
  const newPriceId = priceIdForMembership(env, targetLevel);

  const result = await changeStripeSubscriptionTier(env, {
    subscriptionId,
    newPriceId,
    isUpgrade
  });

  if (isUpgrade) {
    // 升档：差价付成功即生效；否则返回付款页地址（USDC / 无可复用卡）。
    return result.applied
      ? { action: 'upgraded' }
      : { action: 'upgrade_pending', payment_url: result.paymentUrl ?? undefined };
  }
  // 降档（进信用余额）或同价换档（民主↔投票 $9.99）：即时生效。
  return { action: targetCents < currentCents ? 'downgraded' : 'switched' };
}

export async function assertCheckoutEligibility(
  env: Env,
  ownerAccount: string,
  membershipLevel: MembershipLevel
): Promise<void> {
  const plan = membershipPlan(membershipLevel);
  // 精确匹配：读链身份，必须恰好等于该会员所属身份档，禁止降档/越级。
  // 访客身份会员（freedom / democracy）也要确认账户确无 voting/candidate 身份。
  try {
    const identity = await fetchChainIdentityState(env, ownerAccount);
    if (!identityEligibleForPlan(identity.identity_level, plan)) {
      throw new HttpError(
        403,
        'membership_identity_mismatch',
        '当前身份不能订阅该会员（禁止降档或越级）'
      );
    }
  } catch (error) {
    if (error instanceof HttpError) {
      throw error;
    }
    throw new HttpError(503, 'membership_identity_check_failed', '链上身份读取失败，暂不能创建订阅');
  }
}

async function createStripeCheckoutSession(
  env: Env,
  ownerAccount: string,
  membershipLevel: MembershipLevel,
  trialEndSeconds?: number
): Promise<Required<StripeCheckoutSession>> {
  const priceId = priceIdForMembership(env, membershipLevel);
  const successUrl = env.CHECKOUT_SUCCESS_URL;
  const cancelUrl = env.CHECKOUT_CANCEL_URL;
  if (!successUrl || !cancelUrl) {
    throw new HttpError(503, 'stripe_checkout_urls_not_configured', 'Stripe Checkout 回跳地址未配置');
  }
  if (env.STRIPE_DEV_PROXY === '1') {
    return {
      id: `cs_dev_${membershipLevel}`,
      url: `${successUrl}${successUrl.includes('?') ? '&' : '?'}session_id=cs_dev_${membershipLevel}`
    };
  }
  const secretKey = env.STRIPE_API_KEY;
  if (!secretKey) {
    throw new HttpError(503, 'stripe_secret_not_configured', 'Stripe secret key 未配置');
  }

  const params = new URLSearchParams();
  params.set('mode', 'subscription');
  params.set('line_items[0][price]', priceId);
  params.set('line_items[0][quantity]', '1');
  params.set('client_reference_id', ownerAccount);
  params.set('success_url', successUrl);
  params.set('cancel_url', cancelUrl);
  // 权益真源只认 subscription webhook，所以 owner_account 必须写入订阅 metadata。
  params.set('subscription_data[metadata][owner_account]', ownerAccount);
  params.set('subscription_data[metadata][membership_level]', membershipLevel);
  params.set('metadata[owner_account]', ownerAccount);
  params.set('metadata[membership_level]', membershipLevel);
  // 切换支付 USDC→卡：试用期到 USDC 到期日，首次扣费顺延到那时，已付 USDC 时长不浪费。
  if (trialEndSeconds) {
    params.set('subscription_data[trial_end]', String(trialEndSeconds));
  }

  const response = await fetch(stripeCheckoutUrl, {
    method: 'POST',
    headers: {
      authorization: `Bearer ${secretKey}`,
      'content-type': 'application/x-www-form-urlencoded'
    },
    body: params
  });
  const data = (await response.json().catch(() => ({}))) as StripeCheckoutSession & {
    error?: { message?: string };
  };
  if (!response.ok) {
    throw new HttpError(
      502,
      'stripe_checkout_failed',
      data.error?.message ?? 'Stripe Checkout 创建失败'
    );
  }
  if (!data.id || !data.url) {
    throw new HttpError(502, 'stripe_checkout_invalid_response', 'Stripe Checkout 响应不完整');
  }
  return {
    id: data.id,
    url: data.url
  };
}

function priceIdForMembership(env: Env, membershipLevel: MembershipLevel): string {
  const priceId =
    membershipLevel === 'candidate'
      ? env.CANDIDATE_PRICE_ID
      : membershipLevel === 'voting'
        ? env.VOTING_PRICE_ID
        : membershipLevel === 'democracy'
          ? env.DEMOCRACY_PRICE_ID
          : env.FREEDOM_PRICE_ID;
  if (!priceId) {
    throw new HttpError(503, 'stripe_price_not_configured', 'Stripe 会员价格 ID 未配置');
  }
  return priceId;
}

export function ownerAccountFromRequest(body: { owner_account?: unknown }): string {
  if (typeof body.owner_account !== 'string' || body.owner_account.trim().length === 0) {
    throw new HttpError(400, 'owner_account_missing', '缺少钱包账户地址');
  }
  const ownerAccount = body.owner_account.trim();
  try {
    decodeAddress(ownerAccount);
  } catch {
    throw new HttpError(400, 'invalid_owner_account', '钱包账户地址不合法');
  }
  return ownerAccount;
}

export function assertCheckoutMembershipLevel(value: unknown): MembershipLevel {
  try {
    return assertMembershipLevel(value);
  } catch {
    throw new HttpError(400, 'invalid_membership_level', '会员等级不合法');
  }
}
