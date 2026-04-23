import 'dart:typed_data';

import '../qr/bodies/sign_request_body.dart';
import '../wallet/wallet_manager.dart';
import 'payload_decoder.dart';
import 'qr_signer.dart';

enum OfflineSignErrorCode {
  walletNotFound,
  coldWalletUnsupported,
  walletMismatch,
  invalidPayload,
  displayMismatch,
  expired,
}

class OfflineSignException implements Exception {
  const OfflineSignException(this.code, this.message);

  final OfflineSignErrorCode code;
  final String message;

  @override
  String toString() => message;
}

/// 离线签名验证结果。
class OfflineSignVerification {
  const OfflineSignVerification({
    required this.decoded,
    required this.displayMatch,
  });

  final DecodedPayload? decoded;
  final DisplayMatchStatus displayMatch;
}

enum DisplayMatchStatus {
  matched,
  mismatched,
  decodeFailed,
}

/// 离线签名执行服务。
class OfflineSignService {
  OfflineSignService({
    WalletManager? walletManager,
    QrSigner? signer,
  })  : _walletManager = walletManager ?? WalletManager(),
        _signer = signer ?? QrSigner();

  final WalletManager _walletManager;
  final QrSigner _signer;

  SignRequestEnvelope parseRequest(String raw) {
    return _signer.parseRequest(raw);
  }

  OfflineSignVerification verifyPayload(SignRequestEnvelope request) {
    final body = request.body;
    final decoded = PayloadDecoder.decode(
      body.payloadHex,
      specVersion: body.specVersion,
    );

    if (decoded == null) {
      return const OfflineSignVerification(
        decoded: null,
        displayMatch: DisplayMatchStatus.decodeFailed,
      );
    }

    if (body.display.action != decoded.action) {
      return OfflineSignVerification(
        decoded: decoded,
        displayMatch: DisplayMatchStatus.mismatched,
      );
    }

    for (final entry in decoded.fields.entries) {
      final displayValue = _findFieldValue(body.display.fields, entry.key);
      if (displayValue != null && displayValue != entry.value) {
        return OfflineSignVerification(
          decoded: decoded,
          displayMatch: DisplayMatchStatus.mismatched,
        );
      }
    }

    return OfflineSignVerification(
      decoded: decoded,
      displayMatch: DisplayMatchStatus.matched,
    );
  }

  Future<SignResponseEnvelope> signRequestRaw({
    required int walletIndex,
    required String raw,
  }) async {
    final request = parseRequest(raw);
    return signParsedRequest(walletIndex: walletIndex, request: request);
  }

  Future<SignResponseEnvelope> signParsedRequest({
    required int walletIndex,
    required SignRequestEnvelope request,
  }) async {
    final body = request.body;
    // 签名时再次校验过期
    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    if ((request.expiresAt ?? 0) < now) {
      throw const OfflineSignException(
        OfflineSignErrorCode.expired,
        '签名请求已过期,请重新扫描',
      );
    }

    final wallet = await _walletManager.getWalletByIndex(walletIndex);
    if (wallet == null) {
      throw const OfflineSignException(
        OfflineSignErrorCode.walletNotFound,
        '未找到指定钱包',
      );
    }
    if (wallet.isColdWallet) {
      throw const OfflineSignException(
        OfflineSignErrorCode.coldWalletUnsupported,
        '当前钱包为冷钱包,无法作为离线签名设备',
      );
    }

    if (_normalizeHex(wallet.pubkeyHex) != _normalizeHex(body.pubkey)) {
      throw const OfflineSignException(
        OfflineSignErrorCode.walletMismatch,
        '签名请求中的公钥与当前钱包不一致',
      );
    }
    if (wallet.address.trim() != body.address.trim()) {
      throw const OfflineSignException(
        OfflineSignErrorCode.walletMismatch,
        '签名请求中的地址与当前钱包不一致',
      );
    }

    final verification = verifyPayload(request);
    // 两色识别模型(2026-04-22):
    //   matched  → 绿色放行
    //   mismatched / decodeFailed → 红色拒签,不再保留任何"白名单盲签"兜底。
    //   见 memory/05-architecture/qr-signing-recognition.md § 四铁律。
    switch (verification.displayMatch) {
      case DisplayMatchStatus.matched:
        break;
      case DisplayMatchStatus.mismatched:
        throw const OfflineSignException(
          OfflineSignErrorCode.displayMismatch,
          '交易内容与摘要不符,拒绝签名',
        );
      case DisplayMatchStatus.decodeFailed:
        throw const OfflineSignException(
          OfflineSignErrorCode.displayMismatch,
          '无法独立验证交易内容,禁止签名',
        );
    }

    final payloadBytes = _hexToBytes(body.payloadHex);
    if (payloadBytes.isEmpty) {
      throw const OfflineSignException(
        OfflineSignErrorCode.invalidPayload,
        '签名负载为空,无法签名',
      );
    }

    final signature = await _walletManager.signWithWallet(
      wallet.walletIndex,
      Uint8List.fromList(payloadBytes),
    );

    return _signer.buildResponse(
      request: request,
      signatureHex: '0x${_toHex(signature)}',
    );
  }

  /// 从 display.fields 中按 key 查找 value（用于与解码结果交叉比对）。
  static String? _findFieldValue(
      List<SignDisplayField> fields, String key) {
    for (final field in fields) {
      if (field.key == key) return field.value;
    }
    return null;
  }

  List<int> _hexToBytes(String input) {
    final text = _normalizeHex(input);
    if (text.isEmpty || text.length.isOdd) {
      return const <int>[];
    }
    return List<int>.generate(
      text.length ~/ 2,
      (i) => int.parse(text.substring(i * 2, i * 2 + 2), radix: 16),
      growable: false,
    );
  }

  String _toHex(List<int> bytes) {
    const chars = '0123456789abcdef';
    final buffer = StringBuffer();
    for (final byte in bytes) {
      buffer
        ..write(chars[(byte >> 4) & 0x0f])
        ..write(chars[byte & 0x0f]);
    }
    return buffer.toString();
  }
}

String _normalizeHex(String input) {
  final trimmed = input.trim();
  if (trimmed.startsWith('0x') || trimmed.startsWith('0X')) {
    return trimmed.substring(2).toLowerCase();
  }
  return trimmed.toLowerCase();
}
