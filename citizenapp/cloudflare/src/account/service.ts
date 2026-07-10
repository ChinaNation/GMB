import type { Env } from '../types';
import { HttpError, jsonResponse, readJson } from '../shared/http';
import { assertOwnerAccount, ownerPubkeyHex } from '../shared/ids';
import { nowMs } from '../shared/time';
import { getMembership } from '../membership/service';
import { cancelStripeSubscriptionAtPeriodEnd } from '../membership/stripe_api';
import { consumeActionSignature, issueActionChallenge } from './action_challenge';
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
  const deleted = await purgeAccount(env, parsed.ownerAccount);
  return jsonResponse({ ok: true, owner_account: parsed.ownerAccount, deleted });
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

/// POST /v1/square/membership/cancel —— 验钱包签名后到期取消订阅（当期用完再终止）。
export async function cancelMembershipRoute(request: Request, env: Env): Promise<Response> {
  const body = await readJson<ActionConfirmRequest>(request);
  const parsed = parseConfirm(body);
  await consumeActionSignature(env, {
    ownerAccount: parsed.ownerAccount,
    action: 'cancel_membership',
    challengeId: parsed.challengeId,
    signature: parsed.signature
  });

  const membership = await getMembership(env, parsed.ownerAccount);
  if (!membership?.stripe_subscription_id) {
    throw new HttpError(404, 'no_active_subscription', '没有可取消的订阅');
  }
  await cancelStripeSubscriptionAtPeriodEnd(env, membership.stripe_subscription_id);
  // 本地即时反映「到期取消」；最终失效以 Stripe subscription 事件回调为准。
  await env.DB.prepare(
    `UPDATE square_memberships SET cancel_at_period_end = 1, updated_at = ? WHERE owner_account = ?`
  )
    .bind(nowMs(), parsed.ownerAccount)
    .run();

  return jsonResponse({ ok: true, owner_account: parsed.ownerAccount });
}
