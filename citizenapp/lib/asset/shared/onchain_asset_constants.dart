// 链上发行代币(onchain-issuance)pallet_index / call_index / ACTION 常量(ADR-011 v3)。
//
// 业务调用走 propose_X extrinsic(InternalVote 路径):
//   propose_issue / mint / burn / close / transfer  (call_index 0..=4)
// 监管调用走 propose_monitor_X extrinsic(JointVote 路径,NRC admin 发起):
//   propose_monitor_freeze / unfreeze / confiscate / force_transfer / force_close
//   (call_index 10..=14)
//
// 链端权威定义见 citizenchain/runtime/issuance/onchain-issuance/src/lib.rs。
// 冷钱包 citizenwallet/lib/signer/pallet_registry.dart 同步 pallet_index/call_index 常量。

class OnchainAssetActions {
  OnchainAssetActions._();

  // ---------- pallet_index / call_index(与链端 + citizenwallet PalletRegistry 同步) ----------

  /// OnchainIssuance pallet_index(与 citizenchain runtime/src/lib.rs::construct_runtime 一致)。
  static const int onchainIssuancePalletIndex = 25;

  /// pallet_assets 内核 pallet_index(原生 extrinsic 全部被 RuntimeCallFilter reject)。
  static const int assetsPalletIndex = 26;

  // 业务 propose_X(call_index 0..=4)
  static const int callProposeIssue = 0;
  static const int callProposeMint = 1;
  static const int callProposeBurn = 2;
  static const int callProposeClose = 3;
  static const int callProposeTransfer = 4;

  // 监管 propose_monitor_X(call_index 10..=14)
  static const int callProposeMonitorFreeze = 10;
  static const int callProposeMonitorUnfreeze = 11;
  static const int callProposeMonitorConfiscate = 12;
  static const int callProposeMonitorForceTransfer = 13;
  static const int callProposeMonitorForceClose = 14;

  // ---------- ACTION 字符串常量(VotingEngine ProposalData 业务标签) ----------

  // 业务 ACTION(InternalVote)
  static const String actionIssue = 'OAIS';
  static const String actionMint = 'OAMT';
  static const String actionBurn = 'OABN';
  static const String actionClose = 'OACL';
  static const String actionTransfer = 'OATR';

  // 监管 ACTION(JointVote)
  static const String actionMonitorFreeze = 'OMFZ';
  static const String actionMonitorUnfreeze = 'OMUF';
  static const String actionMonitorConfiscate = 'OMCF';
  static const String actionMonitorForceTransfer = 'OMFT';
  static const String actionMonitorForceClose = 'OMFC';

  /// VotingEngine ProposalData 业务标签前缀(与链端 MODULE_TAG 一致)。
  static const String moduleTag = 'onc-iss';

  /// 链端铁律:每个用户代币创建一次性 reserve 1000 GMB(= 100_000 FEN)押金,
  /// 提案通过则 transfer 给 NRC fee_account,否决则退还 proposer。
  ///
  /// 用户在 asset_issue_page 发起前必须本地预校验 GMB 余额 ≥ 1000,
  /// 避免发起后链端因余额不足 reserve 失败浪费一次提交。
  static const int issueCreationFeeFen = 100000;

  /// decimals 合法区间(链端 onchain_issuance::validation 同步约束)。
  static const int minDecimals = 0;
  static const int maxDecimals = 18;
}
