import type { Env, LoginChallengeRow } from '../types';
import { fetchChainIdentityStateByCid } from '../chain/identity';
import { verifyWalletSignature } from '../auth/wallet_signature';
import { HttpError, jsonResponse, readJson } from '../shared/http';
import { sha256Hex } from '../shared/hash';
import {
  assertAccountId,
  assertCidNumber,
  createId,
  signerPublicKeyHex,
} from '../shared/ids';
import { nowMs, secondsFromNow } from '../shared/time';
import { clearStaleIdentitySessions } from '../auth/session_index';
import { closeStaleChatRealtime } from '../chat/realtime';
import {
  OP_SIGN_SQUARE_ACTION,
  bytesToHex,
  concatBytes,
  hexToBytes,
  scaleString,
  signingMessage,
  u64Le,
} from '../shared/signing_message';

const TAKEOVER_ACTION = 'activate_cid_binding';
const TAKEOVER_CHALLENGE_TTL_SECONDS = 300;
const DATA_ROOT_BYTES = 32;
const AES_GCM_NONCE_BYTES = 12;

interface ChallengeRequest {
  cid_number?: unknown;
  account_id?: unknown;
}

interface TakeoverRequest extends ChallengeRequest {
  binding_revision?: unknown;
  challenge_id?: unknown;
  signature?: unknown;
}

interface CidDataRootRow {
  cid_number: string;
  sealed_data_root: string;
  seal_nonce: string;
  data_root_hash: string;
  active_binding_revision: number;
  active_account_id: string;
  created_at: number;
  updated_at: number;
}

interface ParsedBinding {
  cidNumber: string;
  accountId: string;
  bindingRevision: number;
}

/// 接管签名正文只包含新账户和 finalized 绑定事实；自主换绑所需的当前账户授权已经
/// 由 runtime 在换绑交易中验证，本接口不得复刻或替代链上换绑授权。
export function buildCidTakeoverScalePayload(input: {
  genesisHash: string;
  cidNumber: string;
  accountId: string;
  bindingRevision: number;
  challengeId: string;
  expiresAt: number;
}): Uint8Array {
  return concatBytes(
    scaleString(TAKEOVER_ACTION),
    scaleString(requireGenesisHash(input.genesisHash)),
    scaleString(assertCidNumber(input.cidNumber)),
    scaleString(assertAccountId(input.accountId)),
    u64Le(requireBindingRevision(input.bindingRevision)),
    scaleString(input.challengeId),
    u64Le(input.expiresAt),
  );
}

/// POST /v1/square/identity/takeover/challenge
///
/// 无旧会话、旧设备前提：直接读取 finalized CID 当前绑定，只有当前新账户能取得挑战。
export async function cidTakeoverChallengeRoute(
  request: Request,
  env: Env,
): Promise<Response> {
  const body = await readJson<ChallengeRequest>(request);
  const cidNumber = parseCidNumber(body.cid_number);
  const accountId = parseAccountId(body.account_id);
  const binding = await requireCurrentFinalizedBinding(env, cidNumber, accountId);
  const challengeId = createId('cidt');
  const expiresAt = secondsFromNow(TAKEOVER_CHALLENGE_TTL_SECONDS);
  const signingPayload = buildCidTakeoverScalePayload({
    genesisHash: requireGenesisHash(env.CHAIN_GENESIS_HASH),
    cidNumber,
    accountId,
    bindingRevision: binding.bindingRevision,
    challengeId,
    expiresAt,
  });
  const signingPayloadHex = bytesToHex(signingPayload);

  await env.DB.prepare(
    `INSERT INTO square_login_challenges
      (challenge_id, cid_number, binding_revision, account_id, signing_payload, expires_at, used_at)
      VALUES (?, ?, ?, ?, ?, ?, NULL)`,
  )
    .bind(
      challengeId,
      cidNumber,
      binding.bindingRevision,
      accountId,
      signingPayloadHex,
      expiresAt,
    )
    .run();

  return jsonResponse({
    ok: true,
    cid_number: cidNumber,
    account_id: accountId,
    binding_revision: binding.bindingRevision,
    challenge_id: challengeId,
    signing_payload_hex: signingPayloadHex,
    signer_public_key: signerPublicKeyHex(accountId),
    expires_at: expiresAt,
  });
}

