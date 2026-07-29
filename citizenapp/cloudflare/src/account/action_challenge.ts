import type { Env, LoginChallengeRow } from '../types';
import { HttpError } from '../shared/http';
import { createId } from '../shared/ids';
import { nowMs, secondsFromNow } from '../shared/time';
import { verifyWalletSignature } from '../auth/wallet_signature';
import {
  OP_SIGN_SQUARE_ACTION,
  bytesToHex,
  concatBytes,
  hexToBytes,
  scaleString,
  signingMessage,
  u64Le
} from '../shared/signing_message';

/// 需要钱包 sr25519 主钥签名授权的敏感动作。创作者档位已改为链上交易签名，
/// 不得再复用本离链挑战形成第二次业务签名。
export type SignedAction = 'delete_account';

const ACTION_CHALLENGE_TTL_SECONDS = 300;

/// 动作签名 SCALE payload：`action ‖ account_id ‖ challenge_id ‖ cid_number ‖ expires_at`。
/// action 编入正文 → 登录/其它动作的签名无法被重放成本动作；cid_number 固定插在
/// challenge_id 与 expires_at 之间。被签消息 = signing_message(OP_SIGN_SQUARE_ACTION, payload)。
function buildActionScalePayload(
  action: SignedAction,
  accountId: string,
  challengeId: string,
  expiresAt: number,
  cidNumber: string
): Uint8Array {
  return concatBytes(
    scaleString(action),
    scaleString(accountId),
    scaleString(challengeId),
    scaleString(cidNumber),
    u64Le(expiresAt)
  );
}

export interface IssuedActionChallenge {
  challengeId: string;
  opTag: number;
  signingPayloadHex: string;
  expiresAt: number;
}

/// 下发一个动作签名挑战（复用 square_login_challenges 表，signing_payload 存
/// SCALE payload 的 hex，动作编入其中）。
export async function issueActionChallenge(
  env: Env,
  cidNumber: string,
  accountId: string,
  action: SignedAction
): Promise<IssuedActionChallenge> {
  const challengeId = createId('sqa');
  const expiresAt = secondsFromNow(ACTION_CHALLENGE_TTL_SECONDS);
  const signingPayloadHex = bytesToHex(
    buildActionScalePayload(action, accountId, challengeId, expiresAt, cidNumber)
  );

  await env.DB.prepare(
    `INSERT INTO square_login_challenges
      (challenge_id, cid_number, account_id, signing_payload, expires_at, used_at)
      VALUES (?, ?, ?, ?, ?, NULL)`
  )
    .bind(challengeId, cidNumber, accountId, signingPayloadHex, expiresAt)
    .run();

  return { challengeId, opTag: OP_SIGN_SQUARE_ACTION, signingPayloadHex, expiresAt };
}

export interface ActionSignatureInput {
  cidNumber: string;
  accountId: string;
  action: SignedAction;
  challengeId: string;
  signature: string;
}

/// 校验并**一次性消费**动作签名：挑战存在且归属该账户、未用、未过期、动作匹配、
/// 钱包 sr25519 主钥对 signing_message(OP_SIGN_SQUARE_ACTION) 签名有效。
/// 任一不满足抛 401。成功后标记 used_at，防重放。
export async function consumeActionSignature(
  env: Env,
  input: ActionSignatureInput
): Promise<void> {
  const challenge = await env.DB.prepare(
    `SELECT challenge_id, cid_number, account_id, signing_payload, expires_at, used_at
      FROM square_login_challenges
      WHERE challenge_id = ?`
  )
    .bind(input.challengeId)
    .first<LoginChallengeRow>();

  if (
    !challenge ||
    challenge.cid_number !== input.cidNumber ||
    challenge.account_id !== input.accountId
  ) {
    throw new HttpError(401, 'invalid_challenge', '签名挑战不存在');
  }
  if (challenge.used_at !== null) {
    throw new HttpError(401, 'used_challenge', '签名挑战已使用');
  }
  if (challenge.expires_at <= nowMs()) {
    throw new HttpError(401, 'expired_challenge', '签名挑战已过期');
  }
  // 动作必须匹配：用请求的 action 重建 payload，须逐字节等于下发时存的 payload，
  // 杜绝把别的动作（含登录）挑战的签名挪用到本动作。
  const expectedPayloadHex = bytesToHex(
    buildActionScalePayload(
      input.action,
      challenge.account_id,
      challenge.challenge_id,
      challenge.expires_at,
      challenge.cid_number
    )
  );
  if (expectedPayloadHex !== challenge.signing_payload) {
    throw new HttpError(401, 'action_mismatch', '签名挑战动作/CID 不匹配');
  }

  const message = signingMessage(
    OP_SIGN_SQUARE_ACTION,
    hexToBytes(challenge.signing_payload)
  );
  const isValid = await verifyWalletSignature(message, input.signature, input.accountId);
  if (!isValid) {
    throw new HttpError(401, 'invalid_signature', '钱包签名校验失败');
  }

  const claimedAt = nowMs();
  const claimed = await env.DB.prepare(
    `UPDATE square_login_challenges
      SET used_at = ?
      WHERE challenge_id = ?
        AND cid_number = ?
        AND account_id = ?
        AND used_at IS NULL
        AND expires_at > ?`
    )
    .bind(
      claimedAt,
      challenge.challenge_id,
      challenge.cid_number,
      challenge.account_id,
      claimedAt
    )
    .run();
  // 原子占位：并发下只有一方能把 used_at 从 NULL 翻成非空；命中 0 行说明已被
  // 抢先消费（含 SELECT 判空与本 UPDATE 之间的竞态），按已用处理。
  if ((claimed.meta?.changes ?? 0) !== 1) {
    throw new HttpError(401, 'used_challenge', '签名挑战已使用');
  }
}

/// 释放（回滚）一个已消费的动作挑战：used_at 重置为 NULL，供下游副作用
/// （purge 注销）失败后原地重试，避免烧掉签名逼用户重签。
/// 仅应在 consumeActionSignature 成功、随后副作用失败时调用；expires_at 不变，
/// 因此释放不延长挑战寿命、不放大重放窗口。
export async function releaseActionChallenge(
  env: Env,
  challengeId: string
): Promise<void> {
  await env.DB.prepare(
    `UPDATE square_login_challenges SET used_at = NULL WHERE challenge_id = ?`
  )
    .bind(challengeId)
    .run();
}
