import type { Env } from '../types';
import { verifyWalletSignature } from '../auth/wallet_signature';
import { HttpError, jsonResponse, readJson, requireSession } from '../shared/http';
import { accountIdBytes, assertAccountId } from '../shared/ids';
import {
  concatBytes,
  OP_SIGN_CID_REBIND,
  scaleString,
  signingMessage,
} from '../shared/signing_message';
import { revokeRebindOldAccount } from '../account/purge';

interface RebindRevokeRequest {
  old_account_id?: unknown;
  old_account_signature?: unknown;
}

function parseOldAccountId(value: unknown): string {
  try {
    return assertAccountId(value);
  } catch {
    throw new HttpError(400, 'invalid_old_account_id', '旧账户标识格式不合法');
  }
}

function parseOldAccountSignature(value: unknown): string {
  if (typeof value !== 'string' || !/^0x[0-9a-fA-F]{128}$/.test(value)) {
    throw new HttpError(400, 'invalid_old_account_signature', '旧账户换绑授权签名格式不合法');
  }
  return value.toLowerCase();
}

/// POST /v1/square/rebind/revoke —— 换绑后吊销**旧身份账户**的鉴权云端数据。
///
/// request_guard 已保证 Bearer 会话账户仍是该 CID 的当前链上绑定账户；handler 再验证旧账户
/// 对 `signing_message(OP_SIGN_CID_REBIND, SCALE(cid_number, new_account_id))` 的换绑授权。
/// 两项同时成立才允许新账户代清理旧账户，禁止把“旧账户当前未绑定”误当成历史归属证明。
///
/// CID 及其通讯录/动态/文章/粉丝/会员均按 cid_number 归属，不迁移、不删除。本接口只删
/// old_account_id 的账户级鉴权材料；不关闭按 CID 命名的实时 DO，也不删除 CID 级绑定 nonce。
/// 删除幂等，客户端可在网络失败或重启后携带同一授权安全重试。
export async function rebindRevokeRoute(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const body = await readJson<RebindRevokeRequest>(request);
  const oldAccountId = parseOldAccountId(body.old_account_id);
  const oldAccountSignature = parseOldAccountSignature(body.old_account_signature);
  if (oldAccountId === session.account_id) {
    throw new HttpError(409, 'rebind_account_unchanged', '旧账户与当前绑定账户不能相同');
  }

  const payload = concatBytes(
    scaleString(session.cid_number),
    accountIdBytes(session.account_id),
  );
  const digest = signingMessage(OP_SIGN_CID_REBIND, payload);
  if (!(await verifyWalletSignature(digest, oldAccountSignature, oldAccountId))) {
    throw new HttpError(403, 'invalid_rebind_authorization', '旧账户换绑授权验证失败');
  }

  const deleted = await revokeRebindOldAccount(env, oldAccountId);
  return jsonResponse({
    ok: true,
    old_account_id: oldAccountId,
    current_account_id: session.account_id,
    deleted,
  });
}
