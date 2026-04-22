// 中文注释：任务卡 2 新机构/账户两层模型的前端 API 封装。
// 后端对应:backend/src/institutions/handler.rs
// 铁律:feedback_institutions_two_layer.md(一个 sfid_id 下可挂多个 account_name)

import { adminRequest, type AdminAuth, type CpmsSiteRow } from './client';

export type InstitutionCategory = 'PUBLIC_SECURITY' | 'GOV_INSTITUTION' | 'PRIVATE_INSTITUTION';

export const InstitutionCategoryLabel: Record<InstitutionCategory, string> = {
  PUBLIC_SECURITY: '公安局',
  GOV_INSTITUTION: '公权机构',
  PRIVATE_INSTITUTION: '私权机构',
};

/**
 * 多签账户链上状态。
 *
 * 流转(2026-04-21 统一两步激活模式):
 *   INACTIVE  ──点"激活"──▶  PENDING  ──成功──▶  REGISTERED
 *                                     └──失败──▶  FAILED ──点"重试激活"──▶  PENDING
 *
 * - `INACTIVE`:本地已登记账户,未推链。所有账户(含机构创建时自动生成的 2 条默认账户、
 *   管理员手工创建的其他账户)默认都是这个状态。
 * - `PENDING`:激活请求已提交,正在上链确认。
 * - `REGISTERED`:链上已注册,`duoqian_address` + `chain_tx_hash` 已填。
 * - `FAILED`:上链失败,可点"重试激活"重推。
 */
export type MultisigChainStatus = 'INACTIVE' | 'PENDING' | 'REGISTERED' | 'FAILED';

export interface MultisigInstitution {
  sfid_id: string;
  /** 机构名称。两步式创建(2026-04-19):第一步生成时为 null,详情页补填后非空。 */
  institution_name: string | null;
  category: InstitutionCategory;
  a3: string;
  p1: string;
  province: string;
  city: string;
  province_code: string;
  /** 任务卡 6 新增:2 位数字市代码(r5 段后 3 字符),作为公安局对账稳定主键 */
  city_code?: string;
  institution_code: string;
  /** 私法人子类型(仅 A3=SFR 且 P1 填完后才有值) */
  sub_type?: string | null;
  /** 所属法人 sfid_id(仅 A3=FFR 非法人必填;指向 SFR/GFR) */
  parent_sfid_id?: string | null;
  created_by: string;
  created_at: string;
}

export interface ReconcileReport {
  province: string;
  inserted: number;
  updated: number;
  removed: number;
  total_after: number;
}

export interface MultisigAccount {
  sfid_id: string;
  account_name: string;
  duoqian_address: string | null;
  chain_status: MultisigChainStatus;
  chain_tx_hash: string | null;
  chain_block_number: number | null;
  created_by: string;
  created_at: string;
}

export interface InstitutionListRow {
  sfid_id: string;
  /** 两步式创建:第一步仅有 SFID 时为 null,详情页补填后非空 */
  institution_name: string | null;
  category: InstitutionCategory;
  a3: string;
  p1: string;
  province: string;
  city: string;
  institution_code: string;
  sub_type?: string | null;
  parent_sfid_id?: string | null;
  account_count: number;
  created_at: string;
  /** 创建该机构的登录管理员姓名(按 created_by pubkey 反查 admin_users);未命中 null */
  created_by_name?: string | null;
  /** 创建者角色:KEY_ADMIN / SHENG_ADMIN / SHI_ADMIN;未命中 null */
  created_by_role?: string | null;
}

export interface InstitutionDetail {
  institution: MultisigInstitution;
  accounts: MultisigAccount[];
  /** 创建该机构的登录管理员姓名(按 created_by pubkey 反查 admin_users) */
  created_by_name?: string | null;
  /** 创建者角色:KEY_ADMIN / SHENG_ADMIN / SHI_ADMIN */
  created_by_role?: string | null;
}

/** 机构资料库文档 */
export interface InstitutionDocument {
  id: number;
  sfid_id: string;
  file_name: string;
  doc_type: string;
  file_size: number;
  uploaded_by: string;
  uploaded_at: string;
}

/** 文档类型枚举 */
export const DOC_TYPE_OPTIONS = [
  '公司章程',
  '营业许可证',
  '股东会决议',
  '法人授权书',
  '其他',
] as const;

// ─── 请求 DTO ─────────────────────────────────────────────────

export interface CreateInstitutionInput {
  a3: string;
  p1?: string;
  province?: string;
  city: string;
  institution: string;
  /**
   * 机构名称。
   * - 私权(SFR/FFR)两步式:**不传**(或 undefined),由详情页 updateInstitution 补填
   * - 公权(GFR)/公安局:**必传**,同步做查重
   */
  institution_name?: string;
}

