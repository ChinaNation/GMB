import 'dart:convert';

import 'package:wumin/qr/qr_protocols.dart';
import 'package:wumin/qr/bodies/login_challenge_body.dart';
import 'package:wumin/qr/bodies/login_receipt_body.dart';
import 'package:wumin/qr/bodies/sign_request_body.dart';
import 'package:wumin/qr/bodies/sign_response_body.dart';
import 'package:wumin/qr/bodies/user_contact_body.dart';
import 'package:wumin/qr/bodies/user_transfer_body.dart';
import 'package:wumin/qr/bodies/user_duoqian_body.dart';

/// WUMIN_QR_V1 统一 envelope。与 wuminapp/lib/qr/envelope.dart 逐字节一致。
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
      'proto': QrProtocols.v1,
      'kind': kind.wire,
    };
    if (kind.temporary) {
      if (id == null || issuedAt == null || expiresAt == null) {
        throw StateError('临时码 ${kind.wire} 必须包含 id/issued_at/expires_at');
      }
      map['id'] = id;
      map['issued_at'] = issuedAt;
      map['expires_at'] = expiresAt;
    } else {
      if (id != null || issuedAt != null || expiresAt != null) {
        throw StateError('固定码 ${kind.wire} 不能包含 id/issued_at/expires_at');
      }
    }
    map['body'] = body.toJson();
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
    final proto = data['proto'];
    if (proto != QrProtocols.v1) {
      throw FormatException('proto 必须为 ${QrProtocols.v1},实际: $proto');
    }
    final kindWire = data['kind'];
    if (kindWire is! String) {
      throw const FormatException('缺少 kind 字段');
    }
    final kind = QrKind.fromWire(kindWire);

    String? id;
    int? issuedAt;
    int? expiresAt;
    if (kind.temporary) {
      id = _requireString(data, 'id');
      issuedAt = _requireInt(data, 'issued_at');
      expiresAt = _requireInt(data, 'expires_at');
    } else {
      if (data.containsKey('id') ||
          data.containsKey('issued_at') ||
          data.containsKey('expires_at')) {
        throw FormatException('固定码 ${kind.wire} 不应包含 id/issued_at/expires_at');
      }
    }

    final bodyRaw = data['body'];
    if (bodyRaw is! Map<String, dynamic>) {
      throw const FormatException('缺少 body 对象');
    }

    final QrBody body;
    switch (kind) {
      case QrKind.loginChallenge:
        body = LoginChallengeBody.fromJson(bodyRaw);
      case QrKind.loginReceipt:
        body = LoginReceiptBody.fromJson(bodyRaw);
      case QrKind.signRequest:
        body = SignRequestBody.fromJson(bodyRaw);
      case QrKind.signResponse:
        body = SignResponseBody.fromJson(bodyRaw);
      case QrKind.userContact:
        body = UserContactBody.fromJson(bodyRaw);
      case QrKind.userTransfer:
        body = UserTransferBody.fromJson(bodyRaw);
      case QrKind.userDuoqian:
        body = UserDuoqianBody.fromJson(bodyRaw);
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
