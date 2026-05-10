import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/governance/personal-manage/personal_manage_storage_codec.dart';

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

  group('PersonalManageStorageCodec', () {
    test('builds personal subject id', () {
      final personal =
          PersonalManageStorageCodec.subjectIdFromAccountHex('11' * 32);

      expect(personal.length, 48);
      expect(
        personal[0],
        PersonalManageStorageCodec.subjectKindPersonalDuoqian,
      );
      expect(personal.sublist(1, 33), List<int>.filled(32, 0x11));
      expect(personal.sublist(33), List<int>.filled(15, 0));
    });

    test('decodes personal account state', () {
      final data = Uint8List.fromList([
        ...List<int>.filled(32, 0x66),
        ...compactVec('家庭基金'),
        ...u32Le(101),
        1,
      ]);

      final decoded = PersonalManageStorageCodec.decodePersonalDuoqian(data)!;
      expect(decoded.creatorHex, '66' * 32);
      expect(utf8.decode(decoded.accountName), '家庭基金');
      expect(decoded.createdAt, 101);
      expect(decoded.statusByte, 1);
    });
  });
}
