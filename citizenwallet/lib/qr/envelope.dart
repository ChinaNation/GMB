import 'dart:convert';

import 'package:citizenwallet/qr/qr_protocols.dart';
import 'package:citizenwallet/qr/bodies/sign_request_body.dart';
import 'package:citizenwallet/qr/bodies/sign_response_body.dart';
import 'package:citizenwallet/qr/bodies/user_contact_body.dart';
import 'package:citizenwallet/qr/bodies/user_transfer_body.dart';

/// QR_V1 统一 envelope。与 citizenapp/lib/qr/envelope.dart 逐字节一致。
class QrEnvelope<T extends QrBody> {
  const QrEnvelope({
    required this.kind,
    required this.id,
    required this.issuedAt,
    required this.expiresAt,
    required this.body,
  });

  final QrKind kind;
  final String? id;
  final int? issuedAt;
  final int? expiresAt;
  final T body;

  Map<String, dynamic> toJson() {
    final map = <String, dynamic>{
      'p': QrProtocols.v1,
      'k': kind.code,
    };
    if (kind.temporary) {
      if (id == null || expiresAt == null) {
        throw StateError('临时码 ${kind.code} 必须包含 i/e');
      }
      map['i'] = id;
      map['e'] = expiresAt;
    } else {
      if (id != null || issuedAt != null || expiresAt != null) {
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
    if (proto != QrProtocols.v1) {
      throw FormatException('p 必须为 ${QrProtocols.v1},实际: $proto');
    }
    final kindWire = data['k'];
    final kind = QrKind.fromWire(kindWire);

    String? id;
    int? issuedAt;
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
        body = UserTransferBody.fromJson(bodyRaw);
    }

    return QrEnvelope<QrBody>(
      kind: kind,
      id: id,
      issuedAt: issuedAt,
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
