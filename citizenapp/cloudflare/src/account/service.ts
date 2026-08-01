import type { Env } from '../types';
import {
  HttpError,
  jsonResponse,
  readJson,
  requireSession
} from '../shared/http';
import { assertAccountId, signerPublicKeyHex } from '../shared/ids';
import { fetchChainIdentityStateByCid } from '../chain/identity';
import {
  consumeActionSignature,
  issueActionChallenge,
  releaseActionChallenge
} from './action_challenge';
import { purgeIdentity } from './purge';

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
    throw new HttpError(400, 'invalid_account_id', '账户标识格式不合法');
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
  const session = await requireSession(request, env);
  const body = await readJson<ChallengeRequest>(request);
  const accountId = parseAccountId(body.account_id);
  await requireCurrentCidBinding(env, session.cid_number, session.account_id, accountId);
  const challenge = await issueActionChallenge(
    env,
    session.cid_number,
    session.binding_revision,
    accountId,
    'delete_account'
  );
  return jsonResponse({
    ok: true,
    cid_number: session.cid_number,
    account_id: accountId,
    challenge_id: challenge.challengeId,
    op_tag: challenge.opTag,
    signing_payload_hex: challenge.signingPayloadHex,
    signer_public_key: signerPublicKeyHex(accountId),
    expires_at: challenge.expiresAt
  });
}

/// POST /v1/square/account/delete —— 当前绑定账户签名授权后，按会话 CID 硬删除该身份数据。
export async function deleteAccountRoute(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const body = await readJson<ActionConfirmRequest>(request);
  const parsed = parseConfirm(body);
  await requireCurrentCidBinding(
    env,
    session.cid_number,
    session.account_id,
    parsed.accountId
  );
  await consumeActionSignature(env, {
    cidNumber: session.cid_number,
    bindingRevision: session.binding_revision,
    accountId: parsed.accountId,
    action: 'delete_account',
    challengeId: parsed.challengeId,
    signature: parsed.signature
  });
  try {
    // 验签可能跨越一个区块最终化窗口；副作用开始前再次复核，禁止换绑竞态借此前授权删身份。
    await requireCurrentCidBinding(
      env,
      session.cid_number,
      session.account_id,
      parsed.accountId
    );
    // account_id 只完成当前绑定账户授权；唯一删除目标始终是会话 cid_number。
    const deleted = await purgeIdentity(
      env,
      session.cid_number,
      parsed.accountId
    );
    return jsonResponse({
      ok: true,
      cid_number: session.cid_number,
      authorization_account_id: parsed.accountId,
      deleted
    });
  } catch (error) {
    // purge 失败：释放挑战，用户可原地重试而不必重签（purge 幂等）。
    await releaseActionChallenge(env, parsed.challengeId);
    throw error;
  }
}

/// 注销属于动权操作，不能只采信 request_guard 的短缓存；挑战和确认两次都从同一
/// finalized 链身份真源复核 CID → 当前绑定账户，换绑竞态一律拒绝。
async function requireCurrentCidBinding(
  env: Env,
  cidNumber: string,
  sessionAccountId: string,
  requestedAccountId: string
): Promise<void> {
  if (requestedAccountId !== sessionAccountId) {
    throw new HttpError(403, 'delete_account_mismatch', '只能使用当前会话绑定账户注销身份');
  }
  const identity = await fetchChainIdentityStateByCid(env, cidNumber);
  if (
    identity.cid_number !== cidNumber ||
    identity.account_id !== requestedAccountId
  ) {
    throw new HttpError(401, 'cid_binding_changed', 'CID 当前绑定账户已变更，请重新登录');
  }
}
