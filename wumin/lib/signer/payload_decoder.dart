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

  static DecodedPayload? decode(String payloadHex, {int? specVersion}) {
    // 先尝试解码非链上交易：管理员激活 payload。
    // 激活 payload 格式：GMB_ACTIVATE(12B) + shenfen_id(48B) + timestamp(8B) + nonce(16B) = 84B
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
    } catch (_) {
      // 非激活 payload，继续正常解码
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

      // DuoqianTransferPow / propose_transfer
      if (palletIndex == PalletRegistry.duoqianTransferPowPallet &&
          callIndex == PalletRegistry.proposeTransferCall) {
        return _decodeProposeTransfer(bytes);
      }

      // DuoqianTransferPow / vote_transfer
      if (palletIndex == PalletRegistry.duoqianTransferPowPallet &&
          callIndex == PalletRegistry.voteTransferCall) {
        return _decodeVoteTransfer(bytes);
      }

      // VotingEngineSystem / joint_vote
      if (palletIndex == PalletRegistry.votingEngineSystemPallet &&
          callIndex == PalletRegistry.jointVoteCall) {
        return _decodeJointVote(bytes);
      }

      // VotingEngineSystem / citizen_vote
      if (palletIndex == PalletRegistry.votingEngineSystemPallet &&
          callIndex == PalletRegistry.citizenVoteCall) {
        return _decodeCitizenVote(bytes);
      }

      // RuntimeRootUpgrade / propose_runtime_upgrade
      if (palletIndex == PalletRegistry.runtimeRootUpgradePallet &&
          callIndex == PalletRegistry.proposeRuntimeUpgradeCall) {
        return _decodeProposeRuntimeUpgrade(bytes);
      }

      // RuntimeRootUpgrade / developer_direct_upgrade
      if (palletIndex == PalletRegistry.runtimeRootUpgradePallet &&
          callIndex == PalletRegistry.developerDirectUpgradeCall) {
        return _decodeDeveloperUpgrade(bytes);
      }

      // OffchainTransactionPos / bind_clearing_institution
      if (palletIndex == PalletRegistry.offchainTransactionPosPallet &&
          callIndex == PalletRegistry.bindClearingInstitutionCall) {
        return _decodeBindClearingInstitution(bytes);
      }

      // OffchainTransactionPos / offchain_pay（链下支付授权）
      if (palletIndex == PalletRegistry.offchainTransactionPosPallet &&
          callIndex == PalletRegistry.offchainPayCall) {
        return _decodeOffchainPay(bytes);
      }

      // OffchainTransactionPos / propose_institution_rate
      if (palletIndex == PalletRegistry.offchainTransactionPosPallet &&
          callIndex == PalletRegistry.proposeInstitutionRateCall) {
        return _decodeProposeInstitutionRate(bytes);
      }

      // OffchainTransactionPos / vote_institution_rate
      if (palletIndex == PalletRegistry.offchainTransactionPosPallet &&
          callIndex == PalletRegistry.voteInstitutionRateCall) {
        return _decodeVoteProposal(bytes, 'vote_institution_rate', '费率提案');
      }

      // ── DuoqianManagePow(17) ──
      if (palletIndex == PalletRegistry.duoqianManagePowPallet) {
        if (callIndex == PalletRegistry.proposeCreateCall) return _decodeProposeCreate(bytes);
        if (callIndex == PalletRegistry.proposeCloseCall) return _decodeProposeClose(bytes);
        if (callIndex == PalletRegistry.voteCreateCall) return _decodeVoteProposal(bytes, 'vote_create', '多签创建提案');
        if (callIndex == PalletRegistry.proposeCreatePersonalCall) return _decodeProposeCreatePersonal(bytes);
        if (callIndex == PalletRegistry.voteCloseCall) return _decodeVoteProposal(bytes, 'vote_close', '多签关闭提案');
      }

      // ── DuoqianTransferPow(19) 补充 ──
      if (palletIndex == PalletRegistry.duoqianTransferPowPallet) {
        if (callIndex == PalletRegistry.proposeSafetyFundCall) return _decodeProposeSafetyFund(bytes);
        if (callIndex == PalletRegistry.voteSafetyFundCall) return _decodeVoteProposal(bytes, 'vote_safety_fund_transfer', '安全基金提案');
        if (callIndex == PalletRegistry.proposeSweepCall) return _decodeProposeSweep(bytes);
        if (callIndex == PalletRegistry.voteSweepCall) return _decodeVoteProposal(bytes, 'vote_sweep_to_main', '手续费划转提案');
      }

      // ── ResolutionDestroGov(14) ──
      if (palletIndex == PalletRegistry.resolutionDestroGovPallet) {
        if (callIndex == PalletRegistry.proposeDestroyCall) return _decodeProposeDestroy(bytes);
        if (callIndex == PalletRegistry.voteDestroyCall) return _decodeVoteProposal(bytes, 'vote_destroy', '决议销毁提案');
      }

      // ── AdminsOriginGov(12) ──
      if (palletIndex == PalletRegistry.adminsOriginGovPallet) {
        if (callIndex == PalletRegistry.proposeAdminReplacementCall) return _decodeProposeAdminReplacement(bytes);
        if (callIndex == PalletRegistry.voteAdminReplacementCall) return _decodeVoteProposal(bytes, 'vote_admin_replacement', '管理员替换提案');
      }

      // ── GrandpaKeyGov(16) ──
      if (palletIndex == PalletRegistry.grandpaKeyGovPallet) {
        if (callIndex == PalletRegistry.proposeKeyChangeCall) return _decodeProposeKeyChange(bytes);
        if (callIndex == PalletRegistry.voteKeyChangeCall) return _decodeVoteProposal(bytes, 'vote_key_change', 'GRANDPA 密钥提案');
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
    final toAddress = Keyring().encodeAddress(toAccountId.toList(), _ss58Prefix);

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
  // DuoqianTransferPow(19) / propose_transfer(0)
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
      summary: '$orgName 提案转账 $amountYuan GMB 给 ${_truncateAddress(beneficiary)}',
      fields: {
        'org': orgName,
        'beneficiary': beneficiary,
        'amount_yuan': '$amountYuan GMB',
        'remark': remark,
      },
    );
  }

  // ---------------------------------------------------------------------------
  // DuoqianTransferPow(19) / vote_transfer(1)
  // 格式：[0x13][0x01][proposal_id:u64_le][approve:bool]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeVoteTransfer(Uint8List bytes) {
    // 2 + 8 + 1 = 11
    if (bytes.length < 11) return null;

    final proposalId = _readU64Le(bytes, 2);
    final approve = bytes[10] != 0;
    final voteText = approve ? '赞成' : '反对';

    return DecodedPayload(
      action: 'vote_transfer',
      summary: '转账提案 #$proposalId 投票：$voteText',
      fields: {
        'proposal_id': proposalId.toString(),
        'approve': approve.toString(),
      },
    );
  }

  // ---------------------------------------------------------------------------
  // VotingEngineSystem(9) / joint_vote(3)
  // 格式：[0x09][0x03][proposal_id:u64_le][institution:48][approve:bool]
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
  // VotingEngineSystem / citizen_vote
  // 格式：[pallet][call][proposal_id:u64_le][binding_id:32][Vec nonce][Vec sig][approve:bool]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeCitizenVote(Uint8List bytes) {
    // 最小：2 + 8 + 32 + 1(Vec nonce compact len) + 1(Vec sig compact len) + 1(approve) = 45
    if (bytes.length < 45) return null;

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
      },
    );
  }

  // ---------------------------------------------------------------------------
  // RuntimeRootUpgrade(13) / propose_runtime_upgrade(0)
  // 格式：[13][0][Compact<u32> reason_len][reason_bytes][Compact<u32> wasm_len][wasm_bytes][u64_le eligible_total][Compact nonce][Compact sig]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeProposeRuntimeUpgrade(Uint8List bytes) {
    if (bytes.length < 3) return null;

    var offset = 2; // 跳过 pallet_index + call_index

    // reason: Vec<u8>
    final (reasonLen, reasonLenSize) = _decodeCompactU32(bytes, offset);
    offset += reasonLenSize;
    var reason = '';
    if (reasonLen > 0 && offset + reasonLen <= bytes.length) {
      reason = String.fromCharCodes(
        bytes.sublist(offset, offset + reasonLen),
      );
    }
    offset += reasonLen;

    // wasm: Vec<u8>
    if (offset >= bytes.length) return null;
    final (wasmLen, wasmLenSize) = _decodeCompactU32(bytes, offset);
    offset += wasmLenSize;

    final sizeKb = (wasmLen / 1024).toStringAsFixed(0);
    final sizeMb = (wasmLen / (1024 * 1024)).toStringAsFixed(2);
    final sizeDisplay = wasmLen > 1024 * 1024 ? '$sizeMb MB' : '$sizeKb KB';

    return DecodedPayload(
      action: 'propose_runtime_upgrade',
      summary: 'Runtime 升级提案（WASM $sizeDisplay）',
      fields: {
        'reason': reason,
        'wasm_size': sizeDisplay,
      },
    );
  }

  // ---------------------------------------------------------------------------
  // RuntimeRootUpgrade(13) / developer_direct_upgrade(2)
  // 格式：[13][2][Compact<u32> wasm_len][wasm_bytes...]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeDeveloperUpgrade(Uint8List bytes) {
    if (bytes.length < 3) return null;

    var offset = 2; // 跳过 pallet_index + call_index
    final (wasmLen, lenSize) = _decodeCompactU32(bytes, offset);
    offset += lenSize;

    // 计算 wasm 大小（KB 或 MB）
    final sizeKb = (wasmLen / 1024).toStringAsFixed(0);
    final sizeMb = (wasmLen / (1024 * 1024)).toStringAsFixed(2);
    final sizeDisplay = wasmLen > 1024 * 1024 ? '$sizeMb MB' : '$sizeKb KB';

    return DecodedPayload(
      action: 'developer_direct_upgrade',
      summary: '开发者直升 Runtime（WASM $sizeDisplay）',
      fields: {
        'wasm_size': sizeDisplay,
        'wasm_bytes': wasmLen.toString(),
      },
    );
  }

  // ---------------------------------------------------------------------------
  // OffchainTransactionPos(21) / offchain_pay(99) — 链下支付授权
  // 格式：[21][99][payer:32][recipient:32][amount_fen:u128_LE][fee_fen:u128_LE][tx_id:32][bank:48]
  // 总长度 178 字节
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeOffchainPay(Uint8List bytes) {
    if (bytes.length < 178) return null;

    // payer: 32 bytes (offset 2)
    final payerAccountId = bytes.sublist(2, 34);
    final payerAddress =
        Keyring().encodeAddress(payerAccountId.toList(), _ss58Prefix);

    // recipient: 32 bytes (offset 34)
    final recipientAccountId = bytes.sublist(34, 66);
    final recipientAddress =
        Keyring().encodeAddress(recipientAccountId.toList(), _ss58Prefix);

    // amount_fen: u128 LE (offset 66, 16 bytes)
    final amountFen = _readU128Le(bytes, 66);
    final amountYuan = _fenToYuan(amountFen);

    // fee_fen: u128 LE (offset 82, 16 bytes)
    final feeFen = _readU128Le(bytes, 82);
    final feeYuan = _fenToYuan(feeFen);

    // bank: 48 bytes (offset 130), shenfen_id 补零
    final bankBytes = bytes.sublist(130, 178);
    var endIndex = 48;
    while (endIndex > 0 && bankBytes[endIndex - 1] == 0) {
      endIndex--;
    }
    final shenfenId =
        endIndex > 0 ? String.fromCharCodes(bankBytes.sublist(0, endIndex)) : '';
    final bankName = institutionName(shenfenId) ?? shenfenId;

    return DecodedPayload(
      action: 'offchain_pay',
      summary: '扫码支付 $amountYuan GMB 给 ${_truncateAddress(recipientAddress)}',
      fields: {
        'from': payerAddress,
        'to': recipientAddress,
        'amount_yuan': '$amountYuan GMB',
        'fee_yuan': '$feeYuan GMB',
        'bank': bankName,
      },
    );
  }

  // ---------------------------------------------------------------------------
  // OffchainTransactionPos(21) / bind_clearing_institution(9)
  // 格式：[0x15][0x09][institution: [u8; 48]]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeBindClearingInstitution(Uint8List bytes) {
    // 2 (pallet+call) + 48 (InstitutionPalletId) = 50
    if (bytes.length < 50) return null;

    // institution: [u8; 48] — shenfen_id 补零到 48 字节
    final institutionBytes = bytes.sublist(2, 50);
    // 去尾部零字节还原 shenfen_id 字符串
    var endIndex = 48;
    while (endIndex > 0 && institutionBytes[endIndex - 1] == 0) {
      endIndex--;
    }
    if (endIndex == 0) return null;
    final shenfenId = String.fromCharCodes(institutionBytes.sublist(0, endIndex));
    final bankName = institutionName(shenfenId) ?? shenfenId;

    return DecodedPayload(
      action: 'bind_clearing',
      summary: '绑定清算行：$bankName',
      fields: {
        'institution': bankName,
        'shenfen_id': shenfenId,
      },
    );
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
  // OffchainTransactionPos(21) / propose_institution_rate(1)
  // 格式：[21][1][institution:48][new_rate_bp:u32_LE]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeProposeInstitutionRate(Uint8List bytes) {
    // 2 (pallet+call) + 48 (institution) + 4 (u32) = 54
    if (bytes.length < 54) return null;

    // institution: [u8; 48] — shenfen_id 补零到 48 字节
    final institutionBytes = bytes.sublist(2, 50);
    var endIndex = 48;
    while (endIndex > 0 && institutionBytes[endIndex - 1] == 0) {
      endIndex--;
    }
    if (endIndex == 0) return null;
    final shenfenId = String.fromCharCodes(institutionBytes.sublist(0, endIndex));

    // new_rate_bp: u32 LE
    final rateBp = bytes[50] |
        (bytes[51] << 8) |
        (bytes[52] << 16) |
        (bytes[53] << 24);
    final ratePercent = (rateBp / 100.0).toStringAsFixed(2);

    return DecodedPayload(
      action: 'propose_institution_rate',
      summary: '$shenfenId 提案设置链下交易费率为 $ratePercent%',
      fields: {
        'institution': shenfenId,
        'new_rate_bp': '$rateBp bp ($ratePercent%)',
      },
    );
  }

  // ---------------------------------------------------------------------------
  // 通用投票解码（proposal_id:u64 + approve:bool）
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeVoteProposal(Uint8List bytes, String action, String label) {
    if (bytes.length < 11) return null;
    final proposalId = _readU64Le(bytes, 2);
    final approve = bytes[10] != 0;
    return DecodedPayload(
      action: action,
      summary: '$label #$proposalId 投票：${approve ? "赞成" : "反对"}',
      fields: {
        'proposal_id': proposalId.toString(),
        'approve': approve.toString(),
      },
    );
  }

  // ---------------------------------------------------------------------------
  // DuoqianManagePow(17) / propose_create(0)
  // 格式：[17][0][BoundedVec sfid_id][u32 admin_count][BoundedVec<AccountId32> admins][u32 threshold][u128 amount]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeProposeCreate(Uint8List bytes) {
    if (bytes.length < 10) return null;
    var offset = 2;

    // sfid_id: BoundedVec<u8>
    final (sfidLen, sfidLenSize) = _decodeCompactU32(bytes, offset);
    offset += sfidLenSize;
    if (offset + sfidLen > bytes.length) return null;
    final sfidId = utf8.decode(bytes.sublist(offset, offset + sfidLen), allowMalformed: true);
    offset += sfidLen;

    // name: BoundedVec<u8>
    final (nameLen, nameLenSize) = _decodeCompactU32(bytes, offset);
    offset += nameLenSize;
    if (offset + nameLen > bytes.length) return null;
    final name = utf8.decode(bytes.sublist(offset, offset + nameLen), allowMalformed: true);
    offset += nameLen;

    // admin_count: u32
    if (offset + 4 > bytes.length) return null;
    final adminCount = bytes[offset] | (bytes[offset + 1] << 8) | (bytes[offset + 2] << 16) | (bytes[offset + 3] << 24);
    offset += 4;

    // admins: BoundedVec<AccountId32> — 跳过
    final (adminsLen, adminsLenSize) = _decodeCompactU32(bytes, offset);
    offset += adminsLenSize;
    offset += adminsLen * 32;
    if (offset + 4 + 16 > bytes.length) return null;

    // threshold: u32
    final threshold = bytes[offset] | (bytes[offset + 1] << 8) | (bytes[offset + 2] << 16) | (bytes[offset + 3] << 24);
    offset += 4;

    // amount: u128
    final amountFen = _readU128Le(bytes, offset);
    final amountYuan = _fenToYuan(amountFen);

    return DecodedPayload(
      action: 'propose_create',
      summary: '创建多签账户「$name」（$adminCount 管理员，阈值 $threshold，入金 $amountYuan 元）',
      fields: {
        'sfid_id': sfidId,
        'name': name,
        'admin_count': adminCount.toString(),
        'threshold': '$threshold/$adminCount',
        'amount_yuan': '$amountYuan GMB',
      },
    );
  }

  // ---------------------------------------------------------------------------
  // DuoqianManagePow(17) / propose_create_personal(4)
  // 格式：[17][4][BoundedVec name][u32 admin_count][BoundedVec<AccountId32> admins][u32 threshold][u128 amount]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeProposeCreatePersonal(Uint8List bytes) {
    if (bytes.length < 10) return null;
    var offset = 2;

    // name: BoundedVec<u8>
    final (nameLen, nameLenSize) = _decodeCompactU32(bytes, offset);
    offset += nameLenSize;
    if (offset + nameLen > bytes.length) return null;
    final name = utf8.decode(bytes.sublist(offset, offset + nameLen), allowMalformed: true);
    offset += nameLen;

    // admin_count: u32
    if (offset + 4 > bytes.length) return null;
    final adminCount = bytes[offset] | (bytes[offset + 1] << 8) | (bytes[offset + 2] << 16) | (bytes[offset + 3] << 24);
    offset += 4;

    // admins: BoundedVec<AccountId32> — 跳过
    final (adminsLen, adminsLenSize) = _decodeCompactU32(bytes, offset);
    offset += adminsLenSize;
    offset += adminsLen * 32;
    if (offset + 4 + 16 > bytes.length) return null;

    // threshold: u32
    final threshold = bytes[offset] | (bytes[offset + 1] << 8) | (bytes[offset + 2] << 16) | (bytes[offset + 3] << 24);
    offset += 4;

    // amount: u128
    final amountFen = _readU128Le(bytes, offset);
    final amountYuan = _fenToYuan(amountFen);

    return DecodedPayload(
      action: 'propose_create_personal',
      summary: '创建个人多签「$name」（$adminCount 管理员，阈值 $threshold，入金 $amountYuan 元）',
      fields: {
        'name': name,
        'admin_count': adminCount.toString(),
        'threshold': '$threshold/$adminCount',
        'amount_yuan': '$amountYuan GMB',
      },
    );
  }

  // ---------------------------------------------------------------------------
  // DuoqianManagePow(17) / propose_close(1)
  // 格式：[17][1][duoqian_address:32][beneficiary:32]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeProposeClose(Uint8List bytes) {
    if (bytes.length < 66) return null;
    final duoqianId = bytes.sublist(2, 34);
    final beneficiaryId = bytes.sublist(34, 66);
    final duoqian = Keyring().encodeAddress(duoqianId.toList(), _ss58Prefix);
    final beneficiary = Keyring().encodeAddress(beneficiaryId.toList(), _ss58Prefix);
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
  // DuoqianTransferPow(19) / propose_safety_fund(3)
  // 格式：[19][3][beneficiary:32][amount:u128][BoundedVec remark]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeProposeSafetyFund(Uint8List bytes) {
    if (bytes.length < 50) return null;
    var offset = 2;
    final beneficiaryId = bytes.sublist(offset, offset + 32);
    offset += 32;
    final beneficiary = Keyring().encodeAddress(beneficiaryId.toList(), _ss58Prefix);
    final amountFen = _readU128Le(bytes, offset);
    offset += 16;
    final amountYuan = _fenToYuan(amountFen);
    final (remarkLen, remarkLenSize) = _decodeCompactU32(bytes, offset);
    offset += remarkLenSize;
    var remark = '';
    if (remarkLen > 0 && offset + remarkLen <= bytes.length) {
      remark = utf8.decode(bytes.sublist(offset, offset + remarkLen), allowMalformed: true);
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
  // DuoqianTransferPow(19) / propose_sweep(5)
  // 格式：[19][5][institution:48][amount:u128]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeProposeSweep(Uint8List bytes) {
    if (bytes.length < 66) return null;
    var offset = 2;
    final institutionBytes = bytes.sublist(offset, offset + 48);
    offset += 48;
    var endIndex = 48;
    while (endIndex > 0 && institutionBytes[endIndex - 1] == 0) { endIndex--; }
    final shenfenId = endIndex > 0 ? String.fromCharCodes(institutionBytes.sublist(0, endIndex)) : '';
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
  // ResolutionDestroGov(14) / propose_destroy(0)
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
  // AdminsOriginGov(12) / propose_admin_replacement(0)
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
  // GrandpaKeyGov(16) / propose_key_change(0)
  // 格式：[16][0][institution:48][new_key:32]
  // ---------------------------------------------------------------------------
  static DecodedPayload? _decodeProposeKeyChange(Uint8List bytes) {
    if (bytes.length < 82) return null;
    var offset = 2;
    final institutionBytes = bytes.sublist(offset, offset + 48);
    offset += 48;
    var endIndex = 48;
    while (endIndex > 0 && institutionBytes[endIndex - 1] == 0) { endIndex--; }
    final shenfenId = endIndex > 0 ? String.fromCharCodes(institutionBytes.sublist(0, endIndex)) : '';
    final keyBytes = bytes.sublist(offset, offset + 32);
    final keyHex = keyBytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
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
  // 工具方法
  // ---------------------------------------------------------------------------

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