export interface CreateInstitutionOutput {
  sfid_id: string;
  /** 首次创建:私权为 null,公权为已填入的名称 */
  institution_name: string | null;
  category: InstitutionCategory;
}

/** 机构详情页可编辑字段(两步式第二步) */
export interface UpdateInstitutionInput {
  institution_name?: string;
  sub_type?: string | null;
  /** 所属法人 sfid_id(仅 FFR;传空串后端会拒) */
  parent_sfid_id?: string;
}

/** 法人机构搜索结果项(FFR 详情页"所属法人"选择器用) */
export interface ParentInstitutionRow {
  sfid_id: string;
  institution_name: string;
  a3: string;
  /** 私法人子类型(仅 a3=SFR);FFR 判断父 SFR 是否 JOINT_STOCK 以显示清算行设置 */
  sub_type?: string | null;
  category: InstitutionCategory;
  province: string;
  city: string;
}

export interface CreateAccountOutput {
  sfid_id: string;
  account_name: string;
  chain_status: MultisigChainStatus;
  chain_tx_hash: string | null;
  chain_block_number: number | null;
  duoqian_address: string | null;
}

export interface ListInstitutionsQuery {
  category?: InstitutionCategory;
  province?: string;
  city?: string;
  /** 模糊搜索关键字:匹配机构名称或 SFID 子串(大小写不敏感);空=不过滤 */
  q?: string;
}

// ─── API 调用 ─────────────────────────────────────────────────

/**
 * 机构名称查重。
 * - 私权机构(SFR/FFR):全国唯一
 * - 公权机构(GFR):同城唯一,需传 a3='GFR' + city
 */
export async function checkInstitutionName(
  auth: AdminAuth,
  name: string,
  a3?: string,
  city?: string,
): Promise<{ exists: boolean }> {
  const params = new URLSearchParams({ name });
  if (a3) params.set('a3', a3);
  if (city) params.set('city', city);
  return adminRequest<{ exists: boolean }>(`/api/v1/institution/check-name?${params.toString()}`, auth);
}

/**
 * 生成机构(**不上链**)。成功后拿到 sfid_id,再调 `createAccount` 实际上链。
 */
export async function createInstitution(
  auth: AdminAuth,
  input: CreateInstitutionInput
): Promise<CreateInstitutionOutput> {
  return adminRequest<CreateInstitutionOutput>('/api/v1/institution/create', auth, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(input),
  });
}

/**
 * 在机构下创建账户(**只登记本地 Inactive,不触链**)。
 * 2026-04-21 统一两步模式:创建后需在账户列表点"激活"按钮才推链注册。
 * 同一 sfid_id 下 account_name 必须唯一(后端硬校验)。
 */
export async function createAccount(
  auth: AdminAuth,
  sfidId: string,
  accountName: string
): Promise<CreateAccountOutput> {
  return adminRequest<CreateAccountOutput>(
    `/api/v1/institution/${encodeURIComponent(sfidId)}/account/create`,
    auth,
    {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ account_name: accountName }),
    }
  );
}

/**
 * 激活账户(推链注册)。
 * 允许从 INACTIVE / FAILED 触发;PENDING / REGISTERED 会被后端拒绝。
 * 链上派生公式由链端按 account_name 路由到 Role::Main / Role::Fee / Role::Named。
 */
export async function activateAccount(
  auth: AdminAuth,
  sfidId: string,
  accountName: string,
): Promise<CreateAccountOutput> {
  return adminRequest<CreateAccountOutput>(
    `/api/v1/institution/${encodeURIComponent(sfidId)}/account/${encodeURIComponent(accountName)}/activate`,
    auth,
    { method: 'POST' },
  );
}

/**
 * 按 scope(登录管理员角色/省/市)返回机构列表。
 * 可选 category / province / city 二次过滤。
 */
export async function listInstitutions(
  auth: AdminAuth,
  query?: ListInstitutionsQuery
): Promise<InstitutionListRow[]> {
  const params = new URLSearchParams();
  if (query?.category) params.set('category', query.category);
  if (query?.province) params.set('province', query.province);
  if (query?.city) params.set('city', query.city);
  if (query?.q && query.q.trim()) params.set('q', query.q.trim());
  const qs = params.toString();
  const path = qs ? `/api/v1/institution/list?${qs}` : '/api/v1/institution/list';
  return adminRequest<InstitutionListRow[]>(path, auth);
}

/**
 * 搜索可选的法人机构(供 FFR 详情页"所属法人"选择器使用)。
 * q 可匹配 sfid_id 子串或 institution_name 子串,全国范围,最多 20 条。
 */
export async function searchParentInstitutions(
  auth: AdminAuth,
  q: string,
): Promise<ParentInstitutionRow[]> {
  const params = new URLSearchParams({ q });
  return adminRequest<ParentInstitutionRow[]>(
    `/api/v1/institution/search-parents?${params.toString()}`,
    auth,
  );
}

