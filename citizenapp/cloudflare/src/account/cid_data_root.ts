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
const TAKEOVER_CHALLENGE_PREFIX = 'cidt_';
const TAKEOVER_CHALLENGE_TTL_SECONDS = 300;
const DATA_ROOT_BYTES = 32;
const AES_GCM_NONCE_BYTES = 12;
const RECOVERY_KEY_VERSION = 1;
const RECOVERY_AT_REST_DOMAIN = 'citizenapp.cid-data-root/recovery-at-rest';
const RECOVERY_GRANT_DOMAIN = 'citizenapp.cid-data-root/recovery-grant';

interface ChallengeRequest {
  cid_number?: unknown;
  account_id?: unknown;
  recovery_public_key?: unknown;
}

interface TakeoverRequest extends ChallengeRequest {
  binding_revision?: unknown;
  challenge_id?: unknown;
  signature?: unknown;
}

interface CidDataRootRow {
  cid_number: string;
  recovery_ciphertext: string;
  recovery_nonce: string;
  recovery_key_version: number;
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
  recoveryPublicKey: string;
}

interface RecoveryEnvelope {
  senderPublicKey: string;
  nonce: string;
  ciphertext: string;
}

/// 接管签名只证明 finalized 当前账户授权本次临时接收公钥取得 CID 数据根。
/// 自主换绑所需的旧账户授权已经由 runtime 验证；注册局换绑则由注册局权限验证。
/// 本接口不读取、不要求或替代任何此前账户签名。
export function buildCidTakeoverScalePayload(input: {
  genesisHash: string;
  cidNumber: string;
  accountId: string;
  bindingRevision: number;
  recoveryPublicKey: string;
  challengeId: string;
  expiresAt: number;
}): Uint8Array {
  return concatBytes(
    scaleString(TAKEOVER_ACTION),
    scaleString(requireGenesisHash(input.genesisHash)),
    scaleString(assertCidNumber(input.cidNumber)),
    scaleString(assertAccountId(input.accountId)),
    u64Le(requireBindingRevision(input.bindingRevision)),
    scaleString(requireRecoveryPublicKey(input.recoveryPublicKey)),
    scaleString(input.challengeId),
    u64Le(input.expiresAt),
  );
}

/// POST /v1/square/identity/takeover/challenge
///
/// 新设备尚无会话，因此直接读取 finalized CID 当前绑定。挑战把临时 X25519 接收公钥
/// 纳入签名载荷，攻击者不能在确认阶段替换数据根接收者。
export async function cidTakeoverChallengeRoute(
  request: Request,
  env: Env,
): Promise<Response> {
  const body = await readJson<ChallengeRequest>(request);
  const cidNumber = parseCidNumber(body.cid_number);
  const accountId = parseAccountId(body.account_id);
  const recoveryPublicKey = requireRecoveryPublicKey(body.recovery_public_key);
  const binding = await requireCurrentFinalizedBinding(env, cidNumber, accountId);
  const challengeId = createId('cidt');
  const expiresAt = secondsFromNow(TAKEOVER_CHALLENGE_TTL_SECONDS);
  const genesisHash = requireGenesisHash(env.CHAIN_GENESIS_HASH);
  const signingPayload = buildCidTakeoverScalePayload({
    genesisHash,
    cidNumber,
    accountId,
    bindingRevision: binding.bindingRevision,
    recoveryPublicKey,
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
    chain_genesis_hash: genesisHash,
    cid_number: cidNumber,
    account_id: accountId,
    binding_revision: binding.bindingRevision,
    recovery_public_key: recoveryPublicKey,
    challenge_id: challengeId,
    signing_payload_hex: signingPayloadHex,
    signer_public_key: signerPublicKeyHex(accountId),
    expires_at: expiresAt,
  });
}

