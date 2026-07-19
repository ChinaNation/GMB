import 'dart:convert';
import 'dart:typed_data';

import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import '../chain/chain_constants.dart';
import '../qr/generated/qr_action_registry.g.dart';
import 'institution_code.dart';
import 'pallet_registry.dart';

/// payload_hex 中 call data 的解码结果。
///
/// 离线设备用此结果向用户展示交易详情，并把解出的动作与
/// QR_V1 `b.a` 数字动作码交叉比对。
class DecodedPayload {
  const DecodedPayload({
    required this.action,
    required this.summary,
    required this.fields,
    Map<String, String>? reviewFields,
  }) : reviewFields = reviewFields ?? fields;

  /// 动作标识，供本地动作码表映射和确认页展示使用。
  final String action;

  /// 一句话摘要。
  final String summary;

  /// 结构化机器字段，用于从 payload 独立验真。
  final Map<String, String> fields;

  /// 用户确认页展示字段。
  ///
  /// `fields` 保留独立验真的机器字段，`reviewFields` 只放人能判断的
  /// 中文业务信息和 SS58 地址，避免把内部 ID、原始公钥 hex 暴露为确认内容。
  final Map<String, String> reviewFields;
}

/// 管理员人员的钱包端解码结果。
///
/// 字段名与 runtime、CitizenApp 完全一致；账户是唯一授权与去重
/// 字段，姓、名只在确认页按中文顺序合并显示。
typedef _DecodedAdminPerson = ({
  Uint8List accountBytes,
  String adminAccountHex,
  String familyName,
  String givenName,
});

/// SCALE call data 解码器。
///
/// 根据 pallet_index + call_index 识别已知交易类型，
/// 从 call data 中提取关键字段并生成人可读摘要。
///
/// 注意：此解码器工作在 call data 层面（即 SigningPayload.method），
/// 而非完整的 SigningPayload 编码。sign_request 中的 payload_hex
/// 是完整的 SigningPayload 编码，call data 位于其头部。
///
/// 目前仅支持高风险交易类型，不支持的返回 null。
class PayloadDecoder {
  /// SS58 地址前缀。
  static const int _ss58Prefix = ChainConstants.ss58Prefix;

  /// 固定审阅展示值统一来自 qr-protocol 生成表；decoder 只提供动态字段。
  static String _reviewValue(
    String fieldKey,
    Map<String, String> values,
  ) {
    return GeneratedQrActionRegistry.fieldValueForKey(fieldKey, values) ?? '';
  }

  /// 尝试从 payload_hex 中解码交易信息。
  ///
  /// [payloadHex] 为完整 SigningPayload 编码的 hex 字符串。
  /// call data 从 payload 起始位置开始，以 pallet_index 和 call_index 为前两字节。
  ///
  /// 返回 null 表示无法识别或解码失败 → strict 模式下 Reject → 禁止签名。
  // 二进制前缀域:
  //   ACTIVATE_ADMIN → 前 4 字节 GMB(3B) || 0x18
  //   DECRYPT        → 前 4 字节 GMB(3B) || 0x19
  // 这两个域签的是**原始可解析字节**:冷钱包对整段 payloadHex 直接 sr25519 签名,
  // 本 decoder 仅读 4 字节前缀跳过 + 按偏移取字段展示。与
  // primitives::sign::signing_message(blake2_256(GMB||op_tag||SCALE)) 的哈希域
  // 结构不同(它们不进 SIGN_OP_TAGS,不走 signingMessage)。
  //
  // 四方逐字节锁步(node 构造 / node 验签解析 / 本 decoder 解码 / CitizenApp 构造):
  //   - citizenchain node: governance/admins_change/activation.rs(build/decode)、
  //     transaction/.../settlement/admin_unlock.rs(build_challenge_payload)
  //   - CitizenApp: governance/admins-change/services/admin_activation_service.dart
  // 金标布局: citizenchain/runtime/primitives/tests/fixtures/
  //   binary_prefix_domain_vectors.json(本仓副本 test/signer/fixtures/)。
  //
  // 前缀 = core_const::GMB(0x47 0x4D 0x42)|| op_tag(1B)。单源对齐
  // primitives::sign::binary_domain_prefix / BINARY_PREFIX_LEN。

  /// 二进制前缀域 GMB 域分隔符(3 字节 ASCII),单源对齐 core_const::GMB。
  static const _gmbPrefix = [0x47, 0x4D, 0x42]; // "GMB"

  /// 二进制前缀域统一前缀长度 = GMB(3B) + op_tag(1B) = 4(对齐 BINARY_PREFIX_LEN)。
  static const _binaryPrefixLen = 4;

  /// 机构 CID 固定槽长度，与 primitives::core_const::CID_NUMBER_MAX_BYTES 对齐。
  static const _institutionCidSlotLen = 32;

  /// 对齐 primitives::sign::ACTIVATE_ADMIN_PAYLOAD_LEN。
  static const _activateAdminPayloadLen =
      _binaryPrefixLen + _institutionCidSlotLen + 4 + 1 + 32 + 8 + 16;

  /// 对齐 primitives::sign::DECRYPT_ADMIN_PAYLOAD_LEN。
  static const _decryptAdminPayloadLen =
      _binaryPrefixLen + _institutionCidSlotLen + 32 + 8 + 16;

  /// ACTIVATE_ADMIN op_tag(对齐 OP_SIGN_ACTIVATE_ADMIN)。
  static const _opSignActivateAdmin = 0x18;

  /// DECRYPT op_tag(对齐 OP_SIGN_DECRYPT)。
  static const _opSignDecrypt = 0x19;

  /// ACTIVATE_ADMIN 4 字节二进制前缀 GMB || 0x18。
  static final _activateAdminPrefix = Uint8List.fromList(
    [..._gmbPrefix, _opSignActivateAdmin],
  );

  /// DECRYPT 4 字节二进制前缀 GMB || 0x19。
  static final _decryptPrefix = Uint8List.fromList(
    [..._gmbPrefix, _opSignDecrypt],
  );
  // 'onchina_admin_governance' 是链上中国平台管理员 payload 的业务域(JSON
  // envelope 的 domain 字段值),不是 signing_message 的哈希签名域 —— 整段 JSON
  // 字节由 sr25519 直接签名,故不并入 op_tag 注册表。
  static const String _onchinaAdminActionDomain = 'onchina_admin_governance';

  static DecodedPayload? decode(String payloadHex) {
    // 先尝试解码非链上交易：管理员激活 / 清算行管理员解密 challenge。
    try {
      final raw = _hexToBytes(payloadHex);
      final adminAction = _decodeOnchinaAdminAction(raw);
      if (adminAction != null) {
        return adminAction;
      }
      if (raw.length == _activateAdminPayloadLen &&
          _hasPrefix(raw, _activateAdminPrefix)) {
        return _decodeActivateAdminAccount(raw);
      }
      // DECRYPT challenge = prefix(4) + cid_number(32) + pubkey(32)
      //   + timestamp(8) + nonce(16)，长度由 primitives 单源常量锁定。
      if (raw.length == _decryptAdminPayloadLen &&
          _hasPrefix(raw, _decryptPrefix)) {
        return _decodeDecryptAdmin(raw);
      }
      final citizenIdentity = _decodeCitizenIdentityPayload(raw);
      if (citizenIdentity != null) {
        return citizenIdentity;
      }
    } catch (_) {
      // 非 challenge payload，继续正常解码。
    }

    // 防误签由 strict 两色模式独家把关:
    // - decoder 解析失败(任何分支不匹配返回 null) → Reject → 禁止签名
    // - 解析成功但 QR 动作码与 decoded.action 不一致 → Reject → 禁止签名
    // 不按 spec_version 锁布局,合法新 spec 可直接解码,布局变了 strict 模式自动拦截。
    try {
      final bytes = _hexToBytes(payloadHex);
      if (bytes.length < 2) return null;

      final palletIndex = bytes[0];
      final callIndex = bytes[1];

      // OnchainTransaction / transfer_with_remark
      if (palletIndex == PalletRegistry.onchainTransactionPallet &&
          callIndex == PalletRegistry.transferWithRemarkCall) {
        return _decodeTransferWithRemark(bytes);
      }

      // ── InternalVote sub-pallet (20) · 内部投票管理员一人一票 ──
      if (palletIndex == PalletRegistry.internalVotePallet &&
          callIndex == PalletRegistry.internalVoteCall) {
        return _decodeInternalVote(bytes);
      }

      // ── JointVote sub-pallet (21) · 联合投票(内部投票阶段 + 联合公投)──
      if (palletIndex == PalletRegistry.jointVotePallet) {
        if (callIndex == PalletRegistry.jointVoteCall) {
          return _decodeJointVote(bytes);
        }
        if (callIndex == PalletRegistry.castReferendumCall) {
          return _decodeCastReferendum(bytes);
        }
      }

      // ── VotingEngine(9) · 引擎核心生命周期 extrinsic ──
      // 仅承载 finalize_proposal / retry_passed_proposal / cancel_passed_proposal。
      if (palletIndex == PalletRegistry.votingEnginePallet) {
        if (callIndex == PalletRegistry.finalizeProposalCall) {
          return _decodeFinalizeProposal(bytes);
        }
        if (callIndex == PalletRegistry.retryPassedProposalCall) {
          return _decodeProposalIdOnly(
            bytes,
            action: 'retry_passed_proposal',
            summaryTemplate: '手动触发已通过提案 #{id} 执行',
          );
        }
        if (callIndex == PalletRegistry.cancelPassedProposalCall) {
          return _decodeCancelPassedProposal(bytes);
        }
      }

      // ── CitizenIdentity(10) · 公民链上投票/参选身份注册 + 注册局占号/吊销 ──
      if (palletIndex == PalletRegistry.citizenIdentityPallet) {
        if (callIndex == PalletRegistry.registerVotingIdentityCall) {
          return _decodeRegisterVotingIdentity(bytes);
        }
        if (callIndex == PalletRegistry.upgradeToCandidateIdentityCall) {
          return _decodeUpgradeToCandidateIdentity(bytes);
        }
        if (callIndex == PalletRegistry.updateVotingIdentityCall) {
          return _decodeUpdateVotingIdentity(bytes);
        }
        if (callIndex == PalletRegistry.updateCandidateIdentityCall) {
          return _decodeUpdateCandidateIdentity(bytes);
        }
        if (callIndex == PalletRegistry.revokeIdentityCall) {
          return _decodeRevokeIdentity(bytes);
        }
        if (callIndex == PalletRegistry.occupyCidCall) {
          return _decodeOccupyCid(bytes);
        }
        if (callIndex == PalletRegistry.occupyCidsBatchCall) {
          return _decodeOccupyCidsBatch(bytes);
        }
        if (callIndex == PalletRegistry.revokeCidCall) {
          return _decodeRevokeCid(bytes);
        }
      }

      // ── MultisigTransfer(17) ──
      // 投票入口统一到 InternalVote::cast(20.0),
      // 手动重试入口统一到 VotingEngine::retry_passed_proposal(9.4),
      // 本 pallet 仅保留 3 条 propose_X。
      if (palletIndex == PalletRegistry.multisigTransferPallet) {
        if (callIndex == PalletRegistry.proposeTransferCall) {
          return _decodeProposeTransfer(bytes);
        }
        if (callIndex == PalletRegistry.proposeSafetyFundCall) {
          return _decodeProposeSafetyFund(bytes);
        }
        if (callIndex == PalletRegistry.proposeSweepCall) {
          return _decodeProposeSweep(bytes);
        }
      }

      // ── 协议升级 RuntimeUpgrade(12) ──
      // 路由分支删除:propose_runtime_upgrade / developer_direct_upgrade 的
      // call_data 含完整 WASM(600KB+),物理上塞不进 QR。server 在 QR 里只放
      // blake2_256(payload) = 32 字节哈希,decoder 拿不到 call_data,无法
      // SCALE 解析。改走 OfflineSignService.verifyPayload 的"哈希直签例外":
      // 用户在冷钱包屏幕上核对 32 字节哈希后放行。

      // ── PublicManage(30) / PrivateManage(31) ──
      // 公权机构与私权机构生命周期分别由两个 pallet 承载。
      if (palletIndex == PalletRegistry.publicManagePallet ||
          palletIndex == PalletRegistry.privateManagePallet) {
        final isPublic = palletIndex == PalletRegistry.publicManagePallet;
        final entityLabel = isPublic ? '公权机构' : '私权机构';
        final createAction = isPublic
            ? 'propose_create_public_institution'
            : 'propose_create_private_institution';
        final closeAction = isPublic
            ? 'propose_close_public_institution'
            : 'propose_close_private_institution';
        if (callIndex == PalletRegistry.proposeCloseInstitutionCall) {
          // 机构 propose_close 携带注销凭证(nonce/签名/签发机构/签发管理员公钥),
          // 比个人多签多 3 个 Vec + 2×32,需专用解码;个人多签仍走 66 字节 _decodeProposeClose。
          return _decodeProposeCloseInstitution(
            bytes,
            action: closeAction,
            entityLabel: entityLabel,
          );
        }
        if (callIndex == PalletRegistry.proposeCreateInstitutionCall) {
          return _decodeProposeCreateInstitution(
            bytes,
            action: createAction,
            entityLabel: entityLabel,
          );
        }
        if (callIndex == PalletRegistry.updateInstitutionInfoCall) {
          return _decodeUpdateInstitutionInfo(
            bytes,
            action: isPublic
                ? 'update_public_institution_info'
                : 'update_private_institution_info',
            entityLabel: entityLabel,
          );
        }
        if (callIndex == PalletRegistry.addInstitutionAccountCall) {
          return _decodeAddInstitutionAccount(
            bytes,
            action: isPublic
                ? 'add_public_institution_account'
                : 'add_private_institution_account',
            entityLabel: entityLabel,
          );
        }
        if (callIndex == PalletRegistry.proposeInstitutionGovernanceCall) {
          return _decodeProposeInstitutionGovernance(
            bytes,
            action: isPublic
                ? 'propose_public_institution_governance'
                : 'propose_private_institution_governance',
            entityLabel: entityLabel,
          );
        }
        if (callIndex == PalletRegistry.registerInstitutionAdminsCall) {
          return _decodeRegisterInstitutionAdmins(
            bytes,
            action: isPublic
                ? 'register_public_institution_admins'
                : 'register_private_institution_admins',
            entityLabel: entityLabel,
          );
        }
      }

      // ── PersonalManage(7) ──
      // 个人多签生命周期 pallet,MODULE_TAG = b"per-mgmt"。
      // ACTION enum 独立(ACTION_CREATE=0/ACTION_CLOSE=1),与实体生命周期模块互不干扰。
      if (palletIndex == PalletRegistry.personalManagePallet) {
        if (callIndex == PalletRegistry.proposeCreatePersonalCall) {
          return _decodeProposeCreatePersonal(bytes);
        }
        if (callIndex == PalletRegistry.proposeClosePersonalCall) {
          return _decodeProposeClose(
            bytes,
            action: 'propose_close_personal',
            summaryLabel: '个人多签',
          );
        }
      }

      // ── PersonalAdmins(29) ──
      // 个人多签管理员集合变更独立 pallet,propose_admin_set_change(call_index=0)。
      if (PalletRegistry.isPersonalAdminSetChangeCall(palletIndex, callIndex)) {
        return _decodeProposeAdminSetChange(bytes);
      }

      // ── ResolutionIssuance(8) · 决议发行联合提案 ──
      // 全国人口快照随联合提案创建由投票引擎内联生成。
      if (palletIndex == PalletRegistry.resolutionIssuancePallet) {
        if (callIndex == PalletRegistry.proposeIssuanceCall) {
          return _decodeProposeResolutionIssuance(bytes);
        }
      }

      // ── ResolutionDestroy(13) ──
      // execute_destroy 走 VotingEngine::retry_passed_proposal。
      if (palletIndex == PalletRegistry.resolutionDestroyPallet) {
        if (callIndex == PalletRegistry.proposeDestroyCall) {
          return _decodeProposeDestroy(bytes);
        }
      }

      // ── GrandpaKeyChange(15) ──
      // execute_replace_grandpa_key / cancel_failed_replace_grandpa_key
      // 分别走 VotingEngine::retry_passed_proposal / cancel_passed_proposal。
      if (palletIndex == PalletRegistry.grandpaKeyChangePallet) {
        if (callIndex == PalletRegistry.proposeReplaceGrandpaKeyCall) {
          return _decodeProposeKeyChange(bytes);
        }
      }

      // ── OffchainTransaction(19) · 清算行 L2 体系 ──
      if (palletIndex == PalletRegistry.offchainTransactionPallet) {
        if (callIndex == PalletRegistry.bindClearingBankCall) {
          return _decodeAccountIdCall(
            bytes,
            action: 'bind_clearing_bank',
            summaryPrefix: '绑定清算行',
            fieldKey: 'bank_main',
          );
        }
        if (callIndex == PalletRegistry.depositCall) {
          return _decodeAmountOnlyCall(
            bytes,
            action: 'deposit_clearing_bank',
            summaryPrefix: '充值到清算行',
          );
        }
        if (callIndex == PalletRegistry.withdrawCall) {
          return _decodeAmountOnlyCall(
            bytes,
            action: 'withdraw_clearing_bank',
            summaryPrefix: '从清算行提现',
          );
        }
        if (callIndex == PalletRegistry.switchBankCall) {
          return _decodeAccountIdCall(
            bytes,
            action: 'switch_clearing_bank',
            summaryPrefix: '切换清算行',
            fieldKey: 'new_bank',
          );
        }
        if (callIndex == PalletRegistry.registerClearingBankCall) {
          return _decodeRegisterClearingBank(bytes);
        }
        if (callIndex == PalletRegistry.updateClearingBankEndpointCall) {
          return _decodeUpdateClearingBankEndpoint(bytes);
        }
        if (callIndex == PalletRegistry.unregisterClearingBankCall) {
          return _decodeUnregisterClearingBank(bytes);
        }
        if (callIndex == PalletRegistry.proposeL2FeeRateCall) {
          return _decodeProposeL2FeeRate(bytes);
        }
      }

      // ── AddressRegistry(33) · 注册局地址目录 ──
      if (palletIndex == PalletRegistry.addressRegistryPallet) {
        return _decodeAddressRegistryCall(bytes, callIndex);
      }

      // ── SquarePost(34) · 技术公司发起平台会员调价内部投票 ──
      if (palletIndex == PalletRegistry.squarePostPallet &&
          callIndex == PalletRegistry.proposeSetPlatformPriceCall) {
        return _decodeProposeSetPlatformPrice(bytes);
      }

      // ── OnchainIssuance(23) · 链上发行代币(Plain FT) ──
      if (palletIndex == PalletRegistry.onchainIssuancePallet) {
        return switch (callIndex) {
          PalletRegistry.proposeIssueCall => _decodeProposeAssetIssue(bytes),
          PalletRegistry.proposeMintCall => _decodeProposeAssetMint(bytes),
          PalletRegistry.proposeBurnCall => _decodeProposeAssetBurn(bytes),
          PalletRegistry.proposeCloseAssetCall =>
            _decodeProposeAssetClose(bytes),
          PalletRegistry.proposeAssetTransferCall =>
            _decodeProposeAssetTransfer(bytes),
          PalletRegistry.proposeMonitorFreezeCall =>
            _decodeProposeMonitorFreeze(bytes, unfreeze: false),
          PalletRegistry.proposeMonitorUnfreezeCall =>
            _decodeProposeMonitorFreeze(bytes, unfreeze: true),
          PalletRegistry.proposeMonitorConfiscateCall =>
            _decodeProposeMonitorConfiscate(bytes),
          PalletRegistry.proposeMonitorForceTransferCall =>
            _decodeProposeMonitorForceTransfer(bytes),
          PalletRegistry.proposeMonitorForceCloseCall =>
            _decodeProposeMonitorForceClose(bytes),
          _ => null,
        };
      }

      // ── LegislationYuan(25) · 立法/修法/废法发起 ──
      // 发起类 QR 由节点端生成(法律全文 SCALE 入 payload),冷钱包逐字段解码核对。
      if (palletIndex == PalletRegistry.legislationYuanPallet) {
        if (callIndex == PalletRegistry.proposeEnactLawCall) {
          return _decodeProposeEnactLaw(bytes);
        }
        if (callIndex == PalletRegistry.proposeAmendLawCall) {
          return _decodeProposeAmendLaw(bytes);
        }
        if (callIndex == PalletRegistry.proposeRepealLawCall) {
          return _decodeProposeRepealLaw(bytes);
        }
      }

      // ── LegislationVote(26) · 立法专属投票引擎 ──
      // 代表机构表决/公投/行政签署/三人会签/护宪终审走 proposal_id+approve；
      // 特别案人口作用域由投票引擎按 actor CID 推导。
      if (palletIndex == PalletRegistry.legislationVotePallet) {
        if (callIndex == PalletRegistry.castRepresentativeVoteCall) {
          return _decodeProposalApprove(
            bytes,
            action: 'cast_representative_vote',
            summaryTemplate: '代表机构表决 立法提案 #{id}：{vote}',
          );
        }
        if (callIndex == PalletRegistry.castLegislationReferendumCall) {
          return _decodeCastLegislationReferendum(bytes);
        }
        if (callIndex == PalletRegistry.executiveSignCall) {
          return _decodeProposalApprove(
            bytes,
            action: 'executive_sign',
            summaryTemplate: '行政签署 立法提案 #{id}：{vote}',
          );
        }
        if (callIndex == PalletRegistry.overrideSignCall) {
          return _decodeProposalApprove(
            bytes,
            action: 'override_sign',
            summaryTemplate: '三人会签 立法提案 #{id}：{vote}',
          );
        }
        if (callIndex == PalletRegistry.guardVoteCall) {
          return _decodeProposalApprove(
            bytes,
            action: 'guard_vote',
            summaryTemplate: '护宪终审 立法提案 #{id}：{vote}',
          );
        }
      }

      return null;
    } catch (_) {
      return null;
    }
  }

