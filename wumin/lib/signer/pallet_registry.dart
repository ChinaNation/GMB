/// 链上 pallet / call 索引注册表。
///
/// 索引由 runtime 的 `construct_runtime!` 宏中声明顺序决定。
/// 链升级调整 pallet 顺序后，需同步更新此文件中的常量。
///
/// 防误签靠两色严格模式:decoder 解析失败 / display.action 与 decoded.action 不一致
/// 直接拒签,无需依赖 spec_version 门控(原 supportedSpecVersions / isSupported 已删,
/// 因为它跟 strict 模式重叠且阻塞合法新 spec)。
///
/// 投票引擎统一入口:
/// - 业务 pallet 不承载投票,管理员投票走 `InternalVote::cast`(22.0)
/// - 联合投票内部投票阶段走 `JointVote::cast_admin`(23.0),
///   联合公投阶段走 `JointVote::cast_referendum`(23.1)
/// - 引擎核心 `VotingEngine` (9) 仅承载 `finalize_proposal`(9.3) /
///   `retry_passed_proposal`(9.4) / `cancel_passed_proposal`(9.5)。
///
/// 手动执行重试/取消统一走 `VotingEngine::retry_passed_proposal`(9.4) 与
/// `VotingEngine::cancel_passed_proposal`(9.5),业务 pallet 不承载 wrapper extrinsic。
class PalletRegistry {
  const PalletRegistry._();

  // ---- Balances (2) ----
  static const int balancesPallet = 2;
  static const int transferKeepAliveCall = 3;

  // ---- VotingEngine (9) · 引擎核心 ----
  // 仅承载 lifecycle extrinsic:finalize_proposal / retry_passed_proposal /
  // cancel_passed_proposal。mode-specific 投票 extrinsic 在 InternalVote(22) /
  // JointVote(23) sub-pallet。
  static const int votingEnginePallet = 9;

  /// `finalize_proposal(proposal_id)` — 任意人触发终态执行(无需签投票)。
  static const int finalizeProposalCall = 3;

  /// `retry_passed_proposal(proposal_id)` — 已通过提案的手动执行入口。
  static const int retryPassedProposalCall = 4;

  /// `cancel_passed_proposal(proposal_id, reason)` — 已通过但确认不可执行的提案取消入口。
  static const int cancelPassedProposalCall = 5;

  // ---- InternalVote sub-pallet (22) · 内部投票管理员一人一票 ----
  static const int internalVotePallet = 22;

  /// `cast(proposal_id, approve)`。
  static const int internalVoteCall = 0;

  // ---- JointVote sub-pallet (23) · 联合投票(内部投票阶段 + 联合公投)----
  static const int jointVotePallet = 23;

  /// `cast_admin(proposal_id, subject_id_48, approve)` — 联合投票内部投票阶段。
  static const int jointVoteCall = 0;

  /// `cast_referendum(proposal_id, binding_id, nonce, signature, ...)` —
  /// 联合公投联合公投阶段(SFID 持有者投票)。
  static const int castReferendumCall = 1;

  // ---- 业务 pallet:仅承载提案创建与幂等兜底入口 ----
  //
  // 投票统一走 `InternalVote(22).cast(0)`,手动重试/取消统一走
  // `VotingEngine(9).retry_passed_proposal(4)` / `cancel_passed_proposal(5)`。

  // ---- DuoqianTransfer (19) ----
  // call_index 3/4/5 留洞不复用(原 execute_xxx wrapper 已物理删除)。
  static const int duoqianTransferPallet = 19;
  static const int proposeTransferCall = 0;
  static const int proposeSafetyFundCall = 1;
  static const int proposeSweepCall = 2;

  // ---- 协议升级 RuntimeUpgrade (13) ----
  static const int runtimeUpgradePallet = 13;
  static const int proposeRuntimeUpgradeCall = 0;
  static const int developerDirectUpgradeCall = 2;

