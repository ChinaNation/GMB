// 中文注释:管理员安全动作 API。
// 管理端权限统一为 LOGIN_STATE / SCAN_SIGN 两类:
//   - LOGIN_STATE:仅需有效会话(会话已是链上已证管理员),无需 commit,各业务 handler 直接执行。
//   - SCAN_SIGN:会话 + 冷钱包扫码签名(prepare 拿 sign_request → 扫签名响应 → commit 提交 signer_pubkey/signature/payload_hash)。

import type { AdminAuth } from '../auth/types';
import { ApiError, adminRequest } from '../utils/http';

export type AdminActionType =
  | 'CREATE_CITY_REGISTRY'
  | 'DELETE_CITY_REGISTRY'
  | 'CREATE_FEDERAL_REGISTRY'
  | 'DELETE_FEDERAL_REGISTRY'
  | 'INSTITUTION_CREATE'
  | 'INSTITUTION_UPDATE'
  | 'INSTITUTION_CREATE_ACCOUNT'
  | 'INSTITUTION_DELETE_ACCOUNT'
  | 'INSTITUTION_DEREGISTER'
  | 'INSTITUTION_ACCOUNT_DEREGISTER'
  | 'INSTITUTION_UPLOAD_DOCUMENT'
  | 'INSTITUTION_DELETE_DOCUMENT';

export type AdminOperationAuth = 'LOGIN_STATE' | 'SCAN_SIGN';
export type RegistryOrgCodeTarget = 'FEDERAL_REGISTRY' | 'CITY_REGISTRY';

export type PrepareAdminActionOutput = {
  action_id: string;
  action_type: AdminActionType;
  sign_request?: string | null;
  payload_hash: string;
  auth_type: AdminOperationAuth;
  expires_at: number;
};

export type AdminSecurityGrantOutput = {
  grant_id: string;
  action_type: AdminActionType;
  auth_type: AdminOperationAuth;
  target: string;
  expires_at: number;
};

export function formatAdminCreateError(error: unknown, targetRegistryOrgCode: RegistryOrgCodeTarget, fallback: string): string {
  if (!(error instanceof ApiError)) {
    return error instanceof Error ? error.message : fallback;
  }
  // 中文注释:管理员新增失败统一按稳定 error_code 显示,不解析后端 message。
  if (error.errorCode === 'CID_ADMIN_ACCOUNT_EXISTS_AS_FEDERAL_REGISTRY') {
    return targetRegistryOrgCode === 'FEDERAL_REGISTRY'
      ? '该账户已是联邦注册局管理员，不能重复新增'
      : '该账户已是联邦注册局管理员，不能新增为市注册局管理员';
  }
  if (error.errorCode === 'CID_ADMIN_ACCOUNT_EXISTS_AS_CITY_REGISTRY') {
    return targetRegistryOrgCode === 'FEDERAL_REGISTRY'
      ? '该账户已是市注册局管理员，不能新增为联邦注册局管理员'
      : '该账户已是市注册局管理员，不能重复新增';
  }
  if (error.errorCode === 'CID_ADMIN_FEDERAL_REGISTRY_PROVINCE_LIMIT_REACHED') {
    return '联邦注册局管理员已满 5 人，不能继续新增';
  }
  if (error.errorCode === 'CID_ADMIN_CITY_REGISTRY_CITY_LIMIT_REACHED') {
    return '本市市注册局管理员已满 30 人，不能继续新增';
  }
  return error.message || fallback;
}

export async function prepareAdminAction(
  auth: AdminAuth,
  actionType: AdminActionType,
  payload: unknown,
): Promise<PrepareAdminActionOutput> {
  return adminRequest<PrepareAdminActionOutput>('/api/v1/admin/actions/prepare', auth, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ action_type: actionType, payload }),
  });
}

// 中文注释:SCAN_SIGN commit 只携带冷钱包扫码签名字段,不再有 passkey_assertion。
export async function commitAdminAction<T>(
  auth: AdminAuth,
  input: {
    action_id: string;
    signer_pubkey: string;
    signature: string;
    payload_hash: string;
  },
): Promise<T> {
  return adminRequest<T>('/api/v1/admin/actions/commit', auth, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(input),
  });
}

// 中文注释:组件提供的「扫码签名」回调:给定已 prepare 的 SCAN_SIGN 动作,
// 弹出公民钱包二维码并扫描签名响应,解析出 signer_pubkey/signature 回传。
export type ScanSignResolver = (
  prepared: PrepareAdminActionOutput,
) => Promise<{ signer_pubkey: string; signature: string }>;

// 中文注释:统一的 SCAN_SIGN 安全授权:prepare → 组件扫码签名 → commit 取回一次性 grant。
// LOGIN_STATE 动作不走这里(无 commit,业务 handler 仅凭会话执行)。
export async function createScanSignSecurityGrant(
  auth: AdminAuth,
  actionType: AdminActionType,
  payload: unknown,
  signWithScan: ScanSignResolver,
): Promise<AdminSecurityGrantOutput> {
  const prepared = await prepareAdminAction(auth, actionType, payload);
  if (prepared.auth_type !== 'SCAN_SIGN' || !prepared.sign_request) {
    throw new Error('该操作缺少公民钱包扫码签名请求');
  }
  const { signer_pubkey, signature } = await signWithScan(prepared);
  return commitAdminAction<AdminSecurityGrantOutput>(auth, {
    action_id: prepared.action_id,
    signer_pubkey,
    signature,
    payload_hash: prepared.payload_hash,
  });
}