  static DecodedPayload? _decodeOnchinaAdminAction(Uint8List raw) {
    try {
      final text = utf8.decode(raw);
      final value = jsonDecode(text);
      if (value is! Map<String, dynamic>) return null;
      if (value['domain'] != _onchinaAdminActionDomain) return null;
      if (value['qr_proto'] != 'QR_V1') return null;
      final actionType = value['action_type'];
      final actorCidNumber = value['actor_cid_number'];
      final province = value['actor_province_name'];
      final actorPubkey = value['actor_pubkey'];
      final target = value['target'];
      final beforeHash = value['before_hash'];
      final afterHash = value['after_hash'];
      if (actionType is! String ||
          actorCidNumber is! String ||
          !_isStructuredInstitutionCid(actorCidNumber) ||
          province is! String ||
          actorPubkey is! String ||
          target is! String ||
          beforeHash is! String ||
          afterHash is! String) {
        return null;
      }
      final actorAddress = _pubkeyHexToSs58OrRaw(actorPubkey);
      final targetAddress = _pubkeyHexToSs58OrRaw(target);
      return DecodedPayload(
        action: 'onchina_admin_action',
        summary: '链上中国平台管理员治理',
        fields: <String, String>{
          'action_type': _onchinaAdminActionLabel(actionType),
          'actor_cid_number': actorCidNumber,
          'actor_province_name': province,
          'actor_pubkey': actorAddress,
          'target': targetAddress,
          'before_hash': beforeHash,
          'after_hash': afterHash,
        },
        reviewFields: <String, String>{
          'action_type': _onchinaAdminActionLabel(actionType),
          'actor_cid_number': actorCidNumber,
          'actor_province_name': province,
          'actor_pubkey': actorAddress,
          'target': targetAddress,
        },
      );
    } catch (_) {
      return null;
    }
  }

  static String _onchinaAdminActionLabel(String actionType) {
    switch (actionType) {
      case 'PASSKEY_REGISTER':
        return '更新 Passkey';
      case 'CREATE_ADMIN':
        return '新增管理员';
      case 'UPDATE_ADMIN':
        return '编辑管理员';
      case 'DELETE_ADMIN':
        return '删除管理员';
      case 'INSTITUTION_CREATE':
        return '创建机构';
      case 'INSTITUTION_UPDATE':
        return '更新机构';
      case 'INSTITUTION_CREATE_ACCOUNT':
        return '新增机构账户';
      case 'INSTITUTION_DELETE_ACCOUNT':
        return '删除机构账户';
      case 'INSTITUTION_UPLOAD_DOCUMENT':
        return '上传机构文档';
      case 'INSTITUTION_DELETE_DOCUMENT':
        return '删除机构文档';
      case 'CITIZEN_BIND_COMMIT':
        return '确认电子护照绑定';
      default:
        return actionType;
    }
  }

  // OnchainTransaction(4) / transfer_with_remark(0)
  // 格式：[0x04][0x00][beneficiary:AccountId32][amount:u128_le][remark:BoundedVec<u8>]
  static DecodedPayload? _decodeTransferWithRemark(Uint8List bytes) {
    // 2 (pallet+call) + 32 (AccountId) + 16 (u128) + 至少 1 (Vec len)
    if (bytes.length < 51) return null;

    var offset = 2;

    // 收款地址 32 bytes
    final toAccountId = bytes.sublist(offset, offset + 32);
    offset += 32;
    final toAddress =
        Keyring().encodeAddress(toAccountId.toList(), _ss58Prefix);

    // amount: u128 little-endian（分）
    final amountFen = _readU128Le(bytes, offset);
    offset += 16;

    // remark: BoundedVec<u8>
    final (remarkLen, remarkLenSize) = _decodeCompactU32(bytes, offset);
    if (remarkLenSize == 0) return null;
    offset += remarkLenSize;
    if (offset + remarkLen > bytes.length) return null;
    if (!_hasValidSigningTail(bytes, offset + remarkLen)) return null;
    final remark = remarkLen == 0
        ? ''
        : utf8.decode(
            bytes.sublist(offset, offset + remarkLen),
            allowMalformed: true,
          );

    final amountYuan = _fenToYuan(amountFen);
    final remarkSuffix = remark.isEmpty ? '' : '，备注：$remark';

    return DecodedPayload(
      action: 'transfer',
      summary:
          '转账 $amountYuan GMB 给 ${_truncateAddress(toAddress)}$remarkSuffix',
      fields: {
        'to': toAddress,
        'amount_yuan': '$amountYuan GMB',
        'remark': remark,
      },
    );
  }

  // MultisigTransfer(17) / propose_transfer(0)
  // 格式：[0x11][0x00][actor_cid_number:Option<CidNumber>]
  //      [funding_account:AccountId32][beneficiary:AccountId32][amount:u128][remark:Vec]。
  // Some(CID) 是机构账户交易；None 是个人多签交易，禁止用账户反推机构身份。
  static DecodedPayload? _decodeProposeTransfer(Uint8List bytes) {
    if (bytes.length < 2 + 1 + 32 + 32 + 16 + 1) return null;
    var offset = 2;
    final actorRead = _readOptionalCidNumber(bytes, offset);
    if (actorRead == null) return null;
    final actorCidNumber = actorRead.$1;
    offset = actorRead.$2;
    if (offset + 32 + 32 + 16 > bytes.length) return null;
    final fundingAccountBytes = bytes.sublist(offset, offset + 32);
    offset += 32;
    final fundingAccount = _bytesToSs58(fundingAccountBytes);

    // beneficiary: 32 bytes（无 MultiAddress 前缀）
    final beneficiaryId = bytes.sublist(offset, offset + 32);
    offset += 32;
    final beneficiary =
        Keyring().encodeAddress(beneficiaryId.toList(), _ss58Prefix);

    // amount: u128 little-endian（16 字节）
    final amountFen = _readU128Le(bytes, offset);
    offset += 16;
    final amountYuan = _fenToYuan(amountFen);

    // remark: Vec<u8>
    final (remarkLen, remarkLenSize) = _decodeCompactU32(bytes, offset);
    if (remarkLenSize == 0) return null;
    offset += remarkLenSize;
    if (offset + remarkLen > bytes.length) return null;
    if (!_hasValidSigningTail(bytes, offset + remarkLen)) return null;
    var remark = '';
    if (remarkLen > 0) {
      remark = utf8.decode(bytes.sublist(offset, offset + remarkLen),
          allowMalformed: true);
    }

    return DecodedPayload(
      action: 'propose_transfer',
      summary:
          '${actorCidNumber == null ? '个人多签' : '机构 $actorCidNumber'}提案转账 $amountYuan GMB 给 ${_truncateAddress(beneficiary)}',
      fields: <String, String>{
        if (actorCidNumber != null) 'actor_cid_number': actorCidNumber,
        if (actorCidNumber != null) 'institution_account': fundingAccount,
        if (actorCidNumber == null) 'personal_account': fundingAccount,
        'operation_fee_payer': actorCidNumber == null
            ? '签名管理员钱包'
            : _reviewValue(
                'operation_fee_payer',
                {'actor_cid_number': actorCidNumber},
              ),
        'execution_fee_payer': actorCidNumber == null
            ? fundingAccount
            : _reviewValue(
                'execution_fee_payer',
                {'actor_cid_number': actorCidNumber},
              ),
        'beneficiary': beneficiary,
        'amount_yuan': '$amountYuan GMB',
        'remark': remark,
      },
    );
  }

  // 业务 pallet 的 finalize_X / vote_X 全部下线,
  // 冷钱包统一通过 `_decodeInternalVote` 解码一人一票的管理员投票 payload。
  // InternalVote(20) / cast(0)
  // 格式：[0x14][0x00][proposal_id:u64_le][approve:bool]
  //
  // 统一入口:所有业务 pallet(admins/resolution_destroy/grandpa_key/
  // entity_manage/multisig_transfer 五路)的管理员投票都走 InternalVote::cast(20.0),
  // 冷钱包不按业务 pallet 分路解码投票 payload。
  static DecodedPayload? _decodeInternalVote(Uint8List bytes) {
    // call_data: 2 + 8 + 1 = 11
    if (bytes.length < 11 || !_hasValidSigningTail(bytes, 11)) return null;
    final proposalId = _readU64Le(bytes, 2);
    final approve = bytes[10] != 0;
    final voteText = approve ? '赞成' : '反对';
    return DecodedPayload(
      action: 'internal_vote',
      summary: '管理员投票 提案 #$proposalId：$voteText',
      fields: {
        'proposal_id': proposalId.toString(),
        'approve': approve.toString(),
      },
    );
  }

  // VotingEngine(9) / finalize_proposal(3)
  // 格式：[0x09][0x03][proposal_id:u64_le]
  //
  // 任意账户触发终态执行,无需签投票语义。引擎核心 lifecycle extrinsic。
  static DecodedPayload? _decodeFinalizeProposal(Uint8List bytes) {
    // call_data: 2 + 8 = 10
    if (bytes.length < 10 || !_hasValidSigningTail(bytes, 10)) return null;
    final proposalId = _readU64Le(bytes, 2);
    return DecodedPayload(
      action: 'finalize_proposal',
      summary: '触发提案 #$proposalId 终态执行',
      fields: {
        'proposal_id': proposalId.toString(),
      },
    );
  }

  // VotingEngine(9) / cancel_passed_proposal(5)
  //
  // 链端签名(统一取消入口,引擎核心 extrinsic):
  //   pub fn cancel_passed_proposal(
  //     origin,
  //     proposal_id: u64,
  //     _reason: BoundedVec<u8, MaxProposalDataLen>,
  //   )
  //
  // SCALE: [0x09][0x05][proposal_id:u64_le][Compact<u32> reason_len + reason_bytes]
  static DecodedPayload? _decodeCancelPassedProposal(Uint8List bytes) {
    if (bytes.length < 11) return null;
    final proposalId = _readU64Le(bytes, 2);
    var offset = 10;

    // _reason: BoundedVec<u8> — 空 reason 也带 1 字节 Compact(0) 前缀。
    final (reasonLen, reasonLenSize) = _decodeCompactU32(bytes, offset);
    if (reasonLenSize == 0) return null;
    offset += reasonLenSize;
    if (offset + reasonLen > bytes.length) return null;
    var reason = '';
    if (reasonLen > 0) {
      reason = utf8.decode(
        bytes.sublist(offset, offset + reasonLen),
        allowMalformed: true,
      );
    }
    offset += reasonLen;
    if (!_hasValidSigningTail(bytes, offset)) return null;

    return DecodedPayload(
      action: 'cancel_passed_proposal',
      summary: '取消已通过但不可执行的提案 #$proposalId',
      fields: {
        'proposal_id': proposalId.toString(),
        'reason': reason,
      },
    );
  }

  // JointVote(21) / cast_admin(0)
  // 格式：[0x15][0x00][proposal_id:u64_le][cid_number:CidNumber][approve:bool]
  static DecodedPayload? _decodeJointVote(Uint8List bytes) {
    if (bytes.length < 12) return null;
    final proposalId = _readU64Le(bytes, 2);
    final cidRead = _readCidNumber(bytes, 10);
    if (cidRead == null || cidRead.$2 >= bytes.length) return null;
    final cidNumber = cidRead.$1;
    final approve = bytes[cidRead.$2] != 0;
    final callEnd = cidRead.$2 + 1;
    if (!_hasValidSigningTail(bytes, callEnd)) return null;
    final voteText = approve ? '赞成' : '反对';

    return DecodedPayload(
      action: 'joint_vote',
      summary: '联合投票 $cidNumber 对提案 #$proposalId：$voteText',
      fields: {
        'proposal_id': proposalId.toString(),
        'cid_number': cidNumber,
        'approve': approve.toString(),
      },
    );
  }

  // JointVote(21) / cast_referendum(1)
  // 格式：[0x15][0x01][proposal_id:u64_le][approve:bool]
  static DecodedPayload? _decodeCastReferendum(Uint8List bytes) {
    if (bytes.length < 11 || !_hasValidSigningTail(bytes, 11)) return null;
    final proposalId = _readU64Le(bytes, 2);
    final approve = bytes[10] != 0;
    final voteText = approve ? '赞成' : '反对';

    return DecodedPayload(
      action: 'cast_referendum',
      summary: '公民投票 提案 #$proposalId：$voteText',
      fields: {
        'proposal_id': proposalId.toString(),
        'approve': approve.toString(),
      },
    );
  }

  // 协议升级 RuntimeUpgrade(12) / propose_runtime_upgrade(0) / developer_direct_upgrade(2)
  //
  // 无 SCALE decoder:
  //   - call_data 含完整 WASM(600KB+),物理上塞不进 QR
  //   - server signing.rs 对这两个 action 在 QR 里只放 blake2_256(payload)
  //     = 32 字节哈希,decoder 拿不到完整 call_data
  //   - wasm_bytes 不在 QR 里,故 decoder 无法复算 sha256(wasm_bytes)
  //
  // 走 OfflineSignService.verifyPayload 的"哈希直签例外":
  //   - 收到 32 字节 payload + QR 动作码 ∈ {两个 wasm 升级 action}
  //   - 用户在冷钱包屏幕上核对桌面 app 显示的 WASM 哈希后放行,
  //     签的就是这份 WASM 哈希。
  // 管理员激活（非链上交易，二进制前缀域）
  // 格式：prefix(4B = GMB||0x18) + cid_number(32B,右补零)
  //      + institution_code([u8;4]) + kind(u8) + admin_pubkey(32B)
  //      + timestamp(8B, u64 LE) + nonce(16B) = 97B。
  // CID 是机构唯一主键，协议账户不参与本地管理员身份绑定。
  static DecodedPayload? _decodeActivateAdminAccount(Uint8List bytes) {
    if (bytes.length != _activateAdminPayloadLen) return null;

    var offset = _activateAdminPrefix.length;
    final cidRead = _readFixedInstitutionCidSlot(bytes, offset);
    if (cidRead == null) return null;
    final cidNumber = cidRead.$1;
    offset = cidRead.$2;

    final codeBytes = bytes.sublist(offset, offset + 4);
    offset += 4;
    final code = InstitutionCode.codeToString(codeBytes);
    final kind = bytes[offset++];
    if (!_activationAccountKindMatchesCode(code, kind) ||
        !_cidContainsInstitutionCode(cidNumber, code)) {
      return null;
    }

    final pubkey = bytes.sublist(offset, offset + 32);
    final institutionLabel = InstitutionCode.codeLabel(code);

    return DecodedPayload(
      action: 'activate_admin_account',
      summary: '激活$institutionLabel管理员',
      fields: {
        'cid_number': cidNumber,
        'institution_code': institutionLabel,
        'admin_pubkey': _bytesToSs58(pubkey),
      },
      reviewFields: {
        'cid_number': cidNumber,
        'institution_code': institutionLabel,
        'admin_pubkey': _bytesToSs58(pubkey),
      },
    );
  }

