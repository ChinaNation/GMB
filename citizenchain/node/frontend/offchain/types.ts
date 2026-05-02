// 清算行 tab 相关 DTO 与状态机类型,与 Tauri 后端 offchain/types.rs 对齐。

export type EligibleClearingBankCandidate = {
  sfidId: string;
  institutionName: string;
  a3: string;
  subType?: string | null;
  parentSfidId?: string | null;
  parentInstitutionName?: string | null;
  parentA3?: string | null;
  province: string;
  city: string;
  /** 主账户当前链上状态:Inactive / Pending / Registered / Failed。 */
  mainChainStatus: 'Inactive' | 'Pending' | 'Registered' | 'Failed';
  mainAccount?: string | null;
  feeAccount?: string | null;
};

// ── 机构详情(链上 duoqian-manage::Institutions[sfid_id] 的对前端形态)──

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
  sfidId: string;
  institutionName: string;

  mainAccount: AccountWithBalance;
  feeAccount: AccountWithBalance;
  /** 主账户/费用账户之外的其它账户(自定义初始账户)。 */
  otherAccounts: AccountWithBalance[];

  adminCount: number;
  threshold: number;
  /** 管理员公钥 32B 的 SS58 列表。 */
  duoqianAdminsSs58: string[];

  /** 机构生命周期:Pending(投票中)/ Active(已生效)/ Closed(已注销)。 */
  status: 'Pending' | 'Active' | 'Closed';
  creatorSs58: string;
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

/** SFID `/registration-info` 响应形态(snake_case 直传)。 */
export type InstitutionRegistrationInfoResp = {
  sfid_id: string;
  institution_name: string;
  account_names: string[];
  credential: InstitutionRegistrationCredentialResp;
};

/** SFID 对链上注册 payload 签发的凭证。 */
export type InstitutionRegistrationCredentialResp = {
  genesis_hash: string;
  province: string;
  /** 防重放 nonce(本次响应生成的随机字符串)。 */
  register_nonce: string;
  /** 本次签名所用省管理员公钥(32 字节 hex)。 */
  signer_admin_pubkey: string;
  /** 省级签名密钥对凭证 payload 的 sr25519 签名(64 字节 hex)。 */
  signature: string;
  meta?: unknown;
};

/** 创建机构时单账户的初始资金条目(单位"分"用字符串透传 BigInt)。 */
export type InitialAccountInputDto = {
  accountName: string;
  /** u128 字符串形式,单位"分"。 */
  amountFen: string;
};

export type ClearingBankNodeOnChainInfo = {
  sfidId: string;
  peerId: string;
  rpcDomain: string;
  rpcPort: number;
  registeredAt: number;
  registeredByPubkeyHex: string;
  registeredBySs58: string;
};

export type ConnectivityCheck = {
  label: string;
  ok: boolean;
  detail?: string | null;
};

export type ConnectivityTestReport = {
  allOk: boolean;
  checks: ConnectivityCheck[];
};

export type DecryptedAdminInfo = {
  pubkeyHex: string;
  sfidId: string;
  decryptedAtMs: number;
};

export type DecryptAdminRequestResult = {
  requestJson: string;
  requestId: string;
  expectedPayloadHash: string;
  payloadHex: string;
};

/**
 * offchain/section.tsx 状态机(2026-05-01 重构)。
 *
 *   empty                        初始 — 顶部 ＋添加清算行 按钮
 *   add-input-sfid               输入 sfid_id 或机构名,debounce 自动搜 SFID 候选
 *   check-multisig               链上查 Institutions[sfid_id]
 *                                  ├─ 已存在 → institution-detail
 *                                  └─ 不存在 → create-multisig-institution
 *   institution-detail           机构详情卡片栅格 + 折叠子页入口 + 节点信息内联
 *   other-accounts-list          其他账户列表子页(折叠卡片入口)
 *   admin-list                   管理员列表子页(折叠卡片入口)
 *   create-multisig-institution  创建机构多签 propose_create_institution(冷钱包签 + 提交)
 *   wait-vote                    轮询 status === 'Active'(等其他管理员投票通过)
 *   declare-node                 多签 Active 但未声明节点 → 填 RPC + 自测 + 签名声明
 */
export type ClearingBankView =
  | { kind: 'empty' }
  | { kind: 'add-input-sfid' }
  | { kind: 'check-multisig'; sfidId: string; institutionName: string }
  | { kind: 'institution-detail'; sfidId: string }
  | { kind: 'create-multisig-institution'; sfidId: string }
  | { kind: 'wait-vote'; sfidId: string; institutionName: string }
  | { kind: 'declare-node'; sfidId: string; institutionName: string }
  | { kind: 'other-accounts-list'; sfidId: string; otherAccounts: AccountWithBalance[] }
  | { kind: 'admin-list'; sfidId: string; admins: string[]; threshold: number; adminCount: number };
