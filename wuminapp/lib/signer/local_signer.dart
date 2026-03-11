import 'dart:convert';
import 'dart:typed_data';

import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

enum LocalSignerErrorCode {
  emptyPayload,
  unsupportedAlgorithm,
  walletMismatch,
}

class LocalSignerException implements Exception {
  const LocalSignerException(this.code, this.message);

  final LocalSignerErrorCode code;
  final String message;

  @override
  String toString() => message;
}

class LocalSignResult {
  const LocalSignResult({
    required this.account,
    required this.pubkeyHex,
    required this.sigAlg,
    required this.signatureHex,
  });

  final String account;
  final String pubkeyHex;
  final String sigAlg;
  final String signatureHex;
}

class LocalSigner {
  Future<LocalSignResult> signUtf8({
    required WalletSecret walletSecret,
    required String message,
  }) {
    final bytes = Uint8List.fromList(utf8.encode(message));
    return signBytes(walletSecret: walletSecret, payload: bytes);
  }

  Future<LocalSignResult> signBytes({
    required WalletSecret walletSecret,
    required Uint8List payload,
  }) async {
    if (payload.isEmpty) {
      throw const LocalSignerException(
        LocalSignerErrorCode.emptyPayload,
        '签名负载为空，无法签名',
      );
    }

    final wallet = walletSecret.profile;
    if (wallet.alg.toLowerCase() != 'sr25519') {
      throw LocalSignerException(
        LocalSignerErrorCode.unsupportedAlgorithm,
        '不支持的钱包签名算法：${wallet.alg}',
      );
    }

    final pair = await Keyring.sr25519.fromMnemonic(walletSecret.mnemonic);
    pair.ss58Format = wallet.ss58;
    final localPubkeyHex = _toHex(pair.bytes().toList(growable: false));
    if (localPubkeyHex.toLowerCase() != wallet.pubkeyHex.toLowerCase()) {
      throw const LocalSignerException(
        LocalSignerErrorCode.walletMismatch,
        '本地签名密钥与当前钱包不一致，请重新导入钱包',
      );
    }

    final signature = pair.sign(payload);
    return LocalSignResult(
      account: wallet.address,
      pubkeyHex: '0x${wallet.pubkeyHex}',
      sigAlg: 'sr25519',
      signatureHex: '0x${_toHex(signature.toList(growable: false))}',
    );
  }

  String _toHex(List<int> bytes) {
    const chars = '0123456789abcdef';
    final buf = StringBuffer();
    for (final b in bytes) {
      buf
        ..write(chars[(b >> 4) & 0x0f])
        ..write(chars[b & 0x0f]);
    }
    return buf.toString();
  }
}
