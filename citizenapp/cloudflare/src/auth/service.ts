import type { Env, LoginChallengeRow, SessionState } from '../types';
import { HttpError, jsonResponse, parsePositiveInt, readJson } from '../shared/http';
import { assertAccountId, createId } from '../shared/ids';
import { nowMs, secondsFromNow } from '../shared/time';
import { putKvJson } from '../limits/storage';
import { sha256Hex } from '../shared/hash';
import { verifyTurnstile } from '../security/turnstile';
import { verifyWalletSignature } from './wallet_signature';
import { indexSessionToken } from './session_index';
import { fetchChainIdentityStateCached } from '../chain/identity';
import {
  assertP256PublicKeyHex,
  buildDeviceBindingSigningMessage,
  DEVICE_SKEW_MS,
  normalizeP256SignatureHex,
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
  account_id?: unknown;
}

interface SessionRequest {
  challenge_id?: unknown;
  account_id?: unknown;
  signature?: unknown;
}

interface DeviceRegisterRequest {
  account_id?: unknown;
  p256_public_key?: unknown;
  issued_at?: unknown;
  binding_signature?: unknown;
  turnstile_token?: unknown;
}

/// 登录挑战的 SCALE payload：`account_id ‖ challenge_id ‖ expires_at`。
/// 被签消息 = signing_message(OP_SIGN_SQUARE_LOGIN, payload)，由客户端重算摘要后
/// 用 P-256 设备子钥签名。worker 单侧编码 payload，客户端只 hash+sign，杜绝字段漂移。
function buildLoginScalePayload(
  accountId: string,
  challengeId: string,
  expiresAt: number
): Uint8Array {
  return concatBytes(
    scaleString(accountId),
    scaleString(challengeId),
    u64Le(expiresAt)
  );
}

