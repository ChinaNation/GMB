import type { Env } from '../types';
import { HttpError, jsonResponse, readJson } from '../shared/http';
import { assertAccountId, signerPublicKeyHex } from '../shared/ids';
import {
  consumeActionSignature,
  issueActionChallenge,
  releaseActionChallenge
} from './action_challenge';
import { purgeAccount } from './purge';

interface ChallengeRequest {
  account_id?: unknown;
}

interface ActionConfirmRequest {
  account_id?: unknown;
  challenge_id?: unknown;
  signature?: unknown;
}

function parseAccountId(value: unknown): string {
  try {
    return assertAccountId(value);
  } catch {
    throw new HttpError(400, 'invalid_account_id', '钱包账户格式不合法');
  }
}

function parseConfirm(body: ActionConfirmRequest): {
  accountId: string;
  challengeId: string;
  signature: string;
} {
  const accountId = parseAccountId(body.account_id);
  if (typeof body.challenge_id !== 'string' || typeof body.signature !== 'string') {
    throw new HttpError(400, 'invalid_action_request', '请求缺少挑战或签名');
  }
  return { accountId, challengeId: body.challenge_id, signature: body.signature };
}

/// POST /v1/square/account/delete/challenge —— 下发注销签名挑战。
export async function deleteAccountChallengeRoute(request: Request, env: Env): Promise<Response> {
  const body = await readJson<ChallengeRequest>(request);
  const accountId = parseAccountId(body.account_id);
  const challenge = await issueActionChallenge(env, accountId, 'delete_account');
  return jsonResponse({
    ok: true,
    account_id: accountId,
    challenge_id: challenge.challengeId,
    op_tag: challenge.opTag,
    signing_payload_hex: challenge.signingPayloadHex,
    signer_public_key: signerPublicKeyHex(accountId),
    expires_at: challenge.expiresAt
  });
}

/// POST /v1/square/account/delete —— 验钱包签名后硬删除该账户在 Cloudflare 的全部数据。
export async function deleteAccountRoute(request: Request, env: Env): Promise<Response> {
  const body = await readJson<ActionConfirmRequest>(request);
  const parsed = parseConfirm(body);
  await consumeActionSignature(env, {
    accountId: parsed.accountId,
    action: 'delete_account',
    challengeId: parsed.challengeId,
    signature: parsed.signature
  });
  try {
    const deleted = await purgeAccount(env, parsed.accountId);
    return jsonResponse({ ok: true, account_id: parsed.accountId, deleted });
  } catch (error) {
    // purge 失败：释放挑战，用户可原地重试而不必重签（purge 幂等）。
    await releaseActionChallenge(env, parsed.challengeId);
    throw error;
  }
}
