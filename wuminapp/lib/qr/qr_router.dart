import 'dart:convert';

import 'package:wuminapp_mobile/qr/qr_protocols.dart';

/// 扫码内容的路由分类结果。
enum QrRouteType {
  /// 登录挑战码（`WUMINAPP_LOGIN_V1`）。
  login,

  /// 收款码（`WUMINAPP_TRANSFER_V1`）。
  transfer,

  /// 用户码（`WUMINAPP_CONTACT_V1` 或旧版 `WUMINAPP_USER_CARD_V1`）。
  contact,

  /// 扫码签名请求（`WUMINAPP_QR_SIGN_V1`）。
  qrSign,

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
        case QrProtocols.transfer:
          return QrRouteResult(
            type: QrRouteType.transfer,
            raw: raw,
            jsonData: jsonData,
          );
        case QrProtocols.contact:
        case QrProtocols.legacyUserCard:
          return QrRouteResult(
            type: QrRouteType.contact,
            raw: raw,
            jsonData: jsonData,
          );
        case QrProtocols.qrSign:
          return QrRouteResult(
            type: QrRouteType.qrSign,
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
