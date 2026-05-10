import 'dart:convert';
import 'dart:typed_data';

import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import '../chain/chain_constants.dart';
import '../chain/institutions.dart';
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
  });

  /// 动作标识，与 display.action 一致。
  final String action;

  /// 一句话摘要。
  final String summary;

  /// 结构化字段，用于与 display.fields 逐一比对。
  final Map<String, String> fields;
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
  static const int _subjectKindBuiltin = 0x01;
  static const int _subjectKindPersonalDuoqian = 0x03;
  static const int _subjectKindInstitutionAccount = 0x05;

  /// 尝试从 payload_hex 中解码交易信息。
  ///
  /// [payloadHex] 为完整 SigningPayload 编码的 hex 字符串。
  /// call data 从 payload 起始位置开始，以 pallet_index 和 call_index 为前两字节。
  ///
  /// 返回 null 表示无法识别或解码失败 → strict 模式下 decodeFailed → 禁止签名。
  static final _activateSubjectPrefix = Uint8List.fromList(
    'GMB_ACTIVATE_SUBJECT_V1'.codeUnits,
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

  static DecodedPayload? decode(String payloadHex) {
    // 先尝试解码非链上交易：管理员激活 / 清算行管理员解密 challenge。
    try {
      final raw = _hexToBytes(payloadHex);
      if (raw.length ==
              _activateSubjectPrefix.length + 48 + 1 + 1 + 32 + 8 + 16 &&
          _hasPrefix(raw, _activateSubjectPrefix)) {
        return _decodeActivateAdminSubject(raw);
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
      // register_sfid_institution(call=2) 由 sfid 后端 ShengSigningPubkey 直签,
      // 不走冷钱包,decoder 不覆盖。
      // propose_create_institution(call=5) 由 wuminapp 在线端构造、走冷钱包扫码签名;
      // ADR-008 step2b/step2d 凭证带 (province, signer_admin_pubkey) 双层匹配字段。
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
      // ADR-008 step3 后凭证 SCALE 末尾带 (province, signer_admin_pubkey)。
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
      // 框架阶段:10 个 propose_X 解码暂用 _decodeOnchainAssetPlaceholder 兜底,
      // 后续任务卡 D 实装具体 SCALE 解码(参考 _decodeProposeTransfer 等同款)。
      if (palletIndex == PalletRegistry.onchainIssuancePallet) {
        if (callIndex == PalletRegistry.proposeIssueCall) {
          return _decodeOnchainAssetPlaceholder(
            bytes,
            action: 'propose_onchain_asset_issue',
            summary: '发起 创建用户代币 提案(待解码业务字段)',
          );
        }
        if (callIndex == PalletRegistry.proposeMintCall) {
          return _decodeOnchainAssetPlaceholder(
            bytes,
            action: 'propose_onchain_asset_mint',
            summary: '发起 增发用户代币 提案(待解码业务字段)',
          );
        }
        if (callIndex == PalletRegistry.proposeBurnCall) {
          return _decodeOnchainAssetPlaceholder(
            bytes,
            action: 'propose_onchain_asset_burn',
            summary: '发起 销毁用户代币 提案(待解码业务字段)',
          );
        }
        if (callIndex == PalletRegistry.proposeCloseAssetCall) {
          return _decodeOnchainAssetPlaceholder(
            bytes,
            action: 'propose_onchain_asset_close',
            summary: '发起 关闭用户代币 提案(待解码业务字段)',
          );
        }
        if (callIndex == PalletRegistry.proposeAssetTransferCall) {
          return _decodeOnchainAssetPlaceholder(
            bytes,
            action: 'propose_onchain_asset_transfer',
            summary: '发起 用户代币转账 提案(待解码业务字段)',
          );
        }
        if (callIndex == PalletRegistry.proposeMonitorFreezeCall) {
          return _decodeOnchainAssetPlaceholder(
            bytes,
            action: 'propose_monitor_freeze',
            summary: '发起 NRC 监管 冻结持仓 提案(待解码业务字段)',
          );
        }
        if (callIndex == PalletRegistry.proposeMonitorUnfreezeCall) {
          return _decodeOnchainAssetPlaceholder(
            bytes,
            action: 'propose_monitor_unfreeze',
            summary: '发起 NRC 监管 解冻持仓 提案(待解码业务字段)',
          );
        }
        if (callIndex == PalletRegistry.proposeMonitorConfiscateCall) {
          return _decodeOnchainAssetPlaceholder(
            bytes,
            action: 'propose_monitor_confiscate',
            summary: '发起 NRC 监管 强制 burn 提案(待解码业务字段)',
          );
        }
        if (callIndex == PalletRegistry.proposeMonitorForceTransferCall) {
          return _decodeOnchainAssetPlaceholder(
            bytes,
            action: 'propose_monitor_force_transfer',
            summary: '发起 NRC 监管 强制划转 提案(待解码业务字段)',
          );
        }
        if (callIndex == PalletRegistry.proposeMonitorForceCloseCall) {
          return _decodeOnchainAssetPlaceholder(
            bytes,
            action: 'propose_monitor_force_close',
            summary: '发起 NRC 监管 整币封禁 提案(待解码业务字段)',
          );
        }
      }

      return null;
    } catch (_) {
      return null;
    }
  }

  // ---------------------------------------------------------------------------
  // OnchainIssuance 框架阶段占位解码:仅返回 action / summary,业务字段待后续任务卡 D 落地
  // (参考 _decodeProposeTransfer 等同款,把 SCALE 字段映射到 SignDisplayField list)
  // ---------------------------------------------------------------------------
  static DecodedPayload _decodeOnchainAssetPlaceholder(
    Uint8List bytes, {
    required String action,
    required String summary,
  }) {
    return DecodedPayload(
      action: action,
      summary: summary,
      fields: const <String, String>{},
    );
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
    final (amountFen, _) = _decodeCompactBigInt(bytes, offset);
    if (amountFen == null) return null;

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
  // 格式：[0x13][0x00][org:u8][institution:48][beneficiary:32][amount:u128_le][Vec remark]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeProposeTransfer(Uint8List bytes) {
    // 最小长度：2 + 1 + 48 + 32 + 16 + 1 = 100
    if (bytes.length < 100) return null;

    var offset = 2;

    // org: u8 仅用于链端路由,冷钱包展示以 SubjectId 解出的具体账户为准。
    offset += 1;

    // institution: [u8; 48]，必须是 D/ADR-015 SubjectId。
    final institutionBytes = bytes.sublist(offset, offset + 48);
    offset += 48;
    final subject = _decodeSpendSubjectId(institutionBytes);
    if (subject == null) return null;

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
    offset += remarkLenSize;
    var remark = '';
    if (remarkLen > 0 && offset + remarkLen <= bytes.length) {
      remark = utf8.decode(bytes.sublist(offset, offset + remarkLen),
          allowMalformed: true);
    }

    return DecodedPayload(
      action: 'propose_transfer',
      summary:
          '${subject.label} 提案转账 $amountYuan GMB 给 ${_truncateAddress(beneficiary)}',
      fields: {
        'institution': subject.label,
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
    if (bytes.length < 11) return null;
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
    if (bytes.length < 10) return null;
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
    if (bytes.length < 10) return null;
    final proposalId = _readU64Le(bytes, 2);
    var offset = 10;

    var reason = '';
    if (offset < bytes.length) {
      final (reasonLen, reasonLenSize) = _decodeCompactU32(bytes, offset);
      offset += reasonLenSize;
      if (reasonLen > 0 && offset + reasonLen <= bytes.length) {
        reason = utf8.decode(
          bytes.sublist(offset, offset + reasonLen),
          allowMalformed: true,
        );
      }
    }

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
  // 格式：[0x17][0x00][proposal_id:u64_le][subject_id:48][approve:bool]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeJointVote(Uint8List bytes) {
    // 2 + 8 + 48 + 1 = 59
    if (bytes.length < 59) return null;

    final proposalId = _readU64Le(bytes, 2);
    // subject_id 48 bytes 跳过（不在 display 中展示细节）
    final approve = bytes[58] != 0;
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
  // ADR-008 step3 凭证双层匹配。
  // 格式：[0x17][0x01][proposal_id:u64_le][binding_id:32]
  //       [Vec nonce][Vec sig][Vec province][[u8;32] signer_admin_pubkey][approve:bool]
  //
  // (province, signer_admin_pubkey) 必须进 payload — 链端 RuntimeSfidVoteVerifier
  // 走 sheng_signing_pubkey_for_admin(province, admin) 双层匹配查派生公钥,
  // signer_admin_pubkey 不进 SCALE 即被拒签 → decoder 拒绝旧凭证字节流。
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeCastReferendum(Uint8List bytes) {
    // 最小：2 + 8 + 32 + 1(nonce compact) + 1(sig compact)
    //      + 1(province compact) + 32(signer_admin_pubkey) + 1(approve) = 78
    if (bytes.length < 78) return null;

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

    // ADR-008 step3 ★ Vec<u8> province (UTF-8 省份字节,BoundedVec<u8, 64>)
    final (provinceLen, provinceLenSize) = _decodeCompactU32(bytes, offset);
    offset += provinceLenSize;
    if (offset + provinceLen > bytes.length) return null;
    final province = utf8.decode(
      bytes.sublist(offset, offset + provinceLen),
      allowMalformed: true,
    );
    offset += provinceLen;

    // ADR-008 step3 ★ [u8; 32] signer_admin_pubkey (固定 32 字节)
    if (offset + 32 > bytes.length) return null;
    final signerAdminPubkey = bytes.sublist(offset, offset + 32);
    offset += 32;

    // approve: bool（1 字节）
    if (offset >= bytes.length) return null;
    final approve = bytes[offset] != 0;
    final voteText = approve ? '赞成' : '反对';

    return DecodedPayload(
      action: 'cast_referendum',
      summary: '公民投票 提案 #$proposalId：$voteText',
      fields: {
        'proposal_id': proposalId.toString(),
        'approve': approve.toString(),
        'province': province,
        'signer_admin_pubkey': _bytesToLowerHex(signerAdminPubkey),
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
  // 格式：GMB_ACTIVATE_SUBJECT_V1 + subject_id(48B) + org(u8) + kind(u8)
  //      + pubkey(32B) + timestamp(8B, u64 LE) + nonce(16B)
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeActivateAdminSubject(Uint8List bytes) {
    final expectedLength =
        _activateSubjectPrefix.length + 48 + 1 + 1 + 32 + 8 + 16;
    if (bytes.length != expectedLength) return null;

    var offset = _activateSubjectPrefix.length;
    final subjectBytes = bytes.sublist(offset, offset + 48);
    final subject = _decodeSpendSubjectId(subjectBytes);
    if (subject == null) return null;
    offset += 48;

    final org = bytes[offset++];
    final kind = bytes[offset++];
    if (!_activationSubjectKindMatchesOrg(org, kind, subject.kind)) {
      return null;
    }

    final pubkey = bytes.sublist(offset, offset + 32);

    return DecodedPayload(
      action: 'activate_admin_subject',
      summary: '激活${_orgName(org)}管理员',
      fields: {
        'org': _orgName(org),
        'subject': _bytesToLowerHex(subjectBytes),
        'pubkey': _bytesToLowerHex(pubkey),
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
  // 链端签名(ADR-008 step2b 双层凭证):
  //   pub fn propose_create_institution(
  //     origin,
  //     sfid_number: SfidNumberOf<T>,                 // BoundedVec<u8>
  //     institution_name: AccountNameOf<T>,   // BoundedVec<u8>
  //     accounts: InstitutionInitialAccountsOf<T>,
  //         // BoundedVec<{ account_name: BoundedVec<u8>, amount: u128 }>
  //     admin_org: u8,                        // ORG_PUP / ORG_OTH
  //     admin_count: u32,
  //     duoqian_admins: DuoqianAdminsOf<T>,   // BoundedVec<AccountId32>
  //     threshold: u32,
  //     register_nonce: RegisterNonceOf<T>,   // BoundedVec<u8>
  //     signature: RegisterSignatureOf<T>,    // BoundedVec<u8> (64B sr25519)
  //     province: Vec<u8>,                    // ★ ADR-008 step2b 必填省份
  //     signer_admin_pubkey: [u8; 32],        // ★ ADR-008 step2b 签名 admin
  //   )
  //
  // SCALE 顺序与上述完全一致。链端 RuntimeSfidInstitutionVerifier 走
  // sheng_signing_pubkey_for_admin(province, signer_admin_pubkey) 双层匹配。
  // 禁止在尾部追加 a3/sub_type/parent_sfid_number 等旧字段。
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

    // institution_name: BoundedVec<u8>
    final (nameLen, nameLenSize) = _decodeCompactU32(bytes, offset);
    offset += nameLenSize;
    if (offset + nameLen > bytes.length) return null;
    final institutionName = utf8.decode(bytes.sublist(offset, offset + nameLen),
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
      final amount = _readU128Le(bytes, offset);
      accountAmounts[accountName] = amount;
      accountsTotal += amount;
      offset += 16;
    }

    // admin_org: u8。机构账户只能使用 ORG_PUP(4) 或 ORG_OTH(5)。
    if (offset + 1 > bytes.length) return null;
    final adminOrg = bytes[offset];
    if (adminOrg != 4 && adminOrg != 5) return null;
    offset += 1;

    // admin_count: u32 (LE)
    if (offset + 4 > bytes.length) return null;
    final adminCount = bytes[offset] |
        (bytes[offset + 1] << 8) |
        (bytes[offset + 2] << 16) |
        (bytes[offset + 3] << 24);
    offset += 4;

    // duoqian_admins: BoundedVec<AccountId32> — 跳过 N × 32 bytes
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

    // ADR-008 step2b ★ province: Vec<u8>
    final (provinceLen, provinceLenSize) = _decodeCompactU32(bytes, offset);
    offset += provinceLenSize;
    if (offset + provinceLen > bytes.length) return null;
    final province = utf8.decode(
      bytes.sublist(offset, offset + provinceLen),
      allowMalformed: true,
    );
    offset += provinceLen;

    // ADR-008 step2b ★ signer_admin_pubkey: [u8; 32]
    if (offset + 32 > bytes.length) return null;
    final signerAdminPubkey = bytes.sublist(offset, offset + 32);
    offset += 32;

    if (offset != bytes.length) return null;

    final amountYuan = _fenToYuan(accountsTotal);
    final fields = <String, String>{
      'sfid_number': sfidNumber,
      'institution_name': institutionName,
      'org': _orgName(adminOrg),
      'admin_count': adminCount.toString(),
      'threshold': '$threshold/$adminCount',
      'total_amount_yuan': '$amountYuan GMB',
    };
    for (final entry in accountAmounts.entries) {
      fields['amount_${entry.key}'] = '${_fenToYuan(entry.value)} GMB';
    }
    fields['province'] = province;
    fields['signer_admin_pubkey'] = _bytesToLowerHex(signerAdminPubkey);

    return DecodedPayload(
      action: 'propose_create_institution',
      summary:
          '创建机构多签账户「$institutionName」（$adminCount 管理员，阈值 $threshold，入金 $amountYuan 元）',
      fields: fields,
    );
  }

  // ---------------------------------------------------------------------------
  // ResolutionIssuance(8) / propose_resolution_issuance(0)
  //
  // 链端签名(ADR-008 step3 双层凭证):
  //   pub fn propose_resolution_issuance(
  //     origin,
  //     reason: ReasonOf<T>,                  // BoundedVec<u8>
  //     total_amount: BalanceOf<T>,           // u128 LE
  //     allocations: AllocationOf<T>,
  //         // BoundedVec<{ recipient: AccountId32, amount: u128 }>
  //     eligible_total: u64,
  //     snapshot_nonce: SnapshotNonceOf<T>,   // BoundedVec<u8>
  //     signature: SnapshotSignatureOf<T>,    // BoundedVec<u8>
  //     province: BoundedVec<u8, ConstU32<64>>,   // ★ ADR-008 step3
  //     signer_admin_pubkey: [u8; 32],            // ★ ADR-008 step3
  //   )
  //
  // 链端 verifier 走 RuntimePopulationSnapshotVerifier(走
  // sheng_signing_pubkey_for_admin)。decoder 仅展示给用户看,不做验签。
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeProposeResolutionIssuance(Uint8List bytes) {
    if (bytes.length < 3) return null;
    var offset = 2; // 跳过 pallet_index + call_index

    // reason: Vec<u8>
    final (reasonLen, reasonLenSize) = _decodeCompactU32(bytes, offset);
    offset += reasonLenSize;
    var reason = '';
    if (reasonLen > 0 && offset + reasonLen <= bytes.length) {
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

    // eligible_total: u64 LE
    if (offset + 8 > bytes.length) return null;
    final eligibleTotal = _readU64Le(bytes, offset);
    offset += 8;

    // snapshot_nonce: Vec<u8> — 跳过
    final (nonceLen, nonceLenSize) = _decodeCompactU32(bytes, offset);
    offset += nonceLenSize;
    if (offset + nonceLen > bytes.length) return null;
    offset += nonceLen;

    // signature: Vec<u8> — 跳过
    final (sigLen, sigLenSize) = _decodeCompactU32(bytes, offset);
    offset += sigLenSize;
    if (offset + sigLen > bytes.length) return null;
    offset += sigLen;

    // ADR-008 step3 ★ province: BoundedVec<u8, 64>
    final (provinceLen, provinceLenSize) = _decodeCompactU32(bytes, offset);
    offset += provinceLenSize;
    if (offset + provinceLen > bytes.length) return null;
    final province = utf8.decode(
      bytes.sublist(offset, offset + provinceLen),
      allowMalformed: true,
    );
    offset += provinceLen;

    // ADR-008 step3 ★ signer_admin_pubkey: [u8; 32]
    if (offset + 32 > bytes.length) return null;
    final signerAdminPubkey = bytes.sublist(offset, offset + 32);

    final amountYuan = _fenToYuan(totalAmountFen);

    return DecodedPayload(
      action: 'propose_resolution_issuance',
      summary: '决议发行 $amountYuan GMB（$allocLen 项分配,合格人数 $eligibleTotal）',
      fields: {
        'reason': reason,
        'amount_yuan': '$amountYuan GMB',
        'allocation_count': allocLen.toString(),
        'eligible_total': eligibleTotal.toString(),
        'province': province,
        'signer_admin_pubkey': _bytesToLowerHex(signerAdminPubkey),
      },
    );
  }

  // ---------------------------------------------------------------------------
  // PersonalManage(7) / propose_create(0)
  // 格式：[7][0][BoundedVec account_name][BoundedVec<AccountId32> admins][u128 amount]
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

    // admins: BoundedVec<AccountId32>。admin_count 由向量长度派生。
    final (adminsLen, adminsLenSize) = _decodeCompactU32(bytes, offset);
    offset += adminsLenSize;
    if (adminsLen < 2 || adminsLen > 64) return null;
    if (offset + adminsLen * 32 > bytes.length) return null;
    offset += adminsLen * 32;
    if (offset + 16 != bytes.length) return null;

    final regularThreshold = _dynamicThreshold(adminsLen);
    if (regularThreshold == null) return null;

    // amount: u128
    final amountFen = _readU128Le(bytes, offset);
    final amountYuan = _fenToYuan(amountFen);

    return DecodedPayload(
      action: 'propose_create_personal',
      summary:
          '创建个人多签「$accountName」（$adminsLen 管理员，日常阈值 $regularThreshold，创建全员通过，入金 $amountYuan 元）',
      fields: {
        'account_name': accountName,
        'admin_count': adminsLen.toString(),
        'regular_threshold': '$regularThreshold/$adminsLen',
        'create_threshold': '$adminsLen/$adminsLen',
        'amount_yuan': '$amountYuan GMB',
      },
    );
  }

  // ---------------------------------------------------------------------------
  // OrganizationManage(17) / propose_close(1)
  // PersonalManage(7) / propose_close(1)
  // 格式：[17][1][duoqian_address:32][beneficiary:32]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeProposeClose(
    Uint8List bytes, {
    required String action,
    required String summaryLabel,
  }) {
    if (bytes.length < 66) return null;
    final duoqianId = bytes.sublist(2, 34);
    final beneficiaryId = bytes.sublist(34, 66);
    final duoqian = Keyring().encodeAddress(duoqianId.toList(), _ss58Prefix);
    final beneficiary =
        Keyring().encodeAddress(beneficiaryId.toList(), _ss58Prefix);
    return DecodedPayload(
      action: action,
      summary: '提案关闭$summaryLabel ${_truncateAddress(duoqian)}',
      fields: {
        'duoqian_address': duoqian,
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
    offset += remarkLenSize;
    var remark = '';
    if (remarkLen > 0 && offset + remarkLen <= bytes.length) {
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
  // 格式：[19][2][institution:48][amount:u128]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeProposeSweep(Uint8List bytes) {
    if (bytes.length < 66) return null;
    var offset = 2;
    final institutionBytes = bytes.sublist(offset, offset + 48);
    offset += 48;
    final bankName = _decodeBuiltinSubjectLabel(institutionBytes);
    if (bankName == null) return null;
    final amountFen = _readU128Le(bytes, offset);
    final amountYuan = _fenToYuan(amountFen);
    return DecodedPayload(
      action: 'propose_sweep_to_main',
      summary: '手续费划转 $amountYuan GMB：$bankName',
      fields: {
        'institution': bankName,
        'amount_yuan': '$amountYuan GMB',
      },
    );
  }

  // ---------------------------------------------------------------------------
  // ResolutionDestro(14) / propose_destroy(0)
  // 格式：[14][0][org:u8][institution:48][amount:u128]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeProposeDestroy(Uint8List bytes) {
    if (bytes.length < 67) return null;
    var offset = 2;
    final org = bytes[offset];
    offset += 1;
    offset += 48; // institution 跳过
    final amountFen = _readU128Le(bytes, offset);
    final amountYuan = _fenToYuan(amountFen);
    return DecodedPayload(
      action: 'propose_destroy',
      summary: '${_orgName(org)} 决议销毁 $amountYuan GMB',
      fields: {
        'org': _orgName(org),
        'amount_yuan': '$amountYuan GMB',
      },
    );
  }

  // ---------------------------------------------------------------------------
  // AdminsChange(12) / propose_admin_set_change(0)
  // 格式：[12][0][org:u8][subject:48][Compact<N>][new_admins:N*32]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeProposeAdminSetChange(Uint8List bytes) {
    if (bytes.length < 52) return null;
    var offset = 2;
    final org = bytes[offset];
    offset += 1;
    final subjectBytes = bytes.sublist(offset, offset + 48);
    final subject = _decodeSpendSubjectId(subjectBytes);
    if (subject == null) return null;
    if (!_adminSubjectKindMatchesOrg(org, subject.kind)) return null;
    offset += 48;

    final (adminCount, countSize) = _decodeCompactU32(bytes, offset);
    if (countSize == 0 || adminCount < 1) return null;
    offset += countSize;

    final admins = <String>[];
    for (var i = 0; i < adminCount; i++) {
      if (offset + 32 > bytes.length) return null;
      admins.add(_bytesToLowerHex(bytes.sublist(offset, offset + 32)));
      offset += 32;
    }

    return DecodedPayload(
      action: 'propose_admin_set_change',
      summary: '${_orgName(org)} 管理员集合变更：${subject.label} → $adminCount 人',
      fields: {
        'org': _orgName(org),
        'subject': _bytesToLowerHex(subjectBytes),
        'new_admins': admins.join(','),
      },
    );
  }

  // ---------------------------------------------------------------------------
  // GrandpaKeyChange(16) / propose_key_change(0)
  // 格式：[16][0][institution:48][new_key:32]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeProposeKeyChange(Uint8List bytes) {
    if (bytes.length < 82) return null;
    var offset = 2;
    final institutionBytes = bytes.sublist(offset, offset + 48);
    offset += 48;
    var endIndex = 48;
    while (endIndex > 0 && institutionBytes[endIndex - 1] == 0) {
      endIndex--;
    }
    final sfidNumber = endIndex > 0
        ? String.fromCharCodes(institutionBytes.sublist(0, endIndex))
        : '';
    final keyBytes = bytes.sublist(offset, offset + 32);
    final keyHex =
        keyBytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
    return DecodedPayload(
      action: 'propose_replace_grandpa_key',
      summary: 'GRANDPA 密钥替换提案',
      fields: {
        'institution': sfidNumber,
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
    if (bytes.length < 10) return null;
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
    if (bytes.length < 34) return null;
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
    final (sfidNumber, _) = _readUtf8Vec(bytes, 2);
    if (sfidNumber == null) return null;
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

  /// 0x 小写 hex(feedback_pubkey_format_rule.md 铁律)。
  static String _bytesToLowerHex(Uint8List bytes) {
    return '0x${bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
  }

  static ({int kind, String label})? _decodeSpendSubjectId(Uint8List bytes) {
    if (bytes.length != 48) return null;
    switch (bytes[0]) {
      case _subjectKindBuiltin:
        final label = _decodeBuiltinSubjectLabel(bytes);
        return label == null ? null : (kind: _subjectKindBuiltin, label: label);
      case _subjectKindPersonalDuoqian:
        final account =
            _accountHexFromSubject(bytes, _subjectKindPersonalDuoqian);
        return account == null
            ? null
            : (
                kind: _subjectKindPersonalDuoqian,
                label: '个人多签 ${_shortHex(account)}'
              );
      case _subjectKindInstitutionAccount:
        final account =
            _accountHexFromSubject(bytes, _subjectKindInstitutionAccount);
        return account == null
            ? null
            : (
                kind: _subjectKindInstitutionAccount,
                label: '机构账户 ${_shortHex(account)}'
              );
      default:
        return null;
    }
  }

  static String? _decodeBuiltinSubjectLabel(Uint8List bytes) {
    if (bytes.length != 48 || bytes[0] != _subjectKindBuiltin) return null;
    var end = 48;
    while (end > 1 && bytes[end - 1] == 0) {
      end--;
    }
    if (end <= 1) return null;
    final sfidNumber = utf8.decode(bytes.sublist(1, end), allowMalformed: true);
    return institutionName(sfidNumber) ?? sfidNumber;
  }

  static String? _accountHexFromSubject(Uint8List bytes, int expectedKind) {
    if (bytes.length != 48 || bytes[0] != expectedKind) return null;
    for (var i = 33; i < 48; i++) {
      if (bytes[i] != 0) return null;
    }
    return bytes
        .sublist(1, 33)
        .map((b) => b.toRadixString(16).padLeft(2, '0'))
        .join();
  }

  static String _shortHex(String hex) {
    if (hex.length <= 14) return hex;
    return '${hex.substring(0, 8)}...${hex.substring(hex.length - 6)}';
  }

  static Uint8List _hexToBytes(String input) {
    final text = input.startsWith('0x') ? input.substring(2) : input;
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

  static BigInt _readU128Le(Uint8List bytes, int offset) {
    var value = BigInt.zero;
    for (var i = 15; i >= 0; i--) {
      value = (value << 8) | BigInt.from(bytes[offset + i]);
    }
    return value;
  }

  static int? _dynamicThreshold(int adminCount) {
    if (adminCount < 2) return null;
    if (adminCount == 2) return 2;
    return (adminCount + 1) ~/ 2;
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
        return '公权机构账户';
      case 5:
        return '其他机构账户';
      default:
        return '机构$org';
    }
  }

  static bool _adminSubjectKindMatchesOrg(int org, int subjectKind) {
    switch (org) {
      case 0:
      case 1:
      case 2:
        return subjectKind == _subjectKindBuiltin;
      case 3:
        return subjectKind == _subjectKindPersonalDuoqian;
      case 4:
      case 5:
        return subjectKind == _subjectKindInstitutionAccount;
      default:
        return false;
    }
  }

  static bool _activationSubjectKindMatchesOrg(
      int org, int kind, int subjectKind) {
    switch (org) {
      case 0:
      case 1:
      case 2:
        return kind == 0 && subjectKind == _subjectKindBuiltin;
      case 3:
        return kind == 2 && subjectKind == _subjectKindPersonalDuoqian;
      case 4:
      case 5:
        return kind == 3 && subjectKind == _subjectKindInstitutionAccount;
      default:
        return false;
    }
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
