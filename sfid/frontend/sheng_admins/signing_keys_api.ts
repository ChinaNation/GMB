// 中文注释:省管理员本人签名密钥生成/更换 API。
// 该流程只管理 SFID 本地 signing seed,必须由当前管理员本人扫码签名确认。

import type { AdminAuth } from '../auth/types';
import { adminRequest } from '../utils/http';

export type SigningOperation = 'GENERATE' | 'REPLACE';

export interface SignerPrepareResult {
  operation: SigningOperation;
  request_id: string;
  payload_hex: string;
  expires_at: number;
  display_action: string;
  display_summary: string;
  display_fields: Array<{ key: string; label: string; value: string }>;
}

export interface SignerSubmitResult {
  ok: boolean;
  operation_result: 'GENERATED' | 'REPLACED';
  signing_pubkey: string;
  signing_created_at: string;
}

/** POST /api/v1/admin/sheng-signer/prepare */
export async function prepareSignerOperation(
  auth: AdminAuth,
  operation: SigningOperation,
): Promise<SignerPrepareResult> {
  return adminRequest<SignerPrepareResult>('/api/v1/admin/sheng-signer/prepare', auth, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ operation }),
  });
}

/** POST /api/v1/admin/sheng-signer/submit */
export async function submitSignerOperation(
  auth: AdminAuth,
  payload: {
    operation: SigningOperation;
    payload_hex: string;
    signature: string;
    signer_pubkey?: string;
  },
): Promise<SignerSubmitResult> {
  return adminRequest<SignerSubmitResult>('/api/v1/admin/sheng-signer/submit', auth, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(payload),
  });
}
