import type { Env } from '../types';
import { HttpError, jsonResponse, readJson } from '../shared/http';
import { assertOwnerAccount, ownerPubkeyHex } from '../shared/ids';
import { nowMs } from '../shared/time';
import { getMembership, subscriptionIsActive } from '../membership/service';
import { cancelStripeSubscriptionAtPeriodEnd } from '../membership/stripe_api';
import {
  consumeActionSignature,
  issueActionChallenge,
  releaseActionChallenge
} from './action_challenge';
import { purgeAccount } from './purge';

interface ChallengeRequest {
  owner_account?: unknown;
}

interface ActionConfirmRequest {
  owner_account?: unknown;
  challenge_id?: unknown;
  signature?: unknown;
}

function parseOwner(value: unknown): string {
  try {
    return assertOwnerAccount(value);
  } catch {
    throw new HttpError(400, 'invalid_owner_account', '钱包账户格式不合法');
  }
}

function parseConfirm(body: ActionConfirmRequest): {
  ownerAccount: string;
  challengeId: string;
  signature: string;
} {
  const ownerAccount = parseOwner(body.owner_account);
  if (typeof body.challenge_id !== 'string' || typeof body.signature !== 'string') {
    throw new HttpError(400, 'invalid_action_request', '请求缺少挑战或签名');
  }
  return { ownerAccount, challengeId: body.challenge_id, signature: body.signature };
}

/// POST /v1/square/account/delete/challenge —— 下发注销签名挑战。
export async function deleteAccountChallengeRoute(request: Request, env: Env): Promise<Response> {
  const body = await readJson<ChallengeRequest>(request);
  const ownerAccount = parseOwner(body.owner_account);
  const challenge = await issueActionChallenge(env, ownerAccount, 'delete_account');
  return jsonResponse({
    ok: true,
    owner_account: ownerAccount,
    challenge_id: challenge.challengeId,
    op_tag: challenge.opTag,
    signing_payload_hex: challenge.signingPayloadHex,
    owner_pubkey_hex: ownerPubkeyHex(ownerAccount),
    expires_at: challenge.expiresAt
  });
}

/// POST /v1/square/account/delete —— 验钱包签名后硬删除该账户在 Cloudflare 的全部数据。
export async function deleteAccountRoute(request: Request, env: Env): Promise<Response> {
  const body = await readJson<ActionConfirmRequest>(request);
  const parsed = parseConfirm(body);
  await consumeActionSignature(env, {
    ownerAccount: parsed.ownerAccount,
    action: 'delete_account',
    challengeId: parsed.challengeId,
    signature: parsed.signature
  });
  try {
    const deleted = await purgeAccount(env, parsed.ownerAccount);
    return jsonResponse({ ok: true, owner_account: parsed.ownerAccount, deleted });
  } catch (error) {
    // purge 失败：释放挑战，用户可原地重试而不必重签（purge 幂等）。
    await releaseActionChallenge(env, parsed.challengeId);
    throw error;
  }
}

/// POST /v1/square/membership/cancel/challenge —— 下发取消订阅签名挑战。
export async function cancelMembershipChallengeRoute(request: Request, env: Env): Promise<Response> {
  const body = await readJson<ChallengeRequest>(request);
  const ownerAccount = parseOwner(body.owner_account);
  const challenge = await issueActionChallenge(env, ownerAccount, 'cancel_membership');
  return jsonResponse({
    ok: true,
    owner_account: ownerAccount,
    challenge_id: challenge.challengeId,
    op_tag: challenge.opTag,
    signing_payload_hex: challenge.signingPayloadHex,
    owner_pubkey_hex: ownerPubkeyHex(ownerAccount),
    expires_at: challenge.expiresAt
  });
}

/// POST /v1/square/membership/cancel —— 验钱包签名后按支付方式取消（ADR-034 段4）：
/// 卡（连续订阅）→ Stripe 到期取消（当期用完再终止）；USDC 预付无自动续、无扣款，
/// 本就到期自然失效，只回「到期失效」信息不动订阅。cancel_kind 供官网出对应文案。
export async function cancelMembershipRoute(request: Request, env: Env): Promise<Response> {
  const body = await readJson<ActionConfirmRequest>(request);
  const parsed = parseConfirm(body);
  await consumeActionSignature(env, {
    ownerAccount: parsed.ownerAccount,
    action: 'cancel_membership',
    challengeId: parsed.challengeId,
    signature: parsed.signature
  });

  try {
    const membership = await getMembership(env, parsed.ownerAccount);
    if (!membership || !subscriptionIsActive(membership)) {
      throw new HttpError(404, 'no_active_subscription', '没有可取消的订阅');
    }

    // USDC 预付：无自动续、无 stripe_subscription_id，到期日一到自然失效，无需真取消。
    if (membership.subscription_source === 'usdc_prepaid') {
      return jsonResponse({
        ok: true,
        owner_account: parsed.ownerAccount,
        cancel_kind: 'usdc_prepaid',
        expires_at: membership.expires_at
      });
    }

    // 卡（连续订阅）：Stripe 到期取消；最终失效以 subscription 事件回调为准。
    if (!membership.stripe_subscription_id) {
      throw new HttpError(404, 'no_active_subscription', '没有可取消的订阅');
    }
    await cancelStripeSubscriptionAtPeriodEnd(env, membership.stripe_subscription_id);
    await env.DB.prepare(
      `UPDATE square_memberships SET cancel_at_period_end = 1, updated_at = ? WHERE owner_account = ?`
    )
      .bind(nowMs(), parsed.ownerAccount)
      .run();

    return jsonResponse({
      ok: true,
      owner_account: parsed.ownerAccount,
      cancel_kind: 'stripe',
      expires_at: membership.expires_at
    });
  } catch (error) {
    // 无订阅 / Stripe 取消失败：释放挑战，用户可原地重试而不必重签。
    await releaseActionChallenge(env, parsed.challengeId);
    throw error;
  }
}
