import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/chat/crypto/mls_boundary.dart';

void main() {
  group('ChatDevice', () {
    test('accepts wallet account as chat account without wallet private key',
        () {
      const identity = ChatDevice(
        accountId:
            '0x1111111111111111111111111111111111111111111111111111111111111111',
        deviceId: 'alice-phone',
        devicePublicKey: '0xaabbcc',
      );

      expect(identity.validate(), isNull);
      expect(identity.accountId,
          '0x1111111111111111111111111111111111111111111111111111111111111111');
      expect(identity.deviceId, 'alice-phone');
    });

    test('rejects invalid device public key hex', () {
      const identity = ChatDevice(
        accountId:
            '0x1111111111111111111111111111111111111111111111111111111111111111',
        deviceId: 'alice-phone',
        devicePublicKey: 'xyz',
      );

      expect(identity.validate(), contains('hex'));
    });
  });
}
