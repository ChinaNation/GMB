import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/transaction/organization-manage/multisig_storage_codec.dart';

void main() {
  List<int> codeBytes(String code) {
    final out = List<int>.filled(4, 0);
    final raw = code.codeUnits;
    for (var i = 0; i < out.length && i < raw.length; i++) {
      out[i] = raw[i];
    }
    return out;
  }

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

  group('MultisigStorageCodec', () {
    test('builds current account ids', () {
      final institutionAccount =
          MultisigStorageCodec.accountIdFromAccountHex('22' * 32);
      expect(institutionAccount.length, 32);
      expect(institutionAccount, List<int>.filled(32, 0x22));
    });

    test('decodes registered institution ref', () {
      final data = Uint8List.fromList([
        ...compactVec('AH001-SCB0H-202605070-2026'),
        ...compactVec('主账户'),
      ]);

      final decoded = MultisigStorageCodec.decodeRegisteredInstitution(data)!;
      expect(decoded.cidNumberText, 'AH001-SCB0H-202605070-2026');
      expect(decoded.accountNameText, '主账户');
    });

    test('decodes admin account', () {
      final admin1 = List<int>.filled(32, 0xaa);
      final admin2 = List<int>.filled(32, 0xbb);
      final data = Uint8List.fromList([
        ...codeBytes('UNIN'),
        3,
        (2 << 2) & 0xff,
        ...admin1,
        ...admin2,
        ...List<int>.filled(32, 0x44),
        ...u32Le(100),
        ...u32Le(101),
        1,
      ]);

      final decoded = MultisigStorageCodec.decodeAdminAccount(data)!;
      expect(decoded.institutionCode, 'UNIN');
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
        ...codeBytes('UNIN'),
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
          MultisigStorageCodec.decodeInstitutionInfo(institution)!;
      final accountDecoded =
          MultisigStorageCodec.decodeInstitutionAccount(account)!;
      expect(institutionDecoded.institutionCode, 'UNIN');
      expect(institutionDecoded.adminsLen, 2);
      expect(institutionDecoded.threshold, 2);
      expect(institutionDecoded.admins, ['11' * 32, '22' * 32]);
      expect(institutionDecoded.statusByte, 1);
      expect(accountDecoded.statusByte, 1);
      expect(accountDecoded.accountHex, 'd1' * 32);
    });
  });
}
