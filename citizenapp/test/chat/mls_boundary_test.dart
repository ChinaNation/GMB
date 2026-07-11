import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/chat/crypto/mls_boundary.dart';

void main() {
  group('ChatDevice', () {
    test('accepts wallet account as chat account without wallet private key',
        () {
      const identity = ChatDevice(
        ownerAccount: 'alice-wallet',
        deviceId: 'alice-phone',
        devicePublicKeyHex: '0xaabbcc',
      );

      expect(identity.validate(), isNull);
      expect(identity.ownerAccount, 'alice-wallet');
      expect(identity.deviceId, 'alice-phone');
    });

    test('rejects invalid device public key hex', () {
      const identity = ChatDevice(
        ownerAccount: 'alice-wallet',
        deviceId: 'alice-phone',
        devicePublicKeyHex: 'xyz',
      );

      expect(identity.validate(), contains('hex'));
    });
  });
}
