import 'package:citizenwallet/qr/generated/qr_action_registry.g.dart';

/// QR_V1 统一二维码协议常量。
///
/// 唯一事实源:`memory/01-architecture/qr/qr-protocol-spec.md`
/// Golden fixtures:`memory/01-architecture/qr/qr-protocol-fixtures/*.json`
///
/// 与 citizenapp/lib/qr/qr_protocols.dart 逐字节一致(两个独立 Flutter app,
/// 无代码依赖,靠 fixture 对齐)。
class QrProtocols {
  QrProtocols._();

  /// 唯一协议版本字符串。压缩为 5 字符以降低二维码密度。
  static const String v1 = 'QR_V1';
}

/// 统一扫码流向枚举。线上只序列化为数字 `k`。
enum QrKind {
  signRequest(1, temporary: true),
  signResponse(2, temporary: true),
  userContact(3, temporary: false),
  userTransfer(4, temporary: true);

  const QrKind(this.code, {required this.temporary});

  /// JSON 线上数字码。
  final int code;

  /// `true` = 临时码(必填 i/e);`false` = 固定码(不带时效字段)。
  final bool temporary;

  /// 固定码 = 永久有效,JSON 不含时效字段。
  bool get fixed => !temporary;

  static QrKind fromWire(Object? wire) {
    final code = wire is int ? wire : int.tryParse(wire?.toString() ?? '');
    for (final k in QrKind.values) {
      if (k.code == code) return k;
    }
    throw FormatException('未知 k: $wire');
  }
}

/// QR_V1 业务动作码。`k` 只表达扫码流向,业务场景必须放在 `a`。
class QrActions {
  QrActions._();

  static const int login = 1;
  static const int citizenIdentity = 2;
  static const int onchinaAdmin = 3;
  static const int activateAdmin = 5;
  static const int decryptAdmin = 6;
  static const int runtimeUpgradeHash = 7;

  /// 广场账户动作由 CitizenApp 热钱包签名；CitizenWallet 扫到时只能识别后拒绝，
  /// 不能退回成未知数字或盲签。
  static const int squareAccountAction = 9;

  static const int transferWithRemark = 0x0400;
  static const int personalCreate = 0x0700;
  static const int personalClose = 0x0701;
  static const int personalAdminSetChange = 0x1d00;
  static const int resolutionIssuance = 0x0800;
  static const int finalizeProposal = 0x0903;
  static const int retryPassedProposal = 0x0904;
  static const int cancelPassedProposal = 0x0905;
  static const int registerVotingIdentity = 0x0a00;
  static const int upgradeToCandidateIdentity = 0x0a01;
  static const int updateVotingIdentity = 0x0a02;
  static const int updateCandidateIdentity = 0x0a03;
  static const int revokeIdentity = 0x0a04;
  static const int occupyCid = 0x0a06;
  static const int occupyCidsBatch = 0x0a07;
  static const int revokeCid = 0x0a08;
  static const int proposeRuntimeUpgrade = 0x0c00;
  static const int developerDirectUpgrade = 0x0c02;
  static const int resolutionDestroy = 0x0d00;
  static const int grandpaKeyChange = 0x0f00;
  static const int publicInstitutionClose = 0x1e01;
  static const int publicInstitutionUpdateInfo = 0x1e06;
  static const int publicInstitutionAddAccount = 0x1e07;
  static const int publicInstitutionGovernance = 0x1e08;
  static const int publicInstitutionRegisterAdmins = 0x1e09;
  static const int privateInstitutionClose = 0x1f01;
  static const int privateInstitutionUpdateInfo = 0x1f06;
  static const int privateInstitutionAddAccount = 0x1f07;
  static const int privateInstitutionGovernance = 0x1f08;
  static const int privateInstitutionRegisterAdmins = 0x1f09;
  static const int multisigTransfer = 0x1100;
  static const int safetyFundTransfer = 0x1101;
  static const int sweepToMain = 0x1102;
  static const int bindClearingBank = 0x131e;
  static const int depositClearingBank = 0x131f;
  static const int withdrawClearingBank = 0x1320;
  static const int switchClearingBank = 0x1321;
  static const int proposeL2FeeRate = 0x1328;
  static const int registerClearingBank = 0x1332;
  static const int updateClearingBankEndpoint = 0x1333;
  static const int unregisterClearingBank = 0x1334;
  static const int internalVote = 0x1400;
  static const int jointVote = 0x1500;
  static const int castReferendum = 0x1501;
  static const int castPopularVote = 0x1602;
  static const int castMutualVote = 0x1603;

  // 链上资产 OnchainIssuance(23 = 0x17)。动作码与 runtime call_index 一一对应。
  static const int proposeAssetIssue = 0x1700;
  static const int proposeAssetMint = 0x1701;
  static const int proposeAssetBurn = 0x1702;
  static const int proposeAssetClose = 0x1703;
  static const int proposeAssetTransfer = 0x1704;
  static const int proposeMonitorFreeze = 0x170a;
  static const int proposeMonitorUnfreeze = 0x170b;
  static const int proposeMonitorConfiscate = 0x170c;
  static const int proposeMonitorForceTransfer = 0x170d;
  static const int proposeMonitorForceClose = 0x170e;

  // 注册局地址目录 AddressRegistry(33 = 0x21)
  static const int setAddressCatalogVersion = 0x2100;
  static const int setAddressName = 0x2101;
  static const int removeAddressName = 0x2102;
  static const int setAddress = 0x2103;
  static const int removeAddress = 0x2104;

  // 公民链基金会平台调价提案 SquarePost(34 = 0x22)
  static const int proposeSetPlatformPrice = 0x2205;

  // 立法院 LegislationYuan(25 = 0x19)
  static const int proposeEnactLaw = 0x1900;
  static const int proposeAmendLaw = 0x1901;
  static const int proposeRepealLaw = 0x1902;

  // 立法投票 LegislationVote(26 = 0x1a)
  static const int castRepresentativeVote = 0x1a01;
  static const int castLegislationReferendum = 0x1a02;
  static const int executiveSign = 0x1a03;
  static const int overrideSign = 0x1a04;
  static const int guardVote = 0x1a05;

  /// 链交易动作统一按 `(pallet_index << 8) | call_index` 生成。
  static int chain(int palletIndex, int callIndex) =>
      ((palletIndex & 0xff) << 8) | (callIndex & 0xff);

  static bool isChainAction(int action) => action >= 0x0100;

  static bool isBinaryRaw(int action) =>
      action == activateAdmin || action == decryptAdmin;

  static bool isRuntimeHashOnly(int action) =>
      GeneratedQrActionRegistry.isHashOnlyAction(action);

  static int fromDecodedAction(String action) {
    // 公民参选身份确认复用 a=2 的公民身份签名域，具体身份等级由 payload 字段展示。
    if (action == 'citizen_candidate_identity') return citizenIdentity;
    return GeneratedQrActionRegistry.actionCodeForKey(action) ?? 0;
  }
}
