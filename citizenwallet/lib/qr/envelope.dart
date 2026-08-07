import 'dart:convert';

import 'package:citizenwallet/qr/qr_protocols.dart';
import 'package:citizenwallet/qr/bodies/sign_request_body.dart';
import 'package:citizenwallet/qr/bodies/sign_response_body.dart';
import 'package:citizenwallet/qr/bodies/user_contact_body.dart';
import 'package:citizenwallet/qr/bodies/account_id_code_body.dart';

/// QR_V1 统一 envelope。与 citizenapp/lib/qr/envelope.dart 逐字节一致。
class QrEnvelope<T extends QrBody> {
  const QrEnvelope({
    required this.kind,
    required this.id,
    required this.expiresAt,
    required this.body,
  });

  final QrKind kind;
  final String? id;
  final int? expiresAt;
  final T body;

  Map<String, dynamic> toJson() {
    final map = <String, dynamic>{
      'p': QrProtocols.qrV1,
      'k': kind.code,
    };
    if (kind.temporary) {
      if (id == null || expiresAt == null) {
        throw StateError('临时码 ${kind.code} 必须包含 i/e');
      }
      map['i'] = id;
      map['e'] = expiresAt;
    } else {
      if (id != null || expiresAt != null) {
        throw StateError('固定码 ${kind.code} 不能包含 i/e');
      }
    }
    map['b'] = body.toJson();
    return map;
  }

  String toRawJson() => jsonEncode(toJson());

  static QrEnvelope<QrBody> parse(String raw) {
    final decoded = jsonDecode(raw);
    if (decoded is! Map<String, dynamic>) {
      throw const FormatException('QR 内容不是 JSON 对象');
    }
    return fromJson(decoded);
  }

  static QrEnvelope<QrBody> fromJson(Map<String, dynamic> data) {
    final proto = data['p'];
    if (proto != QrProtocols.qrV1) {
      throw FormatException('p 必须为 ${QrProtocols.qrV1},实际: $proto');
    }
    final kindWire = data['k'];
    final kind = QrKind.fromWire(kindWire);
    requireExactKeys(
      data,
      kind.temporary ? const {'p', 'k', 'i', 'e', 'b'} : const {'p', 'k', 'b'},
      'QR envelope',
    );

    String? id;
    int? expiresAt;
    if (kind.temporary) {
      id = _requireString(data, 'i');
      expiresAt = _requireInt(data, 'e');
    } else {
      if (data.containsKey('i') || data.containsKey('e')) {
        throw FormatException('固定码 ${kind.code} 不应包含 i/e');
      }
    }

    final bodyRaw = data['b'];
    if (bodyRaw is! Map<String, dynamic>) {
      throw const FormatException('缺少 b 对象');
    }

    final QrBody body;
    switch (kind) {
      case QrKind.signRequest:
        body = SignRequestBody.fromJson(bodyRaw);
      case QrKind.signResponse:
        body = SignResponseBody.fromJson(bodyRaw);
      case QrKind.userContact:
        body = UserContactBody.fromJson(bodyRaw);
      case QrKind.userTransfer:
        // 收款码只属于 CitizenApp:公民钱包完全离线、发不了交易,扫它没有任何用途。
        // 按角色边界报明确错误,不静默忽略。
        throw const FormatException('收款码请用「公民」App 扫描');
      case QrKind.accountIdCode:
        body = AccountIdCodeBody.fromJson(bodyRaw);
    }

    return QrEnvelope<QrBody>(
      kind: kind,
      id: id,
      expiresAt: expiresAt,
      body: body,
    );
  }

  static String _requireString(Map<String, dynamic> data, String key) {
    final v = data[key];
    if (v is! String || v.isEmpty) {
      throw FormatException('字段 $key 必填且为非空字符串');
    }
    return v;
  }

  static int _requireInt(Map<String, dynamic> data, String key) {
    final v = data[key];
    if (v is! int) {
      throw FormatException('字段 $key 必填且为整数');
    }
    return v;
  }
}

abstract class QrBody {
  Map<String, dynamic> toJson();
}

/// 严格键集合校验：既拒绝未知字段，也拒绝缺失字段。
void requireExactKeys(
  Map<String, dynamic> data,
  Set<String> expected,
  String context,
) {
  final actual = data.keys.toSet();
  if (actual.length != expected.length ||
      !actual.containsAll(expected) ||
      !expected.containsAll(actual)) {
    throw FormatException('$context 字段集合不符合 QR_V1');
  }
}
