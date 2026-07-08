import { decodeAddress } from '@polkadot/util-crypto';
import type { Env } from '../types';
import { fetchChainIdentityState } from '../chain/identity';
import { HttpError, jsonResponse, readJson, requireSession } from '../shared/http';
import {
  assertMembershipLevel,
  identitySatisfies,
  membershipPlan,
  type MembershipLevel
} from './plans';
import { assertSessionOwner } from './service';

interface CheckoutRequestBody {
  owner_account?: unknown;
  membership_level?: unknown;
}

interface StripeCheckoutSession {
  id?: string;
  url?: string | null;
}

const stripeCheckoutUrl = 'https://api.stripe.com/v1/checkout/sessions';

export async function stripeCheckoutRoute(request: Request, env: Env): Promise<Response> {
  const body = await readJson<CheckoutRequestBody>(request);
  const ownerAccount = ownerAccountFromRequest(body);
  const membershipLevel = assertCheckoutMembershipLevel(body.membership_level);

  // 官网订阅允许用户输入钱包账户；App 或后续钱包登录态调用时必须和 session owner 一致。
  const authorization = request.headers.get('authorization');
  if (authorization?.startsWith('Bearer ')) {
    const session = await requireSession(request, env);
    assertSessionOwner(session, ownerAccount);
  }

  await assertCheckoutEligibility(env, ownerAccount, membershipLevel);
  const session = await createStripeCheckoutSession(env, ownerAccount, membershipLevel);

  return jsonResponse({
    ok: true,
    checkout_session_id: session.id,
    checkout_url: session.url,
    membership_level: membershipLevel
  });
}

async function assertCheckoutEligibility(
  env: Env,
  ownerAccount: string,
  membershipLevel: MembershipLevel
): Promise<void> {
  const plan = membershipPlan(membershipLevel);
  if (plan.required_identity_level === 'visitor') {
    return;
  }

  try {
    const identity = await fetchChainIdentityState(env, ownerAccount);
    if (!identitySatisfies(identity.identity_level, plan.required_identity_level)) {
      throw new HttpError(403, 'membership_identity_required', '当前链上身份不满足该会员等级');
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
  membershipLevel: MembershipLevel
): Promise<Required<StripeCheckoutSession>> {
  const priceId = priceIdForMembership(env, membershipLevel);
  const successUrl = env.CITIZENAPP_MEMBERSHIP_SUCCESS_URL;
  const cancelUrl = env.CITIZENAPP_MEMBERSHIP_CANCEL_URL;
  if (!successUrl || !cancelUrl) {
    throw new HttpError(503, 'stripe_checkout_urls_not_configured', 'Stripe Checkout 回跳地址未配置');
  }
  if (env.STRIPE_DEV_CHECKOUT_PROXY === '1') {
    return {
      id: `cs_dev_${membershipLevel}`,
      url: `${successUrl}${successUrl.includes('?') ? '&' : '?'}session_id=cs_dev_${membershipLevel}`
    };
  }
  const secretKey = env.STRIPE_SECRET_KEY;
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
      ? env.STRIPE_PRICE_CANDIDATE
      : membershipLevel === 'voting'
        ? env.STRIPE_PRICE_VOTING
        : env.STRIPE_PRICE_VISITOR;
  if (!priceId) {
    throw new HttpError(503, 'stripe_price_not_configured', 'Stripe 会员价格 ID 未配置');
  }
  return priceId;
}

function ownerAccountFromRequest(body: CheckoutRequestBody): string {
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

function assertCheckoutMembershipLevel(value: unknown): MembershipLevel {
  try {
    return assertMembershipLevel(value);
  } catch {
    throw new HttpError(400, 'invalid_membership_level', '会员等级不合法');
  }
}