  /// 管理员激活载荷中的机构码必须与 CID 核心段一致。
  ///
  /// 这里只核对 CID 内嵌的 3/4 字符机构码，不复制链端校验和与盈利属性规则。
  static bool _cidContainsInstitutionCode(String cidNumber, String code) {
    final segments = cidNumber.split('-');
    return _isStructuredInstitutionCid(cidNumber) &&
        (code.length == 3 || code.length == 4) &&
        segments.length == 4 &&
        segments[1].startsWith(code);
  }

  /// 只镜像 CID 的固定分段/字符布局；完整校验和仍由链端唯一实现裁决。
  static bool _isStructuredInstitutionCid(String cidNumber) {
    return RegExp(r'^[A-Z0-9]{5}-[A-Z0-9]{5}-[0-9]{9}-[0-9]{4}$')
        .hasMatch(cidNumber);
  }

  // 清算行管理员解密（非链上交易，二进制前缀域）。
  // 格式：prefix(4B = GMB||0x19) + cid_number(32B,右补零) + pubkey(32B)
  //      + timestamp(8B, u64 LE) + nonce(16B)。旧 48B 槽位不兼容并直接拒绝。
  static DecodedPayload? _decodeDecryptAdmin(Uint8List bytes) {
    if (bytes.length != _decryptAdminPayloadLen) return null;
    final cidRead = _readFixedInstitutionCidSlot(bytes, _binaryPrefixLen);
    if (cidRead == null) return null;

    return DecodedPayload(
      action: 'decrypt_admin',
      summary: '解密清算行管理员 - ${cidRead.$1}',
      fields: {
        'cid_number': cidRead.$1,
      },
    );
  }

  /// 严格读取右补零的 32 字节机构 CID 槽。
  static (String, int)? _readFixedInstitutionCidSlot(
    Uint8List bytes,
    int offset,
  ) {
    if (offset < 0 || offset + _institutionCidSlotLen > bytes.length) {
      return null;
    }
    final slot = bytes.sublist(offset, offset + _institutionCidSlotLen);
    final zeroIndex = slot.indexOf(0);
    final textLength = zeroIndex < 0 ? slot.length : zeroIndex;
    if (textLength == 0 || slot.sublist(textLength).any((byte) => byte != 0)) {
      return null;
    }
    final cidNumber = utf8.decode(slot.sublist(0, textLength));
    if (!_isStructuredInstitutionCid(cidNumber)) return null;
    return (cidNumber, offset + _institutionCidSlotLen);
  }

  // PublicManage(30) / PrivateManage(31) / propose_create_*_institution(5)
  //
  // 链端调用(外层 origin 是 actor_cid_number 的管理员；不再存在创建机构内层凭证):
  //   pub fn propose_create_*_institution(
  //     origin,
  //     cid_number: CidNumberOf<T>,                 // BoundedVec<u8>
  //     cid_full_name: AccountNameOf<T>,   // BoundedVec<u8>
  //     cid_short_name: AccountNameOf<T>,  // BoundedVec<u8>
  //     town_code: AccountNameOf<T>,       // BoundedVec<u8>
  //     admins: BoundedVec<Admin>,
  //         // 每项固定为 admin_account + family_name + given_name
  //     actor_cid_number: Vec<u8>,
  //   )
  //
  // SCALE 顺序与上述完全一致。机构授权只认 actor CID 下的 admins，
  // 不得恢复主账户身份或主账户管理员根。
  // 禁止在尾部追加 subject_property/sub_type/parent_cid_number 等多余字段。
  static DecodedPayload? _decodeProposeCreateInstitution(
    Uint8List bytes, {
    required String action,
    required String entityLabel,
  }) {
    if (bytes.length < 10) return null;
    var offset = 2;

    // cid_number: BoundedVec<u8>
    final (cidLen, cidLenSize) = _decodeCompactU32(bytes, offset);
    offset += cidLenSize;
    if (offset + cidLen > bytes.length) return null;
    final cidNumber = utf8.decode(bytes.sublist(offset, offset + cidLen),
        allowMalformed: true);
    offset += cidLen;

    // cid_full_name: BoundedVec<u8>
    final (nameLen, nameLenSize) = _decodeCompactU32(bytes, offset);
    offset += nameLenSize;
    if (offset + nameLen > bytes.length) return null;
    final cidFullName = utf8.decode(bytes.sublist(offset, offset + nameLen),
        allowMalformed: true);
    offset += nameLen;

    // cid_short_name: BoundedVec<u8>
    final (shortNameLen, shortNameLenSize) = _decodeCompactU32(bytes, offset);
    offset += shortNameLenSize;
    if (offset + shortNameLen > bytes.length) return null;
    final cidShortName = utf8.decode(
      bytes.sublist(offset, offset + shortNameLen),
      allowMalformed: true,
    );
    offset += shortNameLen;

    // town_code: BoundedVec<u8>。非镇级为空;镇级公权机构为 3 字节镇码。
    final (townCodeLen, townCodeLenSize) = _decodeCompactU32(bytes, offset);
    offset += townCodeLenSize;
    if (offset + townCodeLen > bytes.length) return null;
    final townCode = utf8.decode(
      bytes.sublist(offset, offset + townCodeLen),
      allowMalformed: true,
    );
    offset += townCodeLen;

    final adminsRead = _readAdminPersons(
      bytes,
      offset,
      minCount: 2,
      maxCount: 1989,
    );
    if (adminsRead == null) return null;
    final admins = adminsRead.$1;
    final adminsLen = admins.length;
    offset = adminsRead.$2;

    // actor_cid_number: Vec<u8>。外层 origin 必须属于该 CID 的 admins。
    final actorRead = _readCidNumber(bytes, offset);
    if (actorRead == null) return null;
    final actorCidNumber = actorRead.$1;
    offset = actorRead.$2;

    if (!_hasValidSigningTail(bytes, offset)) return null;

    final reviewValues = {'actor_cid_number': actorCidNumber};
    final fields = <String, String>{
      'cid_number': cidNumber,
      'cid_full_name': cidFullName,
      'cid_short_name': cidShortName,
      'admins_len': adminsLen.toString(),
      'admins': _adminMachineValue(admins),
      'default_role': _reviewValue('default_role', reviewValues),
      'protocol_accounts': _reviewValue('protocol_accounts', reviewValues),
      'fee_payer': _reviewValue('fee_payer', reviewValues),
    };
    fields['actor_cid_number'] = actorCidNumber;
    if (townCode.isNotEmpty) {
      fields['town_code'] = townCode;
    }

    final reviewFields = Map<String, String>.of(fields)
      ..['admins'] = _adminReviewValue(admins);
    return DecodedPayload(
      action: action,
      summary: '创建$entityLabel「$cidFullName」（$adminsLen 名管理员，制度账户自动建立）',
      fields: fields,
      reviewFields: reviewFields,
    );
  }

  // PublicManage(30) / PrivateManage(31) / update_institution_info(6)
  // SCALE: cid_number + cid_full_name + cid_short_name + register_nonce + signature
  //      + actor_cid_number + credential_signer_pubkey + scope_province_name + scope_city_name。
  static DecodedPayload? _decodeUpdateInstitutionInfo(
    Uint8List bytes, {
    required String action,
    required String entityLabel,
  }) {
    var offset = 2;
    final cidRead = _readCidNumber(bytes, offset);
    if (cidRead == null) return null;
    final cidNumber = cidRead.$1;
    offset = cidRead.$2;
    final fullNameRead = _readBoundedUtf8(bytes, offset);
    if (fullNameRead == null || fullNameRead.$1.isEmpty) return null;
    final cidFullName = fullNameRead.$1;
    offset = fullNameRead.$2;
    final shortNameRead = _readBoundedUtf8(bytes, offset);
    if (shortNameRead == null || shortNameRead.$1.isEmpty) return null;
    final cidShortName = shortNameRead.$1;
    offset = shortNameRead.$2;
    offset = _skipBoundedBytes(bytes, offset); // register_nonce
    if (offset < 0) return null;
    offset = _skipBoundedBytes(bytes, offset); // signature
    if (offset < 0) return null;
    final actorRead = _readCidNumber(bytes, offset);
    if (actorRead == null) return null;
    offset = actorRead.$2;
    if (offset + 32 > bytes.length) return null;
    final credentialSigner = bytes.sublist(offset, offset + 32);
    offset += 32;
    final (scopeProvinceName, afterProvince) = _readUtf8Vec(bytes, offset);
    if (scopeProvinceName == null || scopeProvinceName.isEmpty) return null;
    offset = afterProvince;
    final (scopeCityName, afterCity) = _readUtf8Vec(bytes, offset);
    if (scopeCityName == null) return null;
    offset = afterCity;
    if (!_hasValidSigningTail(bytes, offset)) return null;
    return DecodedPayload(
      action: action,
      summary: '更新$entityLabel「$cidFullName」($cidNumber)',
      fields: {
        'cid_number': cidNumber,
        'cid_full_name': cidFullName,
        'cid_short_name': cidShortName,
        'actor_cid_number': actorRead.$1,
        'credential_signer_pubkey': _bytesToSs58(credentialSigner),
        'scope_province_name': scopeProvinceName,
        'scope_city_name': scopeCityName,
      },
    );
  }

  // PublicManage(30) / PrivateManage(31) / add_institution_account(7)
  // SCALE: cid_number + account_names:Vec<BoundedVec<u8>> + register_nonce + signature
  //      + actor_cid_number + credential_signer_pubkey + scope_province_name + scope_city_name。
  static DecodedPayload? _decodeAddInstitutionAccount(
    Uint8List bytes, {
    required String action,
    required String entityLabel,
  }) {
    var offset = 2;
    final cidRead = _readCidNumber(bytes, offset);
    if (cidRead == null) return null;
    final cidNumber = cidRead.$1;
    offset = cidRead.$2;
    final (accountCount, countSize) = _decodeCompactU32(bytes, offset);
    if (countSize == 0 || accountCount == 0) return null;
    offset += countSize;
    final accountNames = <String>[];
    for (var index = 0; index < accountCount; index++) {
      final accountNameRead = _readBoundedUtf8(bytes, offset);
      if (accountNameRead == null ||
          accountNameRead.$1.isEmpty ||
          accountNames.contains(accountNameRead.$1)) {
        return null;
      }
      accountNames.add(accountNameRead.$1);
      offset = accountNameRead.$2;
    }
    offset = _skipBoundedBytes(bytes, offset); // register_nonce
    if (offset < 0) return null;
    offset = _skipBoundedBytes(bytes, offset); // signature
    if (offset < 0) return null;
    final actorRead = _readCidNumber(bytes, offset);
    if (actorRead == null) return null;
    offset = actorRead.$2;
    if (offset + 32 > bytes.length) return null;
    final credentialSigner = bytes.sublist(offset, offset + 32);
    offset += 32;
    final (scopeProvinceName, afterProvince) = _readUtf8Vec(bytes, offset);
    if (scopeProvinceName == null || scopeProvinceName.isEmpty) return null;
    offset = afterProvince;
    final (scopeCityName, afterCity) = _readUtf8Vec(bytes, offset);
    if (scopeCityName == null) return null;
    offset = afterCity;
    if (!_hasValidSigningTail(bytes, offset)) return null;
    return DecodedPayload(
      action: action,
      summary: '为$entityLabel $cidNumber 新增 $accountCount 个机构账户',
      fields: {
        'cid_number': cidNumber,
        'account_names': accountNames.join('、'),
        'account_count': accountCount.toString(),
        'actor_cid_number': actorRead.$1,
        'credential_signer_pubkey': _bytesToSs58(credentialSigner),
        'scope_province_name': scopeProvinceName,
        'scope_city_name': scopeCityName,
      },
    );
  }

  // PublicManage(30) / PrivateManage(31) / propose_institution_governance(8)
  // SCALE: cid_number + InstitutionGovernanceAction + nonce + signature
  //      + actor_cid_number + credential_signer_pubkey + scope_province_name + scope_city_name。
  static DecodedPayload? _decodeProposeInstitutionGovernance(
    Uint8List bytes, {
    required String action,
    required String entityLabel,
  }) {
    var offset = 2;
    final cidRead = _readCidNumber(bytes, offset);
    if (cidRead == null) return null;
    final cidNumber = cidRead.$1;
    offset = cidRead.$2;
    final actionRead = _readInstitutionGovernanceAction(bytes, offset);
    if (actionRead == null) return null;
    offset = actionRead.$3;
    offset = _skipBoundedBytes(bytes, offset); // register_nonce
    if (offset < 0) return null;
    offset = _skipBoundedBytes(bytes, offset); // signature
    if (offset < 0) return null;
    final actorRead = _readCidNumber(bytes, offset);
    if (actorRead == null) return null;
    offset = actorRead.$2;
    if (offset + 32 > bytes.length) return null;
    final credentialSigner = bytes.sublist(offset, offset + 32);
    offset += 32;
    final (scopeProvinceName, afterProvince) = _readUtf8Vec(bytes, offset);
    if (scopeProvinceName == null || scopeProvinceName.isEmpty) return null;
    offset = afterProvince;
    final (scopeCityName, afterCity) = _readUtf8Vec(bytes, offset);
    if (scopeCityName == null) return null;
    offset = afterCity;
    if (!_hasValidSigningTail(bytes, offset)) return null;
    return DecodedPayload(
      action: action,
      summary: '$entityLabel $cidNumber 发起${actionRead.$1}',
      fields: {
        'cid_number': cidNumber,
        'governance_action': actionRead.$1,
        'governance_detail': actionRead.$2,
        'actor_cid_number': actorRead.$1,
        'fee_payer': _reviewValue(
          'fee_payer',
          {'actor_cid_number': actorRead.$1},
        ),
        'credential_signer_pubkey': _bytesToSs58(credentialSigner),
        'scope_province_name': scopeProvinceName,
        'scope_city_name': scopeCityName,
      },
    );
  }

  // PublicManage(30) / PrivateManage(31) / register_institution_admins(9)
  // SCALE: cid_number + admins + nonce + signature + actor_cid_number
  //      + credential_signer_pubkey + scope_province_name + scope_city_name。
  static DecodedPayload? _decodeRegisterInstitutionAdmins(
    Uint8List bytes, {
    required String action,
    required String entityLabel,
  }) {
    var offset = 2;
    final cidRead = _readCidNumber(bytes, offset);
    if (cidRead == null) return null;
    final cidNumber = cidRead.$1;
    offset = cidRead.$2;
    final adminsRead = _readAdminPersons(
      bytes,
      offset,
      minCount: 2,
      maxCount: 1989,
    );
    if (adminsRead == null) return null;
    final admins = adminsRead.$1;
    offset = adminsRead.$2;
    offset = _skipBoundedBytes(bytes, offset); // register_nonce
    if (offset < 0) return null;
    offset = _skipBoundedBytes(bytes, offset); // signature
    if (offset < 0) return null;
    final actorRead = _readCidNumber(bytes, offset);
    if (actorRead == null) return null;
    offset = actorRead.$2;
    if (offset + 32 > bytes.length) return null;
    final credentialSigner = bytes.sublist(offset, offset + 32);
    offset += 32;
    final (scopeProvinceName, afterProvince) = _readUtf8Vec(bytes, offset);
    if (scopeProvinceName == null || scopeProvinceName.isEmpty) return null;
    offset = afterProvince;
    final (scopeCityName, afterCity) = _readUtf8Vec(bytes, offset);
    if (scopeCityName == null) return null;
    offset = afterCity;
    if (!_hasValidSigningTail(bytes, offset)) return null;
    return DecodedPayload(
      action: action,
      summary: '注册局登记$entityLabel $cidNumber 的 ${admins.length} 名管理员',
      fields: {
        'cid_number': cidNumber,
        'admins_len': admins.length.toString(),
        'admins': _adminMachineValue(admins),
        'actor_cid_number': actorRead.$1,
        'fee_payer': _reviewValue(
          'fee_payer',
          {'actor_cid_number': actorRead.$1},
        ),
        'credential_signer_pubkey': _bytesToSs58(credentialSigner),
        'scope_province_name': scopeProvinceName,
        'scope_city_name': scopeCityName,
      },
      reviewFields: {
        'cid_number': cidNumber,
        'admins_len': admins.length.toString(),
        'admins': _adminReviewValue(admins),
        'actor_cid_number': actorRead.$1,
        'fee_payer': _reviewValue(
          'fee_payer',
          {'actor_cid_number': actorRead.$1},
        ),
        'credential_signer_pubkey': _bytesToSs58(credentialSigner),
        'scope_province_name': scopeProvinceName,
        'scope_city_name': scopeCityName,
      },
    );
  }

