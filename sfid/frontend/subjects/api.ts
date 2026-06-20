// 中文注释：主体共享类型。实际业务 API 必须放在
// gov/api.ts、private/<type>/api.ts、accounts/api.ts、docs/api.ts。
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

export type PrivateType =
  | 'SOLE'
  | 'PARTNERSHIP'
  | 'COMPANY'
  | 'CORPORATION'
  | 'WELFARE'
  | 'ASSOCIATION';

export type PartnershipKind = 'GENERAL' | 'LIMITED';

export type EducationType =
  | 'NATIONAL_CITIZEN_EDU_COMMITTEE'
  | 'CITY_CITIZEN_EDU_COMMITTEE'
  | 'EARLY_SCHOOL'
  | 'PRIMARY_SCHOOL'
  | 'SECONDARY_SCHOOL'
  | 'UNIVERSITY';

export interface Institution {
  sfid_number: string;
  /** 详情页展示全称;列表不得与简称同时展示。 */
  sfid_full_name?: string | null;
  /** 详情页展示简称;公权目录默认用简称作为 sfid_full_name。 */
  sfid_short_name?: string | null;
  /** 主体业务状态,只允许 ACTIVE / REVOKED。 */
  status: 'ACTIVE' | 'REVOKED';
  category: InstitutionCategory;
  subject_property: string;
  p1: string;
  province_name: string;
  city_name: string;
  town_name?: string;
  province_code: string;
  /** 任务卡 6 新增:2 位数字市代码(r5 段后 3 字符),作为公安局对账稳定主键 */
  city_code?: string;
  town_code?: string;
  institution_code: string;
  org_code?: string | null;
  /** 教育机构业务分类,只用于教育 tab 分类,不参与身份 ID 生成。 */
  education_type?: EducationType | null;
  /** 私权机构类型。仅私权目标类型机构有值。 */
  private_type?: PrivateType | null;
  /** 合伙企业形态。仅 private_type=PARTNERSHIP 时有值。 */
  partnership_kind?: PartnershipKind | null;
  /** 是否具有法人资格。仅私权目标类型机构有值。 */
  has_legal_personality?: boolean | null;
  /** 从属关系引用:需挂靠的 F 指向所属法人。 */
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

export interface InstitutionAccount {
  sfid_number: string;
  account_name: string;
  duoqian_account: string | null;
  chain_status: MultisigChainStatus;
  chain_synced_at?: string | null;
  chain_tx_hash: string | null;
  chain_block_number: number | null;
  created_by: string;
  created_at: string;
}

export interface InstitutionListRow {
  sfid_number: string;
  sfid_full_name?: string | null;
  sfid_short_name?: string | null;
  status: 'ACTIVE' | 'REVOKED';
  category: InstitutionCategory;
  subject_property: string;
  p1: string;
  province_name: string;
  city_name: string;
  town_name?: string;
  institution_code: string;
  org_code?: string | null;
  education_type?: EducationType | null;
  private_type?: PrivateType | null;
  partnership_kind?: PartnershipKind | null;
  has_legal_personality?: boolean | null;
  parent_sfid_number?: string | null;
  account_count: number;
  cpms_status?: string | null;
  install_token_status?: string | null;
  identity_service_status?: string | null;
  created_at: string;
  /** 创建该机构的登录管理员姓名(按 created_by pubkey 反查 admin_users);未命中 null */
  created_by_name?: string | null;
  /** 创建者角色:FEDERAL_ADMIN / CITY_ADMIN;未命中 null */
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
  institution: Institution;
  accounts: InstitutionAccount[];
  /** 创建该机构的登录管理员姓名(按 created_by pubkey 反查 admin_users) */
  created_by_name?: string | null;
  /** 创建者角色:FEDERAL_ADMIN / CITY_ADMIN */
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
  province_name?: string;
  city_name: string;
  institution: string;
  /** 教育机构业务分类。仅 G/S 学校创建时提交;F+JY 分校不使用。 */
  education_type?: EducationType;
  /**
   * 机构名称。
   * - 私权机构创建时必填,由对应私权类型 tab 锁定身份编码
   * - 法人教育机构(G/S+JY)和手动公权机构(G):**必传**,同步做查重
   * - 自动公权机构/公安局:不走手动创建接口
  */
  sfid_full_name?: string;
  /**
   * 所属法人 sfid_number。仅需挂靠的非法人创建必传;个体经营/无限合伙是独立非法人,
   * 不接受所属法人。规则单一源:后端 subjects/uninorg。
   */
  parent_sfid_number?: string;
  private_type?: PrivateType;
  partnership_kind?: PartnershipKind;
  legal_rep_name?: string;
  legal_rep_sfid_number?: string;
  legal_rep_photo_path?: string;
  legal_rep_photo_name?: string;
  legal_rep_photo_mime?: string;
  legal_rep_photo_size?: number;
}

export interface CreateInstitutionOutput {
  sfid_number: string;
  /** 创建成功后的机构名称。 */
  sfid_full_name: string | null;
  category: InstitutionCategory;
}

/** 机构详情页可编辑字段。私权类型创建后不可改。 */
export interface UpdateInstitutionInput {
  sfid_full_name?: string | null;
  sfid_short_name?: string | null;
  /** 所属法人 sfid_number(仅 F;传空串后端会拒) */
  parent_sfid_number?: string;
  legal_rep_name?: string;
  legal_rep_sfid_number?: string;
  legal_rep_photo_path?: string;
  legal_rep_photo_name?: string;
  legal_rep_photo_mime?: string;
  legal_rep_photo_size?: number;
}

/** 法人机构搜索结果项(非法人新增弹窗 + F 详情页"所属法人"选择器用) */
export interface ParentInstitutionRow {
  sfid_number: string;
  sfid_full_name: string;
  subject_property: string;
  /** 私权机构类型。仅用于展示父级机构事实。 */
  private_type?: PrivateType | null;
  partnership_kind?: PartnershipKind | null;
  category: InstitutionCategory;
  /** 盈利属性。F 创建时按"盈利属性附属于所属法人"推导:公法人父恒 0,私法人父继承该值 */
  p1: string;
  province_name: string;
  city_name: string;
}

/** 所属法人搜索参数。预过滤规则与后端 subjects/uninorg 同源(三处同源缺一有绕过口)。 */
export interface SearchParentsOptions {
  /** 非法人的机构代码:JY=教育分校,GT/GP=私权独立非法人,其它 F 由后端判定是否需挂靠 */
  fInstitution: string;
  /** 非法人落位省/市,用于地域预过滤(市级同市/省级同省/国家级不限) */
  province_name: string;
  city_name: string;
  /** 限定父级属性:S=私权入口 / G=公权入口;不传=两者(详情页改挂) */
  parentProperty?: 'S' | 'G';
}

export interface CreateAccountOutput {
  sfid_number: string;
  account_name: string;
  chain_status: MultisigChainStatus;
  chain_synced_at: string | null;
  chain_tx_hash: string | null;
  chain_block_number: number | null;
  duoqian_account: string | null;
}

export interface ListInstitutionsQuery {
  category?: InstitutionCategory;
  province_name?: string;
  city_name?: string;
  /** 精确搜索关键字:匹配机构名称或 SFID;空=返回空页 */
  q?: string;
  private_type?: PrivateType;
  cursor?: string | null;
  page_size?: number;
}