/// POST /v1/square/identity/takeover
///
/// finalized 当前账户签名成功后，恢复层解封该 CID 的稳定数据根，再用本次 X25519
/// 会话钥加密返回。JSON、D1 和日志都不出现明文数据根。
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

  // 验签可能跨越新的 finalized 区块；发放前再次读链，旧 revision 请求不能穿透。
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

  let dataRoot: Uint8Array | null = null;
  try {
    const granted = await grantCidDataRoot(env, parsed);
    dataRoot = granted.dataRoot;
    const envelope = await encryptDataRootForRecipient(
      env,
      parsed,
      challenge.challenge_id,
      granted.dataRoot,
      granted.dataRootHash,
    );
    return jsonResponse({
      ok: true,
      chain_genesis_hash: requireGenesisHash(env.CHAIN_GENESIS_HASH),
      cid_number: parsed.cidNumber,
      account_id: parsed.accountId,
      binding_revision: parsed.bindingRevision,
      recovery_recipient_public_key: parsed.recoveryPublicKey,
      recovery_sender_public_key: envelope.senderPublicKey,
      recovery_nonce_base64: envelope.nonce,
      encrypted_cid_data_root_base64: envelope.ciphertext,
      data_root_hash: granted.dataRootHash,
    });
  } catch (error) {
    // 数据根密封或加密信封生成失败时允许同一挑战在原 TTL 内重试，不延长有效期。
    try {
      await env.DB.prepare(
        `UPDATE square_login_challenges SET used_at = NULL WHERE challenge_id = ?`,
      )
        .bind(challenge.challenge_id)
        .run();
    } catch (releaseError) {
      console.error(JSON.stringify({
        message: 'CID 数据根挑战释放失败',
        challenge_id: challenge.challenge_id,
        error: releaseError instanceof Error ? releaseError.message : String(releaseError),
      }));
    }
    throw error;
  } finally {
    dataRoot?.fill(0);
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
  if (!parsed.challengeId.startsWith(TAKEOVER_CHALLENGE_PREFIX)) {
    throw new HttpError(401, 'invalid_challenge', '挑战类型不属于 CID 数据根接管');
  }
  const challenge = await env.DB.prepare(
    `SELECT challenge_id, cid_number, binding_revision, account_id, signing_payload, expires_at, used_at
      FROM square_login_challenges WHERE challenge_id = ?`,
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
    recoveryPublicKey: parsed.recoveryPublicKey,
    challengeId: challenge.challenge_id,
    expiresAt: challenge.expires_at,
  });
  if (bytesToHex(expectedPayload) !== challenge.signing_payload) {
    throw new HttpError(401, 'takeover_payload_mismatch', '接管挑战上下文不匹配');
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

async function grantCidDataRoot(
  env: Env,
  binding: ParsedBinding,
): Promise<{ dataRoot: Uint8Array; dataRootHash: string }> {
  let row = await readCidDataRoot(env, binding.cidNumber);
  if (!row) {
    const generatedRoot = crypto.getRandomValues(new Uint8Array(DATA_ROOT_BYTES));
    try {
      const dataRootHash = await sha256Hex(generatedRoot);
      const sealed = await sealDataRoot(env, binding.cidNumber, generatedRoot);
      const now = nowMs();
      await env.DB.prepare(
        `INSERT INTO cid_data_roots
          (cid_number, recovery_ciphertext, recovery_nonce, recovery_key_version,
           data_root_hash, active_binding_revision, active_account_id, created_at, updated_at)
          VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
          ON CONFLICT(cid_number) DO NOTHING`,
      )
        .bind(
          binding.cidNumber,
          sealed.ciphertext,
          sealed.nonce,
          RECOVERY_KEY_VERSION,
          dataRootHash,
          binding.bindingRevision,
          binding.accountId,
          now,
          now,
        )
        .run();
    } finally {
      generatedRoot.fill(0);
    }
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
    dataRoot.fill(0);
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
    dataRoot.fill(0);
    throw new HttpError(409, 'stale_binding_revision', 'CID 接管版本并发推进');
  }
  return { dataRoot, dataRootHash };
}

async function readCidDataRoot(
  env: Env,
  cidNumber: string,
): Promise<CidDataRootRow | null> {
  return env.DB.prepare(
    `SELECT cid_number, recovery_ciphertext, recovery_nonce, recovery_key_version,
      data_root_hash, active_binding_revision, active_account_id, created_at, updated_at
      FROM cid_data_roots WHERE cid_number = ?`,
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
  const encrypted = await crypto.subtle.encrypt(
    {
      name: 'AES-GCM',
      iv: asArrayBuffer(nonce),
      additionalData: asArrayBuffer(recoveryAtRestAad(env, cidNumber, RECOVERY_KEY_VERSION)),
      tagLength: 128,
    },
    await deriveRecoveryAtRestKey(env, cidNumber, RECOVERY_KEY_VERSION),
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
  if (row.recovery_key_version !== RECOVERY_KEY_VERSION) {
    throw new HttpError(503, 'cid_data_root_key_version_unavailable', 'CID 数据根恢复密钥版本不可用');
  }
  try {
    const plaintext = await crypto.subtle.decrypt(
      {
        name: 'AES-GCM',
        iv: asArrayBuffer(base64ToBytes(row.recovery_nonce)),
        additionalData: asArrayBuffer(
          recoveryAtRestAad(env, row.cid_number, row.recovery_key_version),
        ),
        tagLength: 128,
      },
      await deriveRecoveryAtRestKey(env, row.cid_number, row.recovery_key_version),
      asArrayBuffer(base64ToBytes(row.recovery_ciphertext)),
    );
    const dataRoot = new Uint8Array(plaintext);
    if (dataRoot.length !== DATA_ROOT_BYTES) {
      dataRoot.fill(0);
      throw new Error('invalid data root length');
    }
    return dataRoot;
  } catch (error) {
    if (error instanceof HttpError) throw error;
    throw new HttpError(503, 'cid_data_root_corrupted', 'CID 数据根无法解封');
  }
}

async function deriveRecoveryAtRestKey(
  env: Env,
  cidNumber: string,
  keyVersion: number,
): Promise<CryptoKey> {
  const raw = env.CID_DATA_ROOT_RECOVERY_KEY;
  if (typeof raw !== 'string' || !/^0x[0-9a-f]{64}$/.test(raw)) {
    throw new HttpError(503, 'cid_data_root_recovery_key_unavailable', 'CID 数据根恢复密钥未配置');
  }
  const secretBytes = hexToBytes(raw);
  try {
    const baseKey = await crypto.subtle.importKey(
      'raw',
      asArrayBuffer(secretBytes),
      'HKDF',
      false,
      ['deriveKey'],
    );
    return crypto.subtle.deriveKey(
      {
        name: 'HKDF',
        hash: 'SHA-256',
        salt: asArrayBuffer(new TextEncoder().encode(
          `${RECOVERY_AT_REST_DOMAIN}/salt|${requireGenesisHash(env.CHAIN_GENESIS_HASH)}`,
        )),
        info: asArrayBuffer(new TextEncoder().encode(
          `${RECOVERY_AT_REST_DOMAIN}|${assertCidNumber(cidNumber)}|${keyVersion}`,
        )),
      },
      baseKey,
      { name: 'AES-GCM', length: 256 },
      false,
      ['encrypt', 'decrypt'],
    );
  } finally {
    secretBytes.fill(0);
  }
}

function recoveryAtRestAad(
  env: Env,
  cidNumber: string,
  keyVersion: number,
): Uint8Array {
  return new TextEncoder().encode(
    `${RECOVERY_AT_REST_DOMAIN}|${requireGenesisHash(env.CHAIN_GENESIS_HASH)}|`
    + `${assertCidNumber(cidNumber)}|${keyVersion}`,
  );
}

async function encryptDataRootForRecipient(
  env: Env,
  binding: ParsedBinding,
  challengeId: string,
  dataRoot: Uint8Array,
  dataRootHash: string,
): Promise<RecoveryEnvelope> {
  try {
    const recipientPublicKey = await crypto.subtle.importKey(
      'raw',
      asArrayBuffer(hexToBytes(binding.recoveryPublicKey)),
      { name: 'X25519' },
      false,
      [],
    );
    const generated = await crypto.subtle.generateKey(
      { name: 'X25519' },
      true,
      ['deriveBits'],
    );
    if (!isCryptoKeyPair(generated)) {
      throw new Error('X25519 did not return a key pair');
    }
    const senderPublicKeyBytes = new Uint8Array(
      await crypto.subtle.exportKey('raw', generated.publicKey),
    );
    const senderPublicKey = `0x${bytesToHex(senderPublicKeyBytes)}`;
    const sharedBits = new Uint8Array(await crypto.subtle.deriveBits(
      { name: 'X25519', public: recipientPublicKey },
      generated.privateKey,
      256,
    ));
    try {
      const envelopeKey = await deriveRecoveryGrantKey(env, binding, challengeId, sharedBits);
      const nonce = crypto.getRandomValues(new Uint8Array(AES_GCM_NONCE_BYTES));
      const encrypted = await crypto.subtle.encrypt(
        {
          name: 'AES-GCM',
          iv: asArrayBuffer(nonce),
          additionalData: asArrayBuffer(recoveryGrantAad(
            env,
            binding,
            challengeId,
            senderPublicKey,
            dataRootHash,
          )),
          tagLength: 128,
        },
        envelopeKey,
        asArrayBuffer(dataRoot),
      );
      return {
        senderPublicKey,
        nonce: bytesToBase64(nonce),
        ciphertext: bytesToBase64(new Uint8Array(encrypted)),
      };
    } finally {
      sharedBits.fill(0);
    }
  } catch {
    throw new HttpError(503, 'cid_data_root_envelope_failed', 'CID 数据根加密信封生成失败');
  }
}

async function deriveRecoveryGrantKey(
  env: Env,
  binding: ParsedBinding,
  challengeId: string,
  sharedBits: Uint8Array,
): Promise<CryptoKey> {
  const baseKey = await crypto.subtle.importKey(
    'raw',
    asArrayBuffer(sharedBits),
    'HKDF',
    false,
    ['deriveKey'],
  );
  return crypto.subtle.deriveKey(
    {
      name: 'HKDF',
      hash: 'SHA-256',
      salt: asArrayBuffer(new TextEncoder().encode(recoveryGrantSalt(
        env,
        binding,
        challengeId,
      ))),
      info: asArrayBuffer(new TextEncoder().encode(RECOVERY_GRANT_DOMAIN)),
    },
    baseKey,
    { name: 'AES-GCM', length: 256 },
    false,
    ['encrypt'],
  );
}

function recoveryGrantSalt(
  env: Env,
  binding: ParsedBinding,
  challengeId: string,
): string {
  return `${RECOVERY_GRANT_DOMAIN}/salt|${requireGenesisHash(env.CHAIN_GENESIS_HASH)}|`
    + `${binding.cidNumber}|${binding.bindingRevision}|${binding.accountId}|${challengeId}`;
}

function recoveryGrantAad(
  env: Env,
  binding: ParsedBinding,
  challengeId: string,
  senderPublicKey: string,
  dataRootHash: string,
): Uint8Array {
  return new TextEncoder().encode(
    `${RECOVERY_GRANT_DOMAIN}|${requireGenesisHash(env.CHAIN_GENESIS_HASH)}|`
    + `${binding.cidNumber}|${binding.bindingRevision}|${binding.accountId}|${challengeId}|`
    + `${binding.recoveryPublicKey}|${senderPublicKey}|${dataRootHash}`,
  );
}

async function requireCurrentFinalizedBinding(
  env: Env,
  cidNumber: string,
  accountId: string,
  expectedRevision?: number,
): Promise<Omit<ParsedBinding, 'recoveryPublicKey'>> {
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
  if (
    typeof body.challenge_id !== 'string'
    || !body.challenge_id.startsWith(TAKEOVER_CHALLENGE_PREFIX)
  ) {
    throw new HttpError(400, 'invalid_takeover_request', '接管请求缺少专用挑战');
  }
  return {
    cidNumber: parseCidNumber(body.cid_number),
    accountId: parseAccountId(body.account_id),
    bindingRevision,
    recoveryPublicKey: requireRecoveryPublicKey(body.recovery_public_key),
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

function requireRecoveryPublicKey(value: unknown): string {
  if (
    typeof value !== 'string'
    || !/^0x[0-9a-f]{64}$/.test(value)
    || value === `0x${'0'.repeat(64)}`
  ) {
    throw new HttpError(400, 'invalid_recovery_public_key', 'X25519 恢复公钥格式不合法');
  }
  return value;
}

function isCryptoKeyPair(value: CryptoKey | CryptoKeyPair): value is CryptoKeyPair {
  return 'privateKey' in value && 'publicKey' in value;
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
