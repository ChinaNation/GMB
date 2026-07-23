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
  main_account_id?: string | null;
  fee_account_id?: string | null;
};

export type AccountWithBalance = {
  accountName: string;
  /** 唯一账户 ID（小写 0x + 64 位十六进制）。 */
  account_id: string;
  /** 仅用于展示的 SS58 地址（GMB prefix=2027）。 */
  ss58_address: string;
  /** `frame_system::Account[address].data.free`,最小单位"分"。 */
  balanceMinUnits: string;
  /** 友好元字符串 `xxx.xx`。 */
  balanceText: string;
  accountKind: 'main' | 'fee' | 'stake' | 'safety_fund' | 'he' | 'named';
  canClose: boolean;
};

export type InstitutionDetail = {
  cidNumber: string;
  cidFullName: string;
  /** 机构码（CID institution_code，[u8;4] 序列化为数字数组）。 */
  institutionCode: number[];

  main_account_info: AccountWithBalance;
  fee_account_info: AccountWithBalance;
  /** 主账户/费用账户之外的其它账户(自定义初始账户)。 */
  otherAccounts: AccountWithBalance[];

  adminsLen: number;
  threshold: number;
  /** 管理员账户及其全部有效岗位任职。 */
  admins: InstitutionAdminInfo[];

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
  /** 代表签发机构的唯一 CID。 */
  actor_cid_number: string;
  /** 本次凭证签名所用管理员公钥（小写 0x + 64 位十六进制）。 */
  credential_signer_public_key: string;
  /** 业务作用域省名。 */
  scope_province_name: string;
  /** 业务作用域市名。 */
  scope_city_name: string;
  /** 签发管理员对凭证 payload 的 sr25519 签名(64 字节 hex)。 */
  signature: string;
  meta?: unknown;
};
