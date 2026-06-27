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
  static const int citizenBind = 2;
  static const int cidAdmin = 3;
  static const int activateAdmin = 5;
  static const int decryptAdmin = 6;
  static const int runtimeUpgradeHash = 7;

  static const int balancesTransfer = 0x0203;
  static const int personalCreate = 0x0700;
  static const int personalClose = 0x0701;
  static const int personalCleanupRejected = 0x0702;
  static const int personalAdminSetChange = 0x0703;
  static const int resolutionIssuance = 0x0800;
  static const int finalizeProposal = 0x0903;
  static const int retryPassedProposal = 0x0904;
  static const int cancelPassedProposal = 0x0905;
  static const int genesisAdmins = 0x0c00;
  static const int publicAdmins = 0x1d00;
  static const int privateAdmins = 0x1e00;
  static const int proposeRuntimeUpgrade = 0x0d00;
  static const int developerDirectUpgrade = 0x0d02;
  static const int resolutionDestroy = 0x0e00;
  static const int grandpaKeyChange = 0x1000;
  static const int organizationClose = 0x1101;
  static const int organizationCleanupRejected = 0x1104;
  static const int organizationCreate = 0x1105;
  static const int multisigTransfer = 0x1300;
  static const int safetyFundTransfer = 0x1301;
  static const int sweepToMain = 0x1302;
  static const int bindClearingBank = 0x151e;
  static const int depositClearingBank = 0x151f;
  static const int withdrawClearingBank = 0x1520;
  static const int switchClearingBank = 0x1521;
  static const int registerClearingBank = 0x1532;
  static const int updateClearingBankEndpoint = 0x1533;
  static const int unregisterClearingBank = 0x1534;
  static const int internalVote = 0x1600;
  static const int jointVote = 0x1700;
  static const int castReferendum = 0x1701;

  // 立法院 LegislationYuan(27 = 0x1b)
  static const int proposeEnactLaw = 0x1b00;
  static const int proposeAmendLaw = 0x1b01;
  static const int proposeRepealLaw = 0x1b02;

  // 立法投票 LegislationVote(28 = 0x1c)
  static const int prepareLegislationSnapshot = 0x1c00;
  static const int castHouseVote = 0x1c01;
  static const int castLegislationReferendum = 0x1c02;
  static const int executiveSign = 0x1c03;
  static const int overrideSign = 0x1c04;
  static const int guardVote = 0x1c05;

  /// 链交易动作统一按 `(pallet_index << 8) | call_index` 生成。
  static int chain(int palletIndex, int callIndex) =>
      ((palletIndex & 0xff) << 8) | (callIndex & 0xff);

  static bool isChainAction(int action) => action >= 0x0100;

  static bool isBinaryRaw(int action) =>
      action == activateAdmin || action == decryptAdmin;

  static bool isRuntimeHashOnly(int action) =>
      action == runtimeUpgradeHash ||
      action == proposeRuntimeUpgrade ||
      action == developerDirectUpgrade;

  static int fromDecodedAction(String action) => switch (action) {
        'transfer' => balancesTransfer,
        'propose_create_personal' => personalCreate,
        'propose_close_personal' => personalClose,
        'cleanup_rejected_personal_proposal' => personalCleanupRejected,
        'propose_resolution_issuance' => resolutionIssuance,
        'finalize_proposal' => finalizeProposal,
        'retry_passed_proposal' => retryPassedProposal,
        'cancel_passed_proposal' => cancelPassedProposal,
        'propose_personal_admin_set_change' => personalAdminSetChange,
        'propose_genesis_admin_set_change' => genesisAdmins,
        'propose_public_admin_set_change' => publicAdmins,
        'propose_private_admin_set_change' => privateAdmins,
        'propose_runtime_upgrade' => proposeRuntimeUpgrade,
        'developer_direct_upgrade' => developerDirectUpgrade,
        'propose_destroy' => resolutionDestroy,
        'propose_replace_grandpa_key' => grandpaKeyChange,
        'propose_close_institution' => organizationClose,
        'cleanup_rejected_proposal' => organizationCleanupRejected,
        'propose_create_institution' => organizationCreate,
        'propose_transfer' => multisigTransfer,
        'propose_safety_fund_transfer' => safetyFundTransfer,
        'propose_sweep_to_main' => sweepToMain,
        'bind_clearing_bank' => bindClearingBank,
        'deposit_clearing_bank' => depositClearingBank,
        'withdraw_clearing_bank' => withdrawClearingBank,
        'switch_clearing_bank' => switchClearingBank,
        'register_clearing_bank' => registerClearingBank,
        'update_clearing_bank_endpoint' => updateClearingBankEndpoint,
        'unregister_clearing_bank' => unregisterClearingBank,
        'internal_vote' => internalVote,
        'joint_vote' => jointVote,
        'cast_referendum' => castReferendum,
        'propose_enact_law' => proposeEnactLaw,
        'propose_amend_law' => proposeAmendLaw,
        'propose_repeal_law' => proposeRepealLaw,
        'prepare_legislation_snapshot' => prepareLegislationSnapshot,
        'cast_house_vote' => castHouseVote,
        'cast_referendum_vote' => castLegislationReferendum,
        'executive_sign' => executiveSign,
        'override_sign' => overrideSign,
        'guard_vote' => guardVote,
        'activate_admin_account' => activateAdmin,
        'decrypt_admin' => decryptAdmin,
        'citizen_bind' => citizenBind,
        'cid_admin_action' => cidAdmin,
        _ => 0,
      };
}
