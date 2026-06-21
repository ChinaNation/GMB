import 'dart:convert';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import '../chain/chain_constants.dart';
import '../chain/reserved_account_names.dart';
import 'pallet_registry.dart';

/// payload_hex 中 call data 的解码结果。
///
/// 离线设备用此结果向用户展示交易详情，
/// 并与 sign_request 中的 display 字段交叉比对。
class DecodedPayload {
  const DecodedPayload({
    required this.action,
    required this.summary,
    required this.fields,
    Map<String, String>? reviewFields,
  }) : reviewFields = reviewFields ?? fields;

  /// 动作标识，与 display.action 一致。
  final String action;

  /// 一句话摘要。
  final String summary;

  /// 结构化字段，用于与 display.fields 逐一比对。
  final Map<String, String> fields;

  /// 用户确认页展示字段。
  ///
  /// 中文注释：`fields` 保留独立验真的机器字段，`reviewFields` 只放人能判断的
  /// 中文业务信息和 SS58 地址，避免把 payload_hash、内部 ID、原始公钥 hex 暴露为确认内容。
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
  static final _activateAdminPrefix = Uint8List.fromList(
    'GMB_ACTIVATE_ADMIN_V1'.codeUnits,
  );

  /// "GMB_DECRYPT_V1" 前缀（14 字节 ASCII）。
  ///
  /// 这是 `WUMIN_QR_V1` 内部 payload 的业务签名域，不是二维码协议版本。
  static const _decryptPrefix = [
    0x47, 0x4D, 0x42, 0x5F, // GMB_
    0x44, 0x45, 0x43, 0x52, // DECR
    0x59, 0x50, 0x54, 0x5F, // YPT_
    0x56, 0x31, // V1
  ];
  static final _cpmsArchiveDeletePrefix = Uint8List.fromList(
    'CPMS_ARCHIVE_DELETE_V1|'.codeUnits,
  );
  static const String _sfidAdminActionDomain = 'sfid_admin_governance';

