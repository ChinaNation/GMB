import 'dart:convert';
import 'dart:typed_data';

import 'package:bip39_mnemonic/bip39_mnemonic.dart' as bip39m;
import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:substrate_bip39/crypto_scheme.dart';
import 'package:wuminapp_mobile/qr/login/login_models.dart';
import 'package:wuminapp_mobile/qr/login/login_replay_guard.dart';
import 'package:wuminapp_mobile/qr/login/login_service.dart';
import 'package:wuminapp_mobile/signer/qr_signer.dart';
import 'package:wuminapp_mobile/signer/system_signature_verifier.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

void main() {
  group('LoginService', () {
    late _FakeWalletService walletService;
    late _FakeReplayGuard replayGuard;
    late _FakeSystemSignatureVerifier systemSignatureVerifier;
    late LoginService service;

    setUp(() {
      walletService = _FakeWalletService();
      replayGuard = _FakeReplayGuard();
      systemSignatureVerifier = _FakeSystemSignatureVerifier();
      service = LoginService(
        walletManager: walletService,
        replayGuard: replayGuard,
        systemSignatureVerifier: systemSignatureVerifier,
      );
    });

    test('parseChallenge should parse a valid challenge', () {
      final raw = _challengeJson(
        challenge: 'req-1',
        expiresAt: _nowSec() + 90,
      );
      final challenge = service.parseChallenge(raw);

      expect(challenge.proto, 'WUMINAPP_LOGIN_V1');
      expect(challenge.system, 'cpms');
      expect(challenge.challenge, 'req-1');
      expect(challenge.sysPubkey, startsWith('0x'));
      expect(challenge.sysSig, startsWith('0x'));
    });

    test('parseChallenge should reject expired challenge', () {
      final raw = _challengeJson(
        challenge: 'req-expired',
        issuedAt: _nowSec() - 120,
        expiresAt: _nowSec() - 1,
      );
      expect(
        () => service.parseChallenge(raw),
        throwsA(
          isA<Exception>().having(
            (e) => e.toString(),
            'message',
            contains('已过期'),
          ),
        ),
      );
    });

    test('parseChallenge should reject non-90-second ttl', () {
      final raw = _challengeJson(
        challenge: 'req-invalid-ttl',
        issuedAt: _nowSec() - 1,
        expiresAt: _nowSec() + 60,
      );
      expect(
        () => service.parseChallenge(raw),
        throwsA(
          isA<Exception>().having(
            (e) => e.toString(),
            'message',
            contains('必须为 90 秒'),
          ),
        ),
      );
    });

    test('parseChallenge should reject challenge with illegal chars', () {
      final raw = _challengeJson(
        challenge: 'bad id',
        expiresAt: _nowSec() + 90,
      );
      expect(
        () => service.parseChallenge(raw),
        throwsA(
          isA<Exception>().having(
            (e) => e.toString(),
            'message',
            contains('challenge'),
          ),
        ),
      );
    });

    test('parseChallenge should reject challenge with whitespace', () {
      final raw = _challengeJson(
        challenge: 'ab c123',
        expiresAt: _nowSec() + 90,
      );
      expect(
        () => service.parseChallenge(raw),
        throwsA(
          isA<Exception>().having(
            (e) => e.toString(),
            'message',
            contains('challenge'),
          ),
        ),
      );
    });

    test('parseChallenge should reject issued_at too far in future', () {
      final issuedAt = _nowSec() + LoginService.maxClockSkewSeconds + 10;
      final raw = _challengeJson(
        challenge: 'req-future-issued-at',
        issuedAt: issuedAt,
        expiresAt: issuedAt + LoginService.challengeTtlSeconds,
      );
      expect(
        () => service.parseChallenge(raw),
        throwsA(
          isA<Exception>().having(
            (e) => e.toString(),
            'message',
            contains('issued_at'),
          ),
        ),
      );
    });

    test('parseChallenge should allow sfid payload without extra cert fields',
        () {
      final raw = _challengeJson(
        challenge: 'req-sfid-no-cert',
        system: 'sfid',
        expiresAt: _nowSec() + 90,
      );
      final challenge = service.parseChallenge(raw);
      expect(challenge.system, 'sfid');
    });

    test('buildSignMessage should follow canonical format', () {
      final expiresAt = _nowSec() + 90;
      final raw = _challengeJson(
        challenge: 'abc123',
        expiresAt: expiresAt,
      );
      final challenge = service.parseChallenge(raw);
      final signMessage = service.buildSignMessage(challenge);

      expect(
        signMessage,
        'WUMINAPP_LOGIN_V1|cpms|abc123|$expiresAt',
      );
    });

    test('buildReceiptPayload should sign with selected wallet', () async {
      final raw = _challengeJson(
        challenge: 'req-wallet-2',
        system: 'sfid',
        expiresAt: _nowSec() + 90,
      );
      final challenge = service.parseChallenge(raw);

      final receipt = await service.buildReceiptPayload(
        challenge,
        walletIndex: 2,
      );

      final wallet2 = await walletService.getWalletSecretByIndex(2);
      expect(wallet2, isNotNull);
      expect(receipt['pubkey'], '0x${wallet2!.profile.pubkeyHex}');
      expect(receipt['proto'], 'WUMINAPP_LOGIN_V1');
      expect(receipt['sig_alg'], 'sr25519');
      expect(receipt['signature'], startsWith('0x'));
      // 回执码不再包含 account 字段。
      expect(receipt.containsKey('account'), isFalse);
    });

    test('buildReceiptPayload should block replay challenge', () async {
      final raw = _challengeJson(
        challenge: 'req-replay',
        expiresAt: _nowSec() + 90,
      );
      final challenge = service.parseChallenge(raw);

      await service.buildReceiptPayload(challenge, walletIndex: 1);
      expect(
        () => service.buildReceiptPayload(challenge, walletIndex: 1),
        throwsA(
          isA<Exception>().having(
            (e) => e.toString(),
            'message',
            contains('已使用'),
          ),
        ),
      );
    });

    test('buildReceiptPayload should fail when walletIndex not found',
        () async {
      final raw = _challengeJson(
        challenge: 'req-wallet-missing',
        expiresAt: _nowSec() + 90,
      );
      final challenge = service.parseChallenge(raw);

      expect(
        () => service.buildReceiptPayload(challenge, walletIndex: 99),
        throwsA(
          isA<Exception>().having(
            (e) => e.toString(),
            'message',
            contains('未找到指定钱包'),
          ),
        ),
      );
    });

    test('buildExternalSignRequest should build cold-wallet qr request',
        () async {
      final raw = _challengeJson(
        challenge: 'req-cold-login',
        expiresAt: _nowSec() + 90,
      );
      final challenge = service.parseChallenge(raw);
      final coldWallet = await walletService.buildColdWalletProfile(1);

      final bundle = await service.buildExternalSignRequest(
        challenge,
        wallet: coldWallet,
      );

      final parsed = QrSigner().parseRequest(bundle.requestJson);
      expect(parsed.scope, QrSignScope.login);
      expect(parsed.requestId, challenge.challenge);
      expect(parsed.account, coldWallet.address);
      expect(parsed.pubkey, '0x${coldWallet.pubkeyHex}');
      expect(bundle.signMessage, service.buildSignMessage(challenge));
    });

    test('buildReceiptFromSignature should accept cold-wallet signature',
        () async {
      final raw = _challengeJson(
        challenge: 'req-cold-receipt',
        expiresAt: _nowSec() + 90,
      );
      final challenge = service.parseChallenge(raw);
      final coldWallet = await walletService.buildColdWalletProfile(1);
      final signMessage = service.buildSignMessage(challenge);
      final signed = await walletService.signUtf8WithWallet(
        coldWallet.walletIndex,
        signMessage,
      );

      final receipt = await service.buildReceiptFromSignature(
        challenge: challenge,
        pubkeyHex: signed.pubkeyHex,
        signatureHex: signed.signatureHex,
      );

      expect(receipt['challenge'], challenge.challenge);
      expect(receipt['pubkey'], signed.pubkeyHex);
      expect(receipt['signature'], signed.signatureHex);
    });
  });
}

