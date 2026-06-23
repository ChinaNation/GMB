import '../qr/qr_protocols.dart';
import '../wallet/wallet_manager.dart';
import 'payload_decoder.dart';
import 'qr_signer.dart';

enum OfflineSignErrorCode {
  walletNotFound,
  coldWalletUnsupported,
  walletMismatch,
  invalidPayload,
  contentMismatch,
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
    required this.contentMatch,
  });

  final DecodedPayload? decoded;
  final ContentMatchStatus contentMatch;
}

enum ContentMatchStatus {
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

    // Runtime 升级只在 QR 中携带 32B 待签摘要,原始 WASM call_data 留在生成端 session。
    if (QrActions.isRuntimeHashOnly(body.action)) {
      if (body.payloadBytes.length == 32) {
        return const OfflineSignVerification(
          decoded: null,
          contentMatch: ContentMatchStatus.matched,
        );
      }
      // 例外条件不全(payload 不是 32B / 缺 wasm_hash 字段),仍按 decodeFailed 处理。
      return const OfflineSignVerification(
        decoded: null,
        contentMatch: ContentMatchStatus.decodeFailed,
      );
    }

    final decoded = PayloadDecoder.decode(body.payloadHex);

    if (decoded == null) {
      return const OfflineSignVerification(
        decoded: null,
        contentMatch: ContentMatchStatus.decodeFailed,
      );
    }

    final decodedAction = QrActions.fromDecodedAction(decoded.action);
    if (decodedAction != 0 && decodedAction != body.action) {
      return OfflineSignVerification(
        decoded: decoded,
        contentMatch: ContentMatchStatus.mismatched,
      );
    }

    return OfflineSignVerification(
      decoded: decoded,
      contentMatch: ContentMatchStatus.matched,
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

    if (_normalizeHex(wallet.pubkeyHex) != _normalizeHex(body.pubkeyHex)) {
      throw const OfflineSignException(
        OfflineSignErrorCode.walletMismatch,
        '签名请求中的公钥与当前钱包不一致',
      );
    }

    final verification = verifyPayload(request);
    // 两色识别模型:action 与 payload 解码一致才绿色放行。
    switch (verification.contentMatch) {
      case ContentMatchStatus.matched:
        break;
      case ContentMatchStatus.mismatched:
        throw const OfflineSignException(
          OfflineSignErrorCode.contentMismatch,
          '交易内容与摘要不符,拒绝签名',
        );
      case ContentMatchStatus.decodeFailed:
        throw const OfflineSignException(
          OfflineSignErrorCode.contentMismatch,
          '无法独立验证交易内容,禁止签名',
        );
    }

    final payloadBytes = QrSigner.signingBytesFor(body);
    if (payloadBytes.isEmpty) {
      throw const OfflineSignException(
        OfflineSignErrorCode.invalidPayload,
        '签名负载为空,无法签名',
      );
    }

    final signature = await _walletManager.signWithWallet(
      wallet.walletIndex,
      payloadBytes,
    );

    return _signer.buildResponse(
      request: request,
      signatureHex: '0x${_toHex(signature)}',
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
