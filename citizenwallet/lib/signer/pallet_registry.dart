/// 链上 pallet / call 索引注册表。
///
/// 索引由 runtime 的 `construct_runtime!` 宏中声明顺序决定。
/// 链升级调整 pallet 顺序后，需同步更新此文件中的常量。
///
/// 防误签靠两色严格模式:decoder 解析失败 / QR action 与 payload 解码动作不一致
/// 直接红色拒签。
///
/// 投票引擎统一入口:
/// - 业务 pallet 不承载投票，机构岗位选民/个人多签管理员走 `InternalVote::cast`(20.0)
/// - 联合投票内部投票阶段走 `JointVote::cast_admin`(21.0),
///   联合公投阶段走 `JointVote::cast_referendum`(21.1)
/// - 引擎核心 `VotingEngine` (9) 仅承载 `finalize_proposal`(9.3) /
///   `retry_passed_proposal`(9.4) / `cancel_passed_proposal`(9.5)。
///
/// 手动执行重试/取消统一走 `VotingEngine::retry_passed_proposal`(9.4) 与
/// `VotingEngine::cancel_passed_proposal`(9.5),业务 pallet 不承载 wrapper extrinsic。
class PalletRegistry {
  const PalletRegistry._();

  // ---- OnchainTransaction (4) ----
  static const int onchainTransactionPallet = 4;
  static const int transferWithRemarkCall = 0;

  // ---- VotingEngine (9) · 引擎核心 ----
  // 仅承载 lifecycle extrinsic:finalize_proposal / retry_passed_proposal /
  // cancel_passed_proposal。mode-specific 投票 extrinsic 在 InternalVote(20) /
  // JointVote(21) sub-pallet。
  static const int votingEnginePallet = 9;

  /// `finalize_proposal(proposal_id)` — 任意人触发终态执行(无需签投票)。
  static const int finalizeProposalCall = 3;

  /// `retry_passed_proposal(proposal_id)` — 已通过提案的手动执行入口。
  static const int retryPassedProposalCall = 4;

  /// `cancel_passed_proposal(proposal_id, reason)` — 已通过但确认不可执行的提案取消入口。
  static const int cancelPassedProposalCall = 5;

  // ---- CitizenIdentity (10) · 公民链上身份 ----
  static const int citizenIdentityPallet = 10;

  /// `register_voting_identity(actor_cid_number, payload, citizen_signature)`。
  static const int registerVotingIdentityCall = 0;

  /// `upgrade_to_candidate_identity(actor_cid_number, payload, citizen_signature)`。
  static const int upgradeToCandidateIdentityCall = 1;

  /// `update_voting_identity(actor_cid_number, payload, citizen_signature)`。
  static const int updateVotingIdentityCall = 2;

  /// `update_candidate_identity(actor_cid_number, payload, citizen_signature)`。
  static const int updateCandidateIdentityCall = 3;

  /// `revoke_identity(actor_cid_number, cid_number)`。
  static const int revokeIdentityCall = 4;

  // call_index 5 永久留洞：人口快照仅由 votingengine 内联生成。

  /// `occupy_cid(actor_cid_number, cid_number, commitment, province_code, city_code)`
  /// — 注册局建档「占号」:链上原子验格式+查重+登记 CID 号(注册局签名)。
  static const int occupyCidCall = 6;

  /// `occupy_cids_batch(actor_cid_number, items, province_code, city_code)`。
  static const int occupyCidsBatchCall = 7;

  /// `revoke_cid(actor_cid_number, cid_number)`
  /// — 注册局吊销 CID 号(墓碑,永不复用;注册局签名)。
  static const int revokeCidCall = 8;

  // ---- InternalVote sub-pallet (20) · 内部投票管理员一人一票 ----
  static const int internalVotePallet = 20;

  /// `cast(proposal_id, approve)`。
  static const int internalVoteCall = 0;

  // ---- JointVote sub-pallet (21) · 联合投票(内部投票阶段 + 联合公投)----
  static const int jointVotePallet = 21;

  /// `cast_admin(proposal_id, cid_number, voter_role_code, approve)` — 联合投票内部投票阶段。
  static const int jointVoteCall = 0;

  /// `cast_referendum(proposal_id, approve)` — 联合公投阶段,链上按账户读取公民身份。
  static const int castReferendumCall = 1;

  // call_index 2 永久留洞：联合提案创建时由引擎内联生成全国人口快照。

  // ---- 业务 pallet:仅承载提案创建入口 ----
  //
  // 投票统一走 `InternalVote(20).cast(0)`,手动重试/取消统一走
  // `VotingEngine(9).retry_passed_proposal(4)` / `cancel_passed_proposal(5)`。

  // ---- MultisigTransfer (17) ----
  // call_index 3/4/5 留洞不复用。
  static const int multisigTransferPallet = 17;
  static const int proposeTransferCall = 0;
  static const int proposeSafetyFundCall = 1;
  static const int proposeSweepCall = 2;

  // ---- 协议升级 RuntimeUpgrade (12) ----
  static const int runtimeUpgradePallet = 12;
  static const int proposeRuntimeUpgradeCall = 0;
  static const int developerDirectUpgradeCall = 2;

