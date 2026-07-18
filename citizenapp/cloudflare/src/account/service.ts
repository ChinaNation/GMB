import type { Env } from '../types';
import { HttpError, jsonResponse, readJson } from '../shared/http';
import { assertOwnerAccount, ownerPubkeyHex } from '../shared/ids';
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
