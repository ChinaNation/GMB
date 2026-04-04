import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:wuminapp_mobile/trade/offchain/clearing_banks.dart';

/// 链下快捷支付 RPC 调用模块。
///
/// 通过 WSS 连接省储行节点，调用 offchain_submitSignedTx / offchain_queryTxStatus。
class OffchainRpc {
  /// 链下支付 payload 标识（pallet_index=21, call_index=99）。
  static const int offchainPalletIndex = 21;
  static const int offchainPayCallIndex = 99;

  /// 提交签名交易到省储行节点。
  ///
  /// [bankShenfenId] 省储行 shenfen_id（用于查找 wssUrl）。
  /// [payerAddress] 付款方 SS58 地址。
  /// [recipientAddress] 收款方 SS58 地址。
  /// [amountFen] 金额（分）。
  /// [feeFen] 手续费（分）。
  /// [signature] 交易签名（hex）。
  /// [txId] 交易唯一标识（hex）。
  static Future<OffchainTxReceipt> submitSignedTx({
    required String bankShenfenId,
    required String payerAddress,
    required String recipientAddress,
    required int amountFen,
    required int feeFen,
    required String signature,
    required String txId,
  }) async {
    final bank = findClearingBank(bankShenfenId);
    if (bank == null) {
      return OffchainTxReceipt(
        txId: txId,
        status: OffchainTxStatus.failed,
        message: '未知的清算省储行',
      );
    }
    if (!bank.enabled) {
      return OffchainTxReceipt(
        txId: txId,
        status: OffchainTxStatus.failed,
        message: '该省储行链下清算未开通',
      );
    }

    try {
      final result = await _callRpc(bank.wssUrl, 'offchain_submitSignedTx', {
        'bank': bankShenfenId,
        'payer': payerAddress,
        'recipient': recipientAddress,
        'amount_fen': amountFen,
        'fee_fen': feeFen,
        'signature': signature,
        'tx_id': txId,
      });

      final status = result['status'] as String? ?? '';
      return OffchainTxReceipt(
        txId: txId,
        status: status == 'confirmed'
            ? OffchainTxStatus.confirmed
            : OffchainTxStatus.failed,
        message: result['message'] as String?,
      );
    } catch (e) {
      return OffchainTxReceipt(
        txId: txId,
        status: OffchainTxStatus.failed,
        message: '提交失败：$e',
      );
    }
  }

  /// 查询链下交易状态。
  static Future<OffchainTxReceipt> queryTxStatus(
    String bankShenfenId,
    String txId,
  ) async {
    final bank = findClearingBank(bankShenfenId);
    if (bank == null) {
      return OffchainTxReceipt(
        txId: txId,
        status: OffchainTxStatus.failed,
        message: '未知的清算省储行',
      );
    }

    try {
      final result = await _callRpc(bank.wssUrl, 'offchain_queryTxStatus', {
        'tx_id': txId,
      });

      final status = result['status'] as String? ?? 'unknown';
      return OffchainTxReceipt(
        txId: txId,
        status: switch (status) {
          'confirmed' => OffchainTxStatus.confirmed,
          'onchain' => OffchainTxStatus.onchain,
          _ => OffchainTxStatus.failed,
        },
        message: result['message'] as String?,
      );
    } catch (e) {
      return OffchainTxReceipt(
        txId: txId,
        status: OffchainTxStatus.failed,
        message: '查询失败：$e',
      );
    }
  }

  /// 查询省储行的链下交易费率（bp）。
  ///
  /// 通过 WSS 调用省储行节点 offchain_queryInstitutionRate。
  /// 返回费率 bp（1-10），查询失败时默认返回 1 bp。
  static Future<int> queryInstitutionRate(String bankShenfenId) async {
    final bank = findClearingBank(bankShenfenId);
    if (bank == null) return 1;

    try {
      final result = await _callRpc(
        bank.wssUrl,
        'offchain_queryInstitutionRate',
        {},
      );
      return (result['rate_bp'] as num?)?.toInt() ?? 1;
    } catch (_) {
      return 1; // 查询失败默认 1 bp
    }
  }