  // ---- PublicManage (30) / PrivateManage (31) ----
  // 公权机构与私权机构登记管理分别由两个 pallet 承载。
  static const int publicManagePallet = 30;
  static const int privateManagePallet = 31;
  static const int proposeCloseInstitutionCall = 1;
  // call_index 4 永久留洞：机构 pending 状态由投票引擎终态回调清理。

  // call_index 5 永久关闭：普通机构创建必须改由业务模块提交包含初始岗位、权限、
  // 任职和投票规则的原子结果，不保留旧直接创建载荷。
  static const int updateInstitutionInfoCall = 6;
  static const int addInstitutionAccountCall = 7;
  static const int proposeInstitutionGovernanceCall = 8;
  static const int registerInstitutionAdminsCall = 9;

  // ---- PersonalManage (7) · 个人多签生命周期 ----
  // 个人多签独立 pallet,MODULE_TAG = b"per-mgmt",
  // ACTION enum 独立命名空间(ACTION_CREATE=0/ACTION_CLOSE=1)。
  // propose_create(account_name, admins, regular_threshold, amount):
  // admins_len 由 admins 长度派生,regular_threshold 由用户输入且必须严格过半。
  static const int personalManagePallet = 7;
  static const int proposeCreatePersonalCall = 0;
  static const int proposeClosePersonalCall = 1;
  // call_index 2 永久留洞：个人多签 pending 状态由投票引擎终态回调清理。

  // ---- PersonalAdmins (29) · 个人多签管理员集合变更 ----
  // 独立 pallet,承载 propose_admin_set_change(call_index=0)。
  static const int personalAdminsPallet = 29;
  static const int proposePersonalAdminSetChangeCall = 0;

  // ---- ResolutionDestroy (13) ----
  // call_index 1 留洞不复用。
  static const int resolutionDestroyPallet = 13;
  static const int proposeDestroyCall = 0;

  static bool isPersonalAdminSetChangeCall(int palletIndex, int callIndex) {
    return palletIndex == personalAdminsPallet &&
        callIndex == proposePersonalAdminSetChangeCall;
  }

  // ---- GrandpaKeyChange (15) ----
  // call_index 1, 2 留洞不复用。
  static const int grandpaKeyChangePallet = 15;
  static const int proposeReplaceGrandpaKeyCall = 0;

  // ---- ResolutionIssuance (8) ----
  static const int resolutionIssuancePallet = 8;
  static const int proposeIssuanceCall = 0;

  // ---- OnchainIssuance (23) · 链上发行代币(Plain FT) ----
  // call_index 5..=9 / 15+ 留洞不复用(永久 ABI)。
  // 业务调用走 propose_X(InternalVote),监管调用走 propose_monitor_X(JointVote)。
  // 十个调用都以 actor_cid_number 开头；仅 propose_issue 紧随
  // execution_account，机构身份不得从该账户反推。
  // 投票/重试/取消统一走 InternalVote(20)/JointVote(21)/VotingEngine(9.4/9.5)。
  static const int onchainIssuancePallet = 23;
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

  // ---- LegislationYuan (25) · 立法院(立法/修法/废法发起)----
  // 法律结构化上链(章>节>条>款),发起类提案 QR 由节点端生成,冷钱包仅解码核对。
  // 投票/签署统一走 LegislationVote(26),本 pallet 仅承载 3 条 propose_X。
  static const int legislationYuanPallet = 25;
  static const int proposeEnactLawCall = 0;
  static const int proposeAmendLawCall = 1;
  static const int proposeRepealLawCall = 2;

  // ---- LegislationVote (26) · 立法专属投票引擎 ----
  // 立法投票阶段：代表机构表决 / 特别案公投 / 行政签署 / 三人会签 / 护宪终审。
  static const int legislationVotePallet = 26;
  // call_index 0 永久留洞：特别案提案创建时由引擎按 actor CID 内联生成快照。
  static const int castRepresentativeVoteCall = 1;
  static const int castLegislationReferendumCall = 2;
  static const int executiveSignCall = 3;
  static const int overrideSignCall = 4;
  static const int guardVoteCall = 5;

  // ---- OffchainTransaction (19) · 清算行 L2 体系 ----
  static const int offchainTransactionPallet = 19;
  static const int bindClearingBankCall = 30;
  static const int depositCall = 31;
  static const int withdrawCall = 32;
  static const int switchBankCall = 33;
  static const int submitOffchainBatchCall = 34;
  static const int proposeL2FeeRateCall = 40;
  static const int setMaxL2FeeRateCall = 41;
  static const int registerClearingBankCall = 50;
  static const int updateClearingBankEndpointCall = 51;
  static const int unregisterClearingBankCall = 52;

  // ---- AddressRegistry (33) · 注册局地址目录 ----
  static const int addressRegistryPallet = 33;
  static const int setAddressCatalogVersionCall = 0;
  static const int setAddressNameCall = 1;
  static const int removeAddressNameCall = 2;
  static const int setAddressCall = 3;
  static const int removeAddressCall = 4;

  // ---- SquarePost (34) · 平台会员价格治理 ----
  static const int squarePostPallet = 34;
  static const int proposeSetPlatformPriceCall = 5;
}
