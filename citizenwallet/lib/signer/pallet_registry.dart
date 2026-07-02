/// 链上 pallet / call 索引注册表。
///
/// 索引由 runtime 的 `construct_runtime!` 宏中声明顺序决定。
/// 链升级调整 pallet 顺序后，需同步更新此文件中的常量。
///
/// 防误签靠两色严格模式:decoder 解析失败 / QR action 与 payload 解码动作不一致
/// 直接红色拒签。
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

  // ---- CitizenIdentity (10) · 公民链上投票身份 ----
  static const int citizenIdentityPallet = 10;

  /// `register_voting_identity(registrar_account, payload, citizen_signature)`。
  static const int registerVotingIdentityCall = 0;

  // ---- InternalVote sub-pallet (22) · 内部投票管理员一人一票 ----
  static const int internalVotePallet = 22;

  /// `cast(proposal_id, approve)`。
  static const int internalVoteCall = 0;

  // ---- JointVote sub-pallet (23) · 联合投票(内部投票阶段 + 联合公投)----
  static const int jointVotePallet = 23;

  /// `cast_admin(proposal_id, institution_account, approve)` — 联合投票内部投票阶段。
  static const int jointVoteCall = 0;

  /// `cast_referendum(proposal_id, approve)` — 联合公投阶段,链上按账户读取公民身份。
  static const int castReferendumCall = 1;

  /// `prepare_joint_population_snapshot(scope)` — 联合公投提案发起前准备链上人口分母。
  static const int prepareJointPopulationSnapshotCall = 2;

  // ---- 业务 pallet:仅承载提案创建与幂等兜底入口 ----
  //
  // 投票统一走 `InternalVote(22).cast(0)`,手动重试/取消统一走
  // `VotingEngine(9).retry_passed_proposal(4)` / `cancel_passed_proposal(5)`。

  // ---- MultisigTransfer (19) ----
  // call_index 3/4/5 留洞不复用。
  static const int multisigTransferPallet = 19;
  static const int proposeTransferCall = 0;
  static const int proposeSafetyFundCall = 1;
  static const int proposeSweepCall = 2;

  // ---- 协议升级 RuntimeUpgrade (13) ----
  static const int runtimeUpgradePallet = 13;
  static const int proposeRuntimeUpgradeCall = 0;
  static const int developerDirectUpgradeCall = 2;

  // ---- PublicManage (32) / PrivateManage (33) ----
  // 公权机构与私权机构生命周期分别由两个 pallet 承载。
  static const int publicManagePallet = 32;
  static const int privateManagePallet = 33;
  static const int proposeCloseInstitutionCall = 1;
  static const int registerCidInstitutionCall = 2;
  static const int cleanupRejectedInstitutionProposalCall = 4;

  /// `propose_create_*_institution(cid_number, cid_full_name, accounts,
  /// institution_code, admins_len, admins, threshold, register_nonce,
  /// signature, issuer_cid_number, issuer_main_account, signer_pubkey,
  /// scope_*)` — 机构多签账户创建提案。
  static const int proposeCreateInstitutionCall = 5;

  // ---- PersonalAdmins (7) ----
  // 个人多签独立 pallet,MODULE_TAG = b"per-mgmt",
  // ACTION enum 独立命名空间(ACTION_CREATE=0/ACTION_CLOSE=1)。
  // propose_create(account_name, admins, regular_threshold, amount):
  // admins_len 由 admins 长度派生,regular_threshold 由用户输入且必须严格过半。
  static const int personalAdminsPallet = 7;
  static const int proposeCreatePersonalCall = 0;
  static const int proposeClosePersonalCall = 1;
  static const int cleanupRejectedPersonalProposalCall = 2;
  static const int proposePersonalAdminSetChangeCall = 3;

  // ---- ResolutionDestro (14) ----
  // call_index 1 留洞不复用。
  static const int resolutionDestroPallet = 14;
  static const int proposeDestroyCall = 0;

  // ---- 管理员集合变更:PublicAdmins(29) / PrivateAdmins(30) ----
  // PersonalAdmins(7) 的管理员集合变更使用 call_index=3。
  static const int publicAdminsPallet = 29;
  static const int privateAdminsPallet = 30;
  static const int proposeAdminSetChangeCall = 0;

  static bool isAdminSetChangePallet(int palletIndex) {
    return palletIndex == publicAdminsPallet ||
        palletIndex == privateAdminsPallet;
  }

  static bool isPersonalAdminSetChangeCall(int palletIndex, int callIndex) {
    return palletIndex == personalAdminsPallet &&
        callIndex == proposePersonalAdminSetChangeCall;
  }

  // ---- GrandpaKeyChange (16) ----
  // call_index 1, 2 留洞不复用。
  static const int grandpaKeyChangePallet = 16;
  static const int proposeReplaceGrandpaKeyCall = 0;

  // ---- ResolutionIssuance (8) ----
  static const int resolutionIssuancePallet = 8;
  static const int proposeResolutionIssuanceCall = 0;

  // ---- OnchainIssuance (25) · 链上发行代币(Plain FT) ----
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

  // ---- LegislationYuan (27) · 立法院(立法/修法/废法发起)----
  // 法律结构化上链(章>节>条>款),发起类提案 QR 由节点端生成,冷钱包仅解码核对。
  // 投票/签署统一走 LegislationVote(28),本 pallet 仅承载 3 条 propose_X。
  static const int legislationYuanPallet = 27;
  static const int proposeEnactLawCall = 0;
  static const int proposeAmendLawCall = 1;
  static const int proposeRepealLawCall = 2;

  // ---- LegislationVote (28) · 立法专属投票引擎 ----
  // 立法投票阶段:院内表决 / 特别案公投 / 行政签署 / 三人会签 / 护宪终审 / 准备人口快照。
  static const int legislationVotePallet = 28;
  static const int prepareLegislationSnapshotCall = 0;
  static const int castHouseVoteCall = 1;
  static const int castLegislationReferendumCall = 2;
  static const int executiveSignCall = 3;
  static const int overrideSignCall = 4;
  static const int guardVoteCall = 5;

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