int _nowSec() => DateTime.now().millisecondsSinceEpoch ~/ 1000;

const String _fakeSysPubkey =
    '0xabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789';
const String _fakeSysSig =
    '0xabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789';

String _challengeJson({
  required String challenge,
  String system = 'cpms',
  int? issuedAt,
  required int expiresAt,
}) {
  final iat = issuedAt ?? (expiresAt - LoginService.challengeTtlSeconds);
  return '''
{
  "proto": "WUMINAPP_LOGIN_V1",
  "system": "$system",
  "challenge": "$challenge",
  "issued_at": $iat,
  "expires_at": $expiresAt,
  "sys_pubkey": "$_fakeSysPubkey",
  "sys_sig": "$_fakeSysSig",
  "_pad": true
}
''';
}

class _FakeReplayGuard extends LoginReplayGuard {
  final Set<String> _used = <String>{};

  @override
  Future<void> assertNotConsumed(String challenge) async {
    if (_used.contains(challenge)) {
      throw Exception('登录挑战已使用，请刷新二维码后重试');
    }
  }

  @override
  Future<void> consume({
    required String challenge,
    required int expiresAt,
  }) async {
    _used.add(challenge);
  }
}

class _FakeWalletService extends WalletManager {
  static const int _ss58 = 2027;
  static const String _mnemonic1 =
      'bottom drive obey lake curtain smoke basket hold race lonely fit walk';
  static const String _mnemonic2 =
      'legal winner thank year wave sausage worth useful legal winner thank yellow';

  List<_WalletFixture>? _fixtures;

