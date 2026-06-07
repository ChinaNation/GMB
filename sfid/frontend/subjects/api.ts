// 中文注释：主体共享类型。实际业务 API 必须放在
// gov/api.ts、private/api.ts、accounts/api.ts、docs/api.ts。
// 铁律:一个 sfid_number 下可挂多个 account_name。

export type InstitutionCategory = 'PUBLIC_SECURITY' | 'GOV_INSTITUTION' | 'PRIVATE_INSTITUTION';

export const InstitutionCategoryLabel: Record<InstitutionCategory, string> = {
  PUBLIC_SECURITY: '公安局',
  GOV_INSTITUTION: '公权机构',
  PRIVATE_INSTITUTION: '私权机构',
};

export type MultisigChainStatus =
  | 'NOT_ON_CHAIN'
  | 'PENDING_ON_CHAIN'
  | 'ACTIVE_ON_CHAIN'
  | 'REVOKED_ON_CHAIN';

export interface MultisigInstitution {
  sfid_number: string;
  /** 机构名称。两步式创建(2026-04-19):第一步生成时为 null,详情页补填后非空。 */
  institution_name: string | null;
  /** 详情页展示全称;列表不得与简称同时展示。 */
  full_name?: string | null;
  /** 详情页展示简称;公权目录默认用简称作为 institution_name。 */
  short_name?: string | null;
  /** 主体业务状态,只允许 ACTIVE / REVOKED。 */
  status: 'ACTIVE' | 'REVOKED';
  category: InstitutionCategory;
  subject_property: string;
  p1: string;
  province: string;
  city: string;
  town?: string;
  province_code: string;
  /** 任务卡 6 新增:2 位数字市代码(r5 段后 3 字符),作为公安局对账稳定主键 */
  city_code?: string;
  town_code?: string;
  institution_code: string;
  org_code?: string | null;
  /** 私法人子类型(仅 SubjectProperty=S 且 P1 填完后才有值) */
  sub_type?: string | null;
  /** 所属法人 sfid_number(仅 SubjectProperty=F 非法人必填;指向 S/G) */
  parent_sfid_number?: string | null;
  /** 法定代表人资料。初始化目录机构可为空;编辑保存时必填。 */
  legal_rep_name?: string | null;
  legal_rep_sfid_number?: string | null;
  legal_rep_photo_path?: string | null;
  legal_rep_photo_name?: string | null;
  legal_rep_photo_mime?: string | null;
  legal_rep_photo_size?: number | null;
  created_by: string;
  created_at: string;
}

export interface MultisigAccount {
  sfid_number: string;
  account_name: string;
  duoqian_address: string | null;
  chain_status: MultisigChainStatus;
  chain_synced_at?: string | null;
  chain_tx_hash: string | null;
  chain_block_number: number | null;
  created_by: string;
  created_at: string;
}

export interface InstitutionListRow {
  sfid_number: string;
  /** 两步式创建:第一步仅有 SFID 时为 null,详情页补填后非空 */
  institution_name: string | null;
  full_name?: string | null;
  short_name?: string | null;
  status: 'ACTIVE' | 'REVOKED';
  category: InstitutionCategory;
  subject_property: string;
  p1: string;
  province: string;
  city: string;
  town?: string;
  institution_code: string;
  org_code?: string | null;
  sub_type?: string | null;
  parent_sfid_number?: string | null;
  account_count: number;
  cpms_status?: string | null;
  install_token_status?: string | null;
  identity_service_status?: string | null;
  created_at: string;
  /** 创建该机构的登录管理员姓名(按 created_by pubkey 反查 admin_users);未命中 null */
  created_by_name?: string | null;
  /** 创建者角色:FEDERAL_ADMIN / SHI_ADMIN;未命中 null */
  created_by_role?: string | null;
}

export interface PageResult<T> {
  items: T[];
  page_size: number;
  next_cursor?: string | null;
  has_more: boolean;
  /** 确定性目录版本。普通分页接口没有该字段。 */
  manifest_version?: string | null;
  /** 确定性目录状态。普通分页接口没有该字段。 */
  catalog_status?: string | null;
}

export interface InstitutionDetail {
  institution: MultisigInstitution;
  accounts: MultisigAccount[];
  /** 创建该机构的登录管理员姓名(按 created_by pubkey 反查 admin_users) */
  created_by_name?: string | null;
  /** 创建者角色:FEDERAL_ADMIN / SHI_ADMIN */
  created_by_role?: string | null;
}

/** 机构资料库文档 */
export interface InstitutionDocument {
  id: number;
  sfid_number: string;
  file_name: string;
  doc_type: string;
  file_size: number;
  uploaded_by: string;
  uploaded_at: string;
}

export interface LegalRepresentativePhoto {
  file_path: string;
  file_name: string;
  mime_type: string;
  file_size: number;
}

// ─── 请求 DTO ─────────────────────────────────────────────────

export interface CreateInstitutionInput {
  subject_property: string;
  p1?: string;
  province?: string;
  city: string;
  institution: string;
  /**
   * 机构名称。
   * - 私权(S/F)两步式:**不传**(或 undefined),由详情页 updateInstitution 补填
   * - 教育委员会(JY)手动新增学校机构:**必传**,同步做查重
   * - 自动公权机构/公安局:不走手动创建接口
  */
  institution_name?: string;
  legal_rep_name?: string;
  legal_rep_sfid_number?: string;
  legal_rep_photo_path?: string;
  legal_rep_photo_name?: string;
  legal_rep_photo_mime?: string;
  legal_rep_photo_size?: number;
}

export interface CreateInstitutionOutput {
  sfid_number: string;
  /** 首次创建:普通私权为 null,教育委员会(JY)学校机构为已填入的学校名称 */
  institution_name: string | null;
  category: InstitutionCategory;
}

/** 机构详情页可编辑字段(两步式第二步) */
export interface UpdateInstitutionInput {
  institution_name?: string;
  full_name?: string | null;
  short_name?: string | null;
  sub_type?: string | null;
  /** 所属法人 sfid_number(仅 F;传空串后端会拒) */
  parent_sfid_number?: string;
  legal_rep_name?: string;
  legal_rep_sfid_number?: string;
  legal_rep_photo_path?: string;
  legal_rep_photo_name?: string;
  legal_rep_photo_mime?: string;
  legal_rep_photo_size?: number;
}

/** 法人机构搜索结果项(F 详情页"所属法人"选择器用) */
export interface ParentInstitutionRow {
  sfid_number: string;
  institution_name: string;
  subject_property: string;
  /** 私法人子类型(仅 subject_property=S);F 判断父 S 是否 JOINT_STOCK 以显示清算行设置 */
  sub_type?: string | null;
  category: InstitutionCategory;
  province: string;
  city: string;
}

export interface CreateAccountOutput {
  sfid_number: string;
  account_name: string;
  chain_status: MultisigChainStatus;
  chain_synced_at: string | null;
  chain_tx_hash: string | null;
  chain_block_number: number | null;
  duoqian_address: string | null;
}

export interface ListInstitutionsQuery {
  category?: InstitutionCategory;
  province?: string;
  city?: string;
  /** 精确搜索关键字:匹配机构名称或 SFID;空=返回空页 */
  q?: string;
  cursor?: string | null;
  page_size?: number;
}
