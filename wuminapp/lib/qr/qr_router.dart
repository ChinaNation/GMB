import 'package:wuminapp_mobile/qr/qr_protocols.dart';
import 'package:wuminapp_mobile/qr/envelope.dart';

/// 扫码内容的路由分类结果。统一协议 WUMIN_QR_V1 下按 kind 分派。
enum QrRouteType {
  /// 登录挑战码(wuminapp 不处理,展示明确错误)。
  loginChallenge,

  /// 用户码 - 联系人(user_contact)
  userContact,

  /// 用户码 - 收款码(user_transfer)
  userTransfer,

  /// 用户码 - 多签账户(user_duoqian)
  userDuoqian,

  /// 交易签名请求(sign_request)
  signRequest,

  /// 交易签名回执(sign_response)
  signResponse,

  /// login_receipt(wuminapp 既不生成也不扫)
  loginReceipt,

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
    this.envelope,
    this.extractedAddress,
  });

  final QrRouteType type;
  final String raw;

  /// 成功解析的 envelope(仅当 type 非 legacyAddress/unknown 时存在)。
  final QrEnvelope<QrBody>? envelope;

  /// 裸地址/gmb:// 兜底时的地址。
  final String? extractedAddress;
}

/// 统一 QR 码路由器。
///
/// 接收扫码原始字符串,返回 [QrRouteResult] 供上层页面分发处理。
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

    // 1. 尝试 WUMIN_QR_V1 envelope
    if (text.startsWith('{')) {
      try {
        final env = QrEnvelope.parse(text);
        final type = switch (env.kind) {
          QrKind.loginChallenge => QrRouteType.loginChallenge,
          QrKind.loginReceipt => QrRouteType.loginReceipt,
          QrKind.signRequest => QrRouteType.signRequest,
          QrKind.signResponse => QrRouteType.signResponse,
          QrKind.userContact => QrRouteType.userContact,
          QrKind.userTransfer => QrRouteType.userTransfer,
          QrKind.userDuoqian => QrRouteType.userDuoqian,
        };
        return QrRouteResult(type: type, raw: raw, envelope: env);
      } on FormatException {
        // 非 envelope JSON — 继续走兜底分支
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
}