  /// 根据真实费率计算链下交易手续费（元）。
  ///
  /// [amountYuan] 支付金额（元）。
  /// [rateBp] 省储行费率（bp，从 queryInstitutionRate 获取）。
  /// 最低 0.01 元。与链上 pallet round_div 保持一致（四舍五入到分）。
  static double calculateOffchainFeeYuan(double amountYuan, int rateBp) {
    // 转为整数 fen 运算，避免浮点精度偏差
    final amountFen = (amountYuan * 100).round();
    final numerator = amountFen * rateBp;
    final quotient = numerator ~/ 10000;
    final remainder = numerator % 10000;
    // 四舍五入：remainder >= 5000 则进位
    final feeFen = remainder >= 5000 ? quotient + 1 : quotient;
    final result = feeFen < 1 ? 1 : feeFen; // 最低 1 fen = 0.01 元
    return result / 100.0;
  }

  /// 构造链下支付 payload（pallet=21, call=99）。
  ///
  /// 格式：[21][99][payer:32][recipient:32][amount_fen:u128_LE][fee_fen:u128_LE][tx_id:32][bank:48]
  /// 总长度 178 字节。
  static Uint8List buildPayload({
    required Uint8List payerPubkey,
    required Uint8List recipientPubkey,
    required int amountFen,
    required int feeFen,
    required Uint8List txIdBytes,
    required String bankShenfenId,
  }) {
    final buffer = ByteData(178);
    final bytes = buffer.buffer.asUint8List();

    // pallet_index + call_index
    bytes[0] = offchainPalletIndex;
    bytes[1] = offchainPayCallIndex;

    // payer: 32 bytes
    bytes.setRange(2, 34, payerPubkey);

    // recipient: 32 bytes
    bytes.setRange(34, 66, recipientPubkey);

    // amount_fen: u128 LE (16 bytes)
    _writeU128LE(bytes, 66, BigInt.from(amountFen));

    // fee_fen: u128 LE (16 bytes)
    _writeU128LE(bytes, 82, BigInt.from(feeFen));

    // tx_id: 32 bytes
    bytes.setRange(98, 130, txIdBytes);

    // bank: shenfen_id 补零到 48 字节
    final bankBytes = bankShenfenId.codeUnits;
    for (var i = 0; i < 48; i++) {
      bytes[130 + i] = i < bankBytes.length ? bankBytes[i] : 0;
    }

    return bytes;
  }

  // ──── 内部方法 ────

  /// 通过 WSS 调用 JSON-RPC 方法。
  static Future<Map<String, dynamic>> _callRpc(
    String wssUrl,
    String method,
    Map<String, dynamic> params,
  ) async {
    final ws = await WebSocket.connect(wssUrl)
        .timeout(const Duration(seconds: 10));

    try {
      final request = jsonEncode({
        'jsonrpc': '2.0',
        'id': 1,
        'method': method,
        'params': [params],
      });
      ws.add(request);

      final response = await ws.first.timeout(const Duration(seconds: 15));
      final json = jsonDecode(response as String) as Map<String, dynamic>;

      if (json.containsKey('error')) {
        final error = json['error'] as Map<String, dynamic>;
        throw Exception(error['message'] ?? '未知 RPC 错误');
      }

      return (json['result'] as Map<String, dynamic>?) ?? {};
    } finally {
      await ws.close();
    }
  }

  /// 写入 u128 little-endian（16 字节）。
  static void _writeU128LE(Uint8List bytes, int offset, BigInt value) {
    for (var i = 0; i < 16; i++) {
      bytes[offset + i] = (value >> (i * 8)).toInt() & 0xFF;
    }
  }
}

/// 链下交易状态。
enum OffchainTxStatus {
  /// 已确认（省储行链下确认，待上链）。
  confirmed,

  /// 已上链（批量打包完成）。
  onchain,

  /// 失败。
  failed,
}

/// 链下交易确认回执。
class OffchainTxReceipt {
  const OffchainTxReceipt({
    required this.txId,
    required this.status,
    this.message,
  });

  final String txId;
  final OffchainTxStatus status;
  final String? message;
}
