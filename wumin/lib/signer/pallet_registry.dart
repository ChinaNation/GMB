/// 链上 pallet / call 索引注册表。
///
/// 索引由 runtime 的 `construct_runtime!` 宏中声明顺序决定。
/// 链升级调整 pallet 顺序后，需同步更新此文件中的常量。
///
/// [supportedSpecVersions] 列出当前注册表适配的 spec_version 集合。
/// 离线设备收到未知 spec_version 时应拒绝解码，提示用户升级冷钱包。
///
/// Phase 3 · 投票引擎统一入口整改（2026-04-22）：
/// - 业务 pallet 的 `vote_X` 全部下线，所有管理员投票走
///   `VotingEngineSystem::internal_vote`（9.0）。
/// - `joint_vote` / `citizen_vote` / `finalize_proposal` 在投票引擎内部
///   重新排 call_index：0=internal_vote / 1=joint_vote / 2=citizen_vote /
///   3=finalize_proposal。
class PalletRegistry {
  const PalletRegistry._();

  /// 当前注册表适配的链 spec_version 集合。
  ///
  /// Phase 3 runtime 升级到 `spec_version = 2`，冷钱包同步仅接受 2。
  /// 遇到 spec=1 的离线请求（旧版在线端）视为过期，拒绝解码。
  static const Set<int> supportedSpecVersions = {2};

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

  // ---- VotingEngineSystem (9) · 所有治理投票唯一入口 ----
  static const int votingEngineSystemPallet = 9;

  /// `internal_vote(proposal_id, approve)` — 管理员一人一票，
  /// 覆盖所有业务 pallet 的内部投票（admins/resolution_destro/grandpa_key/
  /// duoqian_manage/duoqian_transfer 五路）。
  static const int internalVoteCall = 0;

  /// `joint_vote(proposal_id, institution_id_48, approve)` — 联合投票。
  static const int jointVoteCall = 1;

  /// `citizen_vote(proposal_id, binding_id, nonce, signature, approve)`
  /// — 公民投票（由 SFID 发凭证）。
  static const int citizenVoteCall = 2;

  /// `finalize_proposal(proposal_id)` — 任意人触发终态执行（无需签投票）。
  static const int finalizeProposalCall = 3;

  // ---- 业务 pallet:仅保留提案创建与幂等兜底入口 ----
  //
  // Phase 2/3 已在链端物理删除所有业务 pallet 内部的聚合签名与投票入口
  // (共八条),全部通过 `VotingEngineSystem(9).internal_vote(0)` 统一收敛。
  // 业务 pallet 仅保留 propose 提案创建、execute 执行兜底、cleanup 被拒清理、
  // cancel 失败取消 等幂等入口。冷钱包 decoder 与此对齐。

  // ---- DuoqianTransferPow (19) ----
  static const int duoqianTransferPowPallet = 19;
  static const int proposeTransferCall = 0;
  static const int proposeSafetyFundCall = 1;
  static const int proposeSweepCall = 2;
  static const int executeTransferCall = 3;
  static const int executeSafetyFundCall = 4;
  static const int executeSweepCall = 5;

  // ---- RuntimeRootUpgrade (13) ----
  static const int runtimeRootUpgradePallet = 13;
  static const int proposeRuntimeUpgradeCall = 0;
  static const int developerDirectUpgradeCall = 2;

  // ---- DuoqianManagePow (17) ----
  static const int duoqianManagePowPallet = 17;
  static const int proposeCreateCall = 0;
  static const int proposeCloseCall = 1;
  static const int registerSfidInstitutionCall = 2;
  static const int proposeCreatePersonalCall = 3;
  static const int cleanupRejectedProposalCall = 4;

  // ---- ResolutionDestroGov (14) ----
  static const int resolutionDestroGovPallet = 14;
  static const int proposeDestroyCall = 0;
  static const int executeDestroyCall = 1;

  // ---- AdminsOriginGov (12) ----
  static const int adminsOriginGovPallet = 12;
  static const int proposeAdminReplacementCall = 0;
  static const int executeAdminReplacementCall = 1;

  // ---- GrandpaKeyGov (16) ----
  static const int grandpaKeyGovPallet = 16;
  static const int proposeReplaceGrandpaKeyCall = 0;
  static const int executeReplaceGrandpaKeyCall = 1;
  static const int cancelFailedReplaceGrandpaKeyCall = 2;

  // ---- ResolutionIssuanceGov (8) ----
  static const int resolutionIssuanceGovPallet = 8;
  static const int proposeResolutionIssuanceCall = 0;

  // ---- OffchainTransactionPos (21) · 清算行 L2 体系 ----
  static const int offchainTransactionPosPallet = 21;
  static const int bindClearingBankCall = 30;
  static const int depositCall = 31;
  static const int withdrawCall = 32;
  static const int switchBankCall = 33;
  static const int submitOffchainBatchV2Call = 34;
  static const int proposeL2FeeRateCall = 40;
  static const int setMaxL2FeeRateCall = 41;
}
