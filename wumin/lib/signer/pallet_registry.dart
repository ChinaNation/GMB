/// 链上 pallet / call 索引注册表。
///
/// 索引由 runtime 的 `construct_runtime!` 宏中声明顺序决定。
/// 链升级调整 pallet 顺序后，需同步更新此文件中的常量。
///
/// [supportedSpecVersions] 列出当前注册表适配的 spec_version 集合。
/// 离线设备收到未知 spec_version 时应拒绝解码，提示用户升级冷钱包。
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

  /// 当前注册表适配的链 spec_version 集合。
  /// 遇到旧 spec 的离线请求视为过期,拒绝解码。
  static const Set<int> supportedSpecVersions = {0};

  /// 检查给定 spec_version 是否与当前注册表兼容。
  ///
  /// - 返回 `true`：可安全解码
  /// - 返回 `false`：spec_version 未知，解码可能错位
  /// - [specVersion] 为 null 时（旧版在线端未发送），返回 `false`
  static bool isSupported(int? specVersion) {
    if (specVersion == null) return false;
    return supportedSpecVersions.contains(specVersion);
  }

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
  /// `cast_admin(proposal_id, institution_id_48, approve)` — 联合投票内部投票阶段。
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

  // ---- RuntimeUpgrade (13) ----
  static const int runtimeUpgradePallet = 13;
  static const int proposeRuntimeUpgradeCall = 0;
  static const int developerDirectUpgradeCall = 2;

  // ---- DuoqianManage (17) ----
  // call_index=0 留洞不复用(原 propose_create 单账户机构已物理删除)。
  // 机构多签最少 2 账户,统一走 call_index=5。
  static const int duoqianManagePallet = 17;
  static const int proposeCloseCall = 1;
  static const int registerSfidInstitutionCall = 2;
  static const int proposeCreatePersonalCall = 3;
  static const int cleanupRejectedProposalCall = 4;

  /// `propose_create_institution(sfid_id, institution_name, accounts,
  /// admin_count, duoqian_admins, threshold, register_nonce, signature,
  /// province, signer_admin_pubkey, a3, sub_type, parent_sfid_id)` —
  /// 机构多签账户创建提案,凭证由 SFID 后端按 (province, admin_pubkey)
  /// 双层签发(ADR-008 step2b)。
  static const int proposeCreateInstitutionCall = 5;

  // ---- ResolutionDestro (14) ----
  // call_index 1 留洞不复用。
  static const int resolutionDestroPallet = 14;
  static const int proposeDestroyCall = 0;

  // ---- AdminsChange (12) ----
  // call_index 1 留洞不复用。
  static const int adminsChangePallet = 12;
  static const int proposeAdminReplacementCall = 0;

  // ---- GrandpaKeyChange (16) ----
  // call_index 1, 2 留洞不复用。
  static const int grandpaKeyChangePallet = 16;
  static const int proposeReplaceGrandpaKeyCall = 0;

  // ---- ResolutionIssuance (8) ----
  static const int resolutionIssuancePallet = 8;
  static const int proposeResolutionIssuanceCall = 0;

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
