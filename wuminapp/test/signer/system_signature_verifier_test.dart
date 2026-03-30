import 'dart:convert';
import 'dart:typed_data';

import 'package:bip39_mnemonic/bip39_mnemonic.dart' as bip39m;
import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:substrate_bip39/crypto_scheme.dart';
import 'package:wuminapp_mobile/qr/login/login_models.dart';
import 'package:wuminapp_mobile/signer/system_signature_verifier.dart';

void main() {
  group('LoginSystemSignatureVerifier', () {
    late _KeyFixture sfidKey;
    late LoginSystemSignatureVerifier verifier;

    setUp(() async {
      sfidKey = await _deriveFixture(
        'bottom drive obey lake curtain smoke basket hold race lonely fit walk',
      );
      verifier = LoginSystemSignatureVerifier();
    });

    test('should accept sfid challenge signed by trusted sfid key', () async {
      final challenge = _buildSfidChallenge(sfidKey);
      await expectLater(verifier.verify(challenge), completes);
    });

    test('should reject challenge when signature does not match sys_pubkey',
        () async {
      final challenge = _buildInvalidSignatureChallenge(sfidKey);
      await expectLater(
        verifier.verify(challenge),
        throwsA(
          isA<LoginException>().having(
            (e) => e.code,
            'code',
            LoginErrorCode.invalidSystemSignature,
          ),
        ),
      );
    });
  });
}

LoginChallenge _buildSfidChallenge(_KeyFixture signer) {
  final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
  final expiresAt = now + 90;
  const challengeToken = 'challenge-token';
  final message = utf8.encode(
    'WUMIN_LOGIN_V1.0.0|sfid|$challengeToken|$now|$expiresAt|0x${signer.pubkeyHex}',
  );
  final signature = signer.pair.sign(Uint8List.fromList(message));

  return LoginChallenge(
    proto: 'WUMIN_LOGIN_V1.0.0',
    system: 'sfid',
    challenge: challengeToken,
    issuedAt: now,
    expiresAt: expiresAt,
    sysPubkey: '0x${signer.pubkeyHex}',
    sysSig: '0x${_toHex(signature.toList(growable: false))}',
    raw: '{}',
  );
}

LoginChallenge _buildInvalidSignatureChallenge(_KeyFixture signer) {
  final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
  final expiresAt = now + 90;
  const challengeToken = 'challenge-token';

  return LoginChallenge(
    proto: 'WUMIN_LOGIN_V1.0.0',
    system: 'cpms',
    challenge: challengeToken,
    issuedAt: now,
    expiresAt: expiresAt,
    sysPubkey: '0x${signer.pubkeyHex}',
    sysSig:
        '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
    raw: '{}',
  );
}

Future<_KeyFixture> _deriveFixture(String mnemonic) async {
  final entropy =
      bip39m.Mnemonic.fromSentence(mnemonic, bip39m.Language.english).entropy;
  final miniSecret = await CryptoScheme.miniSecretFromEntropy(entropy);
  final pair = Keyring.sr25519.fromSeed(Uint8List.fromList(miniSecret));
  pair.ss58Format = 2027;
  final pubkeyHex = _toHex(pair.bytes().toList(growable: false));
  return _KeyFixture(pair: pair, pubkeyHex: pubkeyHex);
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

class _KeyFixture {
  const _KeyFixture({
    required this.pair,
    required this.pubkeyHex,
  });

  final KeyPair pair;
  final String pubkeyHex;
}
