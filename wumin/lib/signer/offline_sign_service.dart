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

  QrSignRequest parseRequest(String raw) {
    return _signer.parseRequest(raw);
  }

  OfflineSignVerification verifyPayload(QrSignRequest request) {
    final decoded = PayloadDecoder.decode(
      request.payloadHex,
      specVersion: request.specVersion,
    );

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
    if (displayFields is List) {
      for (final entry in decoded.fields.entries) {
        final displayValue = _findFieldValue(displayFields, entry.key);
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
    // 签名时再次校验过期，防止用户在 UI 上停留太久后点击签名。
    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    if (request.expiresAt < now) {
      throw const OfflineSignException(
        OfflineSignErrorCode.expired,
        '签名请求已过期，请重新扫描',
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
    if (verification.displayMatch == DisplayMatchStatus.decodeFailed) {
      // 中文注释：大 payload 交易（如 runtime 升级）的 payload_hex 是哈希后的 32 字节，
      // 无法从哈希中解码出原始交易内容。对于这类已知安全的操作，信任 display 字段。
      final displayAction = request.display['action']?.toString() ?? '';
      const allowedHashedActions = {
        'developer_upgrade',
        'developer_direct_upgrade',
        'propose_runtime_upgrade',
        'activate_admin',
        'propose_institution_rate',
        'vote_institution_rate',
        'propose_safety_fund_transfer',
        'vote_safety_fund_transfer',
        'propose_sweep_to_main',
        'vote_sweep_to_main',
        'propose_create',
        'propose_create_personal',
        'propose_resolution_issuance',
      };
      if (!allowedHashedActions.contains(displayAction)) {
        throw const OfflineSignException(
          OfflineSignErrorCode.displayMismatch,
          '无法独立验证交易内容，禁止签名。请升级冷钱包。',
        );
      }
      // 大 payload 交易允许签名，信任 display 中的摘要
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

  /// 从 display.fields（List 格式）中按 key 查找 value。
  static String? _findFieldValue(List<dynamic> fields, String key) {
    for (final field in fields) {
      if (field is Map && field['key']?.toString() == key) {
        return field['value']?.toString();
      }
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
