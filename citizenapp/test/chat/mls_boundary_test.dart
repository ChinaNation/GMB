import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/chat/crypto/mls_boundary.dart';

void main() {
  group('ChatDevice', () {
    test('accepts CID as chat identity without wallet private key', () {
      const identity = ChatDevice(
        cidNumber: 'CN220-CTZN2-100000001-2026',
        deviceId: 'alice-phone',
        devicePublicKey: '0xaabbcc',
      );

      expect(identity.validate(), isNull);
      expect(identity.cidNumber, 'CN220-CTZN2-100000001-2026');
      expect(identity.deviceId, 'alice-phone');
    });

    test('rejects invalid device public key hex', () {
      const identity = ChatDevice(
        cidNumber: 'CN220-CTZN2-100000001-2026',
        deviceId: 'alice-phone',
        devicePublicKey: 'xyz',
      );

      expect(identity.validate(), contains('hex'));
    });
  });
}
