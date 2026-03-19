import 'dart:convert';

import 'package:wuminapp_mobile/qr/qr_protocols.dart';

/// 收款码数据模型。
///
/// 收款方生成并展示，付款方扫码后预填转账表单。
class TransferQrPayload {
  const TransferQrPayload({
    required this.to,
    this.name,
    this.amount,
    this.symbol = 'GMB',
    this.memo,
    this.bank,
  });

  /// 协议标识。
  static const String protocol = QrProtocols.transfer;

  /// 收款地址（SS58 格式）。
  final String to;

  /// 钱包名称（= 用户昵称），通讯录扫码时读取。
  final String? name;

  /// 金额（字符串避免浮点精度问题）。
  ///
  /// 为空或 null 时由付款方手动输入。
  final String? amount;

  /// 币种，默认 `GMB`。
  final String symbol;

  /// 备注（展示用）。
  final String? memo;

  /// 清算省储行标识（预留，链下支付用）。
  final String? bank;

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'proto': protocol,
      'to': to,
      'name': name ?? '',
      'amount': amount ?? '',
      'symbol': symbol,
      'memo': memo ?? '',
      'bank': bank ?? '',
    };
  }

  String toRawJson() => jsonEncode(toJson());

  /// 从 JSON Map 解析收款码。
  static TransferQrPayload fromJson(Map<String, dynamic> data) {
    final to = (data['to'] ?? '').toString().trim();
    if (to.isEmpty) {
      throw const FormatException('收款码缺少收款地址');
    }

    final name = _normalizeOptional(data['name']);
    final amount = _normalizeOptional(data['amount']);
    final symbol = (data['symbol'] ?? 'GMB').toString().trim().toUpperCase();
    final memo = _normalizeOptional(data['memo']);
    final bank = _normalizeOptional(data['bank']);

    return TransferQrPayload(
      to: to,
      name: name,
      amount: amount,
      symbol: symbol.isEmpty ? 'GMB' : symbol,
      memo: memo,
      bank: bank,
    );
  }

  /// 从原始 JSON 字符串解析。
  static TransferQrPayload parse(String raw) {
    final decoded = jsonDecode(raw);
    if (decoded is! Map<String, dynamic>) {
      throw const FormatException('收款码数据格式错误');
    }
    final proto = (decoded['proto'] ?? '').toString();
    if (proto != protocol) {
      throw const FormatException('不是收款码二维码');
    }
    return fromJson(decoded);
  }

  static String? _normalizeOptional(Object? value) {
    final normalized = value?.toString().trim() ?? '';
    return normalized.isEmpty ? null : normalized;
  }
}
