import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/organization-manage/shared/duoqian_storage_codec.dart';

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
    test('builds current subject ids', () {
      final builtin = DuoqianStorageCodec.subjectIdFromBuiltin(
        'GFR-LN001-CB0X-944805165-2026',
      );
      expect(builtin.length, 48);
      expect(builtin[0], 0x01);

      final sfid = DuoqianStorageCodec.subjectIdFromSfidBytes(
        Uint8List.fromList(utf8.encode('SFR-AH001-20260507')),
      );
      expect(sfid.length, 48);
      expect(sfid[0], 0x02);

      final institutionAccount =
          DuoqianStorageCodec.subjectIdFromInstitutionAccountHex('22' * 32);
      expect(institutionAccount.length, 48);
      expect(institutionAccount[0], 0x05);
      expect(institutionAccount.sublist(1, 33), List<int>.filled(32, 0x22));
      expect(institutionAccount.sublist(33), List<int>.filled(15, 0));
    });

    test('decodes registered institution ref', () {
      final data = Uint8List.fromList([
        ...compactVec('SFR-AH001-20260507'),
        ...compactVec('主账户'),
      ]);

      final decoded = DuoqianStorageCodec.decodeRegisteredInstitution(data)!;
      expect(decoded.sfidNumberText, 'SFR-AH001-20260507');
      expect(decoded.accountNameText, '主账户');
    });

    test('decodes admin subject', () {
      final admin1 = List<int>.filled(32, 0xaa);
      final admin2 = List<int>.filled(32, 0xbb);
      final data = Uint8List.fromList([
        3,
        1,
        (2 << 2) & 0xff,
        ...admin1,
        ...admin2,
        ...u32Le(2),
      ]);

      final decoded = DuoqianStorageCodec.decodeAdminSubject(data)!;
      expect(decoded.adminCount, 2);
      expect(decoded.threshold, 2);
      expect(decoded.adminPubkeys, ['aa' * 32, 'bb' * 32]);
    });

    test('decodes institution info and account state', () {
      final admin1 = List<int>.filled(32, 0x11);
      final admin2 = List<int>.filled(32, 0x22);
      final institution = Uint8List.fromList([
        ...compactVec('安徽省储行'),
        ...List<int>.filled(32, 0xa1),
        ...List<int>.filled(32, 0xa2),
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
      expect(institutionDecoded.adminCount, 2);
      expect(institutionDecoded.threshold, 2);
      expect(institutionDecoded.adminPubkeys, ['11' * 32, '22' * 32]);
      expect(institutionDecoded.statusByte, 1);
      expect(accountDecoded.statusByte, 1);
      expect(accountDecoded.addressHex, 'd1' * 32);
    });
  });
}
