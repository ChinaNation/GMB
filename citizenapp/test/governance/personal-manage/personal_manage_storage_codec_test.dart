import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/transaction/personal-manage/personal_manage_storage_codec.dart';

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

  group('PersonalManageStorageCodec', () {
    test('builds personal account id', () {
      final personal =
          PersonalManageStorageCodec.accountIdFromAccountHex('11' * 32);

      expect(personal.length, 32);
      expect(personal, List<int>.filled(32, 0x11));
    });

    test('decodes personal account state', () {
      final data = Uint8List.fromList([
        ...List<int>.filled(32, 0x66),
        ...compactVec('家庭基金'),
        ...u32Le(101),
        1,
      ]);

      final decoded = PersonalManageStorageCodec.decodePersonalAccount(data)!;
      expect(decoded.creatorHex, '66' * 32);
      expect(utf8.decode(decoded.accountName), '家庭基金');
      expect(decoded.createdAt, 101);
      expect(decoded.statusByte, 1);
    });

    test('decodes admin account without reading threshold from account', () {
      final admin1 = List<int>.filled(32, 0xaa);
      final admin2 = List<int>.filled(32, 0xbb);
      final data = Uint8List.fromList([
        ...codeBytes('PMUL'),
        2,
        (2 << 2) & 0xff,
        ...admin1,
        ...admin2,
        ...List<int>.filled(32, 0x44),
        ...u32Le(100),
        ...u32Le(101),
        1,
      ]);

      final decoded = PersonalManageStorageCodec.decodeAdminAccount(data)!;

      expect(decoded.institutionCode, 'PMUL');
      expect(decoded.adminsLen, 2);
      expect(decoded.admins, ['aa' * 32, 'bb' * 32]);
    });
  });
}