/// POST /v1/square/identity/takeover
///
/// 当前 finalized 新账户签名成功后取得同一份 CID 稳定数据根。数据根由 CID 层密封，
/// 不读取任何此前绑定账户的私钥、公钥、签名或设备。
export async function cidTakeoverRoute(
  request: Request,
  env: Env,
): Promise<Response> {
  const body = await readJson<TakeoverRequest>(request);
  const parsed = parseTakeoverRequest(body);
  const bindingBefore = await requireCurrentFinalizedBinding(
    env,
    parsed.cidNumber,
    parsed.accountId,
    parsed.bindingRevision,
  );
  const challenge = await consumeTakeoverChallenge(env, parsed, body.signature);

  // 验签可能跨越新的 finalized 区块；发放数据根前必须再次读取链，旧版本请求不能穿透。
  const bindingAfter = await requireCurrentFinalizedBinding(
    env,
    parsed.cidNumber,
    parsed.accountId,
    parsed.bindingRevision,
  );
  if (
    bindingBefore.bindingRevision !== bindingAfter.bindingRevision
    || bindingBefore.accountId !== bindingAfter.accountId
  ) {
    throw new HttpError(409, 'cid_binding_changed', 'CID 绑定已变化，请重新发起接管');
  }

  try {
    const granted = await grantCidDataRoot(env, parsed);
    await revokeStaleBindingCredentials(env, parsed);
    return jsonResponse({
      ok: true,
      cid_number: parsed.cidNumber,
      account_id: parsed.accountId,
      binding_revision: parsed.bindingRevision,
      cid_data_root_base64: bytesToBase64(granted.dataRoot),
      data_root_hash: granted.dataRootHash,
    });
  } catch (error) {
    // 数据根密封或 D1 原子推进失败时允许同一挑战在原过期时间内重试；不会延长 TTL。
    await env.DB.prepare(
      `UPDATE square_login_challenges
        SET used_at = NULL
        WHERE challenge_id = ?`,
    )
      .bind(challenge.challenge_id)
      .run();
    throw error;
  }
}

async function consumeTakeoverChallenge(
  env: Env,
  parsed: ParsedBinding & { challengeId: string },
  rawSignature: unknown,
): Promise<LoginChallengeRow> {
  if (typeof rawSignature !== 'string' || rawSignature.length === 0) {
    throw new HttpError(400, 'invalid_takeover_request', '接管请求缺少新账户签名');
  }
  const challenge = await env.DB.prepare(
    `SELECT challenge_id, cid_number, binding_revision, account_id, signing_payload, expires_at, used_at
      FROM square_login_challenges
      WHERE challenge_id = ?`,
  )
    .bind(parsed.challengeId)
    .first<LoginChallengeRow>();
  if (
    !challenge
    || challenge.cid_number !== parsed.cidNumber
    || challenge.binding_revision !== parsed.bindingRevision
    || challenge.account_id !== parsed.accountId
  ) {
    throw new HttpError(401, 'invalid_challenge', '接管挑战不存在');
  }
  if (challenge.used_at !== null) {
    throw new HttpError(401, 'used_challenge', '接管挑战已使用');
  }
  const now = nowMs();
  if (challenge.expires_at <= now) {
    throw new HttpError(401, 'expired_challenge', '接管挑战已过期');
  }
  const expectedPayload = buildCidTakeoverScalePayload({
    genesisHash: requireGenesisHash(env.CHAIN_GENESIS_HASH),
    cidNumber: parsed.cidNumber,
    accountId: parsed.accountId,
    bindingRevision: parsed.bindingRevision,
    challengeId: challenge.challenge_id,
    expiresAt: challenge.expires_at,
  });
  if (bytesToHex(expectedPayload) !== challenge.signing_payload) {
    throw new HttpError(401, 'takeover_payload_mismatch', '接管挑战绑定版本或账户不匹配');
  }
  const valid = await verifyWalletSignature(
    signingMessage(OP_SIGN_SQUARE_ACTION, expectedPayload),
    rawSignature,
    parsed.accountId,
  );
  if (!valid) {
    throw new HttpError(401, 'invalid_signature', '新钱包账户签名校验失败');
  }
  const claimed = await env.DB.prepare(
    `UPDATE square_login_challenges
      SET used_at = ?
      WHERE challenge_id = ?
        AND cid_number = ?
        AND binding_revision = ?
        AND account_id = ?
        AND used_at IS NULL
        AND expires_at > ?`,
  )
    .bind(
      now,
      challenge.challenge_id,
      parsed.cidNumber,
      parsed.bindingRevision,
      parsed.accountId,
      now,
    )
    .run();
  if ((claimed.meta?.changes ?? 0) !== 1) {
    throw new HttpError(401, 'used_challenge', '接管挑战已使用');
  }
  return challenge;
}

