// 清算行 offchain 网络 DTO 与页面状态机类型。
//
// 中文注释:
// - 机构多签 DTO 已归位到 governance/organization-manage/types.ts。
// - 本文件只保留清算行节点声明、连通性检测、管理员解锁和 offchain 入口状态机。

import type { AdminWalletMatch } from '../../governance/types';
import type { AccountWithBalance } from '../../governance/organization-manage/types';

export type {
  AccountWithBalance,
  EligibleClearingBankCandidate,
  InstitutionDetail,
} from '../../governance/organization-manage/types';

export type ClearingBankNodeOnChainInfo = {
  sfidNumber: string;
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
  sfidNumber: string;
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
 *   empty                        初始 — 顶部 +添加清算行 按钮
 *   add-input-sfid               输入 sfid_number 或机构名,debounce 自动搜 SFID 候选
 *   check-multisig               链上查 Institutions[sfid_number]
 *                                  ├─ 已存在 -> institution-detail
 *                                  └─ 不存在 -> create-multisig-institution
 *   institution-detail           机构详情卡片栅格 + 折叠子页入口 + 节点信息内联
 *   admin-set-change             进入 admins_change 管理员更换流程
 *   other-accounts-list          其他账户列表子页(折叠卡片入口)
 *   admin-list                   管理员列表子页(折叠卡片入口)
 *   create-multisig-institution  创建机构多签 propose_create_institution(冷钱包签 + 提交)
 *   wait-vote                    轮询 status === 'Active'(等其他管理员投票通过)
 *   declare-node                 多签 Active 但未声明节点 -> 填 RPC + 自测 + 签名声明
 */
export type ClearingBankView =
  | { kind: 'empty' }
  | { kind: 'add-input-sfid' }
  | { kind: 'check-multisig'; sfidNumber: string; institutionName: string }
  | { kind: 'institution-detail'; sfidNumber: string }
  | { kind: 'admin-set-change'; sfidNumber: string; institutionName: string; adminWallets: AdminWalletMatch[] }
  | { kind: 'create-multisig-institution'; sfidNumber: string }
  | { kind: 'wait-vote'; sfidNumber: string; institutionName: string }
  | { kind: 'declare-node'; sfidNumber: string; institutionName: string }
  | { kind: 'other-accounts-list'; sfidNumber: string; otherAccounts: AccountWithBalance[] }
  | { kind: 'admin-list'; sfidNumber: string; admins: string[]; threshold: number; adminCount: number };