/**
 * 更新机构详情(两步式第二步)。支持修改机构名称和企业类型。
 * 机构名称后端全国唯一校验(排除自身 sfid_id)。
 * 企业类型与 P1 联动:P1=0 必须 NON_PROFIT;P1=1 不得为 NON_PROFIT。
 */
export async function updateInstitution(
  auth: AdminAuth,
  sfidId: string,
  input: UpdateInstitutionInput,
): Promise<MultisigInstitution> {
  return adminRequest<MultisigInstitution>(
    `/api/v1/institution/${encodeURIComponent(sfidId)}`,
    auth,
    {
      method: 'PATCH',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(input),
    },
  );
}

/**
 * 获取机构详情(含账户列表)。
 */
export async function getInstitution(
  auth: AdminAuth,
  sfidId: string
): Promise<InstitutionDetail> {
  return adminRequest<InstitutionDetail>(
    `/api/v1/institution/${encodeURIComponent(sfidId)}`,
    auth
  );
}

/**
 * 列出机构下所有账户。
 */
export async function listAccounts(
  auth: AdminAuth,
  sfidId: string
): Promise<MultisigAccount[]> {
  return adminRequest<MultisigAccount[]>(
    `/api/v1/institution/${encodeURIComponent(sfidId)}/accounts`,
    auth
  );
}

/**
 * 任务卡 6:按 sfid 工具权威市清单对账公安局机构(增/删/改名)。
 * province 省略时 KeyAdmin 会对 43 省全量对账,其他角色按自己 scope 限制。
 * 进入公安局省详情页前调用,确保数据跟市清单同步。
 */
export async function reconcilePublicSecurity(
  auth: AdminAuth,
  province?: string
): Promise<ReconcileReport[]> {
  const qs = province ? `?province=${encodeURIComponent(province)}` : '';
  return adminRequest<ReconcileReport[]>(
    `/api/v1/public-security/reconcile${qs}`,
    auth,
    { method: 'POST' }
  );
}

/**
 * 任务卡 `20260408-sfid-public-security-cpms-embed`:
 * 按机构 sfid_id 反查其 CPMS 站点。
 * 后端通过 `(province, city, institution_code)` 三元组匹配,无则返回 null。
 */
export async function getCpmsSiteByInstitution(
  auth: AdminAuth,
  sfidId: string
): Promise<CpmsSiteRow | null> {
  return adminRequest<CpmsSiteRow | null>(
    `/api/v1/admin/cpms-keys/by-institution/${encodeURIComponent(sfidId)}`,
    auth
  );
}

/**
 * 删除账户(软删,不触链)。
 */
export async function deleteAccount(
  auth: AdminAuth,
  sfidId: string,
  accountName: string
): Promise<{ deleted: boolean }> {
  return adminRequest<{ deleted: boolean }>(
    `/api/v1/institution/${encodeURIComponent(sfidId)}/account/${encodeURIComponent(accountName)}`,
    auth,
    { method: 'DELETE' }
  );
}

// ─── 机构资料库文档 API ──────────────────────────────────────────

/** 列出机构的所有文档 */
export async function listDocuments(
  auth: AdminAuth,
  sfidId: string,
): Promise<InstitutionDocument[]> {
  return adminRequest<InstitutionDocument[]>(
    `/api/v1/institution/${encodeURIComponent(sfidId)}/documents`,
    auth,
  );
}

/** 上传文档(multipart) */
export async function uploadDocument(
  auth: AdminAuth,
  sfidId: string,
  file: File,
  docType: string,
): Promise<InstitutionDocument> {
  const formData = new FormData();
  formData.append('file', file);
  formData.append('doc_type', docType);
  return adminRequest<InstitutionDocument>(
    `/api/v1/institution/${encodeURIComponent(sfidId)}/documents`,
    auth,
    {
      method: 'POST',
      body: formData,
      // 不设 content-type,让浏览器自动设置 multipart boundary
    },
  );
}

/** 下载文档(返回 Blob) */
export async function downloadDocument(
  auth: AdminAuth,
  sfidId: string,
  docId: number,
  fileName: string,
): Promise<void> {
  const { adminHeaders } = await import('./client');
  const resp = await fetch(
    `/api/v1/institution/${encodeURIComponent(sfidId)}/documents/${docId}/download`,
    { headers: adminHeaders(auth) },
  );
  if (!resp.ok) throw new Error(`下载失败 (${resp.status})`);
  const blob = await resp.blob();
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = fileName;
  a.click();
  URL.revokeObjectURL(url);
}

/** 删除文档 */
export async function deleteDocument(
  auth: AdminAuth,
  sfidId: string,
  docId: number,
): Promise<void> {
  await adminRequest<string>(
    `/api/v1/institution/${encodeURIComponent(sfidId)}/documents/${docId}`,
    auth,
    { method: 'DELETE' },
  );
}