  Future<List<_WalletFixture>> _ensureFixtures() async {
    final existing = _fixtures;
    if (existing != null) {
      return existing;
    }
    final wallet1 = await _deriveFixture(index: 1, mnemonic: _mnemonic1);
    final wallet2 = await _deriveFixture(index: 2, mnemonic: _mnemonic2);
    _fixtures = [wallet1, wallet2];
    return _fixtures!;
  }

  Future<_WalletFixture> _deriveFixture({
    required int index,
    required String mnemonic,
  }) async {
    // 使用与 WalletManager 相同的派生链。
    final entropy =
        bip39m.Mnemonic.fromSentence(mnemonic, bip39m.Language.english).entropy;
    final miniSecret = await CryptoScheme.miniSecretFromEntropy(entropy);

    final pair = Keyring.sr25519.fromSeed(Uint8List.fromList(miniSecret));
    pair.ss58Format = _ss58;
    final pubkeyBytes = pair.bytes().toList(growable: false);
    final pubkeyHex = _toHex(pubkeyBytes);
    final address = pair.address;
    final seedHex = _toHex(miniSecret);
    return _WalletFixture(
      profile: WalletProfile(
        walletIndex: index,
        walletName: '测试钱包$index',
        walletIcon: 'wallet.svg',
        balance: 0,
        address: address,
        pubkeyHex: pubkeyHex,
        alg: 'sr25519',
        ss58: _ss58,
        createdAtMillis: DateTime.now().millisecondsSinceEpoch,
        source: 'test',
        signMode: 'local',
      ),
      seedHex: seedHex,
    );
  }

  @override
  // ignore: deprecated_member_use_from_same_package
  Future<WalletSecret?> getLatestWalletSecret() async {
    final fixtures = await _ensureFixtures();
    final f = fixtures.last;
    return WalletSecret(profile: f.profile, seedHex: f.seedHex);
  }

  @override
  // ignore: deprecated_member_use_from_same_package
  Future<WalletSecret?> getWalletSecretByIndex(int walletIndex) async {
    final fixtures = await _ensureFixtures();
    for (final f in fixtures) {
      if (f.profile.walletIndex == walletIndex) {
        return WalletSecret(profile: f.profile, seedHex: f.seedHex);
      }
    }
    return null;
  }

  @override
  Future<WalletProfile?> getWallet() async {
    final fixtures = await _ensureFixtures();
    return fixtures.last.profile;
  }

  @override
  Future<WalletProfile?> getWalletByIndex(int walletIndex) async {
    final fixtures = await _ensureFixtures();
    for (final f in fixtures) {
      if (f.profile.walletIndex == walletIndex) {
        return f.profile;
      }
    }
    return null;
  }

  Future<WalletProfile> buildColdWalletProfile(int walletIndex) async {
    final fixtures = await _ensureFixtures();
    for (final f in fixtures) {
      if (f.profile.walletIndex == walletIndex) {
        return WalletProfile(
          walletIndex: f.profile.walletIndex,
          walletName: '${f.profile.walletName}-冷',
          walletIcon: f.profile.walletIcon,
          balance: f.profile.balance,
          address: f.profile.address,
          pubkeyHex: f.profile.pubkeyHex,
          alg: f.profile.alg,
          ss58: f.profile.ss58,
          createdAtMillis: f.profile.createdAtMillis,
          source: f.profile.source,
          signMode: 'external',
        );
      }
    }
    throw StateError('未找到测试钱包');
  }

  @override
  Future<WalletSignResult> signUtf8WithWallet(
    int walletIndex,
    String message,
  ) async {
    final fixtures = await _ensureFixtures();
    for (final f in fixtures) {
      if (f.profile.walletIndex == walletIndex) {
        final seedBytes = _hexToBytes(f.seedHex);
        final pair = Keyring.sr25519.fromSeed(Uint8List.fromList(seedBytes));
        pair.ss58Format = _ss58;
        final payload = Uint8List.fromList(utf8.encode(message));
        final signature = pair.sign(payload);
        return WalletSignResult(
          account: f.profile.address,
          pubkeyHex: '0x${f.profile.pubkeyHex}',
          sigAlg: 'sr25519',
          signatureHex: '0x${_toHex(signature.toList(growable: false))}',
        );
      }
    }
    throw const WalletAuthException('未找到指定钱包');
  }

  List<int> _hexToBytes(String input) {
    final text = input.startsWith('0x') ? input.substring(2) : input;
    if (text.isEmpty || text.length.isOdd) return const <int>[];
    final out = <int>[];
    for (var i = 0; i < text.length; i += 2) {
      out.add(int.parse(text.substring(i, i + 2), radix: 16));
    }
    return out;
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

class _WalletFixture {
  const _WalletFixture({
    required this.profile,
    required this.seedHex,
  });

  final WalletProfile profile;
  final String seedHex;
}

class _FakeSystemSignatureVerifier extends LoginSystemSignatureVerifier {
  @override
  Future<void> verify(LoginChallenge challenge) async {}
}
