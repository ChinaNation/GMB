// 中文注释:省级签名密钥 activate / rotate API(ADR-008,Phase 4+5 后端 endpoint 已落地)。
// 三槽各自独立签名密钥,activate = 首次部署该槽签名 keypair 并推链;rotate = 替换为新 keypair。

import { adminRequest, type AdminAuth } from './client';

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
