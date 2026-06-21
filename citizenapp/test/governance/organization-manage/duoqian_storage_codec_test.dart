import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/governance/organization-manage/duoqian_storage_codec.dart';

void main() {
  List<int> compactVec(String text) {
    final bytes = utf8.encode(text);
    return [(bytes.length << 2) & 0xff, ...bytes];
  }

  List<int> u32Le(int value) => [
        value & 0xff,
        (value >> 8) & 0xff,
        (value >> 16) & 0xff,
        (value >> 24) & 0xff,
      ];

  List<int> u128Le(BigInt value) {
    final out = List<int>.filled(16, 0);
    var tmp = value;
    for (var i = 0; i < 16; i++) {
      out[i] = (tmp & BigInt.from(0xff)).toInt();
      tmp = tmp >> 8;
    }
    return out;
  }

  group('DuoqianStorageCodec', () {
    test('builds current account ids', () {
      final institutionAccount =
          DuoqianStorageCodec.accountIdFromAccountHex('22' * 32);
      expect(institutionAccount.length, 32);
      expect(institutionAccount, List<int>.filled(32, 0x22));
    });

    test('decodes registered institution ref', () {
      final data = Uint8List.fromList([
        ...compactVec('AH001-SCB0H-202605070-2026'),
        ...compactVec('主账户'),
      ]);

      final decoded = DuoqianStorageCodec.decodeRegisteredInstitution(data)!;
      expect(decoded.sfidNumberText, 'AH001-SCB0H-202605070-2026');
      expect(decoded.accountNameText, '主账户');
    });

    test('decodes admin account', () {
      final admin1 = List<int>.filled(32, 0xaa);
      final admin2 = List<int>.filled(32, 0xbb);
      final data = Uint8List.fromList([
        5,
        3,
        (2 << 2) & 0xff,
        ...admin1,
        ...admin2,
        ...List<int>.filled(32, 0x44),
        ...u32Le(100),
        ...u32Le(101),
        1,
      ]);

      final decoded = DuoqianStorageCodec.decodeAdminAccount(data)!;
      expect(decoded.org, 5);
      expect(decoded.adminsLen, 2);
      expect(decoded.threshold, isNull);
      expect(decoded.admins, ['aa' * 32, 'bb' * 32]);
    });

    test('decodes institution info and account state', () {
      final admin1 = List<int>.filled(32, 0x11);
      final admin2 = List<int>.filled(32, 0x22);
      final institution = Uint8List.fromList([
        ...compactVec('安徽省储行'),
        ...List<int>.filled(32, 0xa1),
        ...List<int>.filled(32, 0xa2),
        5,
        ...u32Le(2),
        ...u32Le(2),
        (2 << 2) & 0xff,
        ...admin1,
        ...admin2,
        ...List<int>.filled(32, 0xc1),
        ...u32Le(100),
        1,
        ...u32Le(2),
      ]);
      final account = Uint8List.fromList([
        ...List<int>.filled(32, 0xd1),
        ...u128Le(BigInt.from(111)),
        1,
        1,
        ...u32Le(100),
      ]);

      final institutionDecoded =
          DuoqianStorageCodec.decodeInstitutionInfo(institution)!;
      final accountDecoded =
          DuoqianStorageCodec.decodeInstitutionAccount(account)!;
      expect(institutionDecoded.adminsLen, 2);
      expect(institutionDecoded.threshold, 2);
      expect(institutionDecoded.admins, ['11' * 32, '22' * 32]);
      expect(institutionDecoded.statusByte, 1);
      expect(accountDecoded.statusByte, 1);
      expect(accountDecoded.addressHex, 'd1' * 32);
    });
  });
}