export async function createLoginChallenge(request: Request, env: Env): Promise<Response> {
  const body = await readJson<ChallengeRequest>(request);
  let accountId: string;
  try {
    accountId = assertAccountId(body.account_id);
  } catch {
    throw new HttpError(400, 'invalid_account_id', '账户标识格式不合法');
  }

  const challengeId = createId('sqc');
  const expiresAt = secondsFromNow(300);
  const signingPayloadHex = bytesToHex(
    buildLoginScalePayload(accountId, challengeId, expiresAt)
  );

  await env.DB.prepare(
    `INSERT INTO square_login_challenges
      (challenge_id, account_id, signing_payload, expires_at, used_at)
      VALUES (?, ?, ?, ?, NULL)`
  )
    .bind(challengeId, accountId, signingPayloadHex, expiresAt)
    .run();

  return jsonResponse({
    ok: true,
    challenge_id: challengeId,
    account_id: accountId,
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

  let accountId: string;
  try {
    accountId = assertAccountId(body.account_id);
  } catch {
    throw new HttpError(400, 'invalid_account_id', '账户标识格式不合法');
  }

  const challenge = await env.DB.prepare(
    `SELECT challenge_id, account_id, signing_payload, expires_at, used_at
      FROM square_login_challenges
      WHERE challenge_id = ?`
  )
    .bind(body.challenge_id)
    .first<LoginChallengeRow>();

  if (!challenge || challenge.account_id !== accountId) {
    throw new HttpError(401, 'invalid_challenge', '钱包登录挑战不存在');
  }
  if (challenge.used_at !== null) {
    throw new HttpError(401, 'used_challenge', '钱包登录挑战已使用');
  }
  if (challenge.expires_at <= nowMs()) {
    throw new HttpError(401, 'expired_challenge', '钱包登录挑战已过期');
  }

  // 身份主键 = 该钱包账户链上当前绑定的 cid_number;未绑定 → 不能建会话(访客)。
  const identity = await fetchChainIdentityStateCached(env, accountId);
  if (!identity.cid_number) {
    throw new HttpError(403, 'cid_not_bound', '该钱包账户未绑定 CID,无法登录');
  }
  const cidNumber = identity.cid_number;

  // 后台握手用 P-256 设备子钥（硬件、静默）验签 signing_message(OP_SIGN_SQUARE_LOGIN)。
  // 子钥挂在 (cid_number, account_id) 下(同一身份可多设备);逐个验签,任一匹配即通过。
  const subkeys = await env.DB.prepare(
    `SELECT p256_public_key FROM square_device_subkeys WHERE cid_number = ? AND account_id = ?`
  )
    .bind(cidNumber, accountId)
    .all<{ p256_public_key: string }>();
  if (!subkeys.results || subkeys.results.length === 0) {
    throw new HttpError(401, 'device_not_registered', '设备子钥未注册，请先注册设备子钥');
  }
  const loginMessage = signingMessage(
    OP_SIGN_SQUARE_LOGIN,
    hexToBytes(challenge.signing_payload)
  );
  // 跨端签名文本须为 `0x`+128hex（ADR-041）；规范化为裸后交内部裸函数验签，
  // 裸/大写/错长与验签失败一律按既有 401 语义处理（不泄漏格式细节）。
  const signatureBare = normalizeP256SignatureHex(body.signature);
  let matchedP256: string | null = null;
  if (signatureBare !== null) {
    for (const row of subkeys.results) {
      if (await verifyP256Signature(loginMessage, signatureBare, row.p256_public_key)) {
        matchedP256 = row.p256_public_key;
        break;
      }
    }
  }
  if (!matchedP256) {
    throw new HttpError(401, 'invalid_signature', '设备子钥签名校验失败');
  }

  // 中文注释：签名通过后用条件 UPDATE 原子占用挑战。并发请求即使都在上方读到
  // used_at=NULL，也只有一个能把 changes 改成 1；挑战一经占用便不释放。
  const claimedAt = nowMs();
  const claimed = await env.DB.prepare(
    `UPDATE square_login_challenges
      SET used_at = ?
      WHERE challenge_id = ?
        AND account_id = ?
        AND used_at IS NULL
        AND expires_at > ?`
  )
    .bind(claimedAt, challenge.challenge_id, accountId, claimedAt)
    .run();
  if ((claimed.meta?.changes ?? 0) !== 1) {
    if (challenge.expires_at <= claimedAt) {
      throw new HttpError(401, 'expired_challenge', '钱包登录挑战已过期');
    }
    throw new HttpError(401, 'used_challenge', '钱包登录挑战已使用');
  }

  // Session 只证明当前设备控制已登记的钱包子钥。链账户是否存在、余额和公民资格
  // 必须由具体业务动作自行校验，不能阻塞会员页和端到端加密数据同步。
  const sessionTtlSeconds = parsePositiveInt(env.SESSION_TTL_SECONDS, 86_400);
  const sessionToken = createId('sqs');
  const session: SessionState = {
    cid_number: cidNumber,
    account_id: accountId,
    device_key_hash: await sha256Hex(matchedP256),
    created_at: nowMs(),
    expires_at: secondsFromNow(sessionTtlSeconds)
  };

  const sessionKey = `square_session:${sessionToken}`;
  try {
    await putKvJson(env, sessionKey, session, 'session_cache', {
      expirationTtl: sessionTtlSeconds
    });
    // 记入「账户→token」索引，使注销可定向失效该账户全部会话（零残留）。
    await indexSessionToken(env, accountId, sessionToken, sessionTtlSeconds);
  } catch (error) {
    // 中文注释：KV/索引失败时烧毁挑战但删除可能已写入的孤立 Session，客户端只能
    // 重新申请挑战，禁止恢复旧挑战造成并发重放窗口。
    await env.SQUARE_CACHE.delete(sessionKey).catch(() => undefined);
    throw error;
  }

  return jsonResponse({
    ok: true,
    session_token: sessionToken,
    cid_number: cidNumber,
    account_id: accountId,
    expires_at: session.expires_at
  });
}

/// 注册 P-256 设备子钥：客户端用 sr25519 主钥对
/// `signing_message(OP_SIGN_SQUARE_DEVICE_BIND, account_id ‖ p256_public_key ‖ issued_at)`
/// 签名做绑定证明；后端复用 sr25519 验签确认子钥归属，再由链上绑定解析出身份主键
/// cid_number，落库主键 (cid_number, device_id)：同一身份可多设备并存（各一行），
/// 同设备（同 P-256 公钥）重注册按 issued_at 单调覆盖 = 轮换/续期。子钥属生成它的
/// 钱包 account_id；换绑后旧账户子钥由每请求链上绑定复查自然失效。此后登录挑战改由
/// 该子钥静默签名。
export async function registerDeviceSubkey(request: Request, env: Env): Promise<Response> {
  const body = await readJson<DeviceRegisterRequest>(request);
  await verifyTurnstile(request, env, body.turnstile_token);
  let accountId: string;
  try {
    accountId = assertAccountId(body.account_id);
  } catch {
    throw new HttpError(400, 'invalid_account_id', '账户标识格式不合法');
  }
  const p256PublicKey = assertP256PublicKeyHex(body.p256_public_key);
  const now = nowMs();
  if (
    typeof body.issued_at !== 'number' ||
    !Number.isSafeInteger(body.issued_at) ||
    Math.abs(now - body.issued_at) > DEVICE_SKEW_MS
  ) {
    throw new HttpError(400, 'invalid_issued_at', '设备绑定时间戳不合法');
  }
  if (typeof body.binding_signature !== 'string') {
    throw new HttpError(400, 'invalid_binding', '设备绑定签名缺失');
  }

  const bindingMessage = buildDeviceBindingSigningMessage({
    account_id: accountId,
    p256_public_key: p256PublicKey,
    issued_at: body.issued_at
  });
  const isValid = await verifyWalletSignature(
    bindingMessage,
    body.binding_signature,
    accountId
  );
  if (!isValid) {
    throw new HttpError(401, 'invalid_binding_signature', '设备绑定签名校验失败');
  }

  // 身份主键 = 该钱包账户链上当前绑定的 cid_number(占即绑,匿名亦可)。
  // 未绑定 CID 的账户是访客,不能注册社交身份的设备子钥。
  const identity = await fetchChainIdentityStateCached(env, accountId);
  if (!identity.cid_number) {
    throw new HttpError(403, 'cid_not_bound', '该钱包账户未绑定 CID,无法注册设备子钥');
  }
  const cidNumber = identity.cid_number;
  // device_id = 该设备 P-256 公钥的 sha256:同一身份多设备各一行;换机(新公钥)=新设备。
  const deviceId = await sha256Hex(p256PublicKey);

  const updated = await env.DB.prepare(
    `INSERT INTO square_device_subkeys
      (cid_number, device_id, account_id, p256_public_key, issued_at, created_at, updated_at)
      VALUES (?, ?, ?, ?, ?, ?, ?)
      ON CONFLICT(cid_number, device_id) DO UPDATE SET
        account_id = excluded.account_id,
        p256_public_key = excluded.p256_public_key,
        issued_at = excluded.issued_at,
        updated_at = excluded.updated_at
      WHERE excluded.issued_at > square_device_subkeys.issued_at`
  )
    .bind(cidNumber, deviceId, accountId, p256PublicKey, body.issued_at, now, now)
    .run();
  if ((updated.meta?.changes ?? 0) !== 1) {
    throw new HttpError(409, 'stale_device_binding', '设备绑定证明已使用或早于当前绑定');
  }

  return jsonResponse({ ok: true, cid_number: cidNumber });
}
