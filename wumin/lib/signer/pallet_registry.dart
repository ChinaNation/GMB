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
  static const Set<int> supportedSpecVersions = {5};

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
  static const int duoqianTransferPowPallet = 19;
  static const int proposeTransferCall = 0;
  static const int voteTransferCall = 1;

  // ---- VotingEngineSystem ----
  static const int votingEngineSystemPallet = 9;
  static const int jointVoteCall = 3;
  static const int citizenVoteCall = 4;

  // ---- RuntimeRootUpgrade ----
  static const int runtimeRootUpgradePallet = 13;
  static const int proposeRuntimeUpgradeCall = 0;
  static const int developerDirectUpgradeCall = 2;

  // ---- DuoqianManagePow ----
  static const int duoqianManagePowPallet = 17;
  static const int proposeCreateCall = 0;
  static const int proposeCloseCall = 1;
  static const int voteCreateCall = 3;
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
  static const int proposeSafetyFundCall = 3;
  static const int voteSafetyFundCall = 4;
  static const int proposeSweepCall = 5;
  static const int voteSweepCall = 6;

  // ---- OffchainTransactionPos ----
  static const int offchainTransactionPosPallet = 21;
  static const int bindClearingInstitutionCall = 9;
  /// 链下支付授权（虚拟 call_index，非链上 extrinsic）。
  static const int offchainPayCall = 99;
}