  static DecodedPayload? decode(String payloadHex) {
    // 先尝试解码非链上交易：管理员激活 / 清算行管理员解密 challenge。
    try {
      final raw = _hexToBytes(payloadHex);
      final adminAction = _decodeSfidAdminAction(raw);
      if (adminAction != null) {
        return adminAction;
      }
      if (raw.length ==
              _activateAdminPrefix.length + 32 + 1 + 1 + 32 + 8 + 16 &&
          _hasPrefix(raw, _activateAdminPrefix)) {
        return _decodeActivateAdminAccount(raw);
      }
      if (_hasPrefix(raw, _cpmsArchiveDeletePrefix)) {
        return _decodeCpmsArchiveDelete(raw);
      }
      if (raw.length == 118) {
        bool isDecrypt = true;
        for (var i = 0; i < _decryptPrefix.length; i++) {
          if (raw[i] != _decryptPrefix[i]) {
            isDecrypt = false;
            break;
          }
        }
        if (isDecrypt) {
          return _decodeDecryptAdmin(raw);
        }
      }
    } catch (_) {
      // 非 challenge payload，继续正常解码。
    }

    // 防误签由 strict 两色模式独家把关:
    // - decoder 解析失败(任何分支不匹配返回 null) → decodeFailed → 禁止签名
    // - 解析成功但 display.action != decoded.action → mismatched → 禁止签名
    // 不再额外按 spec_version 锁布局,合法新 spec 可直接解码,布局变了 strict 模式自动拦截。
    try {
      final bytes = _hexToBytes(payloadHex);
      if (bytes.length < 2) return null;

      final palletIndex = bytes[0];
      final callIndex = bytes[1];

      // Balances / transfer_keep_alive
      if (palletIndex == PalletRegistry.balancesPallet &&
          callIndex == PalletRegistry.transferKeepAliveCall) {
        return _decodeTransferKeepAlive(bytes);
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

      // ── DuoqianTransfer(19) ──
      // 投票入口统一到 InternalVote::cast(22.0),
      // 手动重试入口统一到 VotingEngine::retry_passed_proposal(9.4),
      // 本 pallet 仅保留 3 条 propose_X。
      if (palletIndex == PalletRegistry.duoqianTransferPallet) {
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
      // SCALE 解析。改走 OfflineSignService.verifyPayload 的"哈希直签例外"
      // 路径:用户在冷钱包屏幕上肉眼核对 display.fields.wasm_hash 即放行。

      // ── OrganizationManage(17) ──
      // 投票入口统一到 InternalVote::cast(22.0)。本 pallet 承载
      // propose_X + cleanup_rejected_proposal(被拒提案残留清理)。
      // register_sfid_institution(call=2) 当前不作为冷钱包 action 暴露;
      // SFID 机构注册凭证等待签发机构管理员业务签名流程接入后再恢复。
      // propose_create_institution(call=5) 由 wuminapp 在线端构造、走冷钱包扫码签名;
      // 凭证尾部带签发机构、签发管理员和业务作用域字段。
      if (palletIndex == PalletRegistry.organizationManagePallet) {
        // call_index=0 留洞不复用(机构多签最少 2 账户,统一走 call_index=5)。
        // call_index=3 留洞不复用(propose_create_personal 已迁至 PersonalManage(7),B 阶段拆分 2026-05-06)。
        if (callIndex == PalletRegistry.proposeCloseCall) {
          return _decodeProposeClose(
            bytes,
            action: 'propose_close_institution',
            summaryLabel: '机构多签',
          );
        }
        if (callIndex == PalletRegistry.proposeCreateInstitutionCall) {
          return _decodeProposeCreateInstitution(bytes);
        }
        if (callIndex == PalletRegistry.cleanupRejectedProposalCall) {
          return _decodeProposalIdOnly(
            bytes,
            action: 'cleanup_rejected_proposal',
            summaryTemplate: '清理被拒机构提案 #{id} 残留',
          );
        }
      }

      // ── PersonalManage(7) ──
      // B 阶段拆分(2026-05-06):个人多签独立 pallet,MODULE_TAG = b"per-mgmt"。
      // ACTION enum 独立(ACTION_CREATE=0/ACTION_CLOSE=1),与 organization-manage 互不干扰。
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
        if (callIndex == PalletRegistry.cleanupRejectedPersonalProposalCall) {
          return _decodeProposalIdOnly(
            bytes,
            action: 'cleanup_rejected_personal_proposal',
            summaryTemplate: '清理被拒个人多签提案 #{id} 残留',
          );
        }
      }

      // ── ResolutionIssuance(8) · 决议发行联合提案 ──
      // 人口快照由 JointVote.prepare_joint_population_snapshot 单独准备。
      if (palletIndex == PalletRegistry.resolutionIssuancePallet) {
        if (callIndex == PalletRegistry.proposeResolutionIssuanceCall) {
          return _decodeProposeResolutionIssuance(bytes);
        }
      }

      // ── ResolutionDestro(14) ──
      // execute_destroy 走 VotingEngine::retry_passed_proposal。
      if (palletIndex == PalletRegistry.resolutionDestroPallet) {
        if (callIndex == PalletRegistry.proposeDestroyCall) {
          return _decodeProposeDestroy(bytes);
        }
      }

      // ── AdminsChange(12) ──
      // 管理员集合变更走 propose_admin_set_change；执行统一由 VotingEngine 重试。
      if (palletIndex == PalletRegistry.adminsChangePallet) {
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

      // ── OnchainIssuance(25) · 链上发行代币(Plain FT, ADR-011 v3) ──
      // 当前未实现完整 SCALE 字段解析,不能独立验证业务内容,因此红色拒签。
      if (palletIndex == PalletRegistry.onchainIssuancePallet) {
        return null;
      }

      return null;
    } catch (_) {
      return null;
    }
  }

  static DecodedPayload? _decodeSfidAdminAction(Uint8List raw) {
    try {
      final text = utf8.decode(raw);
      final value = jsonDecode(text);
      if (value is! Map<String, dynamic>) return null;
      if (value['domain'] != _sfidAdminActionDomain) return null;
      if (value['qr_proto'] != 'WUMIN_QR_V1') return null;
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
      final payloadHash = '0x${sha256.convert(raw).toString()}';
      final actorAddress = _pubkeyHexToSs58OrRaw(actorPubkey);
      final targetAddress = _pubkeyHexToSs58OrRaw(target);
      return DecodedPayload(
        action: 'sfid_admin_action',
        summary: 'SFID 管理员治理',
        fields: <String, String>{
          'action_type': _sfidAdminActionLabel(actionType),
          'actor_province_name': province,
          'actor_pubkey': actorAddress,
          'target': targetAddress,
          'before_hash': beforeHash,
          'after_hash': afterHash,
          'payload_hash': payloadHash,
        },
        reviewFields: <String, String>{
          'action_type': _sfidAdminActionLabel(actionType),
          'actor_province_name': province,
          'actor_pubkey': actorAddress,
          'target': targetAddress,
        },
      );
    } catch (_) {
      return null;
    }
  }

  static String _sfidAdminActionLabel(String actionType) {
    switch (actionType) {
      case 'PASSKEY_REGISTER':
        return '更新 Passkey';
      case 'CREATE_CITY_ADMIN':
        return '新增市管理员';
      case 'UPDATE_CITY_ADMIN':
        return '编辑市管理员';
      case 'DELETE_CITY_ADMIN':
        return '删除市管理员';
      case 'CREATE_FEDERAL_ADMIN':
        return '新增联邦管理员';
      case 'UPDATE_FEDERAL_ADMIN':
        return '编辑联邦管理员';
      case 'DELETE_FEDERAL_ADMIN':
        return '删除联邦管理员';
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
      case 'PUBLIC_SECURITY_RECONCILE':
        return '公安局机构对账';
      case 'CITIZEN_BIND_COMMIT':
        return '确认电子护照绑定';
      case 'CPMS_STATUS_IMPORT_CONFIRM':
        return '导入 CPMS 年度报告';
      case 'CPMS_ISSUE_INSTALL_CODE':
        return '签发 CPMS 安装码';
      case 'CPMS_REVOKE_INSTALL_TOKEN':
        return '作废 CPMS 安装令牌';
      case 'CPMS_REISSUE_INSTALL_TOKEN':
        return '重新签发 CPMS 安装码';
      case 'CPMS_DISABLE_KEYS':
        return '禁用 CPMS 授权';
      case 'CPMS_ENABLE_KEYS':
        return '启用 CPMS 授权';
      case 'CPMS_REVOKE_KEYS':
        return '吊销 CPMS 授权';
      case 'CPMS_DELETE_KEYS':
        return '删除 CPMS 授权';
      default:
        return actionType;
    }
  }

  // ---------------------------------------------------------------------------
  // Balances(2) / transfer_keep_alive(3)
  // 格式：[0x02][0x03][MultiAddress::Id = 0x00 + 32 bytes][Compact amount]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeTransferKeepAlive(Uint8List bytes) {
    // 2 (pallet+call) + 1 (MultiAddress prefix) + 32 (AccountId) + 至少 1 (Compact)
    if (bytes.length < 36) return null;

    var offset = 2;

    // MultiAddress::Id 前缀 0x00
    if (bytes[offset] != 0x00) return null;
    offset += 1;

    // 收款地址 32 bytes
    final toAccountId = bytes.sublist(offset, offset + 32);
    offset += 32;
    final toAddress =
        Keyring().encodeAddress(toAccountId.toList(), _ss58Prefix);

    // Compact<u128> 金额（分）
    final (amountFen, amountSize) = _decodeCompactBigInt(bytes, offset);
    if (amountFen == null || amountSize == 0) return null;
    if (!_hasValidSigningTail(bytes, offset + amountSize)) return null;

    final amountYuan = _fenToYuan(amountFen);

    return DecodedPayload(
      action: 'transfer',
      summary: '转账 $amountYuan GMB 给 ${_truncateAddress(toAddress)}',
      fields: {
        'to': toAddress,
        'amount_yuan': '$amountYuan GMB',
      },
    );
  }

  // ---------------------------------------------------------------------------
  // DuoqianTransfer(19) / propose_transfer(0)
  // 格式：[0x13][0x00][org:u8][institution:AccountId32][beneficiary:32][amount:u128_le][Vec remark]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeProposeTransfer(Uint8List bytes) {
    // 最小长度：2 + 1 + 32 + 32 + 16 + 1 = 84
    if (bytes.length < 84) return null;

    var offset = 2;

    // org: u8 用于展示机构类型；实际治理账户是 32 字节 AccountId。
    final org = bytes[offset];
    offset += 1;

    final institutionBytes = bytes.sublist(offset, offset + 32);
    offset += 32;
    final institutionLabel = _institutionAccountLabel(org, institutionBytes);

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

  // ---------------------------------------------------------------------------
  // InternalVote(22) / cast(0)
  // 格式：[0x16][0x00][proposal_id:u64_le][approve:bool]
  //
  // 统一入口:所有业务 pallet(admins/resolution_destro/grandpa_key/
  // organization_manage/duoqian_transfer 五路)的管理员投票都走 InternalVote::cast(22.0),
  // 冷钱包不按业务 pallet 分路解码投票 payload。
  // ---------------------------------------------------------------------------
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

  // ---------------------------------------------------------------------------
  // VotingEngine(9) / finalize_proposal(3)
  // 格式：[0x09][0x03][proposal_id:u64_le]
  //
  // 任意账户触发终态执行,无需签投票语义。引擎核心 lifecycle extrinsic。
  // ---------------------------------------------------------------------------
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

  // ---------------------------------------------------------------------------
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
  // ---------------------------------------------------------------------------
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

  // ---------------------------------------------------------------------------
  // JointVote(23) / cast_admin(0)
  // 格式：[0x17][0x00][proposal_id:u64_le][institution:AccountId32][approve:bool]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeJointVote(Uint8List bytes) {
    // call_data: 2 + 8 + 32 + 1 = 43
    if (bytes.length < 43 || !_hasValidSigningTail(bytes, 43)) return null;

    final proposalId = _readU64Le(bytes, 2);
    // institution AccountId32 跳过（不在 display 中展示细节）
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

  // ---------------------------------------------------------------------------
  // JointVote(23) / cast_referendum(1)
  //
  // 签发机构 admins 凭证。
  // 格式：[0x17][0x01][proposal_id:u64_le][binding_id:32]
  //       [Vec nonce][Vec sig][Vec issuer_sfid_number][issuer_main_account:32]
  //       [[u8;32] signer_pubkey][Vec scope_province_name][Vec scope_city_name][approve:bool]
  //
  // 签发身份必须进 payload,链端 RuntimeSfidVoteVerifier 按 issuer_main_account
  // 读取 admins-change::AdminAccounts.admins 确认 signer_pubkey。
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeCastReferendum(Uint8List bytes) {
    // 最小：2 + 8 + 32 + 1(nonce) + 1(sig) + 1(issuer)
    //      + 32(issuer account) + 32(signer) + 1(scope province)
    //      + 1(scope city) + 1(approve) = 112
    if (bytes.length < 112) return null;

    final proposalId = _readU64Le(bytes, 2);

    // 逐段解析变长字段，精确定位 approve 偏移量。
    var offset = 2 + 8 + 32; // 跳过 pallet+call, proposal_id, binding_id

    // Vec<u8> nonce — SCALE Compact 前缀 + nonce 字节
    final (nonceLen, nonceLenSize) = _decodeCompactU32(bytes, offset);
    offset += nonceLenSize;
    if (offset + nonceLen > bytes.length) return null;
    offset += nonceLen;

    // Vec<u8> sig — SCALE Compact 前缀 + sig 字节
    final (sigLen, sigLenSize) = _decodeCompactU32(bytes, offset);
    offset += sigLenSize;
    if (offset + sigLen > bytes.length) return null;
    offset += sigLen;

    // Vec<u8> issuer_sfid_number
    final (issuerLen, issuerLenSize) = _decodeCompactU32(bytes, offset);
    offset += issuerLenSize;
    if (offset + issuerLen > bytes.length) return null;
    final issuerSfidNumber = utf8.decode(
      bytes.sublist(offset, offset + issuerLen),
      allowMalformed: true,
    );
    offset += issuerLen;

    // AccountId issuer_main_account
    if (offset + 32 > bytes.length) return null;
    final issuerMainAccount = bytes.sublist(offset, offset + 32);
    offset += 32;

    // [u8; 32] signer_pubkey
    if (offset + 32 > bytes.length) return null;
    final signerPubkey = bytes.sublist(offset, offset + 32);
    offset += 32;

    // Vec<u8> scope_province_name
    final (scopeProvinceLen, scopeProvinceLenSize) =
        _decodeCompactU32(bytes, offset);
    offset += scopeProvinceLenSize;
    if (offset + scopeProvinceLen > bytes.length) return null;
    final scopeProvinceName = utf8.decode(
      bytes.sublist(offset, offset + scopeProvinceLen),
      allowMalformed: true,
    );
    offset += scopeProvinceLen;

    // Vec<u8> scope_city_name
    final (scopeCityLen, scopeCityLenSize) = _decodeCompactU32(bytes, offset);
    offset += scopeCityLenSize;
    if (offset + scopeCityLen > bytes.length) return null;
    final scopeCityName = utf8.decode(
      bytes.sublist(offset, offset + scopeCityLen),
      allowMalformed: true,
    );
    offset += scopeCityLen;

    // approve: bool（1 字节）
    if (offset >= bytes.length) return null;
    final approve = bytes[offset] != 0;
    if (!_hasValidSigningTail(bytes, offset + 1)) return null;
    final voteText = approve ? '赞成' : '反对';

    return DecodedPayload(
      action: 'cast_referendum',
      summary: '公民投票 提案 #$proposalId：$voteText',
      fields: {
        'proposal_id': proposalId.toString(),
        'approve': approve.toString(),
        'issuer_sfid_number': issuerSfidNumber,
        'issuer_main_account': _bytesToSs58(issuerMainAccount),
        'signer_pubkey': _bytesToSs58(signerPubkey),
        'scope_province_name': scopeProvinceName,
        'scope_city_name': scopeCityName,
      },
    );
  }

  // ---------------------------------------------------------------------------
  // 协议升级 RuntimeUpgrade(13) / propose_runtime_upgrade(0) / developer_direct_upgrade(2)
  //
  // 已删除原 SCALE decoder:
  //   - call_data 含完整 WASM(600KB+),物理上塞不进 QR
  //   - server signing.rs 对这两个 action 在 QR 里只放 blake2_256(payload)
  //     = 32 字节哈希,decoder 永远拿不到完整 call_data
  //   - "在 decoder 里复算 sha256(wasm_bytes)"是死路径,因为 wasm_bytes 不在 QR 里
  //
  // 走 OfflineSignService.verifyPayload 的"哈希直签例外":
  //   - 收到 32 字节 payload + display.action ∈ {两个 wasm 升级 action}
  //     + display.fields 含 wasm_hash → matched
  //   - 用户在冷钱包屏幕上肉眼核对 display.fields.wasm_hash 与桌面 app 显示
  //     的 sha256 一致即放行,签的就是这份 WASM。
  // ---------------------------------------------------------------------------

  // ---------------------------------------------------------------------------
  // 管理员激活（非链上交易）
  // 格式：GMB_ACTIVATE_ADMIN_V1 + account_id(32B) + org(u8) + kind(u8)
  //      + pubkey(32B) + timestamp(8B, u64 LE) + nonce(16B)
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeActivateAdminAccount(Uint8List bytes) {
    final expectedLength =
        _activateAdminPrefix.length + 32 + 1 + 1 + 32 + 8 + 16;
    if (bytes.length != expectedLength) return null;

    var offset = _activateAdminPrefix.length;
    final accountBytes = bytes.sublist(offset, offset + 32);
    offset += 32;

    final org = bytes[offset++];
    final kind = bytes[offset++];
    if (!_activationAccountKindMatchesOrg(org, kind)) return null;

    final pubkey = bytes.sublist(offset, offset + 32);
    final accountHex = _bytesToLowerHex(accountBytes);

    return DecodedPayload(
      action: 'activate_admin_account',
      summary: '激活${_orgName(org)}管理员',
      fields: {
        'org': _orgName(org),
        'account': accountHex,
        'pubkey': _bytesToSs58(pubkey),
      },
      reviewFields: {
        'org': _orgName(org),
        'account': _bytesToSs58(accountBytes),
        'pubkey': _bytesToSs58(pubkey),
      },
    );
  }

  // ---------------------------------------------------------------------------
  // 清算行管理员解密（非链上交易）
  // 格式：GMB_DECRYPT_V1(14B) + sfid_number(48B, 右补零) + pubkey(32B)
  //      + timestamp(8B, u64 LE) + nonce(16B)
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeDecryptAdmin(Uint8List bytes) {
    if (bytes.length < 118) return null;

    final idBytes = bytes.sublist(14, 62);
    var endIndex = 48;
    while (endIndex > 0 && idBytes[endIndex - 1] == 0) {
      endIndex--;
    }
    if (endIndex == 0) return null;
    final sfidNumber = String.fromCharCodes(idBytes.sublist(0, endIndex));

    return DecodedPayload(
      action: 'decrypt_admin',
      summary: '解密清算行管理员 - $sfidNumber',
      fields: {
        'sfid_number': sfidNumber,
      },
    );
  }

  // ---------------------------------------------------------------------------
  // OrganizationManage(17) / propose_create_institution(5)
  //
  // 链端签名(签发机构 admins 凭证):
  //   pub fn propose_create_institution(
  //     origin,
  //     sfid_number: SfidNumberOf<T>,                 // BoundedVec<u8>
  //     sfid_full_name: AccountNameOf<T>,   // BoundedVec<u8>
  //     accounts: InstitutionInitialAccountsOf<T>,
  //         // BoundedVec<{ account_name: BoundedVec<u8>, amount: u128 }>
  //     org: u8,                        // ORG_PUP / ORG_OTH
  //     admins_len: u32,
  //     admins: DuoqianAdminsOf<T>,   // BoundedVec<AccountId32>
  //     threshold: u32,
  //     register_nonce: RegisterNonceOf<T>,   // BoundedVec<u8>
  //     signature: RegisterSignatureOf<T>,    // BoundedVec<u8> (64B sr25519)
  //     issuer_sfid_number: Vec<u8>,
  //     issuer_main_account: AccountId32,
  //     signer_pubkey: [u8; 32],
  //     scope_province_name: Vec<u8>,
  //     scope_city_name: Vec<u8>,
  //   )
  //
  // SCALE 顺序与上述完全一致。链端 RuntimeSfidInstitutionVerifier 按
  // issuer_main_account 的 admins 真源确认 signer_pubkey。
  // 禁止在尾部追加 subject_property/sub_type/parent_sfid_number 等多余字段。
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeProposeCreateInstitution(Uint8List bytes) {
    if (bytes.length < 10) return null;
    var offset = 2;

    // sfid_number: BoundedVec<u8>
    final (sfidLen, sfidLenSize) = _decodeCompactU32(bytes, offset);
    offset += sfidLenSize;
    if (offset + sfidLen > bytes.length) return null;
    final sfidNumber = utf8.decode(bytes.sublist(offset, offset + sfidLen),
        allowMalformed: true);
    offset += sfidLen;

    // sfid_full_name: BoundedVec<u8>
    final (nameLen, nameLenSize) = _decodeCompactU32(bytes, offset);
    offset += nameLenSize;
    if (offset + nameLen > bytes.length) return null;
    final sfidFullName = utf8.decode(bytes.sublist(offset, offset + nameLen),
        allowMalformed: true);
    offset += nameLen;

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

    // org: u8。机构账户只能使用 ORG_PUP(4) 或 ORG_OTH(5)。
    if (offset + 1 > bytes.length) return null;
    final adminOrg = bytes[offset];
    if (adminOrg != 4 && adminOrg != 5) return null;
    offset += 1;

    // admins_len: u32 (LE)
    if (offset + 4 > bytes.length) return null;
    final adminCount = bytes[offset] |
        (bytes[offset + 1] << 8) |
        (bytes[offset + 2] << 16) |
        (bytes[offset + 3] << 24);
    offset += 4;

    // admins: BoundedVec<AccountId32> — 跳过 N × 32 bytes
    final (adminsLen, adminsLenSize) = _decodeCompactU32(bytes, offset);
    offset += adminsLenSize;
    if (offset + adminsLen * 32 > bytes.length) return null;
    offset += adminsLen * 32;

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

    // issuer_sfid_number: Vec<u8>
    final (issuerLen, issuerLenSize) = _decodeCompactU32(bytes, offset);
    offset += issuerLenSize;
    if (offset + issuerLen > bytes.length) return null;
    final issuerSfidNumber = utf8.decode(
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
      'sfid_number': sfidNumber,
      'sfid_full_name': sfidFullName,
      'org': _orgName(adminOrg),
      'admins_len': adminCount.toString(),
      'threshold': '$threshold/$adminCount',
      'total_amount_yuan': '$amountYuan GMB',
    };
    for (final entry in accountAmounts.entries) {
      fields['amount_${entry.key}'] = '${_fenToYuan(entry.value)} GMB';
    }
    fields['issuer_sfid_number'] = issuerSfidNumber;
    fields['issuer_main_account'] = _bytesToSs58(issuerMainAccount);
    fields['signer_pubkey'] = _bytesToSs58(signerPubkey);
    fields['scope_province_name'] = scopeProvinceName;
    fields['scope_city_name'] = scopeCityName;

    return DecodedPayload(
      action: 'propose_create_institution',
      summary:
          '创建机构多签账户「$sfidFullName」（$adminCount 管理员，阈值 $threshold，入金 $amountYuan 元）',
      fields: fields,
    );
  }

  // ---------------------------------------------------------------------------
  // ResolutionIssuance(8) / propose_resolution_issuance(0)
  //
  // 链端签名:
  //   pub fn propose_resolution_issuance(
  //     origin,
  //     reason: ReasonOf<T>,                  // BoundedVec<u8>
  //     total_amount: BalanceOf<T>,           // u128 LE
  //     allocations: AllocationOf<T>,
  //         // BoundedVec<{ recipient: AccountId32, amount: u128 }>
  //   )
  //
  // 人口快照由 JointVote.prepare_joint_population_snapshot 先行准备,本交易只携带发行内容。
  // ---------------------------------------------------------------------------
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
      action: 'propose_resolution_issuance',
      summary: '决议发行 $amountYuan GMB（$allocLen 项分配）',
      fields: {
        'reason': reason,
        'amount_yuan': '$amountYuan GMB',
        'allocation_count': allocLen.toString(),
      },
    );
  }

  // ---------------------------------------------------------------------------
  // PersonalManage(7) / propose_create(0)
  // 格式：[7][0][BoundedVec account_name][BoundedVec<AccountId32> admins][u32 regular_threshold][u128 amount]
  // B 阶段拆分(2026-05-06):个人多签独立 pallet,MODULE_TAG = b"per-mgmt"。
  // 历史 OrganizationManage(17) call=3 已废除(留洞不复用)。
  // ---------------------------------------------------------------------------
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

  // ---------------------------------------------------------------------------
  // OrganizationManage(17) / propose_close(1)
  // PersonalManage(7) / propose_close(1)
  // 格式：[17][1][duoqian_account:32][beneficiary:32]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeProposeClose(
    Uint8List bytes, {
    required String action,
    required String summaryLabel,
  }) {
    // call_data: 2 + 32 + 32 = 66
    if (bytes.length < 66 || !_hasValidSigningTail(bytes, 66)) return null;
    final duoqianId = bytes.sublist(2, 34);
    final beneficiaryId = bytes.sublist(34, 66);
    final duoqian = Keyring().encodeAddress(duoqianId.toList(), _ss58Prefix);
    final beneficiary =
        Keyring().encodeAddress(beneficiaryId.toList(), _ss58Prefix);
    return DecodedPayload(
      action: action,
      summary: '提案关闭$summaryLabel ${_truncateAddress(duoqian)}',
      fields: {
        'duoqian_account': duoqian,
        'beneficiary': beneficiary,
      },
    );
  }

  // ---------------------------------------------------------------------------
  // DuoqianTransfer(19) / propose_safety_fund(1)
  // 格式：[19][1][beneficiary:32][amount:u128][BoundedVec remark]
  // ---------------------------------------------------------------------------
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

  // ---------------------------------------------------------------------------
  // DuoqianTransfer(19) / propose_sweep(2)
  // 格式：[19][2][institution:AccountId32][amount:u128]
  // ---------------------------------------------------------------------------
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

  // ---------------------------------------------------------------------------
  // ResolutionDestro(14) / propose_destroy(0)
  // 格式：[14][0][org:u8][institution:AccountId32][amount:u128]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeProposeDestroy(Uint8List bytes) {
    // call_data: 2 + 1 + 32 + 16 = 51
    if (bytes.length < 51 || !_hasValidSigningTail(bytes, 51)) return null;
    var offset = 2;
    final org = bytes[offset];
    offset += 1;
    final institutionBytes = bytes.sublist(offset, offset + 32);
    offset += 32;
    final institutionLabel = _institutionAccountLabel(org, institutionBytes);
    final amountFen = _readU128Le(bytes, offset);
    final amountYuan = _fenToYuan(amountFen);
    return DecodedPayload(
      action: 'propose_destroy',
      summary: '${_orgName(org)} 决议销毁 $amountYuan GMB：$institutionLabel',
      fields: {
        'org': _orgName(org),
        'institution': institutionLabel,
        'amount_yuan': '$amountYuan GMB',
      },
    );
  }

  // ---------------------------------------------------------------------------
  // AdminsChange(12) / propose_admin_set_change(0)
  // 格式：[12][0][org:u8][account:AccountId32][Compact<N>][admins:N*32][new_threshold:u32_le]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeProposeAdminSetChange(Uint8List bytes) {
    if (bytes.length < 72) return null;
    var offset = 2;
    final org = bytes[offset];
    offset += 1;
    final accountBytes = bytes.sublist(offset, offset + 32);
    final accountHex = _bytesToLowerHex(accountBytes);
    offset += 32;

    final (adminCount, countSize) = _decodeCompactU32(bytes, offset);
    if (countSize == 0 || adminCount < 1) return null;
    offset += countSize;

    final admins = <String>[];
    final adminAddresses = <String>[];
    for (var i = 0; i < adminCount; i++) {
      if (offset + 32 > bytes.length) return null;
      final admin = bytes.sublist(offset, offset + 32);
      admins.add(_bytesToLowerHex(admin));
      adminAddresses.add(_bytesToSs58(admin));
      offset += 32;
    }
    if (offset + 4 > bytes.length) return null;
    if (!_hasValidSigningTail(bytes, offset + 4)) return null;
    final newThreshold = _readU32Le(bytes, offset);
    if (!_validAdminChangeThreshold(org, adminCount, newThreshold)) {
      return null;
    }
    final thresholdLabel = '$newThreshold/$adminCount';

    return DecodedPayload(
      action: 'propose_admin_set_change',
      summary:
          '${_orgName(org)} 管理员集合变更：${_bytesToSs58(accountBytes)} → $adminCount 人，阈值 $thresholdLabel',
      fields: {
        'org': _orgName(org),
        'account': accountHex,
        'admins': admins.join(','),
      },
      reviewFields: {
        'org': _orgName(org),
        'account': _bytesToSs58(accountBytes),
        'admins': adminAddresses.join(','),
        'new_threshold': thresholdLabel,
      },
    );
  }

  // ---------------------------------------------------------------------------
  // GrandpaKeyChange(16) / propose_key_change(0)
  // 格式：[16][0][institution:AccountId32][new_key:32]
  // ---------------------------------------------------------------------------
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

  // ---------------------------------------------------------------------------
  // 通用:只取 proposal_id: u64_le 的兜底执行/取消/清理类 call。
  //
  // 链端 Phase 3 保留的若干 `execute_X` / `cancel_failed_X` /
  // `cleanup_rejected_X` 签名完全一致:
  //     pub fn <name>(origin, proposal_id: u64) -> DispatchResult
  // SCALE 编码恒为 `[pallet_idx][call_idx][proposal_id:u64_le]` = 10 bytes。
  //
  // 所有这类 call 的 `display.fields` 按 Registry 统一为
  //   { proposal_id: <decimal string> }
  // 与节点 Tauri UI / wuminapp 的 sign_request 逐字对齐 → 🟢 绿色识别。
  // ---------------------------------------------------------------------------
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

  // ---------------------------------------------------------------------------
  // OffchainTransaction(21) / bind_clearing_bank(30), switch_bank(33)
  // 格式：[21][call][AccountId32]
  // ---------------------------------------------------------------------------
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

  // ---------------------------------------------------------------------------
  // OffchainTransaction(21) / deposit(31), withdraw(32)
  // 格式：[21][call][Compact<u128> amount]
  // ---------------------------------------------------------------------------
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

  // ---------------------------------------------------------------------------
  // OffchainTransaction(21) / register_clearing_bank(50)
  // 格式：[21][50][Vec sfid_number][Vec peer_id][Vec rpc_domain][u16 rpc_port]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeRegisterClearingBank(Uint8List bytes) {
    var offset = 2;
    final (sfidNumber, sfidNext) = _readUtf8Vec(bytes, offset);
    if (sfidNumber == null) return null;
    offset = sfidNext;
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
      summary: '声明清算行节点 $sfidNumber @ $rpcDomain:$rpcPort',
      fields: {
        'sfid_number': sfidNumber,
        'peer_id': peerId,
        'rpc_domain': rpcDomain,
        'rpc_port': rpcPort.toString(),
      },
    );
  }

  // ---------------------------------------------------------------------------
  // OffchainTransaction(21) / update_clearing_bank_endpoint(51)
  // 格式：[21][51][Vec sfid_number][Vec new_domain][u16 new_port]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeUpdateClearingBankEndpoint(Uint8List bytes) {
    var offset = 2;
    final (sfidNumber, sfidNext) = _readUtf8Vec(bytes, offset);
    if (sfidNumber == null) return null;
    offset = sfidNext;
    final (newDomain, domainNext) = _readUtf8Vec(bytes, offset);
    if (newDomain == null) return null;
    offset = domainNext;
    if (offset + 2 > bytes.length) return null;
    final newPort = bytes[offset] | (bytes[offset + 1] << 8);
    if (!_hasValidSigningTail(bytes, offset + 2)) return null;

    return DecodedPayload(
      action: 'update_clearing_bank_endpoint',
      summary: '更新清算行 $sfidNumber 端点 → $newDomain:$newPort',
      fields: {
        'sfid_number': sfidNumber,
        'new_domain': newDomain,
        'new_port': newPort.toString(),
      },
    );
  }

  // ---------------------------------------------------------------------------
  // OffchainTransaction(21) / unregister_clearing_bank(52)
  // 格式：[21][52][Vec sfid_number]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeUnregisterClearingBank(Uint8List bytes) {
    final (sfidNumber, sfidEnd) = _readUtf8Vec(bytes, 2);
    if (sfidNumber == null) return null;
    if (!_hasValidSigningTail(bytes, sfidEnd)) return null;
    return DecodedPayload(
      action: 'unregister_clearing_bank',
      summary: '注销清算行节点 $sfidNumber',
      fields: {
        'sfid_number': sfidNumber,
      },
    );
  }

  // ---------------------------------------------------------------------------
  // 工具方法
  // ---------------------------------------------------------------------------

  /// SigningPayload 扩展尾固定段:spec_version(4) + tx_version(4)
  /// + genesis_hash(32) + birth_hash(32) + CheckMetadataHash None(1)。
  static const int _signingTailFixedLen = 73;

  /// 校验 call_data 在 [callEnd] 处结束,其后是合法的 SigningPayload 扩展尾。
  ///
  /// QR 的 payload_hex 是完整 SigningPayload,call_data 永远不会顶到末尾;
  /// 尾部布局与节点端 build_signing_payload / wuminapp polkadart 编码一致:
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

  static String _institutionAccountLabel(int? org, Uint8List accountBytes) {
    switch (org) {
      case 0:
        return '国储会';
      case 1:
        return '省储会';
      case 2:
        return '省储行';
      case 3:
        return '个人多签 ${_bytesToSs58(accountBytes)}';
      case 4:
      case 5:
        return '机构账户 ${_bytesToSs58(accountBytes)}';
      default:
        return '机构账户 ${_bytesToSs58(accountBytes)}';
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

  static int _minimumRegularThreshold(int adminCount) {
    return (adminCount ~/ 2) + 1;
  }

  static bool _validAdminChangeThreshold(
    int org,
    int adminCount,
    int threshold,
  ) {
    return switch (org) {
      0 => adminCount == 19 && threshold == 13,
      1 || 2 => adminCount == 9 && threshold == 6,
      3 || 4 || 5 => adminCount >= 2 &&
          threshold > adminCount ~/ 2 &&
          threshold <= adminCount,
      _ => false,
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

  /// 机构代号映射。
  static String _orgName(int org) {
    switch (org) {
      case 0:
        return '国储会';
      case 1:
        return '省储会';
      case 2:
        return '省储行';
      case 3:
        return '个人多签';
      case 4:
      case 5:
        // 公权(ORG_PUP=4)/其他(ORG_OTH=5)对外统一展示为"机构账户",
        // 与 _institutionAccountLabel 措辞一致，消除同一 org 数字跨函数标签不一致。
        return '机构账户';
      default:
        return '机构$org';
    }
  }

  static bool _activationAccountKindMatchesOrg(int org, int kind) {
    switch (org) {
      case 0:
      case 1:
      case 2:
        return kind == 0;
      case 3:
        return kind == 1;
      case 4:
      case 5:
        return kind == 2;
      default:
        return false;
    }
  }

  static DecodedPayload? _decodeCpmsArchiveDelete(Uint8List bytes) {
    final text = utf8.decode(bytes, allowMalformed: false);
    final parts = text.split('|');
    if (parts.length != 6 || parts[0] != 'CPMS_ARCHIVE_DELETE_V1') {
      return null;
    }
    final challengeId = parts[1];
    final archiveId = parts[2];
    final archiveNo = parts[3];
    final adminPubkey = parts[4];
    final expiresAt = parts[5];
    final adminAddress = _pubkeyHexToSs58OrRaw(adminPubkey);
    if (challengeId.isEmpty ||
        archiveId.isEmpty ||
        archiveNo.isEmpty ||
        !adminPubkey.startsWith('0x') ||
        expiresAt.isEmpty) {
      return null;
    }
    return DecodedPayload(
      action: 'archive_delete',
      summary: '确认删除 CPMS 公民档案',
      fields: {
        'archive_no': archiveNo,
        'archive_id': archiveId,
        'admin_pubkey': adminAddress,
        'expires_at': expiresAt,
      },
      reviewFields: {
        'archive_no': archiveNo,
        'admin_pubkey': adminAddress,
        'expires_at': expiresAt,
      },
    );
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
