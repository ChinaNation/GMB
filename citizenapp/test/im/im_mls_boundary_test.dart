import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/im/crypto/im_mls_boundary.dart';

void main() {
  group('ImMlsDeviceIdentity', () {
    test('accepts wallet account as chat account without wallet private key',
        () {
      const identity = ImMlsDeviceIdentity(
        walletChatAccount: 'alice-wallet',
        deviceId: 'alice-phone',
        devicePublicKeyHex: '0xaabbcc',
      );

      expect(identity.validate(), isNull);
      expect(identity.walletChatAccount, 'alice-wallet');
      expect(identity.deviceId, 'alice-phone');
    });

    test('rejects invalid device public key hex', () {
      const identity = ImMlsDeviceIdentity(
        walletChatAccount: 'alice-wallet',
        deviceId: 'alice-phone',
        devicePublicKeyHex: 'xyz',
      );

      expect(identity.validate(), contains('hex'));
    });
  });

  group('ImMlsKeyPackage', () {
    test('round-trips node json and hex bytes', () {
      const package = ImMlsKeyPackage(
        ownerChatAccount: 'bob-wallet',
        deviceId: 'bob-phone',
        devicePublicKeyHex: '0xaabb',
        keyPackageId: 'kp-1',
        keyPackageBytes: [0xaa, 0xbb, 0xcc],
        cipherSuite: 'MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519',
        createdAtMillis: 1,
        expiresAtMillis: 2,
      );

      final json = package.toPublishJson();
      expect(json['key_package_hex'], 'aabbcc');

      final restored = ImMlsKeyPackage.fromNodeJson({
        ...json,
        'protocol_version': 1,
        'consumed_at_millis': 3,
      });
      expect(restored.ownerChatAccount, 'bob-wallet');
      expect(restored.devicePublicKeyHex, '0xaabb');
      expect(restored.keyPackageBytes, [0xaa, 0xbb, 0xcc]);
      expect(restored.consumedAtMillis, 3);
    });
  });
}
