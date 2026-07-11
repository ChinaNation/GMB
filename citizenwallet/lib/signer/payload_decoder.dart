import 'dart:convert';
import 'dart:typed_data';

import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import '../chain/chain_constants.dart';
import '../chain/reserved_account_names.dart';
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

  /// 尝试从 payload_hex 中解码交易信息。
  ///
  /// [payloadHex] 为完整 SigningPayload 编码的 hex 字符串。
  /// call data 从 payload 起始位置开始，以 pallet_index 和 call_index 为前两字节。
  ///
  /// 返回 null 表示无法识别或解码失败 → strict 模式下 decodeFailed → 禁止签名。
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
      if (raw.length ==
              _activateAdminPrefix.length + 32 + 4 + 1 + 32 + 8 + 16 &&
          _hasPrefix(raw, _activateAdminPrefix)) {
        return _decodeActivateAdminAccount(raw);
      }
      // DECRYPT challenge = prefix(4) + cid_number(48) + pubkey(32)
      //   + timestamp(8) + nonce(16) = 108 字节(对齐 node CHALLENGE_TOTAL_LEN)。
      if (raw.length == _binaryPrefixLen + 48 + 32 + 8 + 16 &&
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
    // - decoder 解析失败(任何分支不匹配返回 null) → decodeFailed → 禁止签名
    // - 解析成功但 QR 动作码与 decoded.action 不一致 → mismatched → 禁止签名
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

      // ── InternalVote sub-pallet (22) · 内部投票管理员一人一票 ──
      if (palletIndex == PalletRegistry.internalVotePallet &&
          callIndex == PalletRegistry.internalVoteCall) {
        return _decodeInternalVote(bytes);
      }

      // ── JointVote sub-pallet (23) · 联合投票(内部投票阶段 + 联合公投)──
      if (palletIndex == PalletRegistry.jointVotePallet) {
        if (callIndex == PalletRegistry.jointVoteCall) {
          return _decodeJointVote(bytes);
        }
        if (callIndex == PalletRegistry.castReferendumCall) {
          return _decodeCastReferendum(bytes);
        }
        if (callIndex == PalletRegistry.preparePopulationSnapshotCall) {
          return _decodeJointPopulationSnapshot(bytes);
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

      // ── CitizenIdentity(10) · 公民链上投票/参选身份注册 ──
      if (palletIndex == PalletRegistry.citizenIdentityPallet) {
        if (callIndex == PalletRegistry.registerVotingIdentityCall) {
          return _decodeRegisterVotingIdentity(bytes);
        }
        if (callIndex == PalletRegistry.upgradeToCandidateIdentityCall) {
          return _decodeUpgradeToCandidateIdentity(bytes);
        }
      }

      // ── MultisigTransfer(19) ──
      // 投票入口统一到 InternalVote::cast(22.0),
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

      // ── 协议升级 RuntimeUpgrade(13) ──
      // 路由分支删除:propose_runtime_upgrade / developer_direct_upgrade 的
      // call_data 含完整 WASM(600KB+),物理上塞不进 QR。server 在 QR 里只放
      // blake2_256(payload) = 32 字节哈希,decoder 拿不到 call_data,无法
      // SCALE 解析。改走 OfflineSignService.verifyPayload 的"哈希直签例外":
      // 用户在冷钱包屏幕上核对 32 字节哈希后放行。

      // ── PublicManage(32) / PrivateManage(33) ──
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
        final cleanupAction = isPublic
            ? 'cleanup_rejected_public_proposal'
            : 'cleanup_rejected_private_proposal';
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
        if (callIndex ==
            PalletRegistry.cleanupRejectedInstitutionProposalCall) {
          return _decodeProposalIdOnly(
            bytes,
            action: cleanupAction,
            summaryTemplate: '清理被拒$entityLabel提案 #{id} 残留',
          );
        }
      }

      // ── PersonalAdmins(7) ──
      // 个人多签独立 pallet,MODULE_TAG = b"per-mgmt"。
      // ACTION enum 独立(ACTION_CREATE=0/ACTION_CLOSE=1),与实体生命周期模块互不干扰。
      if (palletIndex == PalletRegistry.personalAdminsPallet) {
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
        if (callIndex == PalletRegistry.cleanupRejectedPersonalProposalCall) {
          return _decodeProposalIdOnly(
            bytes,
            action: 'cleanup_rejected_personal_proposal',
            summaryTemplate: '清理被拒个人多签提案 #{id} 残留',
          );
        }
        if (PalletRegistry.isPersonalAdminSetChangeCall(
          palletIndex,
          callIndex,
        )) {
          return _decodeProposeAdminSetChange(bytes);
        }
      }

      // ── ResolutionIssuance(8) · 决议发行联合提案 ──
      // 人口快照由 JointVote.prepare_population_snapshot 单独准备。
      if (palletIndex == PalletRegistry.resolutionIssuancePallet) {
        if (callIndex == PalletRegistry.proposeIssuanceCall) {
          return _decodeProposeResolutionIssuance(bytes);
        }
      }

      // ── ResolutionDestroy(14) ──
      // execute_destroy 走 VotingEngine::retry_passed_proposal。
      if (palletIndex == PalletRegistry.resolutionDestroPallet) {
        if (callIndex == PalletRegistry.proposeDestroyCall) {
          return _decodeProposeDestroy(bytes);
        }
      }

      // ── PublicAdmins(29) / PrivateAdmins(30) ──
      // 管理员集合变更走 propose_admin_set_change；执行统一由 VotingEngine 重试。
      if (PalletRegistry.isAdminSetChangePallet(palletIndex)) {
        if (callIndex == PalletRegistry.proposeAdminSetChangeCall) {
          return _decodeProposeAdminSetChange(bytes);
        }
      }

      // ── GrandpaKeyChange(16) ──
      // execute_replace_grandpa_key / cancel_failed_replace_grandpa_key
      // 分别走 VotingEngine::retry_passed_proposal / cancel_passed_proposal。
      if (palletIndex == PalletRegistry.grandpaKeyChangePallet) {
        if (callIndex == PalletRegistry.proposeReplaceGrandpaKeyCall) {
          return _decodeProposeKeyChange(bytes);
        }
      }

      // ── OffchainTransaction(21) · 清算行 L2 体系 ──
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
      }

      // ── OnchainIssuance(25) · 链上发行代币(Plain FT) ──
      // 当前未实现完整 SCALE 字段解析,不能独立验证业务内容,因此红色拒签。
      if (palletIndex == PalletRegistry.onchainIssuancePallet) {
        return null;
      }

      // ── LegislationYuan(27) · 立法/修法/废法发起 ──
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

      // ── LegislationVote(28) · 立法专属投票引擎 ──
      // 院内表决/公投/行政签署/三人会签/护宪终审走 proposal_id+approve;
      // 人口快照只携带作用域,链端直接读取 citizen-identity。
      if (palletIndex == PalletRegistry.legislationVotePallet) {
        if (callIndex == PalletRegistry.prepareLegislationSnapshotCall) {
          return _decodePrepareLegislationSnapshot(bytes);
        }
        if (callIndex == PalletRegistry.castHouseVoteCall) {
          return _decodeProposalApprove(
            bytes,
            action: 'cast_house_vote',
            summaryTemplate: '院内表决 立法提案 #{id}：{vote}',
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
      final province = value['actor_province_name'];
      final actorPubkey = value['actor_pubkey'];
      final target = value['target'];
      final beforeHash = value['before_hash'];
      final afterHash = value['after_hash'];
      if (actionType is! String ||
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
          'actor_province_name': province,
          'actor_pubkey': actorAddress,
          'target': targetAddress,
          'before_hash': beforeHash,
          'after_hash': afterHash,
        },
        reviewFields: <String, String>{
          'action_type': _onchinaAdminActionLabel(actionType),
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

  // MultisigTransfer(19) / propose_transfer(0)
  // 格式：[0x13][0x00][institution_code:[u8;4]][institution:AccountId32][beneficiary:32][amount:u128_le][Vec remark]
  static DecodedPayload? _decodeProposeTransfer(Uint8List bytes) {
    // 最小长度：2 + 4 + 32 + 32 + 16 + 1 = 87
    if (bytes.length < 87) return null;

    var offset = 2;

    // institution_code: [u8;4] 用于展示机构类型；实际治理账户是 32 字节 AccountId。
    final codeBytes = bytes.sublist(offset, offset + 4);
    offset += 4;
    final code = InstitutionCode.codeToString(codeBytes);

    final institutionBytes = bytes.sublist(offset, offset + 32);
    offset += 32;
    final institutionLabel = _institutionAccountLabel(code, institutionBytes);

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
          '$institutionLabel 提案转账 $amountYuan GMB 给 ${_truncateAddress(beneficiary)}',
      fields: {
        'institution': institutionLabel,
        'beneficiary': beneficiary,
        'amount_yuan': '$amountYuan GMB',
        'remark': remark,
      },
    );
  }

  // 业务 pallet 的 finalize_X / vote_X 全部下线,
  // 冷钱包统一通过 `_decodeInternalVote` 解码一人一票的管理员投票 payload。
  // InternalVote(22) / cast(0)
  // 格式：[0x16][0x00][proposal_id:u64_le][approve:bool]
  //
  // 统一入口:所有业务 pallet(admins/resolution_destroy/grandpa_key/
  // entity_manage/multisig_transfer 五路)的管理员投票都走 InternalVote::cast(22.0),
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

  // JointVote(23) / cast_admin(0)
  // 格式：[0x17][0x00][proposal_id:u64_le][institution:AccountId32][approve:bool]
  static DecodedPayload? _decodeJointVote(Uint8List bytes) {
    // call_data: 2 + 8 + 32 + 1 = 43
    if (bytes.length < 43 || !_hasValidSigningTail(bytes, 43)) return null;

    final proposalId = _readU64Le(bytes, 2);
    // institution AccountId32 跳过,扫码页只展示投票结论所需字段。
    final approve = bytes[42] != 0;
    final voteText = approve ? '赞成' : '反对';

    return DecodedPayload(
      action: 'joint_vote',
      summary: '联合投票 提案 #$proposalId：$voteText',
      fields: {
        'proposal_id': proposalId.toString(),
        'approve': approve.toString(),
      },
    );
  }

  // JointVote(23) / cast_referendum(1)
  // 格式：[0x17][0x01][proposal_id:u64_le][approve:bool]
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

  // JointVote(23) / prepare_population_snapshot(2)
  // 格式：[0x17][0x02][scope:PopulationScope]
  static DecodedPayload? _decodeJointPopulationSnapshot(Uint8List bytes) {
    final (scopeFields, offset) = _decodePopulationScope(bytes, 2);
    if (scopeFields == null || !_hasValidSigningTail(bytes, offset)) {
      return null;
    }

    return DecodedPayload(
      action: 'prepare_population_snapshot',
      summary: '准备联合公投人口快照（${scopeFields['scope_text']}）',
      fields: scopeFields,
    );
  }

  // 协议升级 RuntimeUpgrade(13) / propose_runtime_upgrade(0) / developer_direct_upgrade(2)
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
  // 格式：prefix(4B = GMB||0x18) + account_id(32B) + institution_code([u8;4])
  //      + kind(u8) + pubkey(32B) + timestamp(8B, u64 LE) + nonce(16B) = 97B
  static DecodedPayload? _decodeActivateAdminAccount(Uint8List bytes) {
    final expectedLength =
        _activateAdminPrefix.length + 32 + 4 + 1 + 32 + 8 + 16;
    if (bytes.length != expectedLength) return null;

    var offset = _activateAdminPrefix.length;
    final accountBytes = bytes.sublist(offset, offset + 32);
    offset += 32;

    final codeBytes = bytes.sublist(offset, offset + 4);
    offset += 4;
    final code = InstitutionCode.codeToString(codeBytes);
    final kind = bytes[offset++];
    if (!_activationAccountKindMatchesCode(code, kind)) return null;

    final pubkey = bytes.sublist(offset, offset + 32);
    final accountHex = _bytesToLowerHex(accountBytes);
    final institutionLabel = InstitutionCode.codeLabel(code);

    return DecodedPayload(
      action: 'activate_admin_account',
      summary: '激活$institutionLabel管理员',
      fields: {
        'institution_code': institutionLabel,
        'account': accountHex,
        'pubkey': _bytesToSs58(pubkey),
      },
      reviewFields: {
        'institution_code': institutionLabel,
        'account': _bytesToSs58(accountBytes),
        'pubkey': _bytesToSs58(pubkey),
      },
    );
  }

  // 清算行管理员解密（非链上交易，二进制前缀域）
  // 格式：prefix(4B = GMB||0x19) + cid_number(48B, 右补零) + pubkey(32B)
  //      + timestamp(8B, u64 LE) + nonce(16B) = 108B
  static DecodedPayload? _decodeDecryptAdmin(Uint8List bytes) {
    const totalLen = _binaryPrefixLen + 48 + 32 + 8 + 16;
    if (bytes.length != totalLen) return null;

    const idStart = _binaryPrefixLen;
    final idBytes = bytes.sublist(idStart, idStart + 48);
    var endIndex = 48;
    while (endIndex > 0 && idBytes[endIndex - 1] == 0) {
      endIndex--;
    }
    if (endIndex == 0) return null;
    final cidNumber = String.fromCharCodes(idBytes.sublist(0, endIndex));

    return DecodedPayload(
      action: 'decrypt_admin',
      summary: '解密清算行管理员 - $cidNumber',
      fields: {
        'cid_number': cidNumber,
      },
    );
  }

  // PublicManage(32) / PrivateManage(33) / propose_create_*_institution(5)
  //
  // 链端签名(签发机构 admins 凭证):
  //   pub fn propose_create_*_institution(
  //     origin,
  //     cid_number: CidNumberOf<T>,                 // BoundedVec<u8>
  //     cid_full_name: AccountNameOf<T>,   // BoundedVec<u8>
  //     cid_short_name: AccountNameOf<T>,  // BoundedVec<u8>
  //     town_code: AccountNameOf<T>,       // BoundedVec<u8>
  //     accounts: InstitutionInitialAccountsOf<T>,
  //         // BoundedVec<{ account_name: BoundedVec<u8>, amount: u128 }>
  //     institution_code: [u8; 4],      // 注册多签机构码(公权/私权/非法人法人)
  //     admins_len: u32,
  //     admins: AdminProfilesOf<T>,   // BoundedVec<AdminProfile<AccountId32>>
  //     threshold: u32,
  //     register_nonce: RegisterNonceOf<T>,   // BoundedVec<u8>
  //     signature: RegisterSignatureOf<T>,    // BoundedVec<u8> (64B sr25519)
  //     issuer_cid_number: Vec<u8>,
  //     issuer_main_account: AccountId32,
  //     signer_pubkey: [u8; 32],
  //     scope_province_name: Vec<u8>,
  //     scope_city_name: Vec<u8>,
  //   )
  //
  // SCALE 顺序与上述完全一致。链端 RuntimeCidInstitutionVerifier 按
  // issuer_main_account 的 admins 真源确认 signer_pubkey。
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

    // accounts: BoundedVec<InstitutionInitialAccount>
    //   每项 = (account_name: Vec<u8>, amount: u128)
    final (accountsLen, accountsLenSize) = _decodeCompactU32(bytes, offset);
    offset += accountsLenSize;
    BigInt accountsTotal = BigInt.zero;
    final accountAmounts = <String, BigInt>{};
    for (var i = 0; i < accountsLen; i++) {
      final (subNameLen, subNameLenSize) = _decodeCompactU32(bytes, offset);
      offset += subNameLenSize;
      if (offset + subNameLen + 16 > bytes.length) return null;
      final accountName = utf8.decode(
        bytes.sublist(offset, offset + subNameLen),
        allowMalformed: true,
      );
      offset += subNameLen;
      // 制度专属保留名（永久质押/安全基金/两和基金）不可作为机构自定义账户注册，
      // 命中即判为不可信 payload → 返回 null（两色识别 decodeFailed = 红色拒签）。
      // 主账户/费用账户是强制默认账户，正常出现在创建凭证里，维持识别。
      if (ReservedAccountNames.isForbidden(accountName)) return null;
      final amount = _readU128Le(bytes, offset);
      accountAmounts[accountName] = amount;
      accountsTotal += amount;
      offset += 16;
    }

    // institution_code: [u8;4]。机构账户只能使用注册多签机构码(公权/私权/非法人法人)。
    if (offset + 4 > bytes.length) return null;
    final code =
        InstitutionCode.codeToString(bytes.sublist(offset, offset + 4));
    if (!InstitutionCode.isInstitution(code)) return null;
    offset += 4;

    // admins_len: u32 (LE)
    if (offset + 4 > bytes.length) return null;
    final adminsLen = bytes[offset] |
        (bytes[offset + 1] << 8) |
        (bytes[offset + 2] << 16) |
        (bytes[offset + 3] << 24);
    offset += 4;

    // admins: BoundedVec<AdminProfile<AccountId32>>
    final (adminsVecLen, adminsVecLenSize) = _decodeCompactU32(bytes, offset);
    offset += adminsVecLenSize;
    for (var i = 0; i < adminsVecLen; i++) {
      if (offset + 32 > bytes.length) return null;
      offset += 32; // account
      offset = _skipBoundedBytes(bytes, offset); // admin_cid_number
      if (offset < 0) return null;
      offset = _skipBoundedBytes(bytes, offset); // name
      if (offset < 0) return null;
      offset = _skipBoundedBytes(bytes, offset); // admin_role
      if (offset < 0) return null;
      if (offset + 9 > bytes.length) return null;
      offset += 4; // term_start
      offset += 4; // term_end
      offset += 1; // source
    }

    // threshold: u32 (LE)
    if (offset + 4 > bytes.length) return null;
    final threshold = bytes[offset] |
        (bytes[offset + 1] << 8) |
        (bytes[offset + 2] << 16) |
        (bytes[offset + 3] << 24);
    offset += 4;

    // register_nonce: BoundedVec<u8> — 跳过
    final (nonceLen, nonceLenSize) = _decodeCompactU32(bytes, offset);
    offset += nonceLenSize;
    if (offset + nonceLen > bytes.length) return null;
    offset += nonceLen;

    // signature: BoundedVec<u8> — 跳过
    final (sigLen, sigLenSize) = _decodeCompactU32(bytes, offset);
    offset += sigLenSize;
    if (offset + sigLen > bytes.length) return null;
    offset += sigLen;

    // issuer_cid_number: Vec<u8>
    final (issuerLen, issuerLenSize) = _decodeCompactU32(bytes, offset);
    offset += issuerLenSize;
    if (offset + issuerLen > bytes.length) return null;
    final issuerCidNumber = utf8.decode(
      bytes.sublist(offset, offset + issuerLen),
      allowMalformed: true,
    );
    offset += issuerLen;

    // issuer_main_account: AccountId32
    if (offset + 32 > bytes.length) return null;
    final issuerMainAccount = bytes.sublist(offset, offset + 32);
    offset += 32;

    // signer_pubkey: [u8; 32]
    if (offset + 32 > bytes.length) return null;
    final signerPubkey = bytes.sublist(offset, offset + 32);
    offset += 32;

    // scope_province_name: Vec<u8>
    final (scopeProvinceLen, scopeProvinceLenSize) =
        _decodeCompactU32(bytes, offset);
    offset += scopeProvinceLenSize;
    if (offset + scopeProvinceLen > bytes.length) return null;
    final scopeProvinceName = utf8.decode(
      bytes.sublist(offset, offset + scopeProvinceLen),
      allowMalformed: true,
    );
    offset += scopeProvinceLen;

    // scope_city_name: Vec<u8>
    final (scopeCityLen, scopeCityLenSize) = _decodeCompactU32(bytes, offset);
    offset += scopeCityLenSize;
    if (offset + scopeCityLen > bytes.length) return null;
    final scopeCityName = utf8.decode(
      bytes.sublist(offset, offset + scopeCityLen),
      allowMalformed: true,
    );
    offset += scopeCityLen;

    if (!_hasValidSigningTail(bytes, offset)) return null;

    final amountYuan = _fenToYuan(accountsTotal);
    final fields = <String, String>{
      'cid_number': cidNumber,
      'cid_full_name': cidFullName,
      'cid_short_name': cidShortName,
      'institution_code': InstitutionCode.codeLabel(code),
      'admins_len': adminsLen.toString(),
      'threshold': '$threshold/$adminsLen',
      'total_amount_yuan': '$amountYuan GMB',
    };
    for (final entry in accountAmounts.entries) {
      fields['amount_${entry.key}'] = '${_fenToYuan(entry.value)} GMB';
    }
    fields['issuer_cid_number'] = issuerCidNumber;
    fields['issuer_main_account'] = _bytesToSs58(issuerMainAccount);
    fields['signer_pubkey'] = _bytesToSs58(signerPubkey);
    fields['scope_province_name'] = scopeProvinceName;
    fields['scope_city_name'] = scopeCityName;
    if (townCode.isNotEmpty) {
      fields['town_code'] = townCode;
    }

    return DecodedPayload(
      action: action,
      summary:
          '创建$entityLabel多签账户「$cidFullName」（$adminsLen 管理员，阈值 $threshold，入金 $amountYuan 元）',
      fields: fields,
    );
  }

  // ResolutionIssuance(8) / propose_issuance(0)
  //
  // 链端签名:
  //   pub fn propose_issuance(
  //     origin,
  //     reason: ReasonOf<T>,                  // BoundedVec<u8>
  //     total_amount: BalanceOf<T>,           // u128 LE
  //     allocations: AllocationOf<T>,
  //         // BoundedVec<{ recipient: AccountId32, amount: u128 }>
  //   )
  //
  // 人口快照由 JointVote.prepare_population_snapshot 先行准备,本交易只携带发行内容。
  static DecodedPayload? _decodeProposeResolutionIssuance(Uint8List bytes) {
    if (bytes.length < 3) return null;
    var offset = 2; // 跳过 pallet_index + call_index

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
        'reason': reason,
        'amount_yuan': '$amountYuan GMB',
        'allocation_count': allocLen.toString(),
      },
    );
  }

  // PersonalAdmins(7) / propose_create(0)
  // 格式：[7][0][BoundedVec account_name][BoundedVec<AccountId32> admins][u32 regular_threshold][u128 amount]
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

    // admins: BoundedVec<AccountId32>。admins_len 由向量长度派生。
    final (adminsLen, adminsLenSize) = _decodeCompactU32(bytes, offset);
    offset += adminsLenSize;
    if (adminsLen < 2 || adminsLen > 64) return null;
    if (offset + adminsLen * 32 > bytes.length) return null;
    offset += adminsLen * 32;

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
        'admins_len': adminsLen.toString(),
        'regular_threshold': '$regularThreshold/$adminsLen',
        'create_threshold': '$adminsLen/$adminsLen',
        'amount_yuan': '$amountYuan GMB',
      },
    );
  }

  // PublicManage(32) / PrivateManage(33) / propose_close_*_institution(1)
  // PersonalAdmins(7) / propose_close(1)
  // 格式：[17][1][account:32][beneficiary:32]
  /// 机构注销 propose_close。
  /// call_data:[2][account:32][beneficiary:32]
  ///   [register_nonce:Vec][signature:Vec][issuer_cid_number:Vec]
  ///   [issuer_main_account:32][signer_pubkey:32] + 签名尾。
  /// 注销凭证由注册局在 CID 签发,机构管理员冷签上链(见 ADR-023 §6.3)。
  static DecodedPayload? _decodeProposeCloseInstitution(
    Uint8List bytes, {
    required String action,
    required String entityLabel,
  }) {
    var offset = 2;
    if (bytes.length < offset + 64) return null;
    final accountId = bytes.sublist(offset, offset + 32);
    offset += 32;
    final beneficiaryId = bytes.sublist(offset, offset + 32);
    offset += 32;
    // 依次跳过三个 Vec<u8>:register_nonce / signature / issuer_cid_number。
    for (var i = 0; i < 3; i++) {
      final (len, lenSize) = _decodeCompactU32(bytes, offset);
      if (lenSize == 0) return null;
      offset += lenSize + len;
      if (offset > bytes.length) return null;
    }
    // issuer_main_account:32 + signer_pubkey:32。
    if (offset + 64 > bytes.length) return null;
    offset += 64;
    if (!_hasValidSigningTail(bytes, offset)) return null;
    final account = Keyring().encodeAddress(accountId.toList(), _ss58Prefix);
    final beneficiary =
        Keyring().encodeAddress(beneficiaryId.toList(), _ss58Prefix);
    return DecodedPayload(
      action: action,
      summary:
          '提案注销$entityLabel多签 ${_truncateAddress(account)}(余额转 ${_truncateAddress(beneficiary)})',
      fields: {
        'account': account,
        'beneficiary': beneficiary,
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

  // MultisigTransfer(19) / propose_safety_fund(1)
  // 格式：[19][1][beneficiary:32][amount:u128][BoundedVec remark]
  static DecodedPayload? _decodeProposeSafetyFund(Uint8List bytes) {
    if (bytes.length < 50) return null;
    var offset = 2;
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
        'beneficiary': beneficiary,
        'amount_yuan': '$amountYuan GMB',
        'remark': remark,
      },
    );
  }

  // MultisigTransfer(19) / propose_sweep(2)
  // 格式：[19][2][institution:AccountId32][amount:u128]
  static DecodedPayload? _decodeProposeSweep(Uint8List bytes) {
    // call_data: 2 + 32 + 16 = 50
    if (bytes.length < 50 || !_hasValidSigningTail(bytes, 50)) return null;
    var offset = 2;
    final institutionBytes = bytes.sublist(offset, offset + 32);
    offset += 32;
    final institutionLabel = _institutionAccountLabel(null, institutionBytes);
    final amountFen = _readU128Le(bytes, offset);
    final amountYuan = _fenToYuan(amountFen);
    return DecodedPayload(
      action: 'propose_sweep_to_main',
      summary: '手续费划转 $amountYuan GMB：$institutionLabel',
      fields: {
        'institution': institutionLabel,
        'amount_yuan': '$amountYuan GMB',
      },
    );
  }

  // ResolutionDestroy(14) / propose_destroy(0)
  // 格式：[14][0][institution_code:[u8;4]][institution:AccountId32][amount:u128]
  static DecodedPayload? _decodeProposeDestroy(Uint8List bytes) {
    // call_data: 2 + 4 + 32 + 16 = 54
    if (bytes.length < 54 || !_hasValidSigningTail(bytes, 54)) return null;
    var offset = 2;
    final code =
        InstitutionCode.codeToString(bytes.sublist(offset, offset + 4));
    offset += 4;
    final institutionBytes = bytes.sublist(offset, offset + 32);
    offset += 32;
    final institutionLabel = _institutionAccountLabel(code, institutionBytes);
    final amountFen = _readU128Le(bytes, offset);
    final amountYuan = _fenToYuan(amountFen);
    return DecodedPayload(
      action: 'propose_destroy',
      summary:
          '${InstitutionCode.codeLabel(code)} 决议销毁 $amountYuan GMB：$institutionLabel',
      fields: {
        'institution_code': InstitutionCode.codeLabel(code),
        'institution': institutionLabel,
        'amount_yuan': '$amountYuan GMB',
      },
    );
  }

  // PersonalAdmins(7.3) / PublicAdmins(29.0) / PrivateAdmins(30.0)
  // 格式：[pallet][call][institution_code:[u8;4]][account:AccountId32][Compact<N>][admins:N*32][new_threshold:u32_le]
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

    final (adminsLen, countSize) = _decodeCompactU32(bytes, offset);
    if (countSize == 0 || adminsLen < 1) return null;
    offset += countSize;

    final admins = <String>[];
    final adminAddresses = <String>[];
    for (var i = 0; i < adminsLen; i++) {
      if (offset + 32 > bytes.length) return null;
      final admin = bytes.sublist(offset, offset + 32);
      admins.add(_bytesToLowerHex(admin));
      adminAddresses.add(_bytesToSs58(admin));
      offset += 32;
    }
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
        'admins': admins.join(','),
      },
      reviewFields: {
        'institution_code': institutionLabel,
        'account': _bytesToSs58(accountBytes),
        'admins': adminAddresses.join(','),
        'new_threshold': thresholdLabel,
      },
    );
  }

  // GrandpaKeyChange(16) / propose_key_change(0)
  // 格式：[16][0][institution:AccountId32][new_key:32]
  static DecodedPayload? _decodeProposeKeyChange(Uint8List bytes) {
    // call_data: 2 + 32 + 32 = 66
    if (bytes.length < 66 || !_hasValidSigningTail(bytes, 66)) return null;
    var offset = 2;
    final institutionBytes = bytes.sublist(offset, offset + 32);
    offset += 32;
    final institutionLabel = _institutionAccountLabel(null, institutionBytes);
    final keyBytes = bytes.sublist(offset, offset + 32);
    final keyHex =
        keyBytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
    return DecodedPayload(
      action: 'propose_replace_grandpa_key',
      summary: 'GRANDPA 密钥替换提案',
      fields: {
        'institution': institutionLabel,
        'new_key': '0x$keyHex',
      },
    );
  }

  // LegislationYuan(27) 立法全文章节(章>节>条>款)SCALE 跳读 + 摘要统计。
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

  /// 解码 runtime `citizen_identity::PopulationScope`。
  static (Map<String, String>?, int) _decodePopulationScope(
    Uint8List bytes,
    int offset,
  ) {
    if (offset >= bytes.length) return (null, -1);
    final tag = bytes[offset];
    offset += 1;

    final fields = <String, String>{};
    switch (tag) {
      case 0:
        fields['scope_level'] = 'COUNTRY';
        fields['scope_text'] = '全国';
        return (fields, offset);
      case 1:
        final (province, next) = _readUtf8Vec(bytes, offset);
        if (province == null) return (null, -1);
        fields['scope_level'] = 'PROVINCE';
        fields['scope_province_code'] = province;
        fields['scope_text'] = '省 $province';
        return (fields, next);
      case 2:
        final (province, afterProvince) = _readUtf8Vec(bytes, offset);
        if (province == null) return (null, -1);
        final (city, next) = _readUtf8Vec(bytes, afterProvince);
        if (city == null) return (null, -1);
        fields['scope_level'] = 'CITY';
        fields['scope_province_code'] = province;
        fields['scope_city_code'] = city;
        fields['scope_text'] = '市 $province/$city';
        return (fields, next);
      case 3:
        final (province, afterProvince) = _readUtf8Vec(bytes, offset);
        if (province == null) return (null, -1);
        final (city, afterCity) = _readUtf8Vec(bytes, afterProvince);
        if (city == null) return (null, -1);
        final (town, next) = _readUtf8Vec(bytes, afterCity);
        if (town == null) return (null, -1);
        fields['scope_level'] = 'TOWN';
        fields['scope_province_code'] = province;
        fields['scope_city_code'] = city;
        fields['scope_town_code'] = town;
        fields['scope_text'] = '镇 $province/$city/$town';
        return (fields, next);
      default:
        return (null, -1);
    }
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

  /// 院序列 HousesOf = Compact(count) + count×(InstitutionCode[u8;4] + AccountId32)。
  /// 返回 (新 offset, 机构码标签列表);失败 offset = -1。
  static (int, List<String>) _scanHouses(Uint8List bytes, int offset) {
    if (offset >= bytes.length) return (-1, const []);
    final (count, countSize) = _decodeCompactU32(bytes, offset);
    if (countSize == 0) return (-1, const []);
    offset += countSize;
    final labels = <String>[];
    for (var i = 0; i < count; i++) {
      if (offset + 4 + 32 > bytes.length) return (-1, const []);
      final code =
          InstitutionCode.codeToString(bytes.sublist(offset, offset + 4));
      labels.add(InstitutionCode.codeLabel(code));
      offset += 4 + 32; // 机构码 + 机构账户
    }
    return (offset, labels);
  }

  /// (InstitutionCode[u8;4], AccountId32) 平铺 36 字节 → 机构码标签。
  /// 返回 (新 offset, 标签);失败 offset = -1。
  static (int, String) _scanBody(Uint8List bytes, int offset) {
    if (offset + 36 > bytes.length) return (-1, '');
    final code =
        InstitutionCode.codeToString(bytes.sublist(offset, offset + 4));
    return (offset + 36, InstitutionCode.codeLabel(code));
  }

  /// Option<(InstitutionCode, AccountId32)>:1 tag 字节 + Some 时 36 字节。
  /// 返回 (新 offset, 标签或 null);失败 offset = -1。
  static (int, String?) _scanOptionBody(Uint8List bytes, int offset) {
    if (offset >= bytes.length) return (-1, null);
    final tag = bytes[offset];
    offset += 1;
    if (tag == 0) return (offset, null);
    if (tag != 1) return (-1, null);
    final (next, label) = _scanBody(bytes, offset);
    if (next < 0) return (-1, null);
    return (next, label);
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

  // LegislationYuan(27) / propose_enact_law(0)
  // SCALE: [27][0][tier:u8][scope_code:u32_le][houses][proposer_body:36]
  //        [executive:36][legislature:Option<36>][vote_type:u8]
  //        [title:BoundedVec][title_en:Option<BoundedVec>][chapters][effective_at:u32_le]
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
    final (afterHouses, houseLabels) = _scanHouses(bytes, offset);
    if (afterHouses < 0) return null;
    offset = afterHouses;

    // proposer_body / executive / legislature
    final (afterProposer, _) = _scanBody(bytes, offset);
    if (afterProposer < 0) return null;
    offset = afterProposer;
    final (afterExecutive, _) = _scanBody(bytes, offset);
    if (afterExecutive < 0) return null;
    offset = afterExecutive;
    final (afterLegislature, _) = _scanOptionBody(bytes, offset);
    if (afterLegislature < 0) return null;
    offset = afterLegislature;

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

    // effective_at: u32 LE
    if (offset + 4 > bytes.length) return null;
    final effectiveAt = _readU32Le(bytes, offset);
    offset += 4;

    if (!_hasValidSigningTail(bytes, offset)) return null;

    return DecodedPayload(
      action: 'propose_enact_law',
      summary:
          '发起立法「$title」（$tierLabel·$voteTypeLabel，$chapterCount 章 $articleTotal 条，第 $effectiveAt 块生效）',
      fields: {
        'title': title,
        'tier': tierLabel,
        'vote_type': voteTypeLabel,
        'scope_code': scopeCode.toString(),
        'houses': houseLabels.join('、'),
        'chapter_count': chapterCount.toString(),
        'article_count': articleTotal.toString(),
        'effective_at': effectiveAt.toString(),
      },
    );
  }

  // LegislationYuan(27) / propose_amend_law(1)
  // SCALE: [27][1][law_id:u64_le][proposer_body:36][executive:36]
  //        [legislature:Option<36>][vote_type:u8][title:BoundedVec]
  //        [title_en:Option<BoundedVec>][chapters][effective_at:u32_le]
  static DecodedPayload? _decodeProposeAmendLaw(Uint8List bytes) {
    if (bytes.length < 10) return null;
    var offset = 2;

    final lawId = _readU64Le(bytes, offset);
    offset += 8;

    final (afterProposer, _) = _scanBody(bytes, offset);
    if (afterProposer < 0) return null;
    offset = afterProposer;
    final (afterExecutive, _) = _scanBody(bytes, offset);
    if (afterExecutive < 0) return null;
    offset = afterExecutive;
    final (afterLegislature, _) = _scanOptionBody(bytes, offset);
    if (afterLegislature < 0) return null;
    offset = afterLegislature;

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

    if (offset + 4 > bytes.length) return null;
    final effectiveAt = _readU32Le(bytes, offset);
    offset += 4;

    if (!_hasValidSigningTail(bytes, offset)) return null;

    return DecodedPayload(
      action: 'propose_amend_law',
      summary:
          '发起修法「$title」（法律 #$lawId·$voteTypeLabel，$chapterCount 章 $articleTotal 条，第 $effectiveAt 块生效）',
      fields: {
        'law_id': lawId.toString(),
        'title': title,
        'vote_type': voteTypeLabel,
        'chapter_count': chapterCount.toString(),
        'article_count': articleTotal.toString(),
        'effective_at': effectiveAt.toString(),
      },
    );
  }

  // LegislationYuan(27) / propose_repeal_law(2)
  // SCALE: [27][2][law_id:u64_le][proposer_body:36][executive:36]
  //        [legislature:Option<36>][vote_type:u8]
  static DecodedPayload? _decodeProposeRepealLaw(Uint8List bytes) {
    if (bytes.length < 10) return null;
    var offset = 2;

    final lawId = _readU64Le(bytes, offset);
    offset += 8;

    final (afterProposer, _) = _scanBody(bytes, offset);
    if (afterProposer < 0) return null;
    offset = afterProposer;
    final (afterExecutive, _) = _scanBody(bytes, offset);
    if (afterExecutive < 0) return null;
    offset = afterExecutive;
    final (afterLegislature, _) = _scanOptionBody(bytes, offset);
    if (afterLegislature < 0) return null;
    offset = afterLegislature;

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
        'vote_type': voteTypeLabel,
      },
    );
  }

  // LegislationVote(28) 通用:proposal_id:u64_le + approve:bool。
  // 院内表决(1)/行政签署(3)/三人会签(4)/护宪终审(5) 同形。
  // SCALE: [28][call][proposal_id:u64_le][approve:bool]
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

  // LegislationVote(28) / prepare_population_snapshot(0)
  // SCALE: [28][0][scope:PopulationScope]
  static DecodedPayload? _decodePrepareLegislationSnapshot(Uint8List bytes) {
    final (scopeFields, offset) = _decodePopulationScope(bytes, 2);
    if (scopeFields == null || !_hasValidSigningTail(bytes, offset)) {
      return null;
    }

    return DecodedPayload(
      action: 'prepare_legislation_snapshot',
      summary: '准备立法人口快照（${scopeFields['scope_text']}）',
      fields: scopeFields,
    );
  }

  // LegislationVote(28) / cast_referendum_vote(2)
  // SCALE: [28][2][proposal_id:u64_le][approve:bool]
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
  // SCALE: [10][0][registrar_account:AccountId32][VotingIdentityPayload]
  //        [Vec<u8> citizen_signature]
  static DecodedPayload? _decodeRegisterVotingIdentity(Uint8List bytes) {
    if (bytes.length < 2 + 32) return null;
    var offset = 2;
    final registrar = bytes.sublist(offset, offset + 32);
    offset += 32;
    final payload = _readVotingIdentityPayload(bytes, offset);
    if (payload == null) return null;
    offset = payload.next;

    final (signatureLen, signatureLenSize) = _decodeCompactU32(bytes, offset);
    if (signatureLenSize == 0 || signatureLen != 64) return null;
    offset += signatureLenSize;
    if (offset + signatureLen > bytes.length) return null;
    offset += signatureLen;
    if (!_hasCallDataEnd(bytes, offset)) return null;

    final registrarAddress = _bytesToSs58(registrar);
    return DecodedPayload(
      action: 'register_voting_identity',
      summary: '注册公民链上身份：${payload.cidNumber}',
      fields: <String, String>{
        'registrar_account': registrarAddress,
        ...payload.fields,
        'citizen_signature_len': signatureLen.toString(),
      },
      reviewFields: <String, String>{
        'registrar_account': registrarAddress,
        ...payload.reviewFields,
      },
    );
  }

  // CitizenIdentity(10) / upgrade_to_candidate_identity(1)
  // SCALE: [10][1][registrar_account:AccountId32][CandidateIdentityPayload]
  //        [Vec<u8> citizen_signature]
  static DecodedPayload? _decodeUpgradeToCandidateIdentity(Uint8List bytes) {
    if (bytes.length < 2 + 32) return null;
    var offset = 2;
    final registrar = bytes.sublist(offset, offset + 32);
    offset += 32;
    final payload = _readCandidateIdentityPayload(bytes, offset);
    if (payload == null) return null;
    offset = payload.next;

    final (signatureLen, signatureLenSize) = _decodeCompactU32(bytes, offset);
    if (signatureLenSize == 0 || signatureLen != 64) return null;
    offset += signatureLenSize;
    if (offset + signatureLen > bytes.length) return null;
    offset += signatureLen;
    if (!_hasCallDataEnd(bytes, offset)) return null;

    final registrarAddress = _bytesToSs58(registrar);
    return DecodedPayload(
      action: 'upgrade_to_candidate_identity',
      summary: '注册公民参选身份：${payload.cidNumber}',
      fields: <String, String>{
        'registrar_account': registrarAddress,
        ...payload.fields,
        'citizen_signature_len': signatureLen.toString(),
      },
      reviewFields: <String, String>{
        'registrar_account': registrarAddress,
        ...payload.reviewFields,
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
      },
      reviewFields: <String, String>{
        'identity_level': '参选身份',
        ...voting.reviewFields,
        'birth_place': birthPlace,
        'citizen_full_name': citizenFullName,
        'citizen_sex': sexLabel,
      },
    );
  }

  // 通用:只取 proposal_id: u64_le 的兜底执行/取消/清理类 call。
  //
  // 链端若干 `execute_X` / `cancel_failed_X` /
  // `cleanup_rejected_X` 签名完全一致:
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

  // OffchainTransaction(21) / bind_clearing_bank(30), switch_bank(33)
  // 格式：[21][call][AccountId32]
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

  // OffchainTransaction(21) / deposit(31), withdraw(32)
  // 格式：[21][call][Compact<u128> amount]
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

  // OffchainTransaction(21) / register_clearing_bank(50)
  // 格式：[21][50][Vec cid_number][Vec peer_id][Vec rpc_domain][u16 rpc_port]
  static DecodedPayload? _decodeRegisterClearingBank(Uint8List bytes) {
    var offset = 2;
    final (cidNumber, cidNext) = _readUtf8Vec(bytes, offset);
    if (cidNumber == null) return null;
    offset = cidNext;
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
      summary: '声明清算行节点 $cidNumber @ $rpcDomain:$rpcPort',
      fields: {
        'cid_number': cidNumber,
        'peer_id': peerId,
        'rpc_domain': rpcDomain,
        'rpc_port': rpcPort.toString(),
      },
    );
  }

  // OffchainTransaction(21) / update_clearing_bank_endpoint(51)
  // 格式：[21][51][Vec cid_number][Vec new_domain][u16 new_port]
  static DecodedPayload? _decodeUpdateClearingBankEndpoint(Uint8List bytes) {
    var offset = 2;
    final (cidNumber, cidNext) = _readUtf8Vec(bytes, offset);
    if (cidNumber == null) return null;
    offset = cidNext;
    final (newDomain, domainNext) = _readUtf8Vec(bytes, offset);
    if (newDomain == null) return null;
    offset = domainNext;
    if (offset + 2 > bytes.length) return null;
    final newPort = bytes[offset] | (bytes[offset + 1] << 8);
    if (!_hasValidSigningTail(bytes, offset + 2)) return null;

    return DecodedPayload(
      action: 'update_clearing_bank_endpoint',
      summary: '更新清算行 $cidNumber 端点 → $newDomain:$newPort',
      fields: {
        'cid_number': cidNumber,
        'new_domain': newDomain,
        'new_port': newPort.toString(),
      },
    );
  }

  // OffchainTransaction(21) / unregister_clearing_bank(52)
  // 格式：[21][52][Vec cid_number]
  static DecodedPayload? _decodeUnregisterClearingBank(Uint8List bytes) {
    final (cidNumber, cidEnd) = _readUtf8Vec(bytes, 2);
    if (cidNumber == null) return null;
    if (!_hasValidSigningTail(bytes, cidEnd)) return null;
    return DecodedPayload(
      action: 'unregister_clearing_bank',
      summary: '注销清算行节点 $cidNumber',
      fields: {
        'cid_number': cidNumber,
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
  /// era(0x00 immortal,P-SIGN-001) + Compact<nonce> + Compact<tip>
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
    if (tipValue == null || tipSize == 0) return false;
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

  /// 机构账户人机标签。
  ///
  /// [code] 为机构码字符串(转账/销毁分支从 4 字节 institution_code 解出);
  /// 当 wire 格式不携带机构码时(sweep / grandpa_key)传 null,统一展示为"机构账户"。
  static String _institutionAccountLabel(String? code, Uint8List accountBytes) {
    if (code == null) {
      return '机构账户 ${_bytesToSs58(accountBytes)}';
    }
    if (InstitutionCode.isFixedGovernance(code)) {
      return InstitutionCode.codeLabel(code);
    }
    if (InstitutionCode.isPersonal(code)) {
      return '个人多签 ${_bytesToSs58(accountBytes)}';
    }
    return '机构账户 ${_bytesToSs58(accountBytes)}';
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

  /// 管理员集合变更阈值合法性。
  ///
  /// 固定治理档机构码走制度常量；注册多签账户(个人多签/机构账户)走动态严格过半。
  static bool _validAdminChangeThreshold(
    String code,
    int adminsLen,
    int threshold,
  ) {
    if (code == 'NRC') {
      return adminsLen == 19 && threshold == 13;
    }
    if (code == 'PRC' || code == 'PRB') {
      return adminsLen == 9 && threshold == 6;
    }
    if (code == 'FRG') {
      return adminsLen == 5 && threshold == 3;
    }
    if (code == 'NJD') {
      return adminsLen == 15 && threshold == 8;
    }
    if (InstitutionCode.isRegisteredMultisig(code)) {
      return adminsLen >= 2 &&
          threshold > adminsLen ~/ 2 &&
          threshold <= adminsLen;
    }
    return false;
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
    if (palletIndex == PalletRegistry.publicAdminsPallet) {
      return callIndex == PalletRegistry.proposeAdminSetChangeCall &&
          (InstitutionCode.isPublicLegal(code) ||
              InstitutionCode.isFixedGovernance(code));
    }
    if (palletIndex == PalletRegistry.privateAdminsPallet) {
      return callIndex == PalletRegistry.proposeAdminSetChangeCall &&
          (InstitutionCode.isPrivateLegal(code) ||
              InstitutionCode.isUnincorporated(code));
    }
    return false;
  }

  static String _adminSetChangeActionForPallet(int palletIndex) {
    return switch (palletIndex) {
      PalletRegistry.personalAdminsPallet =>
        'propose_personal_admin_set_change',
      PalletRegistry.publicAdminsPallet => 'propose_public_admin_set_change',
      PalletRegistry.privateAdminsPallet => 'propose_private_admin_set_change',
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

  /// 激活凭证里的账户 kind 与机构码是否匹配。
  ///
  /// kind 语义对齐链端 admin-primitives::AdminAccountKind(SCALE 判别值):
  ///   0 = PublicInstitution
  ///   1 = PrivateInstitution
  ///   2 = PersonalMultisig
  static bool _activationAccountKindMatchesCode(String code, int kind) {
    if (InstitutionCode.isPersonal(code)) {
      return kind == 2;
    }
    if (InstitutionCode.isPublicLegal(code) ||
        InstitutionCode.isFixedGovernance(code)) {
      return kind == 0;
    }
    if (InstitutionCode.isPrivateLegal(code) ||
        InstitutionCode.isUnincorporated(code)) {
      return kind == 1;
    }
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