/// finalized 新绑定完成后收敛所有可撤销凭证；不读取、联系或要求此前账户/设备。
///
/// CID 业务数据和稳定数据根不在删除范围。先删 KeyPackage 再删设备，防止任何旧 MLS
/// 入群材料继续可领取；WebSocket 按三元组精确关闭，不影响已经建立的新账户连接。
async function revokeStaleBindingCredentials(
  env: Env,
  binding: ParsedBinding,
): Promise<void> {
  await clearStaleIdentitySessions(
    env,
    binding.cidNumber,
    binding.bindingRevision,
    binding.accountId,
  );
  await env.DB.batch([
    env.DB.prepare(
      `DELETE FROM chat_keypackages
        WHERE cid_number = ?
          AND (binding_revision <> ? OR account_id <> ?)`,
    ).bind(binding.cidNumber, binding.bindingRevision, binding.accountId),
    env.DB.prepare(
      `DELETE FROM chat_devices
        WHERE cid_number = ?
          AND (binding_revision <> ? OR account_id <> ?)`,
    ).bind(binding.cidNumber, binding.bindingRevision, binding.accountId),
    env.DB.prepare(
      `DELETE FROM square_device_subkeys
        WHERE cid_number = ?
          AND (binding_revision <> ? OR account_id <> ?)`,
    ).bind(binding.cidNumber, binding.bindingRevision, binding.accountId),
  ]);
  await closeStaleChatRealtime(
    env,
    binding.cidNumber,
    binding.bindingRevision,
    binding.accountId,
  );
}

async function grantCidDataRoot(
  env: Env,
  binding: ParsedBinding,
): Promise<{ dataRoot: Uint8Array; dataRootHash: string }> {
  let row = await readCidDataRoot(env, binding.cidNumber);
  if (!row) {
    const dataRoot = crypto.getRandomValues(new Uint8Array(DATA_ROOT_BYTES));
    const dataRootHash = await sha256Hex(dataRoot);
    const sealed = await sealDataRoot(env, binding.cidNumber, dataRoot);
    const now = nowMs();
    await env.DB.prepare(
      `INSERT INTO cid_data_roots
        (cid_number, sealed_data_root, seal_nonce, data_root_hash,
         active_binding_revision, active_account_id, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(cid_number) DO NOTHING`,
    )
      .bind(
        binding.cidNumber,
        sealed.ciphertext,
        sealed.nonce,
        dataRootHash,
        binding.bindingRevision,
        binding.accountId,
        now,
        now,
      )
      .run();
    row = await readCidDataRoot(env, binding.cidNumber);
    if (!row) {
      throw new HttpError(503, 'cid_data_root_unavailable', 'CID 数据根创建失败');
    }
  }

  if (row.active_binding_revision > binding.bindingRevision) {
    throw new HttpError(409, 'stale_binding_revision', 'CID 接管版本已经推进');
  }
  const dataRoot = await unsealDataRoot(env, row);
  const dataRootHash = await sha256Hex(dataRoot);
  if (dataRootHash !== row.data_root_hash) {
    throw new HttpError(503, 'cid_data_root_corrupted', 'CID 数据根完整性校验失败');
  }

  const updated = await env.DB.prepare(
    `UPDATE cid_data_roots
      SET active_binding_revision = ?, active_account_id = ?, updated_at = ?
      WHERE cid_number = ? AND active_binding_revision <= ?`,
  )
    .bind(
      binding.bindingRevision,
      binding.accountId,
      nowMs(),
      binding.cidNumber,
      binding.bindingRevision,
    )
    .run();
  if ((updated.meta?.changes ?? 0) !== 1) {
    throw new HttpError(409, 'stale_binding_revision', 'CID 接管版本并发推进');
  }
  return { dataRoot, dataRootHash };
}

async function readCidDataRoot(
  env: Env,
  cidNumber: string,
): Promise<CidDataRootRow | null> {
  return env.DB.prepare(
    `SELECT cid_number, sealed_data_root, seal_nonce, data_root_hash,
      active_binding_revision, active_account_id, created_at, updated_at
      FROM cid_data_roots
      WHERE cid_number = ?`,
  )
    .bind(cidNumber)
    .first<CidDataRootRow>();
}

async function sealDataRoot(
  env: Env,
  cidNumber: string,
  dataRoot: Uint8Array,
): Promise<{ ciphertext: string; nonce: string }> {
  const nonce = crypto.getRandomValues(new Uint8Array(AES_GCM_NONCE_BYTES));
  const key = await importMasterKey(env);
  const encrypted = await crypto.subtle.encrypt(
    {
      name: 'AES-GCM',
      iv: asArrayBuffer(nonce),
      additionalData: asArrayBuffer(dataRootAad(env, cidNumber)),
      tagLength: 128,
    },
    key,
    asArrayBuffer(dataRoot),
  );
  return {
    ciphertext: bytesToBase64(new Uint8Array(encrypted)),
    nonce: bytesToBase64(nonce),
  };
}

