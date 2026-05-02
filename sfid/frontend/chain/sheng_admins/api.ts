// 中文注释:省管理员 3-tier 名册 API(ADR-008,Phase 4+5 后端 endpoint 已落地,推链先 mock)。
// 仅 SHENG_ADMIN.Main 槽可对名册做写操作,SHI_ADMIN/SHENG_ADMIN.Backup* 仅可读。

import { adminRequest, type AdminAuth } from '../../api/client';
import type { ShengSlot } from './types';

/** 名册一行:三槽中某一槽的当前登记状态 */
export interface RosterEntry {
  slot: ShengSlot;
  /** 0x 小写 hex;空字符串/null 表示该槽未登记 */
  admin_pubkey: string | null;
  /** 显示用名字(可选) */
  admin_name?: string | null;
  /** 该槽对应签名密钥的链上状态:NOT_ACTIVATED / ACTIVATED */
  signing_status?: 'NOT_ACTIVATED' | 'ACTIVATED' | null;
  /** 已激活的签名公钥(0x 小写 hex);未激活时 null */
  signing_pubkey?: string | null;
  /** 最近一次更新时间 */
  updated_at?: string | null;
}

/** 单省 roster 列表 */
export interface ShengAdminRoster {
  province: string;
  entries: RosterEntry[];
}

/** GET /api/v1/admin/sheng-admin/roster */
export async function getRoster(
  auth: AdminAuth,
  province?: string,
): Promise<ShengAdminRoster> {
  const qs = province ? `?province=${encodeURIComponent(province)}` : '';
  return adminRequest<ShengAdminRoster>(`/api/v1/admin/sheng-admin/roster${qs}`, auth);
}

/** POST /api/v1/admin/sheng-admin/roster/add-backup */
export async function addBackup(
  auth: AdminAuth,
  payload: {
    slot: Exclude<ShengSlot, 'Main'>;
    new_pubkey: string;
    new_name?: string;
  },
): Promise<RosterEntry> {
  return adminRequest<RosterEntry>('/api/v1/admin/sheng-admin/roster/add-backup', auth, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(payload),
  });
}

/** POST /api/v1/admin/sheng-admin/roster/remove-backup */
export async function removeBackup(
  auth: AdminAuth,
  payload: { slot: Exclude<ShengSlot, 'Main'> },
): Promise<{ removed: boolean }> {
  return adminRequest<{ removed: boolean }>('/api/v1/admin/sheng-admin/roster/remove-backup', auth, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(payload),
  });
}

/** GET /api/v1/chain/sheng-admin/list?province=XX(公开,无 auth)*/
export interface ChainShengAdminEntry {
  province: string;
  slot: ShengSlot;
  admin_pubkey: string;
  signing_pubkey?: string | null;
}

export async function getChainShengAdminList(province: string): Promise<ChainShengAdminEntry[]> {
  const qs = `?province=${encodeURIComponent(province)}`;
  const resp = await fetch(`/api/v1/chain/sheng-admin/list${qs}`);
  if (!resp.ok) {
    throw new Error(`链上 sheng-admin 列表查询失败 (${resp.status})`);
  }
  const body = (await resp.json()) as { code: number; data: ChainShengAdminEntry[]; message?: string };
  if (body.code !== 0) {
    throw new Error(body.message ?? '链上 sheng-admin 列表查询失败');
  }
  return body.data;
}

export interface SignerActivateResult {
  signing_pubkey: string;
  /** 链上推送状态:PUSHED / PENDING / MOCKED */
  chain_status: 'PUSHED' | 'PENDING' | 'MOCKED';
  chain_tx_hash?: string | null;
}

export interface SignerRotateResult {
  old_signing_pubkey: string;
  new_signing_pubkey: string;
  chain_status: 'PUSHED' | 'PENDING' | 'MOCKED';
  chain_tx_hash?: string | null;
}

/** POST /api/v1/admin/sheng-signer/activate */
export async function activateSigner(auth: AdminAuth): Promise<SignerActivateResult> {
  return adminRequest<SignerActivateResult>('/api/v1/admin/sheng-signer/activate', auth, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({}),
  });
}

/** POST /api/v1/admin/sheng-signer/rotate */
export async function rotateSigner(auth: AdminAuth): Promise<SignerRotateResult> {
  return adminRequest<SignerRotateResult>('/api/v1/admin/sheng-signer/rotate', auth, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({}),
  });
}