  // ---- OrganizationManage (17) ----
  // call_index=0 留洞不复用(原 propose_create 单账户机构已物理删除)。
  // call_index=3 留洞不复用(propose_create_personal 已迁至 PersonalManage(7),B 阶段拆分 2026-05-06)。
  // 机构多签最少 2 账户,统一走 call_index=5。
  static const int organizationManagePallet = 17;
  static const int proposeCloseCall = 1;
  static const int registerSfidInstitutionCall = 2;
  static const int cleanupRejectedProposalCall = 4;

  /// `propose_create_institution(sfid_number, institution_name, accounts,
  /// admin_org, admin_count, duoqian_admins, threshold, register_nonce, signature,
  /// province, signer_admin_pubkey)` —
  /// 机构多签账户创建提案,凭证由 SFID 后端按 (province, admin_pubkey)
  /// 双层签发(ADR-008 step2b)。
  static const int proposeCreateInstitutionCall = 5;

  // ---- PersonalManage (7) ----
  // B 阶段拆分(2026-05-06):个人多签独立 pallet,MODULE_TAG = b"per-mgmt",
  // ACTION enum 独立命名空间(ACTION_CREATE=0/ACTION_CLOSE=1)。
  // propose_create(account_name, duoqian_admins, amount):
  // admin_count 由 admins 长度派生,threshold 由链端 admins-change 动态派生。
  static const int personalManagePallet = 7;
  static const int proposeCreatePersonalCall = 0;
  static const int proposeClosePersonalCall = 1;
  static const int cleanupRejectedPersonalProposalCall = 2;

  // ---- ResolutionDestro (14) ----
  // call_index 1 留洞不复用。
  static const int resolutionDestroPallet = 14;
  static const int proposeDestroyCall = 0;

  // ---- AdminsChange (12) ----
  // call_index 1 留洞不复用。
  static const int adminsChangePallet = 12;
  static const int proposeAdminSetChangeCall = 0;

  // ---- GrandpaKeyChange (16) ----
  // call_index 1, 2 留洞不复用。
  static const int grandpaKeyChangePallet = 16;
  static const int proposeReplaceGrandpaKeyCall = 0;

  // ---- ResolutionIssuance (8) ----
  static const int resolutionIssuancePallet = 8;
  static const int proposeResolutionIssuanceCall = 0;

  // ---- OnchainIssuance (25) · 链上发行代币(Plain FT, ADR-011 v3) ----
  // call_index 5..=9 / 15+ 留洞不复用(永久 ABI)。
  // 业务调用走 propose_X(InternalVote),监管调用走 propose_monitor_X(JointVote)。
  // 投票/重试/取消统一走 InternalVote(22)/JointVote(23)/VotingEngine(9.4/9.5)。
  static const int onchainIssuancePallet = 25;
  // 业务 propose
  static const int proposeIssueCall = 0;
  static const int proposeMintCall = 1;
  static const int proposeBurnCall = 2;
  static const int proposeCloseAssetCall = 3;
  static const int proposeAssetTransferCall = 4;
  // 监管 propose(NRC,JointVote)
  static const int proposeMonitorFreezeCall = 10;
  static const int proposeMonitorUnfreezeCall = 11;
  static const int proposeMonitorConfiscateCall = 12;
  static const int proposeMonitorForceTransferCall = 13;
  static const int proposeMonitorForceCloseCall = 14;

  // ---- OffchainTransaction (21) · 清算行 L2 体系 ----
  static const int offchainTransactionPallet = 21;
  static const int bindClearingBankCall = 30;
  static const int depositCall = 31;
  static const int withdrawCall = 32;
  static const int switchBankCall = 33;
  static const int submitOffchainBatchV2Call = 34;
  static const int proposeL2FeeRateCall = 40;
  static const int setMaxL2FeeRateCall = 41;
  static const int registerClearingBankCall = 50;
  static const int updateClearingBankEndpointCall = 51;
  static const int unregisterClearingBankCall = 52;
}
