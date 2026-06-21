// 中文注释:DUOQIAN 链交互查询 API。机构创建/机构资料维护仍归属
// `frontend/subjects/`;本目录只承载链端 pull CID 信息所需的前端封装。

type ApiEnvelope<T> = {
  code: number;
  message: string;
  data: T;
};

export interface InstitutionInfoDetail {
  cid_number: string;
  cid_full_name?: string | null;
  category: string;
  subject_property: string;
  p1: string;
  province_name: string;
  city_name: string;
  province_code: string;
  city_code: string;
  institution_code: string;
  private_type?: string | null;
  partnership_kind?: string | null;
  has_legal_personality?: boolean | null;
  parent_cid_number?: string | null;
}

export interface InstitutionRegistrationCredential {
  genesis_hash: string;
  register_nonce: string;
  province_name: string;
  signer_pubkey: string;
  signature: string;
  meta: {
    key_id: string;
    key_version: string;
    alg: 'sr25519' | string;
  };
}

export interface InstitutionRegistrationInfo {
  /** 中文注释:链端注册业务字段 1/3。 */
  cid_number: string;
  /** 中文注释:链端注册业务字段 2/3。 */
  cid_full_name: string;
  /** 中文注释:链端注册业务字段 3/3,顺序必须原样交给链端验签。 */
  account_names: string[];
  /** 中文注释:只用于链端验签与防重放,不属于业务注册字段。 */
  credential: InstitutionRegistrationCredential;
}

async function publicAppRequest<T>(path: string): Promise<T> {
  let resp: Response;
  try {
    resp = await fetch(path);
  } catch (error) {
    const msg = error instanceof Error ? error.message : String(error);
    throw new Error(`无法连接服务器：${msg}`);
  }

  const text = await resp.text();
  let body: ApiEnvelope<T> | null = null;
  try {
    body = text ? (JSON.parse(text) as ApiEnvelope<T>) : null;
  } catch {
    const snippet = text.slice(0, 120);
    throw new Error(`服务响应格式错误(${resp.status})：${snippet || 'empty body'}`);
  }
  if (!resp.ok || !body || body.code !== 0) {
    throw new Error(body?.message ?? `request failed (${resp.status})`);
  }
  return body.data;
}

export async function getInstitutionInfo(cidNumber: string): Promise<InstitutionInfoDetail> {
  const encoded = encodeURIComponent(cidNumber);
  return publicAppRequest<InstitutionInfoDetail>(`/api/v1/app/institutions/${encoded}`);
}

export async function getInstitutionRegistrationInfo(
  cidNumber: string,
): Promise<InstitutionRegistrationInfo> {
  const encoded = encodeURIComponent(cidNumber);
  return publicAppRequest<InstitutionRegistrationInfo>(
    `/api/v1/app/institutions/${encoded}/registration-info`,
  );
}