  // ResolutionIssuance(8) / propose_issuance(0)
  //
  // 链端签名:
  //   pub fn propose_issuance(
  //     origin,
  //     actor_cid_number: CidNumber,
  //     reason: ReasonOf<T>,                  // BoundedVec<u8>
  //     total_amount: BalanceOf<T>,           // u128 LE
  //     allocations: AllocationOf<T>,
  //         // BoundedVec<{ recipient: AccountId32, amount: u128 }>
  //   )
  //
  // 全国人口快照随联合提案创建由投票引擎内联生成，本交易只携带发行内容。
  static DecodedPayload? _decodeProposeResolutionIssuance(Uint8List bytes) {
    if (bytes.length < 3) return null;
    var offset = 2; // 跳过 pallet_index + call_index

    final actorRead = _readCidNumber(bytes, offset);
    if (actorRead == null) return null;
    final actorCidNumber = actorRead.$1;
    offset = actorRead.$2;

    // reason: Vec<u8>
    final (reasonLen, reasonLenSize) = _decodeCompactU32(bytes, offset);
    if (reasonLenSize == 0) return null;
    offset += reasonLenSize;
    if (offset + reasonLen > bytes.length) return null;
    var reason = '';
    if (reasonLen > 0) {
      reason = utf8.decode(
        bytes.sublist(offset, offset + reasonLen),
        allowMalformed: true,
      );
    }
    offset += reasonLen;

    // total_amount: u128 LE(16B 非 Compact)
    if (offset + 16 > bytes.length) return null;
    final totalAmountFen = _readU128Le(bytes, offset);
    offset += 16;

    // allocations: BoundedVec<{ recipient(32) + amount(16) }>
    final (allocLen, allocLenSize) = _decodeCompactU32(bytes, offset);
    offset += allocLenSize;
    if (offset + allocLen * 48 > bytes.length) return null;
    offset += allocLen * 48;

    if (!_hasValidSigningTail(bytes, offset)) return null;

    final amountYuan = _fenToYuan(totalAmountFen);

    return DecodedPayload(
      action: 'propose_issuance',
      summary: '决议发行 $amountYuan GMB（$allocLen 项分配）',
      fields: {
        'actor_cid_number': actorCidNumber,
        'reason': reason,
        'amount_yuan': '$amountYuan GMB',
        'allocation_count': allocLen.toString(),
      },
    );
  }

  // OnchainIssuance(23) / propose_issue(0)
  // SCALE:actor_cid_number + execution_account + AssetClass + name + symbol
  //   + description + decimals:u8 + initial_supply:u128。
  static DecodedPayload? _decodeProposeAssetIssue(Uint8List bytes) {
    var offset = 2;
    final actorRead = _readCidNumber(bytes, offset);
    if (actorRead == null) return null;
    offset = actorRead.$2;
    if (offset + 32 + 1 > bytes.length) return null;
    final executionAccount = _bytesToSs58(
      Uint8List.fromList(bytes.sublist(offset, offset + 32)),
    );
    offset += 32;
    final assetClass = switch (bytes[offset++]) {
      0 => 'Plain',
      1 => 'Pegged',
      _ => null,
    };
    if (assetClass == null) return null;

    final nameRead = _readStrictBoundedUtf8(bytes, offset, maxLength: 64);
    if (nameRead == null) return null;
    offset = nameRead.$2;
    final symbolRead = _readStrictBoundedUtf8(bytes, offset, maxLength: 16);
    if (symbolRead == null) return null;
    offset = symbolRead.$2;
    final descriptionRead =
        _readStrictBoundedUtf8(bytes, offset, maxLength: 256);
    if (descriptionRead == null) return null;
    offset = descriptionRead.$2;
    if (offset + 1 + 16 > bytes.length) return null;
    final decimals = bytes[offset++];
    if (decimals > 18) return null;
    final initialSupply = _readU128Le(bytes, offset);
    offset += 16;
    if (!_hasValidSigningTail(bytes, offset)) return null;

    return DecodedPayload(
      action: 'propose_asset_issue',
      summary:
          '发行资产 ${nameRead.$1}（${symbolRead.$1}），初始供应 ${_formatRawAssetAmount(initialSupply, decimals)}',
      fields: <String, String>{
        'actor_cid_number': actorRead.$1,
        'execution_account': executionAccount,
        'asset_class': assetClass,
        'asset_name': nameRead.$1,
        'asset_symbol': symbolRead.$1,
        'asset_description': descriptionRead.$1,
        'decimals': decimals.toString(),
        'initial_supply_raw': initialSupply.toString(),
      },
    );
  }

  // OnchainIssuance(23) / propose_mint(1)
  // SCALE:actor_cid_number + asset_id:u32 + to:AccountId32 + amount:u128。
  static DecodedPayload? _decodeProposeAssetMint(Uint8List bytes) {
    final header = _readOnchainAssetHeader(bytes);
    if (header == null) return null;
    var offset = header.next;
    if (offset + 32 + 16 > bytes.length) return null;
    final to = _bytesToSs58(
      Uint8List.fromList(bytes.sublist(offset, offset + 32)),
    );
    offset += 32;
    final amount = _readU128Le(bytes, offset);
    offset += 16;
    if (!_hasValidSigningTail(bytes, offset)) return null;
    return DecodedPayload(
      action: 'propose_asset_mint',
      summary:
          '资产 #${header.assetId} 增发 ${amount.toString()} raw 到 ${_truncateAddress(to)}',
      fields: {
        'actor_cid_number': header.actorCidNumber,
        'asset_id': header.assetId.toString(),
        'to': to,
        'amount_raw': amount.toString(),
      },
    );
  }

  // OnchainIssuance(23) / propose_burn(2)
  // SCALE:actor_cid_number + asset_id:u32 + from:AccountId32 + amount:u128。
  static DecodedPayload? _decodeProposeAssetBurn(Uint8List bytes) {
    final header = _readOnchainAssetHeader(bytes);
    if (header == null) return null;
    var offset = header.next;
    if (offset + 32 + 16 > bytes.length) return null;
    final from = _bytesToSs58(
      Uint8List.fromList(bytes.sublist(offset, offset + 32)),
    );
    offset += 32;
    final amount = _readU128Le(bytes, offset);
    offset += 16;
    if (!_hasValidSigningTail(bytes, offset)) return null;
    return DecodedPayload(
      action: 'propose_asset_burn',
      summary:
          '资产 #${header.assetId} 从 ${_truncateAddress(from)} 销毁 ${amount.toString()} raw',
      fields: {
        'actor_cid_number': header.actorCidNumber,
        'asset_id': header.assetId.toString(),
        'from': from,
        'amount_raw': amount.toString(),
      },
    );
  }

  // OnchainIssuance(23) / propose_close(3)
  // SCALE:actor_cid_number + asset_id:u32。
  static DecodedPayload? _decodeProposeAssetClose(Uint8List bytes) {
    final header = _readOnchainAssetHeader(bytes);
    if (header == null || !_hasValidSigningTail(bytes, header.next)) {
      return null;
    }
    return DecodedPayload(
      action: 'propose_asset_close',
      summary: '关闭资产 #${header.assetId}',
      fields: {
        'actor_cid_number': header.actorCidNumber,
        'asset_id': header.assetId.toString(),
      },
    );
  }

  // OnchainIssuance(23) / propose_transfer(4)
  // SCALE:actor_cid_number + asset_id:u32 + from + to + amount:u128。
  static DecodedPayload? _decodeProposeAssetTransfer(Uint8List bytes) {
    final header = _readOnchainAssetHeader(bytes);
    if (header == null) return null;
    var offset = header.next;
    if (offset + 32 + 32 + 16 > bytes.length) return null;
    final from = _bytesToSs58(
      Uint8List.fromList(bytes.sublist(offset, offset + 32)),
    );
    offset += 32;
    final to = _bytesToSs58(
      Uint8List.fromList(bytes.sublist(offset, offset + 32)),
    );
    offset += 32;
    final amount = _readU128Le(bytes, offset);
    offset += 16;
    if (!_hasValidSigningTail(bytes, offset)) return null;
    return DecodedPayload(
      action: 'propose_asset_transfer',
      summary:
          '资产 #${header.assetId} 划转 ${amount.toString()} raw：${_truncateAddress(from)} → ${_truncateAddress(to)}',
      fields: {
        'actor_cid_number': header.actorCidNumber,
        'asset_id': header.assetId.toString(),
        'from': from,
        'to': to,
        'amount_raw': amount.toString(),
      },
    );
  }

  // OnchainIssuance(23) / propose_monitor_freeze(10) / unfreeze(11)
  // SCALE:actor_cid_number + asset_id:u32 + who:AccountId32 + reason_hash:[u8;32]。
  static DecodedPayload? _decodeProposeMonitorFreeze(
    Uint8List bytes, {
    required bool unfreeze,
  }) {
    final header = _readOnchainAssetHeader(bytes);
    if (header == null) return null;
    var offset = header.next;
    if (offset + 32 + 32 > bytes.length) return null;
    final who = _bytesToSs58(
      Uint8List.fromList(bytes.sublist(offset, offset + 32)),
    );
    offset += 32;
    final reasonHash = _bytesToLowerHex(
        Uint8List.fromList(bytes.sublist(offset, offset + 32)));
    offset += 32;
    if (!_hasValidSigningTail(bytes, offset)) return null;
    return DecodedPayload(
      action: unfreeze ? 'propose_monitor_unfreeze' : 'propose_monitor_freeze',
      summary:
          '${unfreeze ? '解冻' : '冻结'}资产 #${header.assetId} 持仓 ${_truncateAddress(who)}',
      fields: {
        'actor_cid_number': header.actorCidNumber,
        'asset_id': header.assetId.toString(),
        'who': who,
        'reason_hash': reasonHash,
      },
    );
  }

  // OnchainIssuance(23) / propose_monitor_confiscate(12)
  // SCALE:actor_cid_number + asset_id:u32 + who + amount:u128 + reason_hash。
  static DecodedPayload? _decodeProposeMonitorConfiscate(Uint8List bytes) {
    final header = _readOnchainAssetHeader(bytes);
    if (header == null) return null;
    var offset = header.next;
    if (offset + 32 + 16 + 32 > bytes.length) return null;
    final who = _bytesToSs58(
      Uint8List.fromList(bytes.sublist(offset, offset + 32)),
    );
    offset += 32;
    final amount = _readU128Le(bytes, offset);
    offset += 16;
    final reasonHash = _bytesToLowerHex(
        Uint8List.fromList(bytes.sublist(offset, offset + 32)));
    offset += 32;
    if (!_hasValidSigningTail(bytes, offset)) return null;
    return DecodedPayload(
      action: 'propose_monitor_confiscate',
      summary:
          '监管扣押资产 #${header.assetId} ${amount.toString()} raw（${_truncateAddress(who)}）',
      fields: {
        'actor_cid_number': header.actorCidNumber,
        'asset_id': header.assetId.toString(),
        'who': who,
        'amount_raw': amount.toString(),
        'reason_hash': reasonHash,
      },
    );
  }

  // OnchainIssuance(23) / propose_monitor_force_transfer(13)
  // SCALE:actor_cid_number + asset_id:u32 + from + to + amount:u128 + reason_hash。
  static DecodedPayload? _decodeProposeMonitorForceTransfer(Uint8List bytes) {
    final header = _readOnchainAssetHeader(bytes);
    if (header == null) return null;
    var offset = header.next;
    if (offset + 32 + 32 + 16 + 32 > bytes.length) return null;
    final from = _bytesToSs58(
      Uint8List.fromList(bytes.sublist(offset, offset + 32)),
    );
    offset += 32;
    final to = _bytesToSs58(
      Uint8List.fromList(bytes.sublist(offset, offset + 32)),
    );
    offset += 32;
    final amount = _readU128Le(bytes, offset);
    offset += 16;
    final reasonHash = _bytesToLowerHex(
        Uint8List.fromList(bytes.sublist(offset, offset + 32)));
    offset += 32;
    if (!_hasValidSigningTail(bytes, offset)) return null;
    return DecodedPayload(
      action: 'propose_monitor_force_transfer',
      summary:
          '监管划转资产 #${header.assetId} ${amount.toString()} raw：${_truncateAddress(from)} → ${_truncateAddress(to)}',
      fields: {
        'actor_cid_number': header.actorCidNumber,
        'asset_id': header.assetId.toString(),
        'from': from,
        'to': to,
        'amount_raw': amount.toString(),
        'reason_hash': reasonHash,
      },
    );
  }

  // OnchainIssuance(23) / propose_monitor_force_close(14)
  // SCALE:actor_cid_number + asset_id:u32 + reason_hash:[u8;32]。
  static DecodedPayload? _decodeProposeMonitorForceClose(Uint8List bytes) {
    final header = _readOnchainAssetHeader(bytes);
    if (header == null) return null;
    var offset = header.next;
    if (offset + 32 > bytes.length) return null;
    final reasonHash = _bytesToLowerHex(
        Uint8List.fromList(bytes.sublist(offset, offset + 32)));
    offset += 32;
    if (!_hasValidSigningTail(bytes, offset)) return null;
    return DecodedPayload(
      action: 'propose_monitor_force_close',
      summary: '监管封禁资产 #${header.assetId}',
      fields: {
        'actor_cid_number': header.actorCidNumber,
        'asset_id': header.assetId.toString(),
        'reason_hash': reasonHash,
      },
    );
  }

  static ({String actorCidNumber, int assetId, int next})?
      _readOnchainAssetHeader(Uint8List bytes) {
    final actorRead = _readCidNumber(bytes, 2);
    if (actorRead == null || actorRead.$2 + 4 > bytes.length) return null;
    final assetId = _readU32Le(bytes, actorRead.$2);
    return (
      actorCidNumber: actorRead.$1,
      assetId: assetId,
      next: actorRead.$2 + 4,
    );
  }

  static String _formatRawAssetAmount(BigInt amount, int decimals) {
    if (decimals == 0) return amount.toString();
    final padded = amount.toString().padLeft(decimals + 1, '0');
    final split = padded.length - decimals;
    return '${padded.substring(0, split)}.${padded.substring(split)}';
  }

  // PersonalManage(7) / propose_create(0)
  // 格式：[7][0][BoundedVec account_name][BoundedVec<Admin> admins][u32 regular_threshold][u128 amount]
  // 个人多签独立 pallet,MODULE_TAG = b"per-mgmt"。
  // 机构生命周期模块 call=3 留洞不复用。
  static DecodedPayload? _decodeProposeCreatePersonal(Uint8List bytes) {
    if (bytes.length < 2 + 1 + 1 + 32 * 2 + 16) return null;
    var offset = 2;

    // account_name: BoundedVec<u8>
    final (accountNameLen, accountNameLenSize) =
        _decodeCompactU32(bytes, offset);
    offset += accountNameLenSize;
    if (offset + accountNameLen > bytes.length) return null;
    final accountName = utf8.decode(
        bytes.sublist(offset, offset + accountNameLen),
        allowMalformed: true);
    offset += accountNameLen;

    final adminsRead = _readAdminPersons(
      bytes,
      offset,
      minCount: 2,
      maxCount: 64,
    );
    if (adminsRead == null) return null;
    final admins = adminsRead.$1;
    final adminsLen = admins.length;
    offset = adminsRead.$2;

    if (offset + 4 + 16 > bytes.length) return null;
    if (!_hasValidSigningTail(bytes, offset + 4 + 16)) return null;
    final regularThreshold = bytes[offset] |
        (bytes[offset + 1] << 8) |
        (bytes[offset + 2] << 16) |
        (bytes[offset + 3] << 24);
    offset += 4;
    final minThreshold = _minimumRegularThreshold(adminsLen);
    if (regularThreshold < minThreshold || regularThreshold > adminsLen) {
      return null;
    }

    // amount: u128
    final amountFen = _readU128Le(bytes, offset);
    final amountYuan = _fenToYuan(amountFen);

    return DecodedPayload(
      action: 'propose_create_personal',
      summary:
          '创建个人多签「$accountName」（$adminsLen 管理员，普通阈值 $regularThreshold，注册全员同意，入金 $amountYuan 元）',
      fields: {
        'account_name': accountName,
        'admins': _adminMachineValue(admins),
        'admins_len': adminsLen.toString(),
        'regular_threshold': '$regularThreshold/$adminsLen',
        'create_threshold': '$adminsLen/$adminsLen',
        'amount_yuan': '$amountYuan GMB',
      },
      reviewFields: {
        'account_name': accountName,
        'admins': _adminReviewValue(admins),
        'admins_len': adminsLen.toString(),
        'regular_threshold': '$regularThreshold/$adminsLen',
        'create_threshold': '$adminsLen/$adminsLen',
        'amount_yuan': '$amountYuan GMB',
      },
    );
  }

  // PublicManage(30) / PrivateManage(31) / propose_close_*_institution(1)
  /// 机构自定义账户关闭提案。
  /// SCALE:actor_cid_number + institution_account + beneficiary + register_nonce
  ///   + signature + credential_issuer_cid_number + credential_signer_pubkey。
  /// 外层管理员授权只认 actor CID；注册局凭证不构成第二套交易授权。
  static DecodedPayload? _decodeProposeCloseInstitution(
    Uint8List bytes, {
    required String action,
    required String entityLabel,
  }) {
    var offset = 2;
    final actorRead = _readCidNumber(bytes, offset);
    if (actorRead == null) return null;
    final actorCidNumber = actorRead.$1;
    offset = actorRead.$2;
    if (bytes.length < offset + 64) return null;
    final accountId = bytes.sublist(offset, offset + 32);
    offset += 32;
    final beneficiaryId = bytes.sublist(offset, offset + 32);
    offset += 32;
    offset = _skipBoundedBytes(bytes, offset); // register_nonce
    if (offset < 0) return null;
    offset = _skipBoundedBytes(bytes, offset); // signature
    if (offset < 0) return null;
    final credentialIssuerRead = _readCidNumber(bytes, offset);
    if (credentialIssuerRead == null) return null;
    offset = credentialIssuerRead.$2;
    if (offset + 32 > bytes.length) return null;
    final credentialSigner = bytes.sublist(offset, offset + 32);
    offset += 32;
    if (!_hasValidSigningTail(bytes, offset)) return null;
    final account = Keyring().encodeAddress(accountId.toList(), _ss58Prefix);
    final beneficiary =
        Keyring().encodeAddress(beneficiaryId.toList(), _ss58Prefix);
    return DecodedPayload(
      action: action,
      summary:
          '提案关闭$entityLabel账户 ${_truncateAddress(account)}(余额转 ${_truncateAddress(beneficiary)})',
      fields: {
        'actor_cid_number': actorCidNumber,
        'institution_account': account,
        'beneficiary': beneficiary,
        'credential_issuer_cid_number': credentialIssuerRead.$1,
        'credential_signer_pubkey': _bytesToSs58(credentialSigner),
      },
    );
  }

