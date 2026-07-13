import { decodeAddress } from '@polkadot/util-crypto';
import type { Env } from '../types';
import { fetchChainIdentityState } from '../chain/identity';
import { HttpError, jsonResponse, readJson } from '../shared/http';
import { ownerPubkeyHex } from '../shared/ids';
import {
  consumeActionSignature,
  issueActionChallenge,
  releaseActionChallenge
} from '../account/action_challenge';
import {
  assertMembershipLevel,
  identityEligibleForPlan,
  membershipPlan,
  type MembershipLevel
} from './plans';

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
  return jsonResponse({
    ok: true,
    owner_account: ownerAccount,
    challenge_id: challenge.challengeId,
    op_tag: challenge.opTag,
    signing_payload_hex: challenge.signingPayloadHex,
    owner_pubkey_hex: ownerPubkeyHex(ownerAccount),
    membership_level: membershipLevel,
    expires_at: challenge.expiresAt
  });
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
    const session = await createStripeCheckoutSession(env, ownerAccount, membershipLevel);
    return jsonResponse({
      ok: true,
      checkout_session_id: session.id,
      checkout_url: session.url,
      membership_level: membershipLevel
    });
  } catch (error) {
    // 资格校验 / 建单失败：挑战未真正兑现，释放回未用，用户可原地重试而不必重签。
    await releaseActionChallenge(env, body.challenge_id);
    throw error;
  }
}

async function assertCheckoutEligibility(
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
  membershipLevel: MembershipLevel
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

function ownerAccountFromRequest(body: { owner_account?: unknown }): string {
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
