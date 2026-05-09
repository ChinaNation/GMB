import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/admins_change/codec/admin_set_change_call_codec.dart';
import 'package:wuminapp_mobile/admins_change/codec/admin_subject_codec.dart';
import 'package:wuminapp_mobile/admins_change/codec/subject_id_codec.dart';
import 'package:wuminapp_mobile/admins_change/models/admin_subject.dart';
import 'package:wuminapp_mobile/admins_change/services/admin_set_validation.dart';

void main() {
  List<int> u32Le(int value) => [
        value & 0xff,
        (value >> 8) & 0xff,
        (value >> 16) & 0xff,
        (value >> 24) & 0xff,
      ];

  List<int> u64Le(int value) {
    final out = List<int>.filled(8, 0);
    var tmp = value;
    for (var i = 0; i < out.length; i++) {
      out[i] = tmp & 0xff;
      tmp >>= 8;
    }
    return out;
  }

  group('admins_change codec', () {
    test('builds AdminsChange::Subjects storage key', () {
      final subjectId = AdminSubjectIdCodec.fromBuiltinSfid(
        'GFR-LN001-CB0X-944805165-2026',
      );
      final key = AdminSubjectIdCodec.adminSubjectStorageKey(subjectId);

      expect(subjectId.length, 48);
      expect(subjectId[0], AdminSubjectIdCodec.builtinInstitution);
      expect(key.length, 16 + 16 + 16 + 48);
    });

    test('decodes full AdminSubject value', () {
      final subjectId = AdminSubjectIdCodec.fromBuiltinSfid(
        'GFR-LN001-CB0X-944805165-2026',
      );
      final data = Uint8List.fromList([
        0,
        0,
        0x08,
        ...List<int>.filled(32, 0xaa),
        ...List<int>.filled(32, 0xbb),
        ...u32Le(13),
        ...List<int>.filled(32, 0xcc),
        ...u64Le(7),
        ...u64Le(9),
        1,
      ]);

      final decoded = AdminSubjectCodec.decode(subjectId, data)!;
      expect(decoded.admins, ['aa' * 32, 'bb' * 32]);
      expect(decoded.threshold, 13);
      expect(decoded.creatorHex, 'cc' * 32);
      expect(decoded.statusLabel, '已激活');
    });

    test('builds propose_admin_set_change call data', () {
      final subjectId = Uint8List.fromList(List<int>.filled(48, 0x11));
      final call = AdminSetChangeCallCodec.build(
        org: 0,
        subjectId: subjectId,
        newAdmins: ['22' * 32, '33' * 32],
      );

      expect(call[0], AdminSetChangeCallCodec.palletIndex);
      expect(call[1], AdminSetChangeCallCodec.proposeAdminSetChangeCallIndex);
      expect(call[2], 0);
      expect(call.sublist(3, 51), List<int>.filled(48, 0x11));
      expect(call[51], 0x08);
      expect(call.length, 2 + 1 + 48 + 1 + 64);
    });

    test('validates proposer and changed admin set', () {
      final subject = AdminSubjectState(
        subjectIdHex: '11' * 48,
        org: 3,
        kind: 2,
        admins: ['aa' * 32, 'bb' * 32],
        threshold: 2,
        creatorHex: 'aa' * 32,
        createdAt: 1,
        updatedAt: 1,
        status: 1,
      );

      final normalized = AdminSetValidation.validate(
        subject: subject,
        proposerPubkeyHex: '0x${'aa' * 32}',
        newAdmins: ['0x${'aa' * 32}', '0x${'cc' * 32}'],
      );
      expect(normalized, ['aa' * 32, 'cc' * 32]);
      expect(
        () => AdminSetValidation.validate(
          subject: subject,
          proposerPubkeyHex: '0x${'aa' * 32}',
          newAdmins: ['aa' * 32, 'bb' * 32],
        ),
        throwsStateError,
      );
    });
  });
}
