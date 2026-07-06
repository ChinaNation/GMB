import type { Env, LoginChallengeRow, SessionState } from '../types';
import { HttpError, jsonResponse, parsePositiveInt, readJson } from '../shared/http';
import { assertOwnerAccount, createId } from '../shared/ids';
import { nowMs, secondsFromNow } from '../shared/time';
import { buildLoginPayload, verifyWalletSignature } from './wallet_signature';

interface ChallengeRequest {
  owner_account?: unknown;
}

interface SessionRequest {
  challenge_id?: unknown;
  owner_account?: unknown;
  signature?: unknown;
}

export async function createLoginChallenge(request: Request, env: Env): Promise<Response> {
  const body = await readJson<ChallengeRequest>(request);
  let ownerAccount: string;
  try {
    ownerAccount = assertOwnerAccount(body.owner_account);
  } catch {
    throw new HttpError(400, 'invalid_owner_account', '钱包账户格式不合法');
  }

  const challengeId = createId('sqc');
  const expiresAt = secondsFromNow(300);
  const signingPayload = buildLoginPayload({
    owner_account: ownerAccount,
    challenge_id: challengeId,
    expires_at: expiresAt
  });

  await env.DB.prepare(
    `INSERT INTO square_login_challenges
      (challenge_id, owner_account, signing_payload, expires_at, used_at)
      VALUES (?, ?, ?, ?, NULL)`
  )
    .bind(challengeId, ownerAccount, signingPayload, expiresAt)
    .run();

  return jsonResponse({
    ok: true,
    challenge_id: challengeId,
    owner_account: ownerAccount,
    signing_payload: signingPayload,
    expires_at: expiresAt
  });
}

export async function createSession(request: Request, env: Env): Promise<Response> {
  const body = await readJson<SessionRequest>(request);
  if (typeof body.challenge_id !== 'string' || typeof body.signature !== 'string') {
    throw new HttpError(400, 'invalid_session_request', '登录请求缺少挑战或签名');
  }

  let ownerAccount: string;
  try {
    ownerAccount = assertOwnerAccount(body.owner_account);
  } catch {
    throw new HttpError(400, 'invalid_owner_account', '钱包账户格式不合法');
  }

  const challenge = await env.DB.prepare(
    `SELECT challenge_id, owner_account, signing_payload, expires_at, used_at
      FROM square_login_challenges
      WHERE challenge_id = ?`
  )
    .bind(body.challenge_id)
    .first<LoginChallengeRow>();

  if (!challenge || challenge.owner_account !== ownerAccount) {
    throw new HttpError(401, 'invalid_challenge', '钱包登录挑战不存在');
  }
  if (challenge.used_at !== null) {
    throw new HttpError(401, 'used_challenge', '钱包登录挑战已使用');
  }
  if (challenge.expires_at <= nowMs()) {
    throw new HttpError(401, 'expired_challenge', '钱包登录挑战已过期');
  }

  const isValid = await verifyWalletSignature(
    challenge.signing_payload,
    body.signature,
    ownerAccount
  );
  if (!isValid) {
    throw new HttpError(401, 'invalid_signature', '钱包签名校验失败');
  }

  const sessionTtlSeconds = parsePositiveInt(env.SQUARE_SESSION_TTL_SECONDS, 86_400);
  const sessionToken = createId('sqs');
  const session: SessionState = {
    owner_account: ownerAccount,
    created_at: nowMs(),
    expires_at: secondsFromNow(sessionTtlSeconds)
  };

  await env.DB.prepare(`UPDATE square_login_challenges SET used_at = ? WHERE challenge_id = ?`)
    .bind(nowMs(), challenge.challenge_id)
    .run();
  await env.FEED_CACHE.put(`square_session:${sessionToken}`, JSON.stringify(session), {
    expirationTtl: sessionTtlSeconds
  });

  return jsonResponse({
    ok: true,
    session_token: sessionToken,
    owner_account: ownerAccount,
    expires_at: session.expires_at
  });
}
