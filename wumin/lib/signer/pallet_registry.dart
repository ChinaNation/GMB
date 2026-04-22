/// 链上 pallet / call 索引注册表。
///
/// 索引由 runtime 的 `construct_runtime!` 宏中声明顺序决定。
/// 链升级调整 pallet 顺序后，需同步更新此文件中的常量。
///
/// [supportedSpecVersions] 列出当前注册表适配的 spec_version 集合。
/// 离线设备收到未知 spec_version 时应拒绝解码，提示用户升级冷钱包。
class PalletRegistry {
  const PalletRegistry._();

  /// 当前注册表适配的链 spec_version 集合。
  ///
  /// 链升级后若 pallet 索引未变，将新 spec_version 加入此集合即可。
  /// 若索引发生变化，需同步修改下方常量并更新此集合。
  static const Set<int> supportedSpecVersions = {1};

  /// 检查给定 spec_version 是否与当前注册表兼容。
  ///
  /// - 返回 `true`：可安全解码
  /// - 返回 `false`：spec_version 未知，解码可能错位
  /// - [specVersion] 为 null 时（旧版在线端未发送），返回 `false`
  static bool isSupported(int? specVersion) {
    if (specVersion == null) return false;
    return supportedSpecVersions.contains(specVersion);
  }

  // ---- Balances ----
  static const int balancesPallet = 2;
  static const int transferKeepAliveCall = 3;

  // ---- DuoqianTransferPow ----
  // Step 2 · 离线 QR 聚合签名改造:vote_transfer(call_index=1)已物理删除,
  // 替换为 finalize_transfer(同 call_index=1)。finalize_X 聚合签名路径走
  // 热钱包/wuminapp,冷钱包不负责盲签 finalize_X(sigs 由 sr25519 签名聚合,
  // 本质不是 payload decode 场景)。
  static const int duoqianTransferPowPallet = 19;
  static const int proposeTransferCall = 0;

  // ---- VotingEngineSystem ----
  static const int votingEngineSystemPallet = 9;
  static const int jointVoteCall = 3;
  static const int citizenVoteCall = 4;

  // ---- RuntimeRootUpgrade ----
  static const int runtimeRootUpgradePallet = 13;
  static const int proposeRuntimeUpgradeCall = 0;
  static const int developerDirectUpgradeCall = 2;

  // ---- DuoqianManagePow ----
  // Step 1 · 离线 QR 聚合签名改造:vote_create(call_index=3)已物理删除,
  // 替换为 finalize_create(同 call_index=3)。冷钱包不负责盲签 finalize_X。
  // vote_close(call_index=5)尚未改造(Step 3 待做),保留。
  static const int duoqianManagePowPallet = 17;
  static const int proposeCreateCall = 0;
  static const int proposeCloseCall = 1;
  static const int proposeCreatePersonalCall = 4;
  static const int voteCloseCall = 5;

  // ---- ResolutionDestroGov ----
  static const int resolutionDestroGovPallet = 14;
  static const int proposeDestroyCall = 0;
  static const int voteDestroyCall = 1;

  // ---- AdminsOriginGov ----
  static const int adminsOriginGovPallet = 12;
  static const int proposeAdminReplacementCall = 0;
  static const int voteAdminReplacementCall = 1;

  // ---- GrandpaKeyGov ----
  static const int grandpaKeyGovPallet = 16;
  static const int proposeKeyChangeCall = 0;
  static const int voteKeyChangeCall = 1;

  // ---- ResolutionIssuanceGov ----
  static const int resolutionIssuanceGovPallet = 8;
  static const int proposeResolutionIssuanceCall = 0;

  // ---- DuoqianTransferPow 补充 ----
  // Step 2 · 离线聚合改造:vote_safety_fund_transfer (call_index=4) / vote_sweep_to_main (call_index=6)
  // 已物理删除,替换为 finalize_safety_fund_transfer / finalize_sweep_to_main(同 call_index)。
  // 冷钱包不负责盲签 finalize_X。
  static const int proposeSafetyFundCall = 3;
  static const int proposeSweepCall = 5;

  // ---- OffchainTransactionPos(清算行 L2 体系) ----
  static const int offchainTransactionPosPallet = 21;
  static const int bindClearingBankCall = 30;
  static const int depositCall = 31;
  static const int withdrawCall = 32;
  static const int switchBankCall = 33;
  static const int submitOffchainBatchV2Call = 34;
  static const int proposeL2FeeRateCall = 40;
  static const int setMaxL2FeeRateCall = 41;
}
