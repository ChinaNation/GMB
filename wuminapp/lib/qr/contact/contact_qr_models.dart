import 'dart:convert';

import 'package:wuminapp_mobile/qr/qr_protocols.dart';

/// 用户协议 QR 码数据模型。
///
/// 统一用于联系人交换、付款码等场景，通过 [purpose] 区分用途。
class UserQrPayload {
  const UserQrPayload({
    required this.address,
    required this.name,
    this.purpose = 'contact',
    this.amount,
  });

  /// 协议标识。
  static const String protocol = QrProtocols.user;

  /// 链上地址（SS58 格式）。
  final String address;

  /// 用户昵称。
  final String name;

  /// 用途：contact（联系人）/ transfer（付款）。
  final String purpose;

  /// 转账金额（purpose=transfer 时使用）。
  final String? amount;

  Map<String, dynamic> toJson() {
    final map = <String, dynamic>{
      'proto': protocol,
      'address': address,
      'name': name,
      'purpose': purpose,
    };
    if (amount != null && amount!.isNotEmpty) {
      map['amount'] = amount;
    }
    return map;
  }

  String toRawJson() => jsonEncode(toJson());

  /// 从 JSON Map 解析。
  static UserQrPayload fromJson(Map<String, dynamic> data) {
    final address = (data['address'] ?? '').toString().trim();
    final name = (data['name'] ?? '').toString().trim();
    if (address.isEmpty || name.isEmpty) {
      throw const FormatException('用户码缺少必要字段');
    }
    final purpose = (data['purpose'] ?? 'contact').toString().trim();
    final amount = data['amount']?.toString().trim();
    return UserQrPayload(
      address: address,
      name: name,
      purpose: purpose,
      amount: (amount != null && amount.isNotEmpty) ? amount : null,
    );
  }

  /// 从原始 JSON 字符串解析。
  static UserQrPayload parse(String raw) {
    final decoded = jsonDecode(raw);
    if (decoded is! Map<String, dynamic>) {
      throw const FormatException('用户码数据格式错误');
    }
    final proto = (decoded['proto'] ?? '').toString();
    if (proto != protocol) {
      throw const FormatException('不是用户码二维码');
    }
    return fromJson(decoded);
  }
}
