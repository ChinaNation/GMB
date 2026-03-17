import 'dart:typed_data';

import 'package:bip39_mnemonic/bip39_mnemonic.dart' as bip39m;
import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:substrate_bip39/crypto_scheme.dart';
import 'package:wuminapp_mobile/signer/local_signer.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

void main() {
  group('LocalSigner', () {
    const mnemonic =
        'legal winner thank year wave sausage worth useful legal winner thank yellow';
    const ss58 = 2027;
    final signer = LocalSigner();

    test('signUtf8 should return sr25519 signature result', () async {
      final secret = await _buildWalletSecret(
        mnemonic: mnemonic,
        ss58: ss58,
      );
      final result = await signer.signUtf8(
        walletSecret: secret,
        message: 'WUMINAPP_LOGIN_V1|cpms|app|req|c|n|123',
      );

      expect(result.account, secret.profile.address);
      expect(result.pubkeyHex, '0x${secret.profile.pubkeyHex}');
      expect(result.sigAlg, 'sr25519');
      expect(result.signatureHex, startsWith('0x'));
      expect(result.signatureHex.length, greaterThan(2));
    });

    test('signBytes should reject mismatched wallet pubkey', () async {
      final secret = await _buildWalletSecret(
        mnemonic: mnemonic,
        ss58: ss58,
        mismatchPubkey: true,
      );

      expect(
        () => signer.signUtf8(
          walletSecret: secret,
          message: 'hello',
        ),
        throwsA(
          isA<LocalSignerException>().having(
            (e) => e.code,
            'code',
            LocalSignerErrorCode.walletMismatch,
          ),
        ),
      );
    });
  });
}

/// 从助记词派生 seed → 构建 WalletSecret（与 WalletManager 使用相同的派生链）。
Future<WalletSecret> _buildWalletSecret({
  required String mnemonic,
  required int ss58,
  bool mismatchPubkey = false,
}) async {
  // mnemonic → entropy → miniSecret (32 bytes)
  final entropy =
      bip39m.Mnemonic.fromSentence(mnemonic, bip39m.Language.english).entropy;
  final miniSecret = await CryptoScheme.miniSecretFromEntropy(entropy);

  final pair = Keyring.sr25519.fromSeed(Uint8List.fromList(miniSecret));
  pair.ss58Format = ss58;
  final pubkeyBytes = pair.bytes().toList(growable: false);
  final pubkeyHex = _toHex(pubkeyBytes);
  final address = pair.address;
  final seedHex = _toHex(miniSecret);

  final profile = WalletProfile(
    walletIndex: 1,
    walletName: 'test-wallet',
    walletIcon: 'wallet.svg',
    balance: 0,
    address: address,
    pubkeyHex: mismatchPubkey ? '${pubkeyHex}00' : pubkeyHex,
    alg: 'sr25519',
    ss58: ss58,
    createdAtMillis: DateTime.now().millisecondsSinceEpoch,
    source: 'test',
    signMode: 'local',
  );
  return WalletSecret(profile: profile, seedHex: seedHex);
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
