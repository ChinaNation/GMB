import 'dart:convert';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/wallet/core/device_data_key_vault.dart';
import 'package:citizenapp/wallet/core/device_subkey.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('derEcdsaToRaw', () {
    test('small r/s left-padded to 32 bytes each', () {
      final der =
          Uint8List.fromList([0x30, 0x06, 0x02, 0x01, 0x01, 0x02, 0x01, 0x02]);
      final raw = derEcdsaToRaw(der);
      expect(raw.length, 64);
      expect(raw[31], 1);
      expect(raw[63], 2);
      expect(raw.where((b) => b != 0).length, 2);
    });

    test('strips sign-padding leading zero on r', () {
      final der = Uint8List.fromList(
        [0x30, 0x07, 0x02, 0x02, 0x00, 0x81, 0x02, 0x01, 0x02],
      );
      final raw = derEcdsaToRaw(der);
      expect(raw[31], 0x81);
      expect(raw[63], 2);
    });

    test('full 32-byte r/s preserved', () {
      final r = List<int>.generate(32, (i) => i + 1); // 0x01..0x20
      final s = List<int>.generate(32, (i) => i + 0x21); // 0x21..0x40
      final der = <int>[0x30, 0x44, 0x02, 0x20, ...r, 0x02, 0x20, ...s];
      final raw = derEcdsaToRaw(Uint8List.fromList(der));
      expect(raw.sublist(0, 32), r);
      expect(raw.sublist(32, 64), s);
    });

    test('rejects non-sequence', () {
      expect(
        () => derEcdsaToRaw(Uint8List.fromList([0x31, 0x00])),
        throwsFormatException,
      );
    });
  });

  group('DeviceSubkey channel', () {
    const channel = MethodChannel('org.citizenapp/device_subkey');
    late List<MethodCall> calls;
    late DeviceSubkey subkey;
    String? publicKeyReturn;
    late String signReturnDerHex;

    setUp(() {
      calls = <MethodCall>[];
      publicKeyReturn = '04${'00' * 64}';
      signReturnDerHex = bytesToHex(
        Uint8List.fromList([0x30, 0x06, 0x02, 0x01, 0x01, 0x02, 0x01, 0x02]),
      );
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (MethodCall call) async {
        calls.add(call);
        switch (call.method) {
          case 'publicKey':
            return publicKeyReturn;
          case 'sign':
            return signReturnDerHex;
          case 'delete':
            return null;
        }
        return null;
      });
      subkey = DeviceSubkey(channel: channel);
    });

    tearDown(() {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, null);
    });

    test('publicKeyHex returns native publicKey', () async {
      expect(await subkey.publicKeyHex(1), '04${'00' * 64}');
      expect(calls.single.arguments['walletIndex'], 1);
    });

    test('publicKeyHex throws when native returns null', () async {
      publicKeyReturn = null;
      await expectLater(
        () => subkey.publicKeyHex(1),
        throwsA(isA<DeviceSubkeyException>()),
      );
    });

    test('signRaw base64-encodes payload and converts DER to raw', () async {
      final payload = Uint8List.fromList([1, 2, 3]);
      final raw = await subkey.signRaw(2, payload);
      expect(raw.length, 64);
      expect(raw[31], 1);
      expect(raw[63], 2);
      final signCall = calls.firstWhere((c) => c.method == 'sign');
      expect(signCall.arguments['walletIndex'], 2);
      expect(signCall.arguments['payload'], base64Encode(payload));
    });

    test('signRawHex returns 128-hex-char raw signature', () async {
      final hex = await subkey.signRawHex(2, Uint8List.fromList([9]));
      expect(hex.length, 128);
    });
  });

  group('DeviceDataKeyVault channel', () {
    const channel = MethodChannel('org.citizenapp/device_data_key_vault');
    late List<MethodCall> calls;
    late DeviceDataKeyVault vault;

    setUp(() {
      calls = <MethodCall>[];
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (MethodCall call) async {
        calls.add(call);
        switch (call.method) {
          case 'seal':
            return 'native-sealed-blob';
          case 'open':
            return base64Encode(<int>[7, 8, 9]);
          case 'delete':
            return null;
        }
        return null;
      });
      vault = DeviceDataKeyVault(channel: channel);
    });

    tearDown(() {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, null);
    });

    test('seal/open 逐字节转交明文和 AAD，不触碰钱包通道', () async {
      final plaintext = Uint8List.fromList(<int>[1, 2, 3]);
      final aad = Uint8List.fromList(<int>[4, 5, 6]);

      expect(
        await vault.seal(walletIndex: 3, plaintext: plaintext, aad: aad),
        'native-sealed-blob',
      );
      expect(
        await vault.open(walletIndex: 3, blob: 'blob', aad: aad),
        <int>[7, 8, 9],
      );

      final sealCall = calls.firstWhere((call) => call.method == 'seal');
      expect(sealCall.arguments['walletIndex'], 3);
      expect(sealCall.arguments['plaintext'], base64Encode(plaintext));
      expect(sealCall.arguments['aad'], base64Encode(aad));
      final openCall = calls.firstWhere((call) => call.method == 'open');
      expect(openCall.arguments['blob'], 'blob');
      expect(openCall.arguments['aad'], base64Encode(aad));
    });

    test('空明文、空 AAD 和负 walletIndex 在原生调用前失败关闭', () async {
      await expectLater(
        vault.seal(
          walletIndex: 1,
          plaintext: Uint8List(0),
          aad: Uint8List.fromList(<int>[1]),
        ),
        throwsA(isA<DeviceDataKeyVaultException>()),
      );
      await expectLater(
        vault.open(
          walletIndex: 1,
          blob: 'blob',
          aad: Uint8List(0),
        ),
        throwsA(isA<DeviceDataKeyVaultException>()),
      );
      await expectLater(
        vault.delete(-1),
        throwsA(isA<DeviceDataKeyVaultException>()),
      );
      expect(calls, isEmpty);
    });
  });
}
