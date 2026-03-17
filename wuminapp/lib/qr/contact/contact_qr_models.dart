import 'dart:convert';

import 'package:wuminapp_mobile/qr/qr_protocols.dart';

/// 用户码数据模型。
///
/// 用户展示个人二维码，其他用户扫码后加入通讯录。
class ContactQrPayload {
  const ContactQrPayload({
    required this.address,
    required this.name,
  });

  /// 协议标识。
  static const String protocol = QrProtocols.contact;

  /// 链上地址（SS58 格式）。
  final String address;

  /// 用户昵称。
  final String name;

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'proto': protocol,
      'address': address,
      'name': name,
    };
  }

  String toRawJson() => jsonEncode(toJson());

  /// 从 JSON Map 解析用户码。
  ///
  /// 同时兼容旧版 `WUMINAPP_USER_CARD_V1` 格式。
  static ContactQrPayload fromJson(Map<String, dynamic> data) {
    final proto = (data['proto'] ?? data['type'] ?? '').toString();

    // 旧版格式兼容：WUMINAPP_USER_CARD_V1 使用 account_pubkey + nickname。
    if (proto == QrProtocols.legacyUserCard) {
      final pubkey = (data['account_pubkey'] ?? '').toString().trim();
      final nickname = (data['nickname'] ?? '').toString().trim();
      if (pubkey.isEmpty || nickname.isEmpty) {
        throw const FormatException('用户码缺少必要字段');
      }
      // 旧版使用 pubkey 作为标识，新版统一用 address。
      // 此处保留 pubkey 原值，由调用方处理兼容。
      return ContactQrPayload(address: pubkey, name: nickname);
    }

    // 新版格式：WUMINAPP_CONTACT_V1。
    final address = (data['address'] ?? '').toString().trim();
    final name = (data['name'] ?? '').toString().trim();
    if (address.isEmpty || name.isEmpty) {
      throw const FormatException('用户码缺少必要字段');
    }
    return ContactQrPayload(address: address, name: name);
  }

  /// 从原始 JSON 字符串解析。
  static ContactQrPayload parse(String raw) {
    final decoded = jsonDecode(raw);
    if (decoded is! Map<String, dynamic>) {
      throw const FormatException('用户码数据格式错误');
    }
    final proto = (decoded['proto'] ?? decoded['type'] ?? '').toString();
    if (proto != protocol && proto != QrProtocols.legacyUserCard) {
      throw const FormatException('不是用户码二维码');
    }
    return fromJson(decoded);
  }
}
