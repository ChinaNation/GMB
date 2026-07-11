/// QR_V1 统一二维码协议常量。
///
/// 唯一事实源:`memory/01-architecture/qr/qr-protocol-spec.md`
/// Golden fixtures:`memory/01-architecture/qr/qr-protocol-fixtures/*.json`
///
/// 本文件只有一个协议字符串和一个扫码流向枚举,禁止新增任何旧协议常量。
class QrProtocol {
  QrProtocol._();

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

  /// 广场账户动作（订阅/取消/…）链下签名，走 GMB 哈希域 op_tag 0x1D。
  /// 官网无私钥发起，CitizenApp「扫一扫」扫码用 owner 主钥签名回传。
  static const int squareAccountAction = 9;

  static const int transferWithRemark = 0x0400;
  static const int personalCreate = 0x0700;
  static const int personalClose = 0x0701;
  static const int personalCleanupRejected = 0x0702;
  static const int personalAdminsChange = 0x0703;
  static const int resolutionIssuance = 0x0800;
  static const int finalizeProposal = 0x0903;
  static const int retryPassedProposal = 0x0904;
  static const int cancelPassedProposal = 0x0905;
  static const int publicAdmins = 0x1d00;
  static const int privateAdmins = 0x1e00;
  static const int proposeRuntimeUpgrade = 0x0d00;
  static const int developerDirectUpgrade = 0x0d02;
  static const int resolutionDestroy = 0x0e00;
  static const int grandpaKeyChange = 0x1000;
  // 机构创建/关闭已收归 onchina 控制台 + 冷钱包,citizenapp 不再生成机构创建/关闭签名请求,
  // 故删除旧 OrganizationManage(17) 动作码 organizationCreate/Close/CleanupRejected(0x1105/0x1101/0x1104)。
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
  static const int preparePopulationSnapshot = 0x1702;
  // 立法(LegislationYuan=27=0x1b 发起类节点端;LegislationVote=28=0x1c 投票/签署类)。
  static const int legislationEnact = 0x1b00;
  static const int legislationAmend = 0x1b01;
  static const int legislationRepeal = 0x1b02;
  static const int legislationPrepareSnapshot = 0x1c00;
  static const int legislationHouseVote = 0x1c01;
  static const int legislationReferendum = 0x1c02;
  static const int legislationExecutiveSign = 0x1c03;
  static const int legislationOverrideSign = 0x1c04;
  static const int legislationGuardVote = 0x1c05;

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
        'transfer' => transferWithRemark,
        'propose_create_personal' => personalCreate,
        'propose_close_personal' => personalClose,
        'cleanup_rejected_personal_proposal' => personalCleanupRejected,
        'propose_issuance' => resolutionIssuance,
        'finalize_proposal' => finalizeProposal,
        'retry_passed_proposal' => retryPassedProposal,
        'cancel_passed_proposal' => cancelPassedProposal,
        'propose_personal_admin_set_change' => personalAdminsChange,
        'propose_public_admin_set_change' => publicAdmins,
        'propose_private_admin_set_change' => privateAdmins,
        'propose_runtime_upgrade' => proposeRuntimeUpgrade,
        'developer_direct_upgrade' => developerDirectUpgrade,
        'propose_destroy' => resolutionDestroy,
        'propose_replace_grandpa_key' => grandpaKeyChange,
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
        'prepare_population_snapshot' => preparePopulationSnapshot,
        'propose_enact_law' => legislationEnact,
        'propose_amend_law' => legislationAmend,
        'propose_repeal_law' => legislationRepeal,
        'prepare_joint_population_snapshot' => legislationPrepareSnapshot,
        'cast_house_vote' => legislationHouseVote,
        'cast_referendum_vote' => legislationReferendum,
        'executive_sign' => legislationExecutiveSign,
        'override_sign' => legislationOverrideSign,
        'guard_vote' => legislationGuardVote,
        'activate_admin_account' => activateAdmin,
        'decrypt_admin' => decryptAdmin,
        'citizen_identity' => citizenIdentity,
        'onchina_admin_action' => onchinaAdmin,
        _ => 0,
      };
}
