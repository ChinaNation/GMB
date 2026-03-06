import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/login/models/login_exception.dart';
import 'package:wuminapp_mobile/login/services/login_replay_guard.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('LoginReplayGuard', () {
    setUp(() {
      SharedPreferences.setMockInitialValues(<String, Object>{});
    });

    test('consume should persist consumed request id', () async {
      final guard = LoginReplayGuard();
      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      const requestId = 'req-consumed-1';

      await guard.consume(requestId: requestId, expiresAt: now + 90);

      expect(await guard.isConsumed(requestId), isTrue);
    });

    test('consume should cleanup expired request ids', () async {
      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      SharedPreferences.setMockInitialValues(<String, Object>{
        'login.used_request_ids': jsonEncode(<String, int>{
          'req-expired': now - 1,
        }),
      });
      final guard = LoginReplayGuard();

      await guard.consume(requestId: 'req-new', expiresAt: now + 90);

      expect(await guard.isConsumed('req-expired'), isFalse);
      expect(await guard.isConsumed('req-new'), isTrue);
    });

    test('assertNotConsumed should throw replay for consumed id', () async {
      final guard = LoginReplayGuard();
      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;

      await guard.consume(requestId: 'req-replay', expiresAt: now + 90);

      await expectLater(
        guard.assertNotConsumed('req-replay'),
        throwsA(
          isA<LoginException>().having(
            (e) => e.code,
            'code',
            LoginErrorCode.replay,
          ),
        ),
      );
    });
  });
}