  static DecodedPayload? _decodeProposeClose(
    Uint8List bytes, {
    required String action,
    required String summaryLabel,
  }) {
    // call_data: 2 + 32 + 32 = 66
    if (bytes.length < 66 || !_hasValidSigningTail(bytes, 66)) return null;
    final multisigId = bytes.sublist(2, 34);
    final beneficiaryId = bytes.sublist(34, 66);
    final multisig = Keyring().encodeAddress(multisigId.toList(), _ss58Prefix);
    final beneficiary =
        Keyring().encodeAddress(beneficiaryId.toList(), _ss58Prefix);
    return DecodedPayload(
      action: action,
      summary: '提案关闭$summaryLabel ${_truncateAddress(multisig)}',
      fields: {
        'account': multisig,
        'beneficiary': beneficiary,
      },
    );
  }

  // MultisigTransfer(17) / propose_safety_fund(1)
  // 格式：[17][1][actor_cid_number:CidNumber][institution_account:32]
  //      [beneficiary:32][amount:u128][BoundedVec remark]
  static DecodedPayload? _decodeProposeSafetyFund(Uint8List bytes) {
    var offset = 2;
    final actorRead = _readCidNumber(bytes, offset);
    if (actorRead == null) return null;
    offset = actorRead.$2;
    if (offset + 32 + 32 + 16 > bytes.length) return null;
    final institutionAccount = bytes.sublist(offset, offset + 32);
    offset += 32;
    final beneficiaryId = bytes.sublist(offset, offset + 32);
    offset += 32;
    final beneficiary =
        Keyring().encodeAddress(beneficiaryId.toList(), _ss58Prefix);
    final amountFen = _readU128Le(bytes, offset);
    offset += 16;
    final amountYuan = _fenToYuan(amountFen);
    final (remarkLen, remarkLenSize) = _decodeCompactU32(bytes, offset);
    if (remarkLenSize == 0) return null;
    offset += remarkLenSize;
    if (offset + remarkLen > bytes.length) return null;
    if (!_hasValidSigningTail(bytes, offset + remarkLen)) return null;
    var remark = '';
    if (remarkLen > 0) {
      remark = utf8.decode(bytes.sublist(offset, offset + remarkLen),
          allowMalformed: true);
    }
    return DecodedPayload(
      action: 'propose_safety_fund_transfer',
      summary: '安全基金转账 $amountYuan GMB 给 ${_truncateAddress(beneficiary)}',
      fields: {
        'actor_cid_number': actorRead.$1,
        'institution_account': _bytesToSs58(institutionAccount),
        'operation_fee_payer': _reviewValue(
          'operation_fee_payer',
          {'actor_cid_number': actorRead.$1},
        ),
        'execution_fee_payer': _reviewValue(
          'execution_fee_payer',
          {'actor_cid_number': actorRead.$1},
        ),
        'beneficiary': beneficiary,
        'amount_yuan': '$amountYuan GMB',
        'remark': remark,
      },
    );
  }

  // MultisigTransfer(17) / propose_sweep(2)
  // 格式：[17][2][actor_cid_number:CidNumber][institution_account:AccountId32][amount:u128]
  static DecodedPayload? _decodeProposeSweep(Uint8List bytes) {
    var offset = 2;
    final actorRead = _readCidNumber(bytes, offset);
    if (actorRead == null) return null;
    offset = actorRead.$2;
    if (offset + 32 + 16 > bytes.length) return null;
    final institutionBytes = bytes.sublist(offset, offset + 32);
    offset += 32;
    final institutionAccount = _bytesToSs58(institutionBytes);
    final amountFen = _readU128Le(bytes, offset);
    offset += 16;
    if (!_hasValidSigningTail(bytes, offset)) return null;
    final amountYuan = _fenToYuan(amountFen);
    return DecodedPayload(
      action: 'propose_sweep_to_main',
      summary: '机构 ${actorRead.$1} 费用账户划转 $amountYuan GMB',
      fields: {
        'actor_cid_number': actorRead.$1,
        'institution_account': institutionAccount,
        'operation_fee_payer': _reviewValue(
          'operation_fee_payer',
          {'actor_cid_number': actorRead.$1},
        ),
        'execution_fee_payer': _reviewValue(
          'execution_fee_payer',
          {'actor_cid_number': actorRead.$1},
        ),
        'amount_yuan': '$amountYuan GMB',
      },
    );
  }

  // ResolutionDestroy(13) / propose_destroy(0)
  // 格式：[13][0][actor_cid_number:CidNumber][institution_account:AccountId32][amount:u128]
  static DecodedPayload? _decodeProposeDestroy(Uint8List bytes) {
    var offset = 2;
    final actorRead = _readCidNumber(bytes, offset);
    if (actorRead == null) return null;
    offset = actorRead.$2;
    if (offset + 32 + 16 > bytes.length) return null;
    final institutionBytes = bytes.sublist(offset, offset + 32);
    offset += 32;
    final institutionAccount = _bytesToSs58(institutionBytes);
    final amountFen = _readU128Le(bytes, offset);
    offset += 16;
    if (!_hasValidSigningTail(bytes, offset)) return null;
    final amountYuan = _fenToYuan(amountFen);
    return DecodedPayload(
      action: 'propose_destroy',
      summary: '机构 ${actorRead.$1} 决议销毁 $amountYuan GMB',
      fields: {
        'actor_cid_number': actorRead.$1,
        'institution_account': institutionAccount,
        'operation_fee_payer': _reviewValue(
          'operation_fee_payer',
          {'actor_cid_number': actorRead.$1},
        ),
        'execution_fee_payer': _reviewValue(
          'execution_fee_payer',
          {'actor_cid_number': actorRead.$1},
        ),
        'amount_yuan': '$amountYuan GMB',
      },
    );
  }

  // PersonalAdmins(29.0)。PublicAdmins/PrivateAdmins 只供机构生命周期内部调用，
  // 没有对外管理员变更 extrinsic，禁止把个人账户布局复用于机构 CID。
  // 格式：[pallet][call][institution_code:[u8;4]][account:AccountId32]
  //       [Compact<N>][admins:N*(admin_account+family_name+given_name)][new_threshold:u32_le]
  static DecodedPayload? _decodeProposeAdminSetChange(Uint8List bytes) {
    if (bytes.length < 75) return null;
    final palletIndex = bytes[0];
    final callIndex = bytes[1];
    var offset = 2;
    final code =
        InstitutionCode.codeToString(bytes.sublist(offset, offset + 4));
    if (!_validAdminChangePalletForCode(palletIndex, callIndex, code)) {
      return null;
    }
    offset += 4;
    final accountBytes = bytes.sublist(offset, offset + 32);
    final accountHex = _bytesToLowerHex(accountBytes);
    offset += 32;

    final adminsRead = _readAdminPersons(
      bytes,
      offset,
      minCount: 2,
      maxCount: 64,
    );
    if (adminsRead == null) return null;
    final admins = adminsRead.$1;
    final adminsLen = admins.length;
    offset = adminsRead.$2;
    if (offset + 4 > bytes.length) return null;
    if (!_hasValidSigningTail(bytes, offset + 4)) return null;
    final newThreshold = _readU32Le(bytes, offset);
    if (!_validAdminChangeThreshold(code, adminsLen, newThreshold)) {
      return null;
    }
    final thresholdLabel = '$newThreshold/$adminsLen';
    final institutionLabel = InstitutionCode.codeLabel(code);
    final action = _adminSetChangeActionForPallet(palletIndex);

    return DecodedPayload(
      action: action,
      summary:
          '$institutionLabel 管理员集合变更：${_bytesToSs58(accountBytes)} → $adminsLen 人，阈值 $thresholdLabel',
      fields: {
        'institution_code': institutionLabel,
        'account': accountHex,
        'admins': _adminMachineValue(admins),
      },
      reviewFields: {
        'institution_code': institutionLabel,
        'account': _bytesToSs58(accountBytes),
        'admins': _adminReviewValue(admins),
        'new_threshold': thresholdLabel,
      },
    );
  }

  // GrandpaKeyChange(15) / propose_key_change(0)
  // 格式：[15][0][actor_cid_number:CidNumber][new_key:32]
  static DecodedPayload? _decodeProposeKeyChange(Uint8List bytes) {
    var offset = 2;
    final actorRead = _readCidNumber(bytes, offset);
    if (actorRead == null) return null;
    offset = actorRead.$2;
    if (offset + 32 > bytes.length) return null;
    final keyBytes = bytes.sublist(offset, offset + 32);
    offset += 32;
    if (!_hasValidSigningTail(bytes, offset)) return null;
    final keyHex =
        keyBytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
    return DecodedPayload(
      action: 'propose_replace_grandpa_key',
      summary: '机构 ${actorRead.$1} GRANDPA 密钥替换提案',
      fields: {
        'actor_cid_number': actorRead.$1,
        'new_key': '0x$keyHex',
      },
    );
  }

  // LegislationYuan(25) 立法全文章节(章>节>条>款)SCALE 跳读 + 摘要统计。
  //
  // 布局逐字段对齐 legislation-yuan(ADR-027):
  //   ChaptersOf = Compact(count) + 每个 Chapter
  //   Chapter  = number:u32_le + title:BoundedVec<u8> + title_en:Option<BoundedVec<u8>>
  //            + sections:Compact(count)+Section[]
  //   Section  = number:u32_le + title + title_en + articles:Compact(count)+Article[]
  //   Article  = number:u32_le + title + title_en + body:BoundedVec<u8>
  //            + body_en:Option<BoundedVec<u8>> + clauses:Compact(count)+Clause[]
  //   Clause   = number:u32_le + text:BoundedVec<u8> + text_en:Option<BoundedVec<u8>>
  //
  // 与 citizenapp lib/legislation/data/legislation_codec.dart 同源字段序。
  // propose 解码只需「章数 / 条数」摘要,不逐条展开正文(QR 已是节点端构造的全文)。
  /// 跳过一个 BoundedVec<u8>(Compact 前缀 + 字节),返回新 offset;失败返回 -1。
  static int _skipBoundedBytes(Uint8List bytes, int offset) {
    if (offset >= bytes.length) return -1;
    final (len, lenSize) = _decodeCompactU32(bytes, offset);
    if (lenSize == 0) return -1;
    offset += lenSize + len;
    if (offset > bytes.length) return -1;
    return offset;
  }

  /// 读取 UTF-8 `BoundedVec<u8>`，成功返回文本与新偏移。
  static (String, int)? _readBoundedUtf8(Uint8List bytes, int offset) {
    if (offset >= bytes.length) return null;
    final (len, lenSize) = _decodeCompactU32(bytes, offset);
    if (lenSize == 0) return null;
    final start = offset + lenSize;
    final end = start + len;
    if (end > bytes.length) return null;
    return (utf8.decode(bytes.sublist(start, end), allowMalformed: true), end);
  }

  /// 跳过 Option<BoundedVec<u8>>(1 tag 字节 + Some 时载荷),返回新 offset;失败返回 -1。
  static int _skipOptionBoundedBytes(Uint8List bytes, int offset) {
    if (offset >= bytes.length) return -1;
    final tag = bytes[offset];
    offset += 1;
    if (tag == 0) return offset; // None
    if (tag != 1) return -1; // 非法 Option tag → 红色拒签
    return _skipBoundedBytes(bytes, offset);
  }

  /// 立法全文章节摘要(章数/条数)。返回 (新 offset, 章数, 条数);失败 offset = -1。
  static (int, int, int) _scanChapters(Uint8List bytes, int offset) {
    if (offset >= bytes.length) return (-1, 0, 0);
    final (chapterCount, chapterCountSize) = _decodeCompactU32(bytes, offset);
    if (chapterCountSize == 0) return (-1, 0, 0);
    offset += chapterCountSize;
    var articleTotal = 0;

    for (var c = 0; c < chapterCount; c++) {
      // Chapter.number: u32
      if (offset + 4 > bytes.length) return (-1, 0, 0);
      offset += 4;
      // Chapter.title + title_en
      offset = _skipBoundedBytes(bytes, offset);
      if (offset < 0) return (-1, 0, 0);
      offset = _skipOptionBoundedBytes(bytes, offset);
      if (offset < 0) return (-1, 0, 0);
      // sections
      final (sectionCount, sectionCountSize) = _decodeCompactU32(bytes, offset);
      if (sectionCountSize == 0) return (-1, 0, 0);
      offset += sectionCountSize;

      for (var s = 0; s < sectionCount; s++) {
        if (offset + 4 > bytes.length) return (-1, 0, 0);
        offset += 4; // Section.number
        offset = _skipBoundedBytes(bytes, offset);
        if (offset < 0) return (-1, 0, 0);
        offset = _skipOptionBoundedBytes(bytes, offset);
        if (offset < 0) return (-1, 0, 0);
        final (articleCount, articleCountSize) =
            _decodeCompactU32(bytes, offset);
        if (articleCountSize == 0) return (-1, 0, 0);
        offset += articleCountSize;

        for (var a = 0; a < articleCount; a++) {
          if (offset + 4 > bytes.length) return (-1, 0, 0);
          offset += 4; // Article.number
          offset = _skipBoundedBytes(bytes, offset); // title
          if (offset < 0) return (-1, 0, 0);
          offset = _skipOptionBoundedBytes(bytes, offset); // title_en
          if (offset < 0) return (-1, 0, 0);
          offset = _skipBoundedBytes(bytes, offset); // body
          if (offset < 0) return (-1, 0, 0);
          offset = _skipOptionBoundedBytes(bytes, offset); // body_en
          if (offset < 0) return (-1, 0, 0);
          final (clauseCount, clauseCountSize) =
              _decodeCompactU32(bytes, offset);
          if (clauseCountSize == 0) return (-1, 0, 0);
          offset += clauseCountSize;

          for (var k = 0; k < clauseCount; k++) {
            if (offset + 4 > bytes.length) return (-1, 0, 0);
            offset += 4; // Clause.number
            offset = _skipBoundedBytes(bytes, offset); // text
            if (offset < 0) return (-1, 0, 0);
            offset = _skipOptionBoundedBytes(bytes, offset); // text_en
            if (offset < 0) return (-1, 0, 0);
          }
          articleTotal += 1;
        }
      }
    }
    return (offset, chapterCount, articleTotal);
  }

  /// 院序列 Houses = Compact(count) + count×CidNumber。
  /// 立法院组成只保存机构 CID，不允许机构账户充当机构身份。
  static (int, List<String>) _scanHouses(Uint8List bytes, int offset) {
    if (offset >= bytes.length) return (-1, const []);
    final (count, countSize) = _decodeCompactU32(bytes, offset);
    if (countSize == 0) return (-1, const []);
    offset += countSize;
    if (count == 0) return (-1, const []);
    final cidNumbers = <String>[];
    for (var i = 0; i < count; i++) {
      final cidRead = _readCidNumber(bytes, offset);
      if (cidRead == null || cidNumbers.contains(cidRead.$1)) {
        return (-1, const []);
      }
      cidNumbers.add(cidRead.$1);
      offset = cidRead.$2;
    }
    return (offset, cidNumbers);
  }

  /// 表决类型 5 类(对齐 legislation-yuan VoteType 枚举索引)。
  static String? _voteTypeLabel(int index) {
    switch (index) {
      case 0:
        return '常规案';
      case 1:
        return '常规教育案';
      case 2:
        return '重要案';
      case 3:
        return '重要教育案';
      case 4:
        return '特别案（强制公投）';
      default:
        return null; // 越界枚举 → 红色拒签
    }
  }

  /// 法律层级 4 类(对齐 legislation-yuan Tier 枚举索引)。
  static String? _tierLabel(int index) {
    switch (index) {
      case 0:
        return '宪法';
      case 1:
        return '国家级';
      case 2:
        return '省级';
      case 3:
        return '市级';
      default:
        return null;
    }
  }

  // LegislationYuan(25) / propose_enact_law(0)
  // SCALE: [25][0][tier:u8][scope_code:u32_le][houses:Vec<CidNumber>]
  //        [actor_cid_number][executive_cid_number][legislature_cid_number:Option<CidNumber>]
  //        [vote_type:u8][title][title_en][chapters][effective_at:u64_le]
  static DecodedPayload? _decodeProposeEnactLaw(Uint8List bytes) {
    if (bytes.length < 3) return null;
    var offset = 2;

    // tier: u8 枚举(立法入口禁止新立宪法,tier=0 视为非法 payload)。
    if (offset >= bytes.length) return null;
    final tierIndex = bytes[offset++];
    final tierLabel = _tierLabel(tierIndex);
    if (tierLabel == null || tierIndex == 0) return null;

    // scope_code: u32 LE
    if (offset + 4 > bytes.length) return null;
    final scopeCode = _readU32Le(bytes, offset);
    offset += 4;

    // houses
    final (afterHouses, houseCidNumbers) = _scanHouses(bytes, offset);
    if (afterHouses < 0) return null;
    offset = afterHouses;

    final actorRead = _readCidNumber(bytes, offset);
    if (actorRead == null) return null;
    offset = actorRead.$2;
    final executiveRead = _readCidNumber(bytes, offset);
    if (executiveRead == null) return null;
    offset = executiveRead.$2;
    final legislatureRead = _readOptionalCidNumber(bytes, offset);
    if (legislatureRead == null) return null;
    offset = legislatureRead.$2;

    // vote_type: u8 枚举
    if (offset >= bytes.length) return null;
    final voteTypeIndex = bytes[offset++];
    final voteTypeLabel = _voteTypeLabel(voteTypeIndex);
    if (voteTypeLabel == null) return null;

    // title: BoundedVec<u8>
    final (titleLen, titleLenSize) = _decodeCompactU32(bytes, offset);
    if (titleLenSize == 0) return null;
    offset += titleLenSize;
    if (offset + titleLen > bytes.length) return null;
    final title = utf8.decode(bytes.sublist(offset, offset + titleLen),
        allowMalformed: true);
    offset += titleLen;

    // title_en: Option<BoundedVec<u8>>
    offset = _skipOptionBoundedBytes(bytes, offset);
    if (offset < 0) return null;

    // chapters 摘要
    final (afterChapters, chapterCount, articleTotal) =
        _scanChapters(bytes, offset);
    if (afterChapters < 0) return null;
    offset = afterChapters;

    // effective_at: u64 LE unix 毫秒。
    if (offset + 8 > bytes.length) return null;
    final effectiveAt = _readU64Le(bytes, offset);
    offset += 8;

    if (!_hasValidSigningTail(bytes, offset)) return null;

    return DecodedPayload(
      action: 'propose_enact_law',
      summary:
          '发起立法「$title」（$tierLabel·$voteTypeLabel，$chapterCount 章 $articleTotal 条，时间戳 $effectiveAt 生效）',
      fields: {
        'title': title,
        'tier': tierLabel,
        'vote_type': voteTypeLabel,
        'scope_code': scopeCode.toString(),
        'houses': houseCidNumbers.join('、'),
        'actor_cid_number': actorRead.$1,
        'executive_cid_number': executiveRead.$1,
        if (legislatureRead.$1 != null)
          'legislature_cid_number': legislatureRead.$1!,
        'chapter_count': chapterCount.toString(),
        'article_count': articleTotal.toString(),
        'effective_at': effectiveAt.toString(),
      },
    );
  }

