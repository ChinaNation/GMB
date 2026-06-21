import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/im/crypto/im_mls_boundary.dart';
import 'package:citizenapp/im/crypto/im_mls_native.dart';

import '../support/smoldot_native_probe.dart';

void main() {
  // libsmoldot 不可用(纯 Dart CI 无宿主 .so)则跳过;真机/集成构建照跑。
  final skip = smoldotNativeSkipReason();

  test('native OpenMLS creates a real KeyPackage', () async {
    final crypto = NativeImMlsCrypto();
    final keyPackage = await crypto.createKeyPackage(
      const ImMlsDeviceIdentity(
        walletChatAccount: 'alice-wallet',
        deviceId: 'alice-phone',
        devicePublicKeyHex: 'aabbcc',
      ),
    );

    expect(keyPackage.ownerChatAccount, 'alice-wallet');
    expect(keyPackage.deviceId, 'alice-phone');
    expect(keyPackage.devicePublicKeyHex, isNotEmpty);
    expect(
        RegExp(r'^[0-9a-f]+$').hasMatch(keyPackage.devicePublicKeyHex), isTrue);
    expect(keyPackage.keyPackageBytes.length, greaterThan(100));
    expect(keyPackage.cipherSuite, contains('MLS_128'));
  }, skip: skip);

  test('native OpenMLS two-party smoke decrypts original plaintext', () async {
    final crypto = NativeImMlsCrypto();
    final result = await crypto.runTwoPartySmoke(
      plaintext: 'hello from 公民 IM',
    );

    expect(result.roundTripOk, isTrue);
    expect(result.decryptedPlaintext, 'hello from 公民 IM');
    expect(result.aliceWireMessageHex.length, greaterThan(100));
    expect(result.bobKeyPackageHex.length, greaterThan(100));
    expect(result.welcomeHex.length, greaterThan(100));
  }, skip: skip);
}
