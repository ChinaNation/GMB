import 'dart:convert';
import 'dart:typed_data';

import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:wuminapp_mobile/services/wallet_service.dart';

class FcrcLoginSignatureService {
  FcrcLoginSignatureService({WalletService? walletService})
      : _walletService = walletService ?? WalletService();

  final WalletService _walletService;

  Future<Map<String, dynamic>> buildSignaturePayload(String rawChallenge) async {
    final challenge = parseChallenge(rawChallenge);
    final walletSecret = await _walletService.getLatestWalletSecret();
    if (walletSecret == null) {
      throw Exception('请先创建或导入钱包');
    }
    final wallet = walletSecret.profile;
    final mnemonic = walletSecret.mnemonic;

    final pair = await Keyring.sr25519.fromMnemonic(mnemonic);
    pair.ss58Format = wallet.ss58;

    final localPubkeyHex = _toHex(pair.bytes().toList(growable: false));
    if (localPubkeyHex.toLowerCase() != wallet.pubkeyHex.toLowerCase()) {
      throw Exception('本地签名密钥与当前钱包不一致，请重新导入钱包。');
    }

    final message = Uint8List.fromList(utf8.encode(challenge.nonce));
    final signature = pair.sign(message);

    return {
      'type': 'fcrc.login.signature',
      'version': 1,
      'crypto': 'sr25519',
      'publicKey': '0x${wallet.pubkeyHex}',
      'nonce': challenge.nonce,
      'signature': '0x${_toHex(signature.toList(growable: false))}',
    };
  }

  FcrcLoginChallenge parseChallenge(String raw) {
    final text = raw.trim();

    try {
      final data = jsonDecode(text);
      if (data is Map<String, dynamic>) {
        final nonce = _pickNonceFromMap(data);
        if (nonce != null && nonce.isNotEmpty) {
          return FcrcLoginChallenge(nonce: nonce, raw: raw);
        }
      }
    } catch (_) {
      // Not JSON
    }

    final uri = Uri.tryParse(text);
    if (uri != null) {
      final nonce =
          uri.queryParameters['nonce'] ?? uri.queryParameters['challenge_nonce'];
      if (nonce != null && nonce.isNotEmpty) {
        return FcrcLoginChallenge(nonce: nonce, raw: raw);
      }
    }

    throw Exception('无法识别登录挑战 nonce');
  }

  String? _pickNonceFromMap(Map<String, dynamic> data) {
    final direct = data['nonce']?.toString();
    if (direct != null && direct.isNotEmpty) {
      return direct;
    }

    final challenge = data['challenge'];
    if (challenge is Map<String, dynamic>) {
      final nested = challenge['nonce']?.toString();
      if (nested != null && nested.isNotEmpty) {
        return nested;
      }
    }

    final challengeNonce = data['challenge_nonce']?.toString();
    if (challengeNonce != null && challengeNonce.isNotEmpty) {
      return challengeNonce;
    }

    return null;
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

class FcrcLoginChallenge {
  const FcrcLoginChallenge({required this.nonce, required this.raw});

  final String nonce;
  final String raw;
}
