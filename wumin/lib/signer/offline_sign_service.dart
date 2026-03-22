import 'dart:typed_data';

import '../wallet/wallet_manager.dart';
import 'payload_decoder.dart';
import 'qr_signer.dart';

enum OfflineSignErrorCode {
  walletNotFound,
  coldWalletUnsupported,
  walletMismatch,
  invalidPayload,
  displayMismatch,
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

  QrSignRequest parseRequest(String raw) {
    return _signer.parseRequest(raw);
  }

  OfflineSignVerification verifyPayload(QrSignRequest request) {
    final decoded = PayloadDecoder.decode(request.payloadHex);

    if (decoded == null) {
      return const OfflineSignVerification(
        decoded: null,
        displayMatch: DisplayMatchStatus.decodeFailed,
      );
    }

    final displayAction = request.display['action']?.toString();
    if (displayAction != decoded.action) {
      return OfflineSignVerification(
        decoded: decoded,
        displayMatch: DisplayMatchStatus.mismatched,
      );
    }

    final displayFields = request.display['fields'];
    if (displayFields is Map) {
      for (final entry in decoded.fields.entries) {
        final displayValue = displayFields[entry.key]?.toString();
        if (displayValue != null && displayValue != entry.value) {
          return OfflineSignVerification(
            decoded: decoded,
            displayMatch: DisplayMatchStatus.mismatched,
          );
        }
      }
    }

    return OfflineSignVerification(
      decoded: decoded,
      displayMatch: DisplayMatchStatus.matched,
    );
  }

  Future<QrSignResponse> signRequestRaw({
    required int walletIndex,
    required String raw,
  }) async {
    final request = parseRequest(raw);
    return signParsedRequest(walletIndex: walletIndex, request: request);
  }

  Future<QrSignResponse> signParsedRequest({
    required int walletIndex,
    required QrSignRequest request,
  }) async {
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
        '当前钱包为冷钱包，无法作为离线签名设备',
      );
    }

    if (_normalizeHex(wallet.pubkeyHex) != _normalizeHex(request.pubkey)) {
      throw const OfflineSignException(
        OfflineSignErrorCode.walletMismatch,
        '签名请求中的公钥与当前钱包不一致',
      );
    }
    if (wallet.address.trim() != request.account.trim()) {
      throw const OfflineSignException(
        OfflineSignErrorCode.walletMismatch,
        '签名请求中的地址与当前钱包不一致',
      );
    }

    final verification = verifyPayload(request);
    if (verification.displayMatch == DisplayMatchStatus.mismatched) {
      throw const OfflineSignException(
        OfflineSignErrorCode.displayMismatch,
        '交易内容与摘要不符，拒绝签名',
      );
    }

    final payloadBytes = _hexToBytes(request.payloadHex);
    if (payloadBytes.isEmpty) {
      throw const OfflineSignException(
        OfflineSignErrorCode.invalidPayload,
        '签名负载为空，无法签名',
      );
    }

    final signature = await _walletManager.signWithWallet(
      wallet.walletIndex,
      Uint8List.fromList(payloadBytes),
    );

    final payloadHash = QrSigner.computePayloadHash(request.payloadHex);

    return QrSignResponse(
      proto: QrSigner.protocol,
      requestId: request.requestId,
      pubkey: '0x${wallet.pubkeyHex}',
      sigAlg: request.sigAlg,
      signature: '0x${_toHex(signature)}',
      payloadHash: payloadHash,
      signedAt: DateTime.now().millisecondsSinceEpoch ~/ 1000,
    );
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
