import 'dart:convert';

import 'package:wuminapp_mobile/qr/qr_protocols.dart';

/// 扫码内容的路由分类结果。
enum QrRouteType {
  /// 登录挑战码（`WUMIN_LOGIN_V1.0.0`）。
  login,

  /// 用户码 - 联系人（`WUMIN_USER_V1.0.0`，purpose=contact）。
  contact,

  /// 用户码 - 收款码（`WUMIN_USER_V1.0.0`，purpose=transfer）。
  transfer,

  /// 交易签名请求（`WUMIN_SIGN_V1.0.0`）。
  sign,

  /// 裸 SS58 地址或 `gmb://account/<addr>` — 向后兼容转账。
  legacyAddress,

  /// 无法识别。
  unknown,
}

/// 路由分析结果。
class QrRouteResult {
  const QrRouteResult({
    required this.type,
    required this.raw,
    this.jsonData,
    this.extractedAddress,
  });

  final QrRouteType type;
  final String raw;
  final Map<String, dynamic>? jsonData;
  final String? extractedAddress;
}

/// 统一 QR 码路由器。
///
/// 接收扫码原始字符串，返回 [QrRouteResult] 供上层页面分发处理。
class QrRouter {
  static final RegExp _ss58Pattern =
      RegExp(r'^[1-9A-HJ-NP-Za-km-z]{30,80}$');
  static const String _gmbSchemePrefix = 'gmb://account/';

  /// 分析扫码内容并返回路由结果。
  QrRouteResult route(String raw) {
    final text = raw.trim();
    if (text.isEmpty) {
      return QrRouteResult(type: QrRouteType.unknown, raw: raw);
    }

    // 1. 尝试 JSON 解析。
    final jsonData = _tryParseJson(text);
    if (jsonData != null) {
      final proto = (jsonData['proto'] ?? jsonData['type'] ?? '').toString();
      switch (proto) {
        case QrProtocols.login:
          return QrRouteResult(
            type: QrRouteType.login,
            raw: raw,
            jsonData: jsonData,
          );
        case QrProtocols.user:
          // 根据 purpose 字段区分联系人/收款
          final purpose = (jsonData['purpose'] ?? 'contact').toString();
          return QrRouteResult(
            type: purpose == 'transfer'
                ? QrRouteType.transfer
                : QrRouteType.contact,
            raw: raw,
            jsonData: jsonData,
          );
        case QrProtocols.sign:
          return QrRouteResult(
            type: QrRouteType.sign,
            raw: raw,
            jsonData: jsonData,
          );
        default:
          return QrRouteResult(
            type: QrRouteType.unknown,
            raw: raw,
            jsonData: jsonData,
          );
      }
    }

    // 2. gmb://account/<address> 格式。
    if (text.toLowerCase().startsWith(_gmbSchemePrefix)) {
      final address = text.substring(_gmbSchemePrefix.length).trim();
      if (address.isNotEmpty) {
        return QrRouteResult(
          type: QrRouteType.legacyAddress,
          raw: raw,
          extractedAddress: address,
        );
      }
    }

    // 3. 裸 SS58 地址。
    if (_ss58Pattern.hasMatch(text)) {
      return QrRouteResult(
        type: QrRouteType.legacyAddress,
        raw: raw,
        extractedAddress: text,
      );
    }

    return QrRouteResult(type: QrRouteType.unknown, raw: raw);
  }

  Map<String, dynamic>? _tryParseJson(String text) {
    try {
      final decoded = jsonDecode(text);
      if (decoded is Map) {
        return decoded.map((k, v) => MapEntry(k.toString(), v));
      }
    } catch (_) {
      // not json
    }
    return null;
  }
}
