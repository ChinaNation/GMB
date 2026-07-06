import type { Env, MembershipRow, SessionState } from '../types';
import { HttpError, jsonResponse, requireSession } from '../shared/http';
import { nowMs } from '../shared/time';

export async function getMembership(env: Env, ownerAccount: string): Promise<MembershipRow | null> {
  return env.DB.prepare(
    `SELECT owner_account, membership_level, storage_quota_bytes, storage_used_bytes, expires_at, updated_at
      FROM square_memberships
      WHERE owner_account = ?`
  )
    .bind(ownerAccount)
    .first<MembershipRow>();
}

export async function requireActiveMembership(
  env: Env,
  ownerAccount: string,
  requiredBytes: number
): Promise<MembershipRow> {
  const membership = await getMembership(env, ownerAccount);
  if (!membership || membership.expires_at <= nowMs()) {
    throw new HttpError(402, 'membership_required', '需要有效会员才能使用广场内容存储');
  }

  const remainingBytes = membership.storage_quota_bytes - membership.storage_used_bytes;
  if (requiredBytes > remainingBytes) {
    throw new HttpError(402, 'storage_quota_exceeded', '会员存储容量不足');
  }

  return membership;
}

export async function addStorageUsage(
  env: Env,
  ownerAccount: string,
  usedBytes: number
): Promise<void> {
  await env.DB.prepare(
    `UPDATE square_memberships
      SET storage_used_bytes = storage_used_bytes + ?, updated_at = ?
      WHERE owner_account = ?`
  )
    .bind(usedBytes, nowMs(), ownerAccount)
    .run();
}

export async function membershipRoute(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const membership = await getMembership(env, session.owner_account);

  return jsonResponse({
    ok: true,
    membership,
    active: Boolean(membership && membership.expires_at > nowMs())
  });
}

export function assertSessionOwner(session: SessionState, ownerAccount: string): void {
  if (session.owner_account !== ownerAccount) {
    throw new HttpError(403, 'owner_account_mismatch', '登录钱包与请求钱包不一致');
  }
}