  // LegislationYuan(25) / propose_amend_law(1)
  // SCALE: [25][1][law_id:u64_le][actor_cid_number][executive_cid_number]
  //        [legislature_cid_number:Option<CidNumber>][vote_type:u8][title]
  //        [title_en][chapters][effective_at:u64_le]
  static DecodedPayload? _decodeProposeAmendLaw(Uint8List bytes) {
    if (bytes.length < 10) return null;
    var offset = 2;

    final lawId = _readU64Le(bytes, offset);
    offset += 8;

    final actorRead = _readCidNumber(bytes, offset);
    if (actorRead == null) return null;
    offset = actorRead.$2;
    final executiveRead = _readCidNumber(bytes, offset);
    if (executiveRead == null) return null;
    offset = executiveRead.$2;
    final legislatureRead = _readOptionalCidNumber(bytes, offset);
    if (legislatureRead == null) return null;
    offset = legislatureRead.$2;

    if (offset >= bytes.length) return null;
    final voteTypeIndex = bytes[offset++];
    final voteTypeLabel = _voteTypeLabel(voteTypeIndex);
    if (voteTypeLabel == null) return null;

    final (titleLen, titleLenSize) = _decodeCompactU32(bytes, offset);
    if (titleLenSize == 0) return null;
    offset += titleLenSize;
    if (offset + titleLen > bytes.length) return null;
    final title = utf8.decode(bytes.sublist(offset, offset + titleLen),
        allowMalformed: true);
    offset += titleLen;

    offset = _skipOptionBoundedBytes(bytes, offset);
    if (offset < 0) return null;

    final (afterChapters, chapterCount, articleTotal) =
        _scanChapters(bytes, offset);
    if (afterChapters < 0) return null;
    offset = afterChapters;

    if (offset + 8 > bytes.length) return null;
    final effectiveAt = _readU64Le(bytes, offset);
    offset += 8;

    if (!_hasValidSigningTail(bytes, offset)) return null;

    return DecodedPayload(
      action: 'propose_amend_law',
      summary:
          '发起修法「$title」（法律 #$lawId·$voteTypeLabel，$chapterCount 章 $articleTotal 条，时间戳 $effectiveAt 生效）',
      fields: {
        'law_id': lawId.toString(),
        'actor_cid_number': actorRead.$1,
        'executive_cid_number': executiveRead.$1,
        if (legislatureRead.$1 != null)
          'legislature_cid_number': legislatureRead.$1!,
        'title': title,
        'vote_type': voteTypeLabel,
        'chapter_count': chapterCount.toString(),
        'article_count': articleTotal.toString(),
        'effective_at': effectiveAt.toString(),
      },
    );
  }

  // LegislationYuan(25) / propose_repeal_law(2)
  // SCALE: [25][2][law_id:u64_le][actor_cid_number][executive_cid_number]
  //        [legislature_cid_number:Option<CidNumber>][vote_type:u8]
  static DecodedPayload? _decodeProposeRepealLaw(Uint8List bytes) {
    if (bytes.length < 10) return null;
    var offset = 2;

    final lawId = _readU64Le(bytes, offset);
    offset += 8;

    final actorRead = _readCidNumber(bytes, offset);
    if (actorRead == null) return null;
    offset = actorRead.$2;
    final executiveRead = _readCidNumber(bytes, offset);
    if (executiveRead == null) return null;
    offset = executiveRead.$2;
    final legislatureRead = _readOptionalCidNumber(bytes, offset);
    if (legislatureRead == null) return null;
    offset = legislatureRead.$2;

    if (offset >= bytes.length) return null;
    final voteTypeIndex = bytes[offset++];
    final voteTypeLabel = _voteTypeLabel(voteTypeIndex);
    if (voteTypeLabel == null) return null;

    if (!_hasValidSigningTail(bytes, offset)) return null;

    return DecodedPayload(
      action: 'propose_repeal_law',
      summary: '发起废法 法律 #$lawId（$voteTypeLabel）',
      fields: {
        'law_id': lawId.toString(),
        'actor_cid_number': actorRead.$1,
        'executive_cid_number': executiveRead.$1,
        if (legislatureRead.$1 != null)
          'legislature_cid_number': legislatureRead.$1!,
        'vote_type': voteTypeLabel,
      },
    );
  }

  // LegislationVote(26) 通用:proposal_id:u64_le + approve:bool。
  // 代表机构表决(1)/行政签署(3)/三人会签(4)/护宪终审(5) 同形。
  // SCALE: [26][call][proposal_id:u64_le][approve:bool]
  static DecodedPayload? _decodeProposalApprove(
    Uint8List bytes, {
    required String action,
    required String summaryTemplate,
  }) {
    // call_data: 2 + 8 + 1 = 11
    if (bytes.length < 11 || !_hasValidSigningTail(bytes, 11)) return null;
    final proposalId = _readU64Le(bytes, 2);
    final approve = bytes[10] != 0;
    final voteText = approve ? '赞成' : '反对';
    return DecodedPayload(
      action: action,
      summary: summaryTemplate
          .replaceAll('{id}', proposalId.toString())
          .replaceAll('{vote}', voteText),
      fields: {
        'proposal_id': proposalId.toString(),
        'approve': approve.toString(),
      },
    );
  }

  // LegislationVote(26) / cast_referendum_vote(2)
  // SCALE: [26][2][proposal_id:u64_le][approve:bool]
  static DecodedPayload? _decodeCastLegislationReferendum(Uint8List bytes) {
    if (bytes.length < 11 || !_hasValidSigningTail(bytes, 11)) return null;
    final proposalId = _readU64Le(bytes, 2);
    final approve = bytes[10] != 0;
    final voteText = approve ? '赞成' : '反对';

    return DecodedPayload(
      action: 'cast_referendum_vote',
      summary: '特别案公投 立法提案 #$proposalId：$voteText',
      fields: {
        'proposal_id': proposalId.toString(),
        'approve': approve.toString(),
      },
    );
  }

  // CitizenIdentity 原始身份载荷。
  // SCALE: VotingIdentityPayload {
  //   cid_number, wallet_account, citizen_age_years, valid_from, valid_until,
  //   citizen_status, residence_province_code, residence_city_code,
  //   residence_town_code
  // }
  static DecodedPayload? _decodeCitizenIdentityPayload(Uint8List bytes) {
    final candidate = _readCandidateIdentityPayload(bytes, 0);
    if (candidate != null && candidate.next == bytes.length) {
      return DecodedPayload(
        action: 'citizen_candidate_identity',
        summary: '确认公民参选身份上链：${candidate.cidNumber}',
        fields: candidate.fields,
        reviewFields: candidate.reviewFields,
      );
    }
    final payload = _readVotingIdentityPayload(bytes, 0);
    if (payload == null || payload.next != bytes.length) return null;
    return DecodedPayload(
      action: 'citizen_identity',
      summary: '确认公民身份上链：${payload.cidNumber}',
      fields: payload.fields,
      reviewFields: payload.reviewFields,
    );
  }

  // CitizenIdentity(10) / register_voting_identity(0)
  // SCALE: [10][0][actor_cid_number:CidNumber][VotingIdentityPayload][citizen_signature:Vec]。
  static DecodedPayload? _decodeRegisterVotingIdentity(Uint8List bytes) {
    return _decodeVotingIdentityCall(
      bytes,
      action: 'register_voting_identity',
      summaryPrefix: '注册公民链上身份',
    );
  }

  static DecodedPayload? _decodeUpdateVotingIdentity(Uint8List bytes) {
    return _decodeVotingIdentityCall(
      bytes,
      action: 'update_voting_identity',
      summaryPrefix: '更新公民链上身份',
    );
  }

  static DecodedPayload? _decodeVotingIdentityCall(
    Uint8List bytes, {
    required String action,
    required String summaryPrefix,
  }) {
    var offset = 2;
    final actorRead = _readCidNumber(bytes, offset);
    if (actorRead == null) return null;
    offset = actorRead.$2;
    final payload = _readVotingIdentityPayload(bytes, offset);
    if (payload == null) return null;
    offset = payload.next;

    final (signatureLen, signatureLenSize) = _decodeCompactU32(bytes, offset);
    if (signatureLenSize == 0 || signatureLen != 64) return null;
    offset += signatureLenSize;
    if (offset + signatureLen > bytes.length) return null;
    offset += signatureLen;
    if (!_hasCallDataEnd(bytes, offset)) return null;

    return DecodedPayload(
      action: action,
      summary: '$summaryPrefix：${payload.cidNumber}',
      fields: <String, String>{
        'actor_cid_number': actorRead.$1,
        ...payload.fields,
        'citizen_signature_len': signatureLen.toString(),
      },
      reviewFields: <String, String>{
        'actor_cid_number': actorRead.$1,
        ...payload.reviewFields,
      },
    );
  }

  // CitizenIdentity(10) / upgrade_to_candidate_identity(1)
  // SCALE: [10][1][actor_cid_number:CidNumber][CandidateIdentityPayload][citizen_signature:Vec]。
  static DecodedPayload? _decodeUpgradeToCandidateIdentity(Uint8List bytes) {
    return _decodeCandidateIdentityCall(
      bytes,
      action: 'upgrade_to_candidate_identity',
      summaryPrefix: '注册公民参选身份',
    );
  }

  static DecodedPayload? _decodeUpdateCandidateIdentity(Uint8List bytes) {
    return _decodeCandidateIdentityCall(
      bytes,
      action: 'update_candidate_identity',
      summaryPrefix: '更新公民参选身份',
    );
  }

  static DecodedPayload? _decodeCandidateIdentityCall(
    Uint8List bytes, {
    required String action,
    required String summaryPrefix,
  }) {
    var offset = 2;
    final actorRead = _readCidNumber(bytes, offset);
    if (actorRead == null) return null;
    offset = actorRead.$2;
    final payload = _readCandidateIdentityPayload(bytes, offset);
    if (payload == null) return null;
    offset = payload.next;

    final (signatureLen, signatureLenSize) = _decodeCompactU32(bytes, offset);
    if (signatureLenSize == 0 || signatureLen != 64) return null;
    offset += signatureLenSize;
    if (offset + signatureLen > bytes.length) return null;
    offset += signatureLen;
    if (!_hasCallDataEnd(bytes, offset)) return null;

    return DecodedPayload(
      action: action,
      summary: '$summaryPrefix：${payload.cidNumber}',
      fields: <String, String>{
        'actor_cid_number': actorRead.$1,
        ...payload.fields,
        'citizen_signature_len': signatureLen.toString(),
      },
      reviewFields: <String, String>{
        'actor_cid_number': actorRead.$1,
        ...payload.reviewFields,
      },
    );
  }

  // CitizenIdentity(10) / revoke_identity(4)
  // SCALE:actor_cid_number + 被吊销的公民 cid_number。
  static DecodedPayload? _decodeRevokeIdentity(Uint8List bytes) {
    final actorRead = _readCidNumber(bytes, 2);
    if (actorRead == null) return null;
    final cidRead = _readCidNumber(bytes, actorRead.$2);
    if (cidRead == null || !_hasCallDataEnd(bytes, cidRead.$2)) return null;
    return DecodedPayload(
      action: 'revoke_identity',
      summary: '吊销公民链上身份：${cidRead.$1}',
      fields: {
        'actor_cid_number': actorRead.$1,
        'cid_number': cidRead.$1,
      },
    );
  }

  // CitizenIdentity(10) / occupy_cid(6) · 注册局建档占号(注册局签名)。
  // SCALE: [10][6][actor_cid_number:CidNumber][cid_number:CidNumber]
  //        [commitment:[u8;32]][residence_province_code:Vec<u8>]
  //        [residence_city_code:Vec<u8>]
  // 逐字节对齐 onchina occupy.rs::encode_occupy_cid_call。
  static DecodedPayload? _decodeOccupyCid(Uint8List bytes) {
    var offset = 2;
    final actorRead = _readCidNumber(bytes, offset);
    if (actorRead == null) return null;
    offset = actorRead.$2;
    final cidRead = _readCidNumber(bytes, offset);
    if (cidRead == null) return null;
    final cidNumber = cidRead.$1;
    offset = cidRead.$2;

    if (offset + 32 > bytes.length) return null;
    final commitment = bytes.sublist(offset, offset + 32);
    offset += 32;

    final (provinceCode, afterProvince) = _readUtf8Vec(bytes, offset);
    if (provinceCode == null ||
        provinceCode.isEmpty ||
        provinceCode.length > 16) {
      return null;
    }
    offset = afterProvince;
    final (cityCode, afterCity) = _readUtf8Vec(bytes, offset);
    if (cityCode == null || cityCode.isEmpty || cityCode.length > 16) {
      return null;
    }
    offset = afterCity;
    if (!_hasCallDataEnd(bytes, offset)) return null;

    return DecodedPayload(
      action: 'occupy_cid',
      summary: '注册局占号(登记 CID 号):$cidNumber',
      fields: <String, String>{
        'actor_cid_number': actorRead.$1,
        'cid_number': cidNumber,
        'commitment': _bytesToLowerHex(commitment),
        'residence_province_code': provinceCode,
        'residence_city_code': cityCode,
      },
      reviewFields: <String, String>{
        'actor_cid_number': actorRead.$1,
        'cid_number': cidNumber,
        'residence': '$provinceCode / $cityCode',
      },
    );
  }

  // CitizenIdentity(10) / occupy_cids_batch(7)
  // SCALE:actor_cid_number + Vec<{cid_number, commitment}> + province_code + city_code。
  static DecodedPayload? _decodeOccupyCidsBatch(Uint8List bytes) {
    var offset = 2;
    final actorRead = _readCidNumber(bytes, offset);
    if (actorRead == null) return null;
    offset = actorRead.$2;
    final (itemCount, countSize) = _decodeCompactU32(bytes, offset);
    if (countSize == 0 || itemCount == 0) return null;
    offset += countSize;
    final cidNumbers = <String>[];
    for (var index = 0; index < itemCount; index++) {
      final cidRead = _readCidNumber(bytes, offset);
      if (cidRead == null || cidNumbers.contains(cidRead.$1)) return null;
      cidNumbers.add(cidRead.$1);
      offset = cidRead.$2;
      if (offset + 32 > bytes.length) return null;
      offset += 32; // commitment
    }
    final (provinceCode, afterProvince) = _readUtf8Vec(bytes, offset);
    if (provinceCode == null ||
        provinceCode.isEmpty ||
        provinceCode.length > 16) {
      return null;
    }
    offset = afterProvince;
    final (cityCode, afterCity) = _readUtf8Vec(bytes, offset);
    if (cityCode == null || cityCode.isEmpty || cityCode.length > 16) {
      return null;
    }
    offset = afterCity;
    if (!_hasCallDataEnd(bytes, offset)) return null;
    return DecodedPayload(
      action: 'occupy_cids_batch',
      summary: '注册局批量占用 $itemCount 个 CID',
      fields: {
        'actor_cid_number': actorRead.$1,
        'cid_number': cidNumbers.join('、'),
        'cid_count': itemCount.toString(),
        'residence_province_code': provinceCode,
        'residence_city_code': cityCode,
      },
      reviewFields: {
        'actor_cid_number': actorRead.$1,
        'cid_number': cidNumbers.join('、'),
        'cid_count': itemCount.toString(),
        'residence': '$provinceCode / $cityCode',
      },
    );
  }

  // CitizenIdentity(10) / revoke_cid(8) · 注册局吊销 CID 号(注册局签名)。
  // SCALE: [10][8][actor_cid_number:CidNumber][cid_number:CidNumber]
  // 逐字节对齐 onchina occupy.rs::encode_revoke_cid_call。
  static DecodedPayload? _decodeRevokeCid(Uint8List bytes) {
    var offset = 2;
    final actorRead = _readCidNumber(bytes, offset);
    if (actorRead == null) return null;
    offset = actorRead.$2;
    final cidRead = _readCidNumber(bytes, offset);
    if (cidRead == null) return null;
    final cidNumber = cidRead.$1;
    offset = cidRead.$2;
    if (!_hasCallDataEnd(bytes, offset)) return null;

    return DecodedPayload(
      action: 'revoke_cid',
      summary: '注册局吊销 CID 号:$cidNumber',
      fields: <String, String>{
        'actor_cid_number': actorRead.$1,
        'cid_number': cidNumber,
      },
      reviewFields: <String, String>{
        'actor_cid_number': actorRead.$1,
        'cid_number': cidNumber,
      },
    );
  }

