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
 * offchain/section.tsx 状态机,8 步串联完成"添加清算行"流程。
 *
 *   empty            初始 — 顶部 +添加 按钮
 *   add-input-sfid   输入 sfid_id 或搜索机构名
 *   check-status     调链 + SFID 综合判定状态
 *   register-sfid    SFID 端未注册 → 提示去 sfid 系统注册
 *   propose-create   多签账户未创建 → 走 propose_create 投票
 *   wait-vote        提案已发起,等其他管理员投票
 *   declare-node     多签 Active 但未声明节点 → 填 RPC + 自测 + 签名
 *   detail           已声明节点 → 复用治理详情页 + 节点信息卡 + 端点更新/注销入口
 */
export type ClearingBankView =
  | { kind: 'empty' }
  | { kind: 'add-input-sfid' }
  | { kind: 'check-status'; sfidId: string }
  | { kind: 'register-sfid'; candidate: EligibleClearingBankCandidate }
  | { kind: 'propose-create'; candidate: EligibleClearingBankCandidate }
  | { kind: 'wait-vote'; sfidId: string; institutionName: string }
  | { kind: 'declare-node'; sfidId: string; institutionName: string }
  | { kind: 'detail'; sfidId: string };
