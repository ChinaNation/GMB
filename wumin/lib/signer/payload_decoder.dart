import 'dart:convert';
import 'dart:typed_data';

import 'package:crypto/crypto.dart' show sha256;
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

  /// 尝试从 payload_hex 中解码交易信息。
  ///
  /// [payloadHex] 为完整 SigningPayload 编码的 hex 字符串。
  /// call data 从 payload 起始位置开始，以 pallet_index 和 call_index 为前两字节。
  ///
  /// [specVersion] 为链 runtime 版本号。当 spec_version 不在
  /// [PalletRegistry.supportedSpecVersions] 中时直接返回 null，
  /// 避免因 pallet 索引偏移导致错误解码。
  ///
  /// 返回 null 表示无法识别或解码失败。
  /// "GMB_ACTIVATE" 前缀（12 字节 ASCII）。
  static const _activatePrefix = [
    0x47, 0x4D, 0x42, 0x5F, // GMB_
    0x41, 0x43, 0x54, 0x49, // ACTI
    0x56, 0x41, 0x54, 0x45, // VATE
  ];

  /// "GMB_DECRYPT_V1" 前缀（14 字节 ASCII）。
  ///
  /// 这是 `WUMIN_QR_V1` 内部 payload 的业务签名域，不是二维码协议版本。
  static const _decryptPrefix = [
    0x47, 0x4D, 0x42, 0x5F, // GMB_
    0x44, 0x45, 0x43, 0x52, // DECR
    0x59, 0x50, 0x54, 0x5F, // YPT_
    0x56, 0x31, // V1
  ];

  static DecodedPayload? decode(String payloadHex, {int? specVersion}) {
    // 先尝试解码非链上交易：管理员激活 / 清算行管理员解密 challenge。
    try {
      final raw = _hexToBytes(payloadHex);
      if (raw.length == 84) {
        bool isActivate = true;
        for (var i = 0; i < 12; i++) {
          if (raw[i] != _activatePrefix[i]) {
            isActivate = false;
            break;
          }
        }
        if (isActivate) {
          return _decodeActivateAdmin(raw);
        }
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

    // spec_version 不在已知列表中时放弃解码，由调用方走 decodeFailed 路径。
    if (!PalletRegistry.isSupported(specVersion)) return null;
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

      // ── VotingEngine(9) · 统一投票入口 ──
      // Phase 3：业务 pallet 的 vote_X 全部下线，冷钱包只在这里解码投票 payload。
      // Phase 4：业务 pallet 的 execute_xxx / cancel_failed_xxx 全部物理删除,
      // 手动重试/取消统一收口至 retry_passed_proposal(9.4) / cancel_passed_proposal(9.5)。
      if (palletIndex == PalletRegistry.votingEnginePallet) {
        if (callIndex == PalletRegistry.internalVoteCall) {
          return _decodeInternalVote(bytes);
        }
        if (callIndex == PalletRegistry.jointVoteCall) {
          return _decodeJointVote(bytes);
        }
        if (callIndex == PalletRegistry.citizenVoteCall) {
          return _decodeCitizenVote(bytes);
        }
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
      // Phase 3/4：投票入口统一到 VotingEngine::internal_vote,
      // 手动重试入口统一到 VotingEngine::retry_passed_proposal,
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

      // ── RuntimeUpgrade(13) ──
      if (palletIndex == PalletRegistry.runtimeUpgradePallet) {
        if (callIndex == PalletRegistry.proposeRuntimeUpgradeCall) {
          return _decodeProposeRuntimeUpgrade(bytes);
        }
        if (callIndex == PalletRegistry.developerDirectUpgradeCall) {
          return _decodeDeveloperUpgrade(bytes);
        }
      }

      // ── DuoqianManage(17) ──
      // Phase 3：投票入口统一到 VotingEngine::internal_vote。本 pallet
      // 保留 propose_X + cleanup_rejected_proposal(被拒提案残留清理)。
      // register_sfid_institution(call=2) 由 sfid 后端 ShengSigningPubkey 直签,
      // 不走冷钱包,decoder 不覆盖。
      // propose_create_institution(call=5) 由 wuminapp 在线端构造、走冷钱包扫码签名;
      // ADR-008 step2b/step2d 后凭证带 (province, signer_admin_pubkey) 双层匹配字段。
      if (palletIndex == PalletRegistry.duoqianManagePallet) {
        if (callIndex == PalletRegistry.proposeCreateCall) {
          return _decodeProposeCreate(bytes);
        }
        if (callIndex == PalletRegistry.proposeCloseCall) {
          return _decodeProposeClose(bytes);
        }
        if (callIndex == PalletRegistry.proposeCreatePersonalCall) {
          return _decodeProposeCreatePersonal(bytes);
        }
        if (callIndex == PalletRegistry.proposeCreateInstitutionCall) {
          return _decodeProposeCreateInstitution(bytes);
        }
        if (callIndex == PalletRegistry.cleanupRejectedProposalCall) {
          return _decodeProposalIdOnly(
            bytes,
            action: 'cleanup_rejected_proposal',
            summaryTemplate: '清理被拒提案 #{id} 残留',
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
      // Phase 4: execute_destroy 已统一到 VotingEngine::retry_passed_proposal。
      if (palletIndex == PalletRegistry.resolutionDestroPallet) {
        if (callIndex == PalletRegistry.proposeDestroyCall) {
          return _decodeProposeDestroy(bytes);
        }
      }

      // ── AdminsChange(12) ──
      // Phase 4: execute_admin_replacement 已统一到 VotingEngine::retry_passed_proposal。
      if (palletIndex == PalletRegistry.adminsChangePallet) {
        if (callIndex == PalletRegistry.proposeAdminReplacementCall) {
          return _decodeProposeAdminReplacement(bytes);
        }
      }

      // ── GrandpaKeyChange(16) ──
      // Phase 4: execute_replace_grandpa_key / cancel_failed_replace_grandpa_key
      // 已分别统一到 VotingEngine::retry_passed_proposal / cancel_passed_proposal。
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

      return null;
    } catch (_) {
      return null;
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

    // org: u8
    final org = bytes[offset];
    offset += 1;
    final orgName = _orgName(org);

    // institution: [u8; 48]
    offset += 48;

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
          '$orgName 提案转账 $amountYuan GMB 给 ${_truncateAddress(beneficiary)}',
      fields: {
        'org': orgName,
        'beneficiary': beneficiary,
        'amount_yuan': '$amountYuan GMB',
        'remark': remark,
      },
    );
  }

  // Phase 3 · 业务 pallet 的 finalize_X / vote_X 全部下线,
  // 冷钱包统一通过 `_decodeInternalVote` 解码一人一票的管理员投票 payload。

  // ---------------------------------------------------------------------------
  // VotingEngine(9) / internal_vote(0)
  // 格式：[0x09][0x00][proposal_id:u64_le][approve:bool]
  //
  // Phase 3 统一入口：所有业务 pallet(admins/resolution_destro/grandpa_key/
  // duoqian_manage/duoqian_transfer 五路)的管理员投票都走这里,冷钱包不再按
  // 业务 pallet 分路解码投票 payload。
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
  // 任意账户触发终态执行，无需签投票语义。
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
  // 链端签名（Phase 4 整改 2026-05-02 后的统一取消入口）：
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
  // VotingEngine(9) / joint_vote(1)
  // 格式：[0x09][0x01][proposal_id:u64_le][institution:48][approve:bool]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeJointVote(Uint8List bytes) {
    // 2 + 8 + 48 + 1 = 59
    if (bytes.length < 59) return null;

    final proposalId = _readU64Le(bytes, 2);
    // institution_id 48 bytes 跳过（不在 display 中展示细节）
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
  // VotingEngine(9) / citizen_vote(2)
  //
  // ADR-008 step3 凭证双层匹配:
  // 格式：[0x09][0x02][proposal_id:u64_le][binding_id:32]
  //       [Vec nonce][Vec sig][Vec province][[u8;32] signer_admin_pubkey][approve:bool]
  //
  // (province, signer_admin_pubkey) 必须进 payload — 链端 RuntimeSfidVoteVerifier
  // 走 sheng_signing_pubkey_for_admin(province, admin) 双层匹配查派生公钥,
  // signer_admin_pubkey 不进 SCALE 即被拒签 → decoder 拒绝旧凭证字节流。
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeCitizenVote(Uint8List bytes) {
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
      action: 'citizen_vote',
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
  // RuntimeUpgrade(13) / propose_runtime_upgrade(0)
  //
  // 链端签名(ADR-008 step3 双层凭证):
  //   pub fn propose_runtime_upgrade(
  //     origin,
  //     reason: ReasonOf<T>,                  // BoundedVec<u8>
  //     code: CodeOf<T>,                      // BoundedVec<u8> —— 实际 WASM
  //     eligible_total: u64,
  //     snapshot_nonce: SnapshotNonceOf<T>,   // BoundedVec<u8>
  //     signature: SnapshotSignatureOf<T>,    // BoundedVec<u8>
  //     province: BoundedVec<u8, ConstU32<64>>,   // ★ ADR-008 step3
  //     signer_admin_pubkey: [u8; 32],            // ★ ADR-008 step3
  //   )
  //
  // SCALE 编码：
  //   [13][0]
  //   + Compact<u32> reason_len + reason_bytes
  //   + Compact<u32> wasm_len   + wasm_bytes
  //   + u64_le eligible_total
  //   + Compact<u32> nonce_len  + nonce_bytes
  //   + Compact<u32> sig_len    + sig_bytes
  //   + Compact<u32> province_len + province_bytes
  //   + 32B signer_admin_pubkey
  //
  // display.fields 对齐 Registry: `reason` / `wasm_size` / `wasm_hash` /
  // `eligible_total` / `province` / `signer_admin_pubkey`。
  // `wasm_hash` 由 decoder 现场 `sha256(wasm_bytes)` 计算,
  // 与节点 Tauri UI 用同一算法生成的哈希逐字对齐 — 这样用户才能独立核对
  // "我签的就是这份 WASM"。signer_admin_pubkey 用小写 0x hex 展示,
  // 内部统一格式遵循 feedback_pubkey_format_rule.md。
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeProposeRuntimeUpgrade(Uint8List bytes) {
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

    // wasm: Vec<u8>
    if (offset >= bytes.length) return null;
    final (wasmLen, wasmLenSize) = _decodeCompactU32(bytes, offset);
    offset += wasmLenSize;
    if (offset + wasmLen > bytes.length) return null;

    final wasmBytes = bytes.sublist(offset, offset + wasmLen);
    final wasmHash = sha256.convert(wasmBytes).toString(); // 64 hex (小写)
    offset += wasmLen;

    // eligible_total: u64_le
    if (offset + 8 > bytes.length) return null;
    final eligibleTotal = _readU64Le(bytes, offset);
    offset += 8;

    // snapshot_nonce: Vec<u8>(本卡只跳过)
    final (nonceLen, nonceLenSize) = _decodeCompactU32(bytes, offset);
    offset += nonceLenSize;
    if (offset + nonceLen > bytes.length) return null;
    offset += nonceLen;

    // signature: Vec<u8>(本卡只跳过)
    final (sigLen, sigLenSize) = _decodeCompactU32(bytes, offset);
    offset += sigLenSize;
    if (offset + sigLen > bytes.length) return null;
    offset += sigLen;

    // ADR-008 step3 ★ province: BoundedVec<u8, 64>(SCALE Vec)
    final (provinceLen, provinceLenSize) = _decodeCompactU32(bytes, offset);
    offset += provinceLenSize;
    if (offset + provinceLen > bytes.length) return null;
    final province = utf8.decode(
      bytes.sublist(offset, offset + provinceLen),
      allowMalformed: true,
    );
    offset += provinceLen;

    // ADR-008 step3 ★ signer_admin_pubkey: [u8; 32](固定 32 字节)
    if (offset + 32 > bytes.length) return null;
    final signerAdminPubkey = bytes.sublist(offset, offset + 32);

    return DecodedPayload(
      action: 'propose_runtime_upgrade',
      summary: 'Runtime 升级提案（WASM ${_formatWasmSize(wasmLen)}, '
          '合格人数 $eligibleTotal）',
      fields: {
        'reason': reason,
        'wasm_size': _formatWasmSize(wasmLen),
        'wasm_hash': '0x$wasmHash',
        'eligible_total': eligibleTotal.toString(),
        'province': province,
        'signer_admin_pubkey': _bytesToLowerHex(signerAdminPubkey),
      },
    );
  }

  // ---------------------------------------------------------------------------
  // RuntimeUpgrade(13) / developer_direct_upgrade(2)
  //
  // 链端签名：pub fn developer_direct_upgrade(origin, code: CodeOf<T>)
  // SCALE 编码：[13][2] + Compact<u32> wasm_len + wasm_bytes
  //
  // display.fields 对齐 Registry: `wasm_size` / `wasm_hash`。hash 同 sha256。
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeDeveloperUpgrade(Uint8List bytes) {
    if (bytes.length < 3) return null;

    var offset = 2; // 跳过 pallet_index + call_index
    final (wasmLen, lenSize) = _decodeCompactU32(bytes, offset);
    offset += lenSize;
    if (offset + wasmLen > bytes.length) return null;

    final wasmBytes = bytes.sublist(offset, offset + wasmLen);
    final wasmHash = sha256.convert(wasmBytes).toString(); // 64 hex (小写)

    return DecodedPayload(
      action: 'developer_direct_upgrade',
      summary: '开发者直升 Runtime（WASM ${_formatWasmSize(wasmLen)}）',
      fields: {
        'wasm_size': _formatWasmSize(wasmLen),
        'wasm_hash': '0x$wasmHash',
      },
    );
  }

  /// WASM 字节数的人可读渲染: `X.XX MB`(≥1 MB) 或 `X KB`。
  static String _formatWasmSize(int wasmLen) {
    if (wasmLen > 1024 * 1024) {
      return '${(wasmLen / (1024 * 1024)).toStringAsFixed(2)} MB';
    }
    return '${(wasmLen / 1024).toStringAsFixed(0)} KB';
  }

  // ---------------------------------------------------------------------------
  // 管理员激活（非链上交易）
  // 格式：GMB_ACTIVATE(12B) + shenfen_id(48B, 右补零) + timestamp(8B, u64 LE) + nonce(16B)
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeActivateAdmin(Uint8List bytes) {
    if (bytes.length < 84) return null;

    // shenfen_id: 48 bytes (offset 12), 去尾部零字节
    final idBytes = bytes.sublist(12, 60);
    var endIndex = 48;
    while (endIndex > 0 && idBytes[endIndex - 1] == 0) {
      endIndex--;
    }
    if (endIndex == 0) return null;
    final shenfenId = String.fromCharCodes(idBytes.sublist(0, endIndex));

    return DecodedPayload(
      action: 'activate_admin',
      summary: '激活管理员 - $shenfenId',
      fields: {
        'shenfen_id': shenfenId,
      },
    );
  }

  // ---------------------------------------------------------------------------
  // 清算行管理员解密（非链上交易）
  // 格式：GMB_DECRYPT_V1(14B) + sfid_id(48B, 右补零) + pubkey(32B)
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
    final sfidId = String.fromCharCodes(idBytes.sublist(0, endIndex));

    return DecodedPayload(
      action: 'decrypt_admin',
      summary: '解密清算行管理员 - $sfidId',
      fields: {
        'sfid_id': sfidId,
      },
    );
  }

  // ---------------------------------------------------------------------------
  // DuoqianManage(17) / propose_create(0)
  // 格式：[17][0][BoundedVec sfid_id][u32 admin_count][BoundedVec<AccountId32> admins][u32 threshold][u128 amount]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeProposeCreate(Uint8List bytes) {
    if (bytes.length < 10) return null;
    var offset = 2;

    // sfid_id: BoundedVec<u8>
    final (sfidLen, sfidLenSize) = _decodeCompactU32(bytes, offset);
    offset += sfidLenSize;
    if (offset + sfidLen > bytes.length) return null;
    final sfidId = utf8.decode(bytes.sublist(offset, offset + sfidLen),
        allowMalformed: true);
    offset += sfidLen;

    // account_name: BoundedVec<u8>
    final (accountNameLen, accountNameLenSize) =
        _decodeCompactU32(bytes, offset);
    offset += accountNameLenSize;
    if (offset + accountNameLen > bytes.length) return null;
    final accountName = utf8.decode(
        bytes.sublist(offset, offset + accountNameLen),
        allowMalformed: true);
    offset += accountNameLen;

    // admin_count: u32
    if (offset + 4 > bytes.length) return null;
    final adminCount = bytes[offset] |
        (bytes[offset + 1] << 8) |
        (bytes[offset + 2] << 16) |
        (bytes[offset + 3] << 24);
    offset += 4;

    // admins: BoundedVec<AccountId32> — 跳过
    final (adminsLen, adminsLenSize) = _decodeCompactU32(bytes, offset);
    offset += adminsLenSize;
    offset += adminsLen * 32;
    if (offset + 4 + 16 > bytes.length) return null;

    // threshold: u32
    final threshold = bytes[offset] |
        (bytes[offset + 1] << 8) |
        (bytes[offset + 2] << 16) |
        (bytes[offset + 3] << 24);
    offset += 4;

    // amount: u128
    final amountFen = _readU128Le(bytes, offset);
    final amountYuan = _fenToYuan(amountFen);

    return DecodedPayload(
      action: 'propose_create',
      summary:
          '创建多签账户「$accountName」（$adminCount 管理员，阈值 $threshold，入金 $amountYuan 元）',
      fields: {
        'sfid_id': sfidId,
        'account_name': accountName,
        'admin_count': adminCount.toString(),
        'threshold': '$threshold/$adminCount',
        'amount_yuan': '$amountYuan GMB',
      },
    );
  }

  // ---------------------------------------------------------------------------
  // DuoqianManage(17) / propose_create_institution(5)
  //
  // 链端签名(ADR-008 step2b 双层凭证):
  //   pub fn propose_create_institution(
  //     origin,
  //     sfid_id: SfidIdOf<T>,                 // BoundedVec<u8>
  //     institution_name: AccountNameOf<T>,   // BoundedVec<u8>
  //     accounts: InstitutionInitialAccountsOf<T>,
  //         // BoundedVec<{ account_name: BoundedVec<u8>, amount: u128 }>
  //     admin_count: u32,
  //     duoqian_admins: DuoqianAdminsOf<T>,   // BoundedVec<AccountId32>
  //     threshold: u32,
  //     register_nonce: RegisterNonceOf<T>,   // BoundedVec<u8>
  //     signature: RegisterSignatureOf<T>,    // BoundedVec<u8> (64B sr25519)
  //     province: Vec<u8>,                    // ★ ADR-008 step2b 必填省份
  //     signer_admin_pubkey: [u8; 32],        // ★ ADR-008 step2b 签名 admin
  //     a3: A3Of<T>,                          // BoundedVec<u8> (3 ASCII chars)
  //     sub_type: Option<SubTypeOf<T>>,       // Option<BoundedVec<u8>>
  //     parent_sfid_id: Option<SfidIdOf<T>>,  // Option<BoundedVec<u8>>
  //   )
  //
  // SCALE 顺序与上述完全一致;新字段位置在 register_nonce/signature 后、
  // a3/sub_type/parent_sfid_id 前。链端 RuntimeSfidInstitutionVerifier 走
  // sheng_signing_pubkey_for_admin(province, signer_admin_pubkey) 双层匹配。
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeProposeCreateInstitution(Uint8List bytes) {
    if (bytes.length < 10) return null;
    var offset = 2;

    // sfid_id: BoundedVec<u8>
    final (sfidLen, sfidLenSize) = _decodeCompactU32(bytes, offset);
    offset += sfidLenSize;
    if (offset + sfidLen > bytes.length) return null;
    final sfidId = utf8.decode(bytes.sublist(offset, offset + sfidLen),
        allowMalformed: true);
    offset += sfidLen;

    // institution_name: BoundedVec<u8>
    final (nameLen, nameLenSize) = _decodeCompactU32(bytes, offset);
    offset += nameLenSize;
    if (offset + nameLen > bytes.length) return null;
    final institutionName = utf8.decode(
        bytes.sublist(offset, offset + nameLen),
        allowMalformed: true);
    offset += nameLen;

    // accounts: BoundedVec<InstitutionInitialAccount>
    //   每项 = (account_name: Vec<u8>, amount: u128)
    final (accountsLen, accountsLenSize) = _decodeCompactU32(bytes, offset);
    offset += accountsLenSize;
    BigInt accountsTotal = BigInt.zero;
    for (var i = 0; i < accountsLen; i++) {
      final (subNameLen, subNameLenSize) = _decodeCompactU32(bytes, offset);
      offset += subNameLenSize;
      if (offset + subNameLen + 16 > bytes.length) return null;
      offset += subNameLen;
      accountsTotal += _readU128Le(bytes, offset);
      offset += 16;
    }

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

    // a3: BoundedVec<u8> (3 ASCII chars - SFR/FFR/GFR/SF)
    final (a3Len, a3LenSize) = _decodeCompactU32(bytes, offset);
    offset += a3LenSize;
    if (offset + a3Len > bytes.length) return null;
    final a3 = utf8.decode(
      bytes.sublist(offset, offset + a3Len),
      allowMalformed: true,
    );
    offset += a3Len;

    final amountYuan = _fenToYuan(accountsTotal);

    return DecodedPayload(
      action: 'propose_create_institution',
      summary:
          '创建机构多签账户「$institutionName」（$adminCount 管理员，阈值 $threshold，入金 $amountYuan 元）',
      fields: {
        'sfid_id': sfidId,
        'institution_name': institutionName,
        'admin_count': adminCount.toString(),
        'threshold': '$threshold/$adminCount',
        'amount_yuan': '$amountYuan GMB',
        'a3': a3,
        'province': province,
        'signer_admin_pubkey': _bytesToLowerHex(signerAdminPubkey),
      },
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
      summary:
          '决议发行 $amountYuan GMB（$allocLen 项分配,合格人数 $eligibleTotal）',
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
  // DuoqianManage(17) / propose_create_personal(4)
  // 格式：[17][4][BoundedVec account_name][u32 admin_count][BoundedVec<AccountId32> admins][u32 threshold][u128 amount]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeProposeCreatePersonal(Uint8List bytes) {
    if (bytes.length < 10) return null;
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

    // admin_count: u32
    if (offset + 4 > bytes.length) return null;
    final adminCount = bytes[offset] |
        (bytes[offset + 1] << 8) |
        (bytes[offset + 2] << 16) |
        (bytes[offset + 3] << 24);
    offset += 4;

    // admins: BoundedVec<AccountId32> — 跳过
    final (adminsLen, adminsLenSize) = _decodeCompactU32(bytes, offset);
    offset += adminsLenSize;
    offset += adminsLen * 32;
    if (offset + 4 + 16 > bytes.length) return null;

    // threshold: u32
    final threshold = bytes[offset] |
        (bytes[offset + 1] << 8) |
        (bytes[offset + 2] << 16) |
        (bytes[offset + 3] << 24);
    offset += 4;

    // amount: u128
    final amountFen = _readU128Le(bytes, offset);
    final amountYuan = _fenToYuan(amountFen);

    return DecodedPayload(
      action: 'propose_create_personal',
      summary:
          '创建个人多签「$accountName」（$adminCount 管理员，阈值 $threshold，入金 $amountYuan 元）',
      fields: {
        'account_name': accountName,
        'admin_count': adminCount.toString(),
        'threshold': '$threshold/$adminCount',
        'amount_yuan': '$amountYuan GMB',
      },
    );
  }

  // ---------------------------------------------------------------------------
  // DuoqianManage(17) / propose_close(1)
  // 格式：[17][1][duoqian_address:32][beneficiary:32]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeProposeClose(Uint8List bytes) {
    if (bytes.length < 66) return null;
    final duoqianId = bytes.sublist(2, 34);
    final beneficiaryId = bytes.sublist(34, 66);
    final duoqian = Keyring().encodeAddress(duoqianId.toList(), _ss58Prefix);
    final beneficiary =
        Keyring().encodeAddress(beneficiaryId.toList(), _ss58Prefix);
    return DecodedPayload(
      action: 'propose_close',
      summary: '提案关闭多签 ${_truncateAddress(duoqian)}',
      fields: {
        'duoqian_address': duoqian,
        'beneficiary': beneficiary,
      },
    );
  }

  // ---------------------------------------------------------------------------
  // DuoqianTransfer(19) / propose_safety_fund(3)
  // 格式：[19][3][beneficiary:32][amount:u128][BoundedVec remark]
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
  // DuoqianTransfer(19) / propose_sweep(5)
  // 格式：[19][5][institution:48][amount:u128]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeProposeSweep(Uint8List bytes) {
    if (bytes.length < 66) return null;
    var offset = 2;
    final institutionBytes = bytes.sublist(offset, offset + 48);
    offset += 48;
    var endIndex = 48;
    while (endIndex > 0 && institutionBytes[endIndex - 1] == 0) {
      endIndex--;
    }
    final shenfenId = endIndex > 0
        ? String.fromCharCodes(institutionBytes.sublist(0, endIndex))
        : '';
    final bankName = institutionName(shenfenId) ?? shenfenId;
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
  // AdminsChange(12) / propose_admin_replacement(0)
  // 格式：[12][0][org:u8][institution:48][old_admin:32][new_admin:32]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeProposeAdminReplacement(Uint8List bytes) {
    if (bytes.length < 115) return null;
    var offset = 2;
    final org = bytes[offset];
    offset += 1;
    offset += 48; // institution 跳过
    final oldAdminId = bytes.sublist(offset, offset + 32);
    offset += 32;
    final newAdminId = bytes.sublist(offset, offset + 32);
    final oldAdmin = Keyring().encodeAddress(oldAdminId.toList(), _ss58Prefix);
    final newAdmin = Keyring().encodeAddress(newAdminId.toList(), _ss58Prefix);
    return DecodedPayload(
      action: 'propose_admin_replacement',
      summary: '${_orgName(org)} 替换管理员',
      fields: {
        'org': _orgName(org),
        'old_admin': oldAdmin,
        'new_admin': newAdmin,
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
    final shenfenId = endIndex > 0
        ? String.fromCharCodes(institutionBytes.sublist(0, endIndex))
        : '';
    final keyBytes = bytes.sublist(offset, offset + 32);
    final keyHex =
        keyBytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
    return DecodedPayload(
      action: 'propose_replace_grandpa_key',
      summary: 'GRANDPA 密钥替换提案',
      fields: {
        'institution': shenfenId,
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
  // 格式：[21][50][Vec sfid_id][Vec peer_id][Vec rpc_domain][u16 rpc_port]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeRegisterClearingBank(Uint8List bytes) {
    var offset = 2;
    final (sfidId, sfidNext) = _readUtf8Vec(bytes, offset);
    if (sfidId == null) return null;
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
      summary: '声明清算行节点 $sfidId @ $rpcDomain:$rpcPort',
      fields: {
        'sfid_id': sfidId,
        'peer_id': peerId,
        'rpc_domain': rpcDomain,
        'rpc_port': rpcPort.toString(),
      },
    );
  }

  // ---------------------------------------------------------------------------
  // OffchainTransaction(21) / update_clearing_bank_endpoint(51)
  // 格式：[21][51][Vec sfid_id][Vec new_domain][u16 new_port]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeUpdateClearingBankEndpoint(Uint8List bytes) {
    var offset = 2;
    final (sfidId, sfidNext) = _readUtf8Vec(bytes, offset);
    if (sfidId == null) return null;
    offset = sfidNext;
    final (newDomain, domainNext) = _readUtf8Vec(bytes, offset);
    if (newDomain == null) return null;
    offset = domainNext;
    if (offset + 2 > bytes.length) return null;
    final newPort = bytes[offset] | (bytes[offset + 1] << 8);

    return DecodedPayload(
      action: 'update_clearing_bank_endpoint',
      summary: '更新清算行 $sfidId 端点 → $newDomain:$newPort',
      fields: {
        'sfid_id': sfidId,
        'new_domain': newDomain,
        'new_port': newPort.toString(),
      },
    );
  }

  // ---------------------------------------------------------------------------
  // OffchainTransaction(21) / unregister_clearing_bank(52)
  // 格式：[21][52][Vec sfid_id]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeUnregisterClearingBank(Uint8List bytes) {
    final (sfidId, _) = _readUtf8Vec(bytes, 2);
    if (sfidId == null) return null;
    return DecodedPayload(
      action: 'unregister_clearing_bank',
      summary: '注销清算行节点 $sfidId',
      fields: {
        'sfid_id': sfidId,
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
        return '注册多签机构';
      default:
        return '机构$org';
    }
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
