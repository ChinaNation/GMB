import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:wuminapp_mobile/login/models/login_models.dart';
import 'package:wuminapp_mobile/login/services/login_replay_guard.dart';
import 'package:wuminapp_mobile/login/services/login_whitelist_policy.dart';
import 'package:wuminapp_mobile/login/services/wuminapp_login_service.dart';
import 'package:wuminapp_mobile/services/wallet_service.dart';

void main() {
  group('WuminLoginService', () {
    late _FakeWalletService walletService;
    late _FakeReplayGuard replayGuard;
    late _FakeWhitelistPolicy whitelistPolicy;
    late WuminLoginService service;

    setUp(() {
      walletService = _FakeWalletService();
      replayGuard = _FakeReplayGuard();
      whitelistPolicy = _FakeWhitelistPolicy();
      service = WuminLoginService(
        walletService: walletService,
        replayGuard: replayGuard,
        whitelistPolicy: whitelistPolicy,
      );
    });

    test('parseChallenge should parse a valid challenge', () {
      final raw = _challengeJson(
        requestId: 'req-1',
        expiresAt: _nowSec() + 60,
      );
      final challenge = service.parseChallenge(raw);

      expect(challenge.proto, WuminLoginService.protocol);
      expect(challenge.system, 'cpms');
      expect(challenge.requestId, 'req-1');
      expect(challenge.aud, 'cpms-local-app');
      expect(challenge.origin, 'cpms-device-id');
    });

    test('parseChallenge should reject expired challenge', () {
      final raw = _challengeJson(
        requestId: 'req-expired',
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

    test('buildSignPreviewForChallenge should follow canonical format', () {
      final raw = _challengeJson(
        requestId: 'req-preview',
        challenge: 'abc123',
        nonce: 'nonce-xyz',
        expiresAt: 4102444800,
      );
      final challenge = service.parseChallenge(raw);
      final signPreview = service.buildSignPreviewForChallenge(challenge);

      expect(
        signPreview,
        'WUMINAPP_LOGIN_V1|cpms|cpms-local-app|cpms-device-id|req-preview|abc123|nonce-xyz|4102444800',
      );
    });

    test('buildReceiptPayloadForChallenge should sign with selected wallet', () async {
      final raw = _challengeJson(
        requestId: 'req-wallet-2',
        system: 'sfid',
        aud: 'sfid-local-app',
        origin: 'sfid-device-id',
        expiresAt: _nowSec() + 60,
      );
      final challenge = service.parseChallenge(raw);

      final receipt = await service.buildReceiptPayloadForChallenge(
        challenge,
        walletIndex: 2,
      );

      final wallet2 = await walletService.getWalletSecretByIndex(2);
      expect(wallet2, isNotNull);
      expect(receipt['account'], wallet2!.profile.address);
      expect(receipt['pubkey'], '0x${wallet2.profile.pubkeyHex}');
      expect(receipt['proto'], WuminLoginService.protocol);
      expect(receipt['sig_alg'], 'sr25519');
      expect(receipt['signature'], startsWith('0x'));
    });

    test('buildReceiptPayloadForChallenge should block replay request_id', () async {
      final raw = _challengeJson(
        requestId: 'req-replay',
        expiresAt: _nowSec() + 60,
      );
      final challenge = service.parseChallenge(raw);

      await service.buildReceiptPayloadForChallenge(challenge, walletIndex: 1);
      expect(
        () => service.buildReceiptPayloadForChallenge(challenge, walletIndex: 1),
        throwsA(
          isA<Exception>().having(
            (e) => e.toString(),
            'message',
            contains('已使用'),
          ),
        ),
      );
    });

    test('buildReceiptPayloadForChallenge should fail when walletIndex not found', () async {
      final raw = _challengeJson(
        requestId: 'req-wallet-missing',
        expiresAt: _nowSec() + 60,
      );
      final challenge = service.parseChallenge(raw);

      expect(
        () => service.buildReceiptPayloadForChallenge(challenge, walletIndex: 99),
        throwsA(
          isA<Exception>().having(
            (e) => e.toString(),
            'message',
            contains('未找到指定钱包'),
          ),
        ),
      );
    });
  });
}

int _nowSec() => DateTime.now().millisecondsSinceEpoch ~/ 1000;

String _challengeJson({
  required String requestId,
  String system = 'cpms',
  String challenge = 'base64-rand',
  String nonce = 'nonce-1',
  int? issuedAt,
  required int expiresAt,
  String aud = 'cpms-local-app',
  String origin = 'cpms-device-id',
}) {
  final iat = issuedAt ?? (_nowSec() - 1);
  return '''
{
  "proto": "WUMINAPP_LOGIN_V1",
  "system": "$system",
  "request_id": "$requestId",
  "challenge": "$challenge",
  "nonce": "$nonce",
  "issued_at": $iat,
  "expires_at": $expiresAt,
  "aud": "$aud",
  "origin": "$origin"
}
''';
}

class _FakeWhitelistPolicy extends LoginWhitelistPolicy {
  _FakeWhitelistPolicy();

  @override
  Future<void> assertAllowed(WuminLoginChallenge challenge) async {
    return;
  }
}

class _FakeReplayGuard extends LoginReplayGuard {
  final Set<String> _used = <String>{};

  @override
  Future<void> assertNotConsumed(String requestId) async {
    if (_used.contains(requestId)) {
      throw Exception('登录挑战已使用，请刷新二维码后重试');
    }
  }

  @override
  Future<void> consume({
    required String requestId,
    required int expiresAt,
  }) async {
    _used.add(requestId);
  }
}

class _FakeWalletService extends WalletService {
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
    final pair = await Keyring.sr25519.fromMnemonic(mnemonic);
    pair.ss58Format = _ss58;
    final pubkeyHex = _toHex(pair.bytes().toList(growable: false));
    return _WalletFixture(
      profile: WalletProfile(
        walletIndex: index,
        address: pair.address,
        pubkeyHex: pubkeyHex,
        alg: 'sr25519',
        ss58: _ss58,
        createdAtMillis: DateTime.now().millisecondsSinceEpoch,
        source: 'test',
      ),
      mnemonic: mnemonic,
    );
  }

  @override
  Future<WalletSecret?> getLatestWalletSecret() async {
    final fixtures = await _ensureFixtures();
    final f = fixtures.last;
    return WalletSecret(profile: f.profile, mnemonic: f.mnemonic);
  }

  @override
  Future<WalletSecret?> getWalletSecretByIndex(int walletIndex) async {
    final fixtures = await _ensureFixtures();
    for (final f in fixtures) {
      if (f.profile.walletIndex == walletIndex) {
        return WalletSecret(profile: f.profile, mnemonic: f.mnemonic);
      }
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

class _WalletFixture {
  const _WalletFixture({
    required this.profile,
    required this.mnemonic,
  });

  final WalletProfile profile;
  final String mnemonic;
}
