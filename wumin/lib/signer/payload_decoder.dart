import 'dart:convert';
import 'dart:typed_data';

import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import '../chain/chain_constants.dart';
import '../chain/clearing_banks.dart';
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
  static DecodedPayload? decode(String payloadHex, {int? specVersion}) {
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
      action: 'developer_upgrade',
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
    final bankName = clearingBankName(shenfenId) ?? shenfenId;

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
    final bankName = clearingBankName(shenfenId) ?? shenfenId;

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

  /// 分 → 元字符串。
  static String _fenToYuan(BigInt fen) {
    final yuan = fen ~/ BigInt.from(100);
    final remainder = (fen % BigInt.from(100)).toInt().abs();
    return '$yuan.${remainder.toString().padLeft(2, '0')}';
  }

  /// 截断地址显示。
  static String _truncateAddress(String addr) {
    if (addr.length <= 16) return addr;
    return '${addr.substring(0, 8)}...${addr.substring(addr.length - 6)}';
  }
}