  static ({
    String cidNumber,
    String walletAddress,
    Map<String, String> fields,
    Map<String, String> reviewFields,
    int next,
  })? _readVotingIdentityPayload(Uint8List bytes, int offset) {
    final (cidNumber, afterCid) = _readUtf8Vec(bytes, offset);
    if (cidNumber == null || cidNumber.isEmpty || cidNumber.length > 32) {
      return null;
    }
    offset = afterCid;
    if (offset + 32 + 1 + 4 + 4 + 1 > bytes.length) return null;

    final walletBytes = bytes.sublist(offset, offset + 32);
    offset += 32;
    final walletAddress = _bytesToSs58(walletBytes);

    final age = bytes[offset];
    offset += 1;
    if (age < 16) return null;

    final validFrom = _readU32Le(bytes, offset);
    offset += 4;
    final validUntil = _readU32Le(bytes, offset);
    offset += 4;
    if (!_isValidDateInt(validFrom) || !_isValidDateInt(validUntil)) {
      return null;
    }
    if (validUntil < validFrom) return null;

    final status = bytes[offset];
    offset += 1;
    final statusLabel = switch (status) {
      0 => 'NORMAL',
      1 => 'REVOKED',
      _ => null,
    };
    if (statusLabel == null) return null;

    final (provinceCode, afterProvince) = _readUtf8Vec(bytes, offset);
    if (provinceCode == null ||
        provinceCode.isEmpty ||
        provinceCode.length > 16) {
      return null;
    }
    offset = afterProvince;
    final (cityCode, afterCity) = _readUtf8Vec(bytes, offset);
    if (cityCode == null || cityCode.isEmpty || cityCode.length > 16) {
      return null;
    }
    offset = afterCity;
    final (townCode, afterTown) = _readUtf8Vec(bytes, offset);
    if (townCode == null || townCode.isEmpty || townCode.length > 16) {
      return null;
    }
    offset = afterTown;

    final validRange =
        '${_formatDateInt(validFrom)} 至 ${_formatDateInt(validUntil)}';
    final residence = '$provinceCode / $cityCode / $townCode';
    return (
      cidNumber: cidNumber,
      walletAddress: walletAddress,
      next: offset,
      fields: <String, String>{
        'cid_number': cidNumber,
        'wallet_account': walletAddress,
        'citizen_age_years': age.toString(),
        'valid_from': validFrom.toString(),
        'valid_until': validUntil.toString(),
        'citizen_status': statusLabel,
        'residence_province_code': provinceCode,
        'residence_city_code': cityCode,
        'residence_town_code': townCode,
      },
      reviewFields: <String, String>{
        'cid_number': cidNumber,
        'wallet_account': walletAddress,
        'citizen_age_years': '$age周岁',
        'valid_range': validRange,
        'citizen_status': statusLabel == 'NORMAL' ? '正常' : '注销',
        'residence': residence,
      },
    );
  }

  static ({
    String cidNumber,
    String walletAddress,
    Map<String, String> fields,
    Map<String, String> reviewFields,
    int next,
  })? _readCandidateIdentityPayload(Uint8List bytes, int offset) {
    final voting = _readVotingIdentityPayload(bytes, offset);
    if (voting == null) return null;
    offset = voting.next;

    final (birthProvinceCode, afterBirthProvince) = _readUtf8Vec(bytes, offset);
    if (birthProvinceCode == null ||
        birthProvinceCode.isEmpty ||
        birthProvinceCode.length > 16) {
      return null;
    }
    offset = afterBirthProvince;
    final (birthCityCode, afterBirthCity) = _readUtf8Vec(bytes, offset);
    if (birthCityCode == null ||
        birthCityCode.isEmpty ||
        birthCityCode.length > 16) {
      return null;
    }
    offset = afterBirthCity;
    final (birthTownCode, afterBirthTown) = _readUtf8Vec(bytes, offset);
    if (birthTownCode == null ||
        birthTownCode.isEmpty ||
        birthTownCode.length > 16) {
      return null;
    }
    offset = afterBirthTown;
    final (citizenFullName, afterFullName) = _readUtf8Vec(bytes, offset);
    if (citizenFullName == null ||
        citizenFullName.isEmpty ||
        citizenFullName.length > 128) {
      return null;
    }
    offset = afterFullName;
    if (offset >= bytes.length) return null;
    final sex = bytes[offset];
    offset += 1;
    final sexLabel = switch (sex) {
      0 => '男',
      1 => '女',
      _ => null,
    };
    if (sexLabel == null) return null;

    // birth_date: u32 YYYYMMDD(LE),CandidateIdentityPayload 末字段。
    if (offset + 4 > bytes.length) return null;
    final birthDate = _readU32Le(bytes, offset);
    offset += 4;
    if (!_isValidDateInt(birthDate)) return null;

    final birthPlace = '$birthProvinceCode / $birthCityCode / $birthTownCode';
    return (
      cidNumber: voting.cidNumber,
      walletAddress: voting.walletAddress,
      next: offset,
      fields: <String, String>{
        ...voting.fields,
        'birth_province_code': birthProvinceCode,
        'birth_city_code': birthCityCode,
        'birth_town_code': birthTownCode,
        'citizen_full_name': citizenFullName,
        'citizen_sex': sex.toString(),
        'birth_date': birthDate.toString(),
      },
      reviewFields: <String, String>{
        'identity_level': '参选身份',
        ...voting.reviewFields,
        'birth_place': birthPlace,
        'citizen_full_name': citizenFullName,
        'citizen_sex': sexLabel,
        'birth_date': _formatDateInt(birthDate),
      },
    );
  }

  // 通用:只取 proposal_id: u64_le 的引擎重试/取消类 call。
  //
  // 链端若干引擎重试/取消调用签名完全一致:
  //     pub fn <name>(origin, proposal_id: u64) -> DispatchResult
  // SCALE 编码恒为 `[pallet_idx][call_idx][proposal_id:u64_le]` = 10 bytes。
  //
  // 所有这类 call 由扫码端从 payload 解出 proposal_id,再按 QR 动作码确认场景。
  static DecodedPayload? _decodeProposalIdOnly(
    Uint8List bytes, {
    required String action,
    required String summaryTemplate,
  }) {
    // call_data: 2 + 8 = 10
    if (bytes.length < 10 || !_hasValidSigningTail(bytes, 10)) return null;
    final proposalId = _readU64Le(bytes, 2);
    return DecodedPayload(
      action: action,
      summary: summaryTemplate.replaceAll('{id}', proposalId.toString()),
      fields: {
        'proposal_id': proposalId.toString(),
      },
    );
  }

  // OffchainTransaction(19) / bind_clearing_bank(30), switch_bank(33)
  // 格式：[19][call][AccountId32]
  static DecodedPayload? _decodeAccountIdCall(
    Uint8List bytes, {
    required String action,
    required String summaryPrefix,
    required String fieldKey,
  }) {
    // call_data: 2 + 32 = 34
    if (bytes.length < 34 || !_hasValidSigningTail(bytes, 34)) return null;
    final accountId = bytes.sublist(2, 34);
    final address = Keyring().encodeAddress(accountId.toList(), _ss58Prefix);
    return DecodedPayload(
      action: action,
      summary: '$summaryPrefix ${_truncateAddress(address)}',
      fields: {
        fieldKey: address,
      },
    );
  }

  // OffchainTransaction(19) / deposit(31), withdraw(32)
  // 格式：[19][call][Compact<u128> amount]
  static DecodedPayload? _decodeAmountOnlyCall(
    Uint8List bytes, {
    required String action,
    required String summaryPrefix,
  }) {
    if (bytes.length < 3) return null;
    final (amountFen, size) = _decodeCompactBigInt(bytes, 2);
    if (amountFen == null || size == 0) return null;
    if (!_hasValidSigningTail(bytes, 2 + size)) return null;
    final amountYuan = _fenToYuan(amountFen);
    return DecodedPayload(
      action: action,
      summary: '$summaryPrefix $amountYuan GMB',
      fields: {
        'amount_yuan': '$amountYuan GMB',
      },
    );
  }

  // OffchainTransaction(19) / register_clearing_bank(50)
  // 格式：[19][50][actor_cid_number][peer_id][rpc_domain][u16 rpc_port]
  static DecodedPayload? _decodeRegisterClearingBank(Uint8List bytes) {
    var offset = 2;
    final actorRead = _readCidNumber(bytes, offset);
    if (actorRead == null) return null;
    offset = actorRead.$2;
    final (peerId, peerNext) = _readUtf8Vec(bytes, offset);
    if (peerId == null) return null;
    offset = peerNext;
    final (rpcDomain, domainNext) = _readUtf8Vec(bytes, offset);
    if (rpcDomain == null) return null;
    offset = domainNext;
    if (offset + 2 > bytes.length) return null;
    final rpcPort = bytes[offset] | (bytes[offset + 1] << 8);
    if (!_hasValidSigningTail(bytes, offset + 2)) return null;

    return DecodedPayload(
      action: 'register_clearing_bank',
      summary: '声明清算行节点 ${actorRead.$1} @ $rpcDomain:$rpcPort',
      fields: {
        'actor_cid_number': actorRead.$1,
        'peer_id': peerId,
        'rpc_domain': rpcDomain,
        'rpc_port': rpcPort.toString(),
      },
    );
  }

  // OffchainTransaction(19) / update_clearing_bank_endpoint(51)
  // 格式：[19][51][actor_cid_number][new_domain][u16 new_port]
  static DecodedPayload? _decodeUpdateClearingBankEndpoint(Uint8List bytes) {
    var offset = 2;
    final actorRead = _readCidNumber(bytes, offset);
    if (actorRead == null) return null;
    offset = actorRead.$2;
    final (newDomain, domainNext) = _readUtf8Vec(bytes, offset);
    if (newDomain == null) return null;
    offset = domainNext;
    if (offset + 2 > bytes.length) return null;
    final newPort = bytes[offset] | (bytes[offset + 1] << 8);
    if (!_hasValidSigningTail(bytes, offset + 2)) return null;

    return DecodedPayload(
      action: 'update_clearing_bank_endpoint',
      summary: '更新清算行 ${actorRead.$1} 端点 → $newDomain:$newPort',
      fields: {
        'actor_cid_number': actorRead.$1,
        'new_domain': newDomain,
        'new_port': newPort.toString(),
      },
    );
  }

  // OffchainTransaction(19) / unregister_clearing_bank(52)
  // 格式：[19][52][actor_cid_number]
  static DecodedPayload? _decodeUnregisterClearingBank(Uint8List bytes) {
    final actorRead = _readCidNumber(bytes, 2);
    if (actorRead == null || !_hasValidSigningTail(bytes, actorRead.$2)) {
      return null;
    }
    return DecodedPayload(
      action: 'unregister_clearing_bank',
      summary: '注销清算行节点 ${actorRead.$1}',
      fields: {
        'actor_cid_number': actorRead.$1,
      },
    );
  }

  // OffchainTransaction(19) / propose_l2_fee_rate(40)
  // SCALE:actor_cid_number + institution_account + new_rate_bp:u32。
  static DecodedPayload? _decodeProposeL2FeeRate(Uint8List bytes) {
    final actorRead = _readCidNumber(bytes, 2);
    if (actorRead == null) return null;
    var offset = actorRead.$2;
    if (offset + 32 + 4 > bytes.length) return null;
    final institutionAccount = bytes.sublist(offset, offset + 32);
    offset += 32;
    final newRateBp = _readU32Le(bytes, offset);
    offset += 4;
    if (!_hasValidSigningTail(bytes, offset)) return null;
    return DecodedPayload(
      action: 'propose_l2_fee_rate',
      summary: '清算行 ${actorRead.$1} 提案调整链下费率为 $newRateBp BP',
      fields: {
        'actor_cid_number': actorRead.$1,
        'institution_account': _bytesToSs58(institutionAccount),
        'new_rate_bp': newRateBp.toString(),
      },
    );
  }

  // AddressRegistry(33) / call 0..=4。所有调用以 actor CID 开头，
  // 注册局管理员只通过外层 origin 授权，地址业务字段随后按 runtime 顺序解码。
  static DecodedPayload? _decodeAddressRegistryCall(
    Uint8List bytes,
    int callIndex,
  ) {
    var offset = 2;
    final actorRead = _readCidNumber(bytes, offset);
    if (actorRead == null) return null;
    offset = actorRead.$2;
    if (callIndex == PalletRegistry.setAddressCatalogVersionCall) {
      final (catalogVersion, afterVersion) = _readUtf8Vec(bytes, offset);
      if (catalogVersion == null || catalogVersion.isEmpty) return null;
      offset = afterVersion;
      if (offset + 32 > bytes.length) return null;
      final catalogHash = bytes.sublist(offset, offset + 32);
      offset += 32;
      if (!_hasValidSigningTail(bytes, offset)) return null;
      return DecodedPayload(
        action: 'set_address_catalog_version',
        summary: '设置地址库版本 $catalogVersion',
        fields: {
          'actor_cid_number': actorRead.$1,
          'catalog_version': catalogVersion,
          'catalog_hash': _bytesToLowerHex(catalogHash),
        },
      );
    }

    final fieldNames = switch (callIndex) {
      PalletRegistry.setAddressNameCall => const [
          'province_code',
          'city_code',
          'town_code',
          'address_name_code',
          'address_name',
        ],
      PalletRegistry.removeAddressNameCall => const [
          'province_code',
          'city_code',
          'town_code',
          'address_name_code',
        ],
      PalletRegistry.setAddressCall ||
      PalletRegistry.removeAddressCall =>
        const [
          'province_code',
          'city_code',
          'town_code',
          'address_name_code',
          'address_local_no',
          'address_detail',
        ],
      _ => null,
    };
    if (fieldNames == null) return null;
    final fields = <String, String>{
      'actor_cid_number': actorRead.$1,
    };
    for (final fieldName in fieldNames) {
      final (value, next) = _readUtf8Vec(bytes, offset);
      if (value == null) return null;
      final canBeEmpty =
          fieldName == 'address_local_no' || fieldName == 'address_detail';
      if (!canBeEmpty && value.isEmpty) return null;
      fields[fieldName] = value;
      offset = next;
    }
    if (!_hasValidSigningTail(bytes, offset)) return null;
    final action = switch (callIndex) {
      PalletRegistry.setAddressNameCall => 'set_address_name',
      PalletRegistry.removeAddressNameCall => 'remove_address_name',
      PalletRegistry.setAddressCall => 'set_address',
      PalletRegistry.removeAddressCall => 'remove_address',
      _ => '',
    };
    final actionText = switch (callIndex) {
      PalletRegistry.setAddressNameCall => '设置地址名称',
      PalletRegistry.removeAddressNameCall => '删除地址名称',
      PalletRegistry.setAddressCall => '设置完整地址',
      PalletRegistry.removeAddressCall => '删除完整地址',
      _ => '',
    };
    return DecodedPayload(
      action: action,
      summary:
          '$actionText ${fields['province_code']}/${fields['city_code']}/${fields['town_code']}',
      fields: fields,
    );
  }

  // SquarePost(34) / propose_set_platform_price(5)
  // SCALE:actor_cid_number + membership_level:u8 + new_price_fen:u128。
  static DecodedPayload? _decodeProposeSetPlatformPrice(Uint8List bytes) {
    final actorRead = _readCidNumber(bytes, 2);
    if (actorRead == null) return null;
    var offset = actorRead.$2;
    if (offset + 1 + 16 > bytes.length) return null;
    final membershipLevel = bytes[offset++];
    final membershipLabel = switch (membershipLevel) {
      0 => '自由会员',
      1 => '民主会员',
      2 => '薪火会员',
      _ => null,
    };
    if (membershipLabel == null) return null;
    final newPriceFen = _readU128Le(bytes, offset);
    offset += 16;
    if (newPriceFen <= BigInt.zero || !_hasCallDataEnd(bytes, offset)) {
      return null;
    }
    final priceText = '${_fenToYuan(newPriceFen)} 元（$newPriceFen 分）';
    return DecodedPayload(
      action: 'propose_set_platform_price',
      summary: '由 ${actorRead.$1} 发起$membershipLabel调价提案：$priceText',
      fields: {
        'actor_cid_number': actorRead.$1,
        'membership_level': membershipLabel,
        'new_price_fen': priceText,
      },
    );
  }

  // 工具方法
  /// SigningPayload 扩展尾固定段:spec_version(4) + tx_version(4)
  /// + genesis_hash(32) + birth_hash(32) + CheckMetadataHash None(1)。
  static const int _signingTailFixedLen = 73;

  /// 校验 call_data 在 [callEnd] 处结束,其后是合法的 SigningPayload 扩展尾。
  ///
  /// QR 的 payload_hex 是完整 SigningPayload,call_data 永远不会顶到末尾;
  /// 尾部布局与节点端 build_signing_payload / CitizenApp polkadart 编码一致:
  /// era(0x00 immortal,P-SIGN-001) + Compact<nonce> + Compact<tip=0>
  /// + mode(0x00) + 固定 73 字节(末字节 Option::None=0x00,immortal 下
  /// birth hash 必等于 genesis hash)。
  /// 所有链上 extrinsic 分支统一以此判定 call_data 边界:既接受真实 payload,
  /// 又防止在 call_data 后夹带任意字节骗签。
  static bool _hasValidSigningTail(Uint8List bytes, int callEnd) {
    if (callEnd < 2 || callEnd >= bytes.length) return false;
    var offset = callEnd;
    // CheckEra: immortal 单字节 0x00。
    if (bytes[offset] != 0x00) return false;
    offset += 1;
    // CheckNonce: Compact<u32>
    final (nonceValue, nonceSize) = _decodeCompactBigInt(bytes, offset);
    if (nonceValue == null || nonceSize == 0) return false;
    offset += nonceSize;
    // ChargeTransactionPayment: Compact<u128> tip
    final (tipValue, tipSize) = _decodeCompactBigInt(bytes, offset);
    // tip 不属于五类交易费；冷钱包必须在签名前拒绝任何非零 tip。
    if (tipValue == null || tipSize == 0 || tipValue != BigInt.zero) {
      return false;
    }
    offset += tipSize;
    // CheckMetadataHash: mode=Disabled(0x00)
    if (offset >= bytes.length || bytes[offset] != 0x00) return false;
    offset += 1;
    if (bytes.length - offset != _signingTailFixedLen) return false;
    // 末字节 CheckMetadataHash Option::None。
    if (bytes[bytes.length - 1] != 0x00) return false;
    // immortal:CheckEra additional 的 birth hash 必等于 genesis hash。
    final genesisStart = offset + 8;
    for (var i = 0; i < 32; i++) {
      if (bytes[genesisStart + i] != bytes[genesisStart + 32 + i]) {
        return false;
      }
    }
    return true;
  }

