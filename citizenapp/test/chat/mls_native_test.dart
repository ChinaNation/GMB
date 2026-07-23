import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/chat/crypto/mls_boundary.dart';
import 'package:citizenapp/chat/crypto/mls_native.dart';

import '../support/smoldot_native_probe.dart';

void main() {
  // libsmoldot 不可用(纯 Dart CI 无宿主 .so)则跳过;真机/集成构建照跑。
  final skip = smoldotNativeSkipReason();

  test('native OpenMLS creates a real KeyPackage', () async {
    final crypto = NativeMlsCrypto();
    final keyPackage = await crypto.createKeyPackage(
      const ChatDevice(
        accountId:
            '0x1111111111111111111111111111111111111111111111111111111111111111',
        deviceId: 'alice-phone',
        devicePublicKey: 'aabbcc',
      ),
    );

    expect(keyPackage.accountId,
        '0x1111111111111111111111111111111111111111111111111111111111111111');
    expect(keyPackage.deviceId, 'alice-phone');
    expect(keyPackage.devicePublicKey, isNotEmpty);
    expect(RegExp(r'^[0-9a-f]+$').hasMatch(keyPackage.devicePublicKey), isTrue);
    expect(keyPackage.keyPackageBytes.length, greaterThan(100));
    expect(keyPackage.cipherSuite, contains('MLS_128'));
  }, skip: skip);

  test('native OpenMLS two-party smoke decrypts original plaintext', () async {
    final crypto = NativeMlsCrypto();
    final result = await crypto.runTwoPartySmoke(
      plaintext: 'hello from 公民 Chat',
    );

    expect(result.roundTripOk, isTrue);
    expect(result.decryptedPlaintext, 'hello from 公民 Chat');
    expect(result.aliceWireMessageHex.length, greaterThan(100));
    expect(result.bobKeyPackageHex.length, greaterThan(100));
    expect(result.welcomeHex.length, greaterThan(100));
  }, skip: skip);
}
