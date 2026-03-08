import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/login/models/login_exception.dart';
import 'package:wuminapp_mobile/login/models/login_models.dart';
import 'package:wuminapp_mobile/login/services/login_whitelist_policy.dart';
import 'package:wuminapp_mobile/login/services/login_whitelist_store.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('LoginWhitelistPolicy', () {
    test('assertAllowed should pass when aud is whitelisted', () async {
      final policy = LoginWhitelistPolicy(
        whitelistStore: _FakeWhitelistStore(
          const LoginWhitelistConfig(
            audWhitelist: <String, Set<String>>{
              'cpms': <String>{'cpms-local-app'},
            },
          ),
        ),
      );
      final challenge = _challenge(system: 'cpms', aud: 'cpms-local-app');

      await policy.assertAllowed(challenge);
    });

    test('assertAllowed should reject when aud is not whitelisted', () async {
      final policy = LoginWhitelistPolicy(
        whitelistStore: _FakeWhitelistStore(
          const LoginWhitelistConfig(
            audWhitelist: <String, Set<String>>{
              'cpms': <String>{'cpms-local-app'},
            },
          ),
        ),
      );
      final challenge = _challenge(system: 'cpms', aud: 'cpms-unknown-app');

      await expectLater(
        policy.assertAllowed(challenge),
        throwsA(
          isA<LoginException>().having(
            (e) => e.code,
            'code',
            LoginErrorCode.unauthorizedAud,
          ),
        ),
      );
    });
  });
}

WuminLoginChallenge _challenge({
  required String system,
  required String aud,
}) {
  final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
  return WuminLoginChallenge(
    proto: 'WUMINAPP_LOGIN_V1',
    system: system,
    requestId: 'req-whitelist-1',
    challenge: 'challenge-token',
    nonce: 'nonce-token',
    issuedAt: now - 1,
    expiresAt: now + 89,
    aud: aud,
    raw: '{}',
  );
}

class _FakeWhitelistStore extends LoginWhitelistStore {
  _FakeWhitelistStore(this._config);

  final LoginWhitelistConfig _config;

  @override
  Future<LoginWhitelistConfig> load() async {
    return _config;
  }
}
