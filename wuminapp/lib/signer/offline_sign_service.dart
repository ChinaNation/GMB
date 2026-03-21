import 'dart:typed_data';

import '../wallet/core/wallet_manager.dart';
import 'qr_signer.dart';

enum OfflineSignErrorCode {
  walletNotFound,
  coldWalletUnsupported,
  walletMismatch,
  invalidPayload,
}

class OfflineSignException implements Exception {
  const OfflineSignException(this.code, this.message);

  final OfflineSignErrorCode code;
  final String message;

  @override
  String toString() => message;
}

/// 离线签名执行服务。
///
/// 用于“另一台设备”扫描在线手机展示的签名请求二维码，
/// 并使用本机热钱包完成签名后生成回执二维码。
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
    return QrSignResponse(
      proto: QrSigner.protocol,
      requestId: request.requestId,
      pubkey: '0x${wallet.pubkeyHex}',
      sigAlg: request.sigAlg,
      signature: '0x${_toHex(signature)}',
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