  /// OnChina 部分 QR 直接携带裸 call_data；完整 SigningPayload 仍走严格尾部校验。
  static bool _hasCallDataEnd(Uint8List bytes, int callEnd) {
    return callEnd == bytes.length || _hasValidSigningTail(bytes, callEnd);
  }

  static bool _isValidDateInt(int value) {
    final year = value ~/ 10000;
    final month = (value ~/ 100) % 100;
    final day = value % 100;
    return year >= 1900 &&
        year <= 9999 &&
        month >= 1 &&
        month <= 12 &&
        day >= 1 &&
        day <= 31;
  }

  static String _formatDateInt(int value) {
    final year = value ~/ 10000;
    final month = (value ~/ 100) % 100;
    final day = value % 100;
    return '${year.toString().padLeft(4, '0')}-${month.toString().padLeft(2, '0')}-${day.toString().padLeft(2, '0')}';
  }

  /// 0x 小写 hex。
  static String _bytesToLowerHex(Uint8List bytes) {
    return '0x${bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
  }

  /// 32 字节账户/公钥 bytes → CitizenChain SS58 地址。
  static String _bytesToSs58(Uint8List bytes) {
    return Keyring().encodeAddress(bytes.toList(), _ss58Prefix);
  }

  /// 人机界面统一显示 SS58 地址；无法确认是 32 字节账户时保留原值。
  static String _pubkeyHexToSs58OrRaw(String value) {
    final trimmed = value.trim();
    final clean = trimmed.startsWith('0x') || trimmed.startsWith('0X')
        ? trimmed.substring(2)
        : trimmed;
    if (clean.length != 64 || !RegExp(r'^[0-9a-fA-F]+$').hasMatch(clean)) {
      return value;
    }
    try {
      return _bytesToSs58(_hexToBytes(clean));
    } catch (_) {
      return value;
    }
  }

  static Uint8List _hexToBytes(String input) {
    final text = (input.startsWith('0x') || input.startsWith('0X'))
        ? input.substring(2)
        : input;
    if (text.isEmpty || text.length.isOdd) return Uint8List(0);
    return Uint8List.fromList(List<int>.generate(
      text.length ~/ 2,
      (i) => int.parse(text.substring(i * 2, i * 2 + 2), radix: 16),
      growable: false,
    ));
  }

  /// 读取 little-endian u64。
  ///
  /// 注意：Dart int 在 native 平台为 64 位，在 Web 平台为 53 位。
  /// 此方法仅适用于 Flutter mobile（native），Web 环境下大 u64 值会溢出。
  static int _readU64Le(Uint8List bytes, int offset) {
    var value = 0;
    for (var i = 7; i >= 0; i--) {
      value = (value << 8) | bytes[offset + i];
    }
    return value;
  }

  static int _readU32Le(Uint8List bytes, int offset) {
    return bytes[offset] |
        (bytes[offset + 1] << 8) |
        (bytes[offset + 2] << 16) |
        (bytes[offset + 3] << 24);
  }

  static BigInt _readU128Le(Uint8List bytes, int offset) {
    var value = BigInt.zero;
    for (var i = 15; i >= 0; i--) {
      value = (value << 8) | BigInt.from(bytes[offset + i]);
    }
    return value;
  }

  static int _minimumRegularThreshold(int adminsLen) {
    return (adminsLen ~/ 2) + 1;
  }

  /// 个人多签管理员集合变更阈值合法性。
  static bool _validAdminChangeThreshold(
    String code,
    int adminsLen,
    int threshold,
  ) {
    return InstitutionCode.isPersonal(code) &&
        adminsLen >= 2 &&
        threshold > adminsLen ~/ 2 &&
        threshold <= adminsLen;
  }

  static bool _validAdminChangePalletForCode(
    int palletIndex,
    int callIndex,
    String code,
  ) {
    if (palletIndex == PalletRegistry.personalAdminsPallet) {
      return callIndex == PalletRegistry.proposePersonalAdminSetChangeCall &&
          InstitutionCode.isPersonal(code);
    }
    return false;
  }

  static String _adminSetChangeActionForPallet(int palletIndex) {
    return switch (palletIndex) {
      PalletRegistry.personalAdminsPallet =>
        'propose_personal_admin_set_change',
      _ => 'propose_unknown_admin_set_change',
    };
  }

  /// 解码 SCALE Compact<BigInt>，返回 (值, 消耗字节数)。
  static (BigInt?, int) _decodeCompactBigInt(Uint8List bytes, int offset) {
    if (offset >= bytes.length) return (null, 0);
    final mode = bytes[offset] & 0x03;
    switch (mode) {
      case 0:
        return (BigInt.from(bytes[offset] >> 2), 1);
      case 1:
        if (offset + 2 > bytes.length) return (null, 0);
        final val = ((bytes[offset + 1] << 8) | bytes[offset]) >> 2;
        return (BigInt.from(val), 2);
      case 2:
        if (offset + 4 > bytes.length) return (null, 0);
        final val = ((bytes[offset + 3] << 24) |
                (bytes[offset + 2] << 16) |
                (bytes[offset + 1] << 8) |
                bytes[offset]) >>
            2;
        return (BigInt.from(val), 4);
      case 3:
        final byteLen = (bytes[offset] >> 2) + 4;
        if (offset + 1 + byteLen > bytes.length) return (null, 0);
        var value = BigInt.zero;
        for (var i = byteLen - 1; i >= 0; i--) {
          value = (value << 8) | BigInt.from(bytes[offset + 1 + i]);
        }
        return (value, 1 + byteLen);
      default:
        return (null, 0);
    }
  }

  /// 解码 SCALE Compact<u32>，返回 (值, 消耗字节数)。
  static (int, int) _decodeCompactU32(Uint8List bytes, int offset) {
    final (value, size) = _decodeCompactBigInt(bytes, offset);
    return (value?.toInt() ?? 0, size);
  }

  /// 解码 SCALE Vec<u8> 并按 UTF-8 转字符串。
  static (String?, int) _readUtf8Vec(Uint8List bytes, int offset) {
    if (offset >= bytes.length) return (null, offset);
    final (len, lenSize) = _decodeCompactU32(bytes, offset);
    if (lenSize == 0) return (null, offset);
    offset += lenSize;
    if (offset + len > bytes.length) return (null, offset);
    final text = utf8.decode(
      bytes.sublist(offset, offset + len),
      allowMalformed: true,
    );
    return (text, offset + len);
  }

  /// 严格解码全仓统一的 `Vec<Admin>`。
  ///
  /// SCALE 顺序只允许 `admin_account + family_name + given_name`。旧纯账户、
  /// 旧合并姓名、空姓名、非法 UTF-8 和重复账户全部拒签。
  static (List<_DecodedAdminPerson>, int)? _readAdminPersons(
    Uint8List bytes,
    int offset, {
    required int minCount,
    required int maxCount,
  }) {
    final (count, countSize) = _decodeCompactU32(bytes, offset);
    if (countSize == 0 || count < minCount || count > maxCount) return null;
    offset += countSize;
    final seen = <String>{};
    final admins = <_DecodedAdminPerson>[];
    for (var index = 0; index < count; index++) {
      if (offset + 32 > bytes.length) return null;
      final accountBytes =
          Uint8List.fromList(bytes.sublist(offset, offset + 32));
      offset += 32;
      final accountHex = _bytesToLowerHex(accountBytes);
      if (!seen.add(accountHex)) return null;
      final familyNameRead = _readAdminName(bytes, offset);
      if (familyNameRead == null) return null;
      offset = familyNameRead.$2;
      final givenNameRead = _readAdminName(bytes, offset);
      if (givenNameRead == null) return null;
      offset = givenNameRead.$2;
      admins.add((
        accountBytes: accountBytes,
        adminAccountHex: accountHex,
        familyName: familyNameRead.$1,
        givenName: givenNameRead.$1,
      ));
    }
    return (List<_DecodedAdminPerson>.unmodifiable(admins), offset);
  }

  /// 管理员姓、名各自必须是 1..=128 字节的严格 UTF-8。
  static (String, int)? _readAdminName(Uint8List bytes, int offset) {
    if (offset >= bytes.length) return null;
    final (length, lengthSize) = _decodeCompactU32(bytes, offset);
    if (lengthSize == 0 || length < 1 || length > 128) return null;
    final start = offset + lengthSize;
    final end = start + length;
    if (end > bytes.length) return null;
    try {
      return (utf8.decode(bytes.sublist(start, end)), end);
    } on FormatException {
      return null;
    }
  }

  static String _adminMachineValue(List<_DecodedAdminPerson> admins) {
    return jsonEncode([
      for (final admin in admins)
        {
          'admin_account': admin.adminAccountHex,
          'family_name': admin.familyName,
          'given_name': admin.givenName,
        },
    ]);
  }

  static String _adminReviewValue(List<_DecodedAdminPerson> admins) {
    return admins
        .map(
          (admin) =>
              '${admin.familyName}${admin.givenName}(${_bytesToSs58(admin.accountBytes)})',
        )
        .join('、');
  }

  /// 机构治理共用读取器；对外只返回人数、展示文本和新偏移。
  static (int, String, int)? _readInstitutionAdmins(
    Uint8List bytes,
    int offset,
  ) {
    final read = _readAdminPersons(
      bytes,
      offset,
      minCount: 0,
      maxCount: 1989,
    );
    if (read == null) return null;
    return (read.$1.length, _adminReviewValue(read.$1), read.$2);
  }

  static (String, String, int)? _readInstitutionGovernanceAction(
    Uint8List bytes,
    int offset,
  ) {
    if (offset >= bytes.length) return null;
    final variant = bytes[offset++];
    if (variant == 0) {
      final admins = _readInstitutionAdmins(bytes, offset);
      if (admins == null || admins.$1 < 2) return null;
      return ('替换管理员集合', '${admins.$1} 名管理员：${admins.$2}', admins.$3);
    }
    if (variant == 1) {
      final next = _skipRoleGovernance(bytes, offset);
      if (next == null) return null;
      return ('岗位/任职治理', next.$1, next.$2);
    }
    if (variant == 2) {
      final admins = _readInstitutionAdmins(bytes, offset);
      if (admins == null || admins.$1 < 2) return null;
      final next = _skipRoleGovernance(bytes, admins.$3);
      if (next == null) return null;
      return (
        '替换管理员并治理岗位',
        '${admins.$1} 名管理员；${next.$1}',
        next.$2,
      );
    }
    return null;
  }

  static (String, int)? _skipRoleGovernance(Uint8List bytes, int offset) {
    final roleChanges = _skipRoleChanges(bytes, offset);
    if (roleChanges == null) return null;
    offset = roleChanges.$2;
    final assignmentChanges = _skipAssignmentChanges(bytes, offset);
    if (assignmentChanges == null) return null;
    offset = assignmentChanges.$2;
    final legal = _skipLegalRepresentativeChange(bytes, offset);
    if (legal == null) return null;
    offset = legal.$2;
    if (roleChanges.$1 == 0 && assignmentChanges.$1 == 0 && legal.$1.isEmpty) {
      return null;
    }
    final legalText = legal.$1.isEmpty ? '' : '，${legal.$1}';
    return (
      '${roleChanges.$1} 个岗位变更，${assignmentChanges.$1} 个任职集合变更$legalText',
      offset,
    );
  }

  static (int, int)? _skipRoleChanges(Uint8List bytes, int offset) {
    final (count, countSize) = _decodeCompactU32(bytes, offset);
    if (countSize == 0) return null;
    offset += countSize;
    final seen = <String>{};
    for (var index = 0; index < count; index++) {
      final code = _readUtf8Vec(bytes, offset);
      if (code.$1 == null || code.$1!.isEmpty || !seen.add(code.$1!)) {
        return null;
      }
      offset = code.$2;
      final name = _readUtf8Vec(bytes, offset);
      if (name.$1 == null || name.$1!.isEmpty) return null;
      offset = name.$2;
      if (offset + 2 > bytes.length) return null;
      final termRequired = bytes[offset++];
      final status = bytes[offset++];
      if (termRequired > 1 || status > 1) return null;
    }
    return (count, offset);
  }

  static (int, int)? _skipAssignmentChanges(Uint8List bytes, int offset) {
    final (count, countSize) = _decodeCompactU32(bytes, offset);
    if (countSize == 0) return null;
    offset += countSize;
    final seenRoles = <String>{};
    for (var index = 0; index < count; index++) {
      final role = _readUtf8Vec(bytes, offset);
      if (role.$1 == null || role.$1!.isEmpty || !seenRoles.add(role.$1!)) {
        return null;
      }
      offset = role.$2;
      final (assignmentCount, assignmentSize) =
          _decodeCompactU32(bytes, offset);
      if (assignmentSize == 0) return null;
      offset += assignmentSize;
      final seenAccounts = <String>{};
      for (var i = 0; i < assignmentCount; i++) {
        if (offset + 32 + 4 + 4 + 1 > bytes.length) return null;
        final account = _bytesToLowerHex(bytes.sublist(offset, offset + 32));
        if (!seenAccounts.add(account)) return null;
        offset += 32;
        offset += 4; // term_start:u32
        offset += 4; // term_end:u32
        final source = bytes[offset++];
        // 0 Genesis,1 Registry,2 PopularElection,3 MutualElection,
        // 4 NominationAppointment,5 InstitutionGovernance。
        if (source > 5) return null;
        offset = _skipBoundedBytes(bytes, offset); // assignment_source_ref
        if (offset < 0 || offset >= bytes.length) return null;
        final status = bytes[offset++];
        if (status > 1) return null;
      }
    }
    return (count, offset);
  }

  static (String, int)? _skipLegalRepresentativeChange(
    Uint8List bytes,
    int offset,
  ) {
    if (offset >= bytes.length) return null;
    final optionTag = bytes[offset++];
    if (optionTag == 0) return ('', offset);
    if (optionTag != 1 || offset >= bytes.length) return null;
    final variant = bytes[offset++];
    if (variant == 1) {
      return ('含法定代表人解除', offset);
    }
    if (variant != 0) return null;
    final name = _readUtf8Vec(bytes, offset);
    if (name.$1 == null || name.$1!.isEmpty) return null;
    offset = name.$2;
    final cid = _readCidNumber(bytes, offset);
    if (cid == null) return null;
    offset = cid.$2;
    if (offset + 32 > bytes.length) return null;
    offset += 32;
    return ('含法定代表人任命/更换', offset);
  }

  /// OnchainIssuance 元数据字段的严格 BoundedVec<u8> 解码。
  /// 非法 UTF-8、越界长度或截断一律拒绝，禁止展示替换字符后继续签名。
  static (String, int)? _readStrictBoundedUtf8(
    Uint8List bytes,
    int offset, {
    required int maxLength,
  }) {
    final (length, compactSize) = _decodeCompactU32(bytes, offset);
    if (compactSize == 0 || length > maxLength) return null;
    final start = offset + compactSize;
    final end = start + length;
    if (end > bytes.length) return null;
    try {
      return (utf8.decode(bytes.sublist(start, end)), end);
    } on FormatException {
      return null;
    }
  }

  /// 解码机构/公民 CID。CID 是最多 32 字节的非空 ASCII，所有机构交易都显式携带，
  /// 离线端不得从机构账户反推或回落出一个 CID。
  static (String, int)? _readCidNumber(Uint8List bytes, int offset) {
    if (offset >= bytes.length) return null;
    final (len, lenSize) = _decodeCompactU32(bytes, offset);
    if (lenSize == 0 || len == 0 || len > 32) return null;
    final start = offset + lenSize;
    final end = start + len;
    if (end > bytes.length) return null;
    final raw = bytes.sublist(start, end);
    if (raw.any((byte) => byte < 0x21 || byte > 0x7e)) return null;
    return (ascii.decode(raw), end);
  }

  /// 解码 `Option<CidNumber>`；只接受规范的 0/1 判别值。
  static (String?, int)? _readOptionalCidNumber(
    Uint8List bytes,
    int offset,
  ) {
    if (offset >= bytes.length) return null;
    final tag = bytes[offset++];
    if (tag == 0) return (null, offset);
    if (tag != 1) return null;
    final cidRead = _readCidNumber(bytes, offset);
    if (cidRead == null) return null;
    return (cidRead.$1, cidRead.$2);
  }

  /// 激活凭证里的账户 kind 与机构码是否匹配。
  ///
  /// kind 语义对齐链端 admin-primitives::AdminAccountKind(SCALE 判别值):
  ///   0 = PublicInstitution
  ///   1 = PrivateInstitution
  ///   2 = PersonalMultisig
  static bool _activationAccountKindMatchesCode(String code, int kind) {
    if (InstitutionCode.isPublicLegal(code) ||
        InstitutionCode.isFixedGovernance(code)) {
      return kind == 0;
    }
    if (InstitutionCode.isPrivateLegal(code)) {
      return kind == 1;
    }
    // 非法人机构由上层明确路由到 public/private admins，两个 kind 都合法。
    if (InstitutionCode.isUnincorporated(code)) return kind == 0 || kind == 1;
    return false;
  }

  static bool _hasPrefix(Uint8List bytes, Uint8List prefix) {
    if (bytes.length < prefix.length) return false;
    for (var i = 0; i < prefix.length; i++) {
      if (bytes[i] != prefix[i]) return false;
    }
    return true;
  }

  /// 分 → 元字符串（带千分位）。
  static String _fenToYuan(BigInt fen) {
    final yuan = fen ~/ BigInt.from(100);
    final remainder = (fen % BigInt.from(100)).toInt().abs();
    final intStr = yuan.toString().replaceAllMapped(
          RegExp(r'(\d)(?=(\d{3})+(?!\d))'),
          (m) => '${m[1]},',
        );
    return '$intStr.${remainder.toString().padLeft(2, '0')}';
  }

  /// 截断地址显示。
  static String _truncateAddress(String addr) {
    if (addr.length <= 16) return addr;
    return '${addr.substring(0, 8)}...${addr.substring(addr.length - 6)}';
  }
}
