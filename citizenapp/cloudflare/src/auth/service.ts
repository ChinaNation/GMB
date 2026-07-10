import type { Env, LoginChallengeRow, SessionState } from '../types';
import { HttpError, jsonResponse, parsePositiveInt, readJson } from '../shared/http';
import { assertOwnerAccount, createId } from '../shared/ids';
import { nowMs, secondsFromNow } from '../shared/time';
import { verifyWalletSignature } from './wallet_signature';
import { indexSessionToken } from './session_index';
import {
  assertP256PublicKeyHex,
  buildDeviceBindingSigningMessage,
  verifyP256Signature
} from './device_subkey';
import {
  OP_SIGN_SQUARE_LOGIN,
  bytesToHex,
  concatBytes,
  hexToBytes,
  scaleString,
  signingMessage,
  u64Le
} from '../shared/signing_message';

interface ChallengeRequest {
  owner_account?: unknown;
}

interface SessionRequest {
  challenge_id?: unknown;
  owner_account?: unknown;
  signature?: unknown;
}

interface DeviceRegisterRequest {
  owner_account?: unknown;
  p256_pubkey?: unknown;
  issued_at?: unknown;
  binding_signature?: unknown;
}

/// 登录挑战的 SCALE payload：`owner ‖ challenge_id ‖ expires_at`。
/// 被签消息 = signing_message(OP_SIGN_SQUARE_LOGIN, payload)，由客户端重算摘要后
/// 用 P-256 设备子钥签名。worker 单侧编码 payload，客户端只 hash+sign，杜绝字段漂移。
function buildLoginScalePayload(
  ownerAccount: string,
  challengeId: string,
  expiresAt: number
): Uint8Array {
  return concatBytes(
    scaleString(ownerAccount),
    scaleString(challengeId),
    u64Le(expiresAt)
  );
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
  const signingPayloadHex = bytesToHex(
    buildLoginScalePayload(ownerAccount, challengeId, expiresAt)
  );

  await env.DB.prepare(
    `INSERT INTO square_login_challenges
      (challenge_id, owner_account, signing_payload, expires_at, used_at)
      VALUES (?, ?, ?, ?, NULL)`
  )
    .bind(challengeId, ownerAccount, signingPayloadHex, expiresAt)
    .run();

  return jsonResponse({
    ok: true,
    challenge_id: challengeId,
    owner_account: ownerAccount,
    op_tag: OP_SIGN_SQUARE_LOGIN,
    signing_payload_hex: signingPayloadHex,
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

  // 后台握手用 P-256 设备子钥（硬件、静默）验签 signing_message(OP_SIGN_SQUARE_LOGIN)。
  const subkey = await env.DB.prepare(
    `SELECT p256_pubkey FROM square_device_subkeys WHERE owner_account = ?`
  )
    .bind(ownerAccount)
    .first<{ p256_pubkey: string }>();
  if (!subkey) {
    throw new HttpError(401, 'device_not_registered', '设备子钥未注册，请先注册设备子钥');
  }
  const loginMessage = signingMessage(
    OP_SIGN_SQUARE_LOGIN,
    hexToBytes(challenge.signing_payload)
  );
  const isValid = await verifyP256Signature(
    loginMessage,
    body.signature,
    subkey.p256_pubkey
  );
  if (!isValid) {
    throw new HttpError(401, 'invalid_signature', '设备子钥签名校验失败');
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
  // 记入「账户→token」索引，使注销可定向失效该账户全部会话（零残留）。
  await indexSessionToken(env, ownerAccount, sessionToken, sessionTtlSeconds);

  return jsonResponse({
    ok: true,
    session_token: sessionToken,
    owner_account: ownerAccount,
    expires_at: session.expires_at
  });
}

/// 注册 P-256 设备子钥：客户端用 sr25519 主钥对
/// `signing_message(OP_SIGN_SQUARE_DEVICE_BIND, owner ‖ p256_pubkey ‖ issued_at)`
/// 签名做绑定证明；后端复用 sr25519 验签确认子钥归属，落库（一账户一活跃子钥，
/// 重注册覆盖 = 换机/轮换）。此后登录挑战改由该子钥静默签名。
export async function registerDeviceSubkey(request: Request, env: Env): Promise<Response> {
  const body = await readJson<DeviceRegisterRequest>(request);
  let ownerAccount: string;
  try {
    ownerAccount = assertOwnerAccount(body.owner_account);
  } catch {
    throw new HttpError(400, 'invalid_owner_account', '钱包账户格式不合法');
  }
  const p256Pubkey = assertP256PublicKeyHex(body.p256_pubkey);
  if (typeof body.issued_at !== 'number' || !Number.isFinite(body.issued_at)) {
    throw new HttpError(400, 'invalid_issued_at', '设备绑定时间戳不合法');
  }
  if (typeof body.binding_signature !== 'string') {
    throw new HttpError(400, 'invalid_binding', '设备绑定签名缺失');
  }

  const bindingMessage = buildDeviceBindingSigningMessage({
    owner_account: ownerAccount,
    p256_pubkey: p256Pubkey,
    issued_at: body.issued_at
  });
  const isValid = await verifyWalletSignature(
    bindingMessage,
    body.binding_signature,
    ownerAccount
  );
  if (!isValid) {
    throw new HttpError(401, 'invalid_binding_signature', '设备绑定签名校验失败');
  }

  const now = nowMs();
  await env.DB.prepare(
    `INSERT INTO square_device_subkeys
      (owner_account, p256_pubkey, issued_at, created_at, updated_at)
      VALUES (?, ?, ?, ?, ?)
      ON CONFLICT(owner_account) DO UPDATE SET
        p256_pubkey = excluded.p256_pubkey,
        issued_at = excluded.issued_at,
        updated_at = excluded.updated_at`
  )
    .bind(ownerAccount, p256Pubkey, body.issued_at, now, now)
    .run();

  return jsonResponse({ ok: true, owner_account: ownerAccount });
}
