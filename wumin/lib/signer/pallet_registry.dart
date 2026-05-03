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
///   `VotingEngine::internal_vote`（9.0）。
/// - `joint_vote` / `citizen_vote` / `finalize_proposal` 在投票引擎内部
///   重新排 call_index：0=internal_vote / 1=joint_vote / 2=citizen_vote /
///   3=finalize_proposal。
///
/// Phase 4 · 业务 wrapper 物理删除（2026-05-02）：
/// - 业务 pallet 的 `execute_xxx` / `cancel_failed_xxx` wrapper extrinsic
///   全部物理删除，统一到 `VotingEngine::retry_passed_proposal`(9.4) 与
///   `VotingEngine::cancel_passed_proposal`(9.5)。冷钱包 decoder 删除 7 个
///   旧分支：`execute_admin_replacement` / `execute_replace_grandpa_key` /
///   `cancel_failed_replace_grandpa_key` / `execute_destroy` /
///   `execute_transfer` / `execute_safety_fund_transfer` / `execute_sweep_to_main`。
class PalletRegistry {
  const PalletRegistry._();

  /// 当前注册表适配的链 spec_version 集合。
  ///
  /// 2026-04-29 重新创世前 runtime wasm 版本整体归零,冷钱包同步仅接受
  /// 当前 fresh genesis 版本。
  /// 遇到旧 spec 的离线请求视为过期，拒绝解码。
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

  // ---- VotingEngine (9) · 所有治理投票唯一入口 ----
  static const int votingEnginePallet = 9;

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

  /// `retry_passed_proposal(proposal_id)` — 已通过提案的手动执行入口
  /// （Phase 4 整改后,所有业务 pallet 的 execute_xxx wrapper 统一收口至此）。
  static const int retryPassedProposalCall = 4;

  /// `cancel_passed_proposal(proposal_id, reason)` — 已通过但确认不可执行
  /// 的提案取消入口（Phase 4 整改后,所有 cancel_failed_xxx 统一收口至此）。
  static const int cancelPassedProposalCall = 5;

  // ---- 业务 pallet:仅保留提案创建与幂等兜底入口 ----
  //
  // Phase 2/3 已在链端物理删除所有业务 pallet 内部的聚合签名与投票入口
  // (共八条),全部通过 `VotingEngine(9).internal_vote(0)` 统一收敛。
  // Phase 4(2026-05-02) 进一步删除了所有业务 pallet 的 execute_xxx /
  // cancel_failed_xxx wrapper extrinsic,手动重试/取消统一走
  // `VotingEngine(9).retry_passed_proposal(4)` / `cancel_passed_proposal(5)`。
  // 业务 pallet 仅保留 propose 提案创建与 cleanup 被拒清理 等幂等入口。

  // ---- DuoqianTransfer (19) ----
  // call_index 3/4/5 (execute_transfer / execute_safety_fund_transfer /
  // execute_sweep_to_main) 已于 Phase 4 物理删除,call_index 留洞不复用。
  static const int duoqianTransferPallet = 19;
  static const int proposeTransferCall = 0;
  static const int proposeSafetyFundCall = 1;
  static const int proposeSweepCall = 2;

  // ---- RuntimeUpgrade (13) ----
  static const int runtimeUpgradePallet = 13;
  static const int proposeRuntimeUpgradeCall = 0;
  static const int developerDirectUpgradeCall = 2;

  // ---- DuoqianManage (17) ----
  static const int duoqianManagePallet = 17;
  static const int proposeCreateCall = 0;
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
  // call_index 1 (execute_destroy) 已于 Phase 4 物理删除,留洞不复用。
  static const int resolutionDestroPallet = 14;
  static const int proposeDestroyCall = 0;

  // ---- AdminsChange (12) ----
  // call_index 1 (execute_admin_replacement) 已于 Phase 4 物理删除,留洞不复用。
  static const int adminsChangePallet = 12;
  static const int proposeAdminReplacementCall = 0;

  // ---- GrandpaKeyChange (16) ----
  // call_index 1, 2 (execute_replace_grandpa_key /
  // cancel_failed_replace_grandpa_key) 已于 Phase 4 物理删除,留洞不复用。
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
