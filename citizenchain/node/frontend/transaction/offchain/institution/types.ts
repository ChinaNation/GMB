import type { InstitutionAdminInfo } from '../../../governance/types';

// 清算行机构身份只读 DTO,与 Tauri 后端 transaction/offchain_transaction/institution_read/types.rs 对齐。

export type EligibleClearingBankCandidate = {
  cidNumber: string;
  cidFullName: string;
  refProperty: string;
  subType?: string | null;
  parentCidNumber?: string | null;
  parentCidFullName?: string | null;
  parentRefProperty?: string | null;
  provinceName: string;
  cityName: string;
  /** 主账户当前链上状态:Pending / Active / Closed / Failed。 */
  mainChainStatus: 'Pending' | 'Active' | 'Closed' | 'Failed';
  mainAccount?: string | null;
  feeAccount?: string | null;
};

export type AccountWithBalance = {
  accountName: string;
  /** 32 字节链上地址的 SS58 形式(GMB prefix=2027)。 */
  addressSs58: string;
  /** `frame_system::Account[address].data.free`,最小单位"分"。 */
  balanceMinUnits: string;
  /** 友好元字符串 `xxx.xx`。 */
  balanceText: string;
  isDefault: boolean;
};

export type InstitutionDetail = {
  cidNumber: string;
  cidFullName: string;
  /** 机构管理员读取使用的机构多签 AccountId，清算行指向主账户。 */
  adminAccountHex: string;
  /** 机构码（CID institution_code，[u8;4] 序列化为数字数组）。 */
  institutionCode: number[];

  mainAccount: AccountWithBalance;
  feeAccount: AccountWithBalance;
  /** 主账户/费用账户之外的其它账户(自定义初始账户)。 */
  otherAccounts: AccountWithBalance[];

  adminsLen: number;
  threshold: number;
  /** 管理员钱包及其全部有效岗位任职。 */
  admins: InstitutionAdminInfo[];

  /** 机构生命周期:Pending(投票中)/ Active(已生效)/ Closed(已注销)。 */
  status: 'Pending' | 'Active' | 'Closed';
  createdAt: number;
  accountCount: number;
};

export type InstitutionProposalItem = {
  proposalId: number;
  kindLabel: string;
  statusLabel: string;
  summary: string;
};

export type InstitutionProposalPage = {
  items: InstitutionProposalItem[];
  hasMore: boolean;
};

/** CID `/registration-info` 响应形态(snake_case 直传)。 */
export type InstitutionRegistrationInfoResp = {
  cid_number: string;
  cid_full_name: string;
  account_names: string[];
  credential: InstitutionRegistrationCredentialResp;
};

/** CID 对链上注册 payload 签发的凭证。 */
export type InstitutionRegistrationCredentialResp = {
  genesis_hash: string;
  /** 防重放 nonce(本次响应生成的随机字符串)。 */
  register_nonce: string;
  /** 签发机构 CID 号。 */
  issuer_cid_number: string;
  /** 签发机构主账户(32 字节 hex)。 */
  issuer_main_account: string;
  /** 本次签名所用机构管理员公钥(32 字节 hex)。 */
  signer_pubkey: string;
  /** 业务作用域省名。 */
  scope_province_name: string;
  /** 业务作用域市名。 */
  scope_city_name: string;
  /** 签发管理员对凭证 payload 的 sr25519 签名(64 字节 hex)。 */
  signature: string;
  meta?: unknown;
};