async function unsealDataRoot(
  env: Env,
  row: CidDataRootRow,
): Promise<Uint8Array> {
  try {
    const plaintext = await crypto.subtle.decrypt(
      {
        name: 'AES-GCM',
        iv: asArrayBuffer(base64ToBytes(row.seal_nonce)),
        additionalData: asArrayBuffer(dataRootAad(env, row.cid_number)),
        tagLength: 128,
      },
      await importMasterKey(env),
      asArrayBuffer(base64ToBytes(row.sealed_data_root)),
    );
    const dataRoot = new Uint8Array(plaintext);
    if (dataRoot.length !== DATA_ROOT_BYTES) {
      throw new Error('invalid data root length');
    }
    return dataRoot;
  } catch {
    throw new HttpError(503, 'cid_data_root_corrupted', 'CID 数据根无法解封');
  }
}

async function importMasterKey(env: Env): Promise<CryptoKey> {
  const raw = env.CID_DATA_ROOT_MASTER_KEY;
  if (typeof raw !== 'string' || !/^0x[0-9a-f]{64}$/.test(raw)) {
    throw new HttpError(503, 'cid_data_root_key_unavailable', 'CID 数据根主密钥未配置');
  }
  return crypto.subtle.importKey(
    'raw',
    asArrayBuffer(hexToBytes(raw)),
    { name: 'AES-GCM' },
    false,
    ['encrypt', 'decrypt'],
  );
}

function dataRootAad(env: Env, cidNumber: string): Uint8Array {
  return new TextEncoder().encode(
    `${requireGenesisHash(env.CHAIN_GENESIS_HASH)}|${assertCidNumber(cidNumber)}`,
  );
}

async function requireCurrentFinalizedBinding(
  env: Env,
  cidNumber: string,
  accountId: string,
  expectedRevision?: number,
): Promise<ParsedBinding> {
  const identity = await fetchChainIdentityStateByCid(env, cidNumber);
  if (
    identity.cid_number !== cidNumber
    || identity.account_id !== accountId
    || identity.binding_revision <= 0
  ) {
    throw new HttpError(401, 'cid_binding_changed', '新账户不是 CID 当前绑定账户');
  }
  if (
    expectedRevision !== undefined
    && identity.binding_revision !== expectedRevision
  ) {
    throw new HttpError(409, 'binding_revision_changed', 'CID 绑定版本已经变化');
  }
  return {
    cidNumber,
    accountId,
    bindingRevision: identity.binding_revision,
  };
}

function parseTakeoverRequest(
  body: TakeoverRequest,
): ParsedBinding & { challengeId: string } {
  const bindingRevision = requireBindingRevision(body.binding_revision);
  if (typeof body.challenge_id !== 'string' || body.challenge_id.length === 0) {
    throw new HttpError(400, 'invalid_takeover_request', '接管请求缺少挑战');
  }
  return {
    cidNumber: parseCidNumber(body.cid_number),
    accountId: parseAccountId(body.account_id),
    bindingRevision,
    challengeId: body.challenge_id,
  };
}

function parseCidNumber(value: unknown): string {
  try {
    return assertCidNumber(value);
  } catch {
    throw new HttpError(400, 'invalid_cid_number', 'CID 格式不合法');
  }
}

function parseAccountId(value: unknown): string {
  try {
    return assertAccountId(value);
  } catch {
    throw new HttpError(400, 'invalid_account_id', '账户标识格式不合法');
  }
}

function requireBindingRevision(value: unknown): number {
  if (!Number.isSafeInteger(value) || (value as number) <= 0) {
    throw new HttpError(400, 'invalid_binding_revision', '绑定版本必须是正整数');
  }
  return value as number;
}

function requireGenesisHash(value: unknown): string {
  if (typeof value !== 'string' || !/^0x[0-9a-f]{64}$/.test(value)) {
    throw new HttpError(503, 'chain_genesis_hash_unavailable', '创世哈希未正确配置');
  }
  return value;
}

function bytesToBase64(bytes: Uint8Array): string {
  let binary = '';
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary);
}

function base64ToBytes(value: string): Uint8Array {
  if (!/^[A-Za-z0-9+/]+={0,2}$/.test(value)) {
    throw new Error('invalid base64');
  }
  const binary = atob(value);
  return Uint8Array.from(binary, (char) => char.charCodeAt(0));
}

function asArrayBuffer(bytes: Uint8Array): ArrayBuffer {
  return bytes.buffer.slice(
    bytes.byteOffset,
    bytes.byteOffset + bytes.byteLength,
  ) as ArrayBuffer;
}
