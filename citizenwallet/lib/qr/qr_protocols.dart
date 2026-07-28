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
    if (wire is! int) {
      throw FormatException('k 必须为整数,实际: $wire');
    }
    final code = wire;
    for (final k in QrKind.values) {
      if (k.code == code) return k;
    }
    throw FormatException('未知 k: $wire');
  }
}

/// QR_V1 业务动作码。`k` 只表达扫码流向,业务场景必须放在 `a`。
class QrActions {
  QrActions._();

  static int _code(String key) =>
      GeneratedQrActionRegistry.actionCodeForKey(key)!;

  static int get login => _code('login');
  static int get citizenIdentity => _code('citizen_identity');
  static int get onchinaAdmin => _code('onchina_admin_action');
  static int get activateAdmin => _code('activate_admin_account');
  static int get decryptAdmin => _code('decrypt_admin');
  static int get runtimeUpgradeHash => _code('runtime_upgrade_hash');

  /// 广场账户动作由 CitizenApp 热钱包签名；CitizenWallet 扫到时只能识别后拒绝，
  /// 不能退回成未知数字或盲签。
  static int get squareAccountAction => _code('square_account_action');

  static int get transferWithRemark => _code('transfer');
  static int get personalCreate => _code('propose_create_personal');
  static int get personalClose => _code('propose_close_personal');
  static int get personalAdminSetChange =>
      _code('propose_personal_admin_set_change');
  static int get resolutionIssuance => _code('propose_issuance');
  static int get finalizeProposal => _code('finalize_proposal');
  static int get retryPassedProposal => _code('retry_passed_proposal');
  static int get cancelPassedProposal => _code('cancel_passed_proposal');
  static int get registerVotingIdentity => _code('register_voting_identity');
  static int get upgradeToCandidateIdentity =>
      _code('upgrade_to_candidate_identity');
  static int get updateVotingIdentity => _code('update_voting_identity');
  static int get updateCandidateIdentity => _code('update_candidate_identity');
  static int get revokeIdentity => _code('revoke_identity');
  static int get occupyCid => _code('occupy_cid');
  static int get occupyCidsBatch => _code('occupy_cids_batch');
  static int get revokeCid => _code('revoke_cid');
  static int get proposeRuntimeUpgrade => _code('propose_runtime_upgrade');
  static int get developerDirectUpgrade => _code('developer_direct_upgrade');
  static int get resolutionDestroy => _code('propose_destroy');
  static int get grandpaKeyChange =>
      _code('propose_emergency_grandpa_key_recovery');
  static int get publicInstitutionClose =>
      _code('propose_close_public_institution');
  static int get publicInstitutionUpdateInfo =>
      _code('update_public_institution_info');
  static int get publicInstitutionAddAccount =>
      _code('add_public_institution_account');
  static int get publicInstitutionGovernance =>
      _code('propose_public_institution_governance');
  static int get publicInstitutionRegisterAdmins =>
      _code('register_public_institution_admins');
  static int get privateInstitutionClose =>
      _code('propose_close_private_institution');
  static int get privateInstitutionUpdateInfo =>
      _code('update_private_institution_info');
  static int get privateInstitutionAddAccount =>
      _code('add_private_institution_account');
  static int get privateInstitutionGovernance =>
      _code('propose_private_institution_governance');
  static int get privateInstitutionRegisterAdmins =>
      _code('register_private_institution_admins');
  static int get multisigTransfer => _code('propose_transfer');
  static int get safetyFundTransfer => _code('propose_safety_fund_transfer');
  static int get sweepToMain => _code('propose_sweep_to_main');
  static int get bindClearingBank => _code('bind_clearing_bank');
  static int get depositClearingBank => _code('deposit_clearing_bank');
  static int get withdrawClearingBank => _code('withdraw_clearing_bank');
  static int get switchClearingBank => _code('switch_clearing_bank');
  static int get proposeL2FeeRate => _code('propose_l2_fee_rate');
  static int get registerClearingBank => _code('register_clearing_bank');
  static int get updateClearingBankEndpoint =>
      _code('update_clearing_bank_endpoint');
  static int get unregisterClearingBank => _code('unregister_clearing_bank');
  static int get internalVote => _code('internal_vote');
  static int get jointVote => _code('joint_vote');
  static int get castReferendum => _code('cast_referendum');
  static int get castPopularVote => _code('cast_popular_vote');
  static int get castMutualVote => _code('cast_mutual_vote');

  // 链上资产 OnchainIssuance(23 = 0x17)。动作码与 runtime call_index 一一对应。
  static int get proposeAssetIssue => _code('propose_asset_issue');
  static int get proposeAssetMint => _code('propose_asset_mint');
  static int get proposeAssetBurn => _code('propose_asset_burn');
  static int get proposeAssetClose => _code('propose_asset_close');
  static int get proposeAssetTransfer => _code('propose_asset_transfer');
  static int get proposeMonitorFreeze => _code('propose_monitor_freeze');
  static int get proposeMonitorUnfreeze => _code('propose_monitor_unfreeze');
  static int get proposeMonitorConfiscate =>
      _code('propose_monitor_confiscate');
  static int get proposeMonitorForceTransfer =>
      _code('propose_monitor_force_transfer');
  static int get proposeMonitorForceClose =>
      _code('propose_monitor_force_close');

  // 注册局地址目录 AddressRegistry(33 = 0x21)
  static int get setAddressCatalogVersion =>
      _code('set_address_catalog_version');
  static int get setAddressName => _code('set_address_name');
  static int get removeAddressName => _code('remove_address_name');
  static int get setAddress => _code('set_address');
  static int get removeAddress => _code('remove_address');

  // 公民链基金会平台调价提案 SquarePost(34 = 0x22)
  static int get proposeSetPlatformPrice => _code('propose_set_platform_price');

  // 立法院 LegislationYuan(25 = 0x19)
  static int get proposeEnactLaw => _code('propose_enact_law');
  static int get proposeAmendLaw => _code('propose_amend_law');
  static int get proposeRepealLaw => _code('propose_repeal_law');

  // 立法投票 LegislationVote(26 = 0x1a)
  static int get castRepresentativeVote => _code('cast_representative_vote');
  static int get castLegislationReferendum => _code('cast_referendum_vote');
  static int get executiveSign => _code('executive_sign');
  static int get overrideSign => _code('override_sign');
  static int get guardVote => _code('guard_vote');

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
