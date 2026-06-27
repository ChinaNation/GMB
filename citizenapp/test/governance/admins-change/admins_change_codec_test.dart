import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/citizen/proposal/admins-change/codec/admin_set_change_call_codec.dart';
import 'package:citizenapp/citizen/proposal/admins-change/codec/admin_account_codec.dart';
import 'package:citizenapp/citizen/proposal/admins-change/codec/account_id_codec.dart';
import 'package:citizenapp/citizen/proposal/admins-change/models/admin_account.dart';
import 'package:citizenapp/citizen/proposal/admins-change/services/admin_set_validation.dart';

void main() {
  List<int> codeBytes(String code) {
    final out = List<int>.filled(4, 0);
    final raw = code.codeUnits;
    for (var i = 0; i < out.length && i < raw.length; i++) {
      out[i] = raw[i];
    }
    return out;
  }

  List<int> u32Le(int value) => [
        value & 0xff,
        (value >> 8) & 0xff,
        (value >> 16) & 0xff,
        (value >> 24) & 0xff,
      ];

  group('admins codec', () {
    test('builds classified AdminAccounts storage key', () {
      final accountId = AdminAccountIdCodec.fromAccountHex('11' * 32);
      final key = AdminAccountIdCodec.adminAccountStorageKey(
        accountId,
        institutionCode: 'NRC',
      );

      expect(accountId.length, 32);
      expect(key.length, 16 + 16 + 16 + 32);
    });

    test('decodes full AdminAccount value', () {
      final accountId = AdminAccountIdCodec.fromAccountHex('11' * 32);
      final data = Uint8List.fromList([
        ...codeBytes('NRC'),
        0,
        0x08,
        ...List<int>.filled(32, 0xaa),
        ...List<int>.filled(32, 0xbb),
        ...List<int>.filled(32, 0xcc),
        ...u32Le(7),
        ...u32Le(9),
        1,
      ]);

      final decoded = AdminAccountCodec.decode(accountId, data)!;
      expect(decoded.admins, ['aa' * 32, 'bb' * 32]);
      expect(decoded.threshold, 0);
      expect(decoded.creatorHex, 'cc' * 32);
      expect(decoded.statusLabel, '已激活');
    });

    test('builds propose_admin_set_change call data', () {
      final accountId = Uint8List.fromList(List<int>.filled(32, 0x11));
      final call = AdminSetChangeCallCodec.build(
        institutionCode: 'NRC',
        accountId: accountId,
        admins: ['22' * 32, '33' * 32],
        newThreshold: 13,
      );

      expect(call[0], AdminSetChangeCallCodec.palletIndexForCode('NRC'));
      expect(call[1], AdminSetChangeCallCodec.callIndexForCode('NRC'));
      expect(call.sublist(2, 6), codeBytes('NRC'));
      expect(call.sublist(6, 38), List<int>.filled(32, 0x11));
      expect(call[38], 0x08);
      expect(call.sublist(call.length - 4), u32Le(13));
      expect(call.length, 2 + 4 + 32 + 1 + 64 + 4);

      final personalCall = AdminSetChangeCallCodec.build(
        institutionCode: 'PMUL',
        accountId: accountId,
        admins: ['22' * 32, '33' * 32],
        newThreshold: 2,
      );
      expect(
          personalCall[0], AdminSetChangeCallCodec.palletIndexForCode('PMUL'));
      expect(personalCall[1], AdminSetChangeCallCodec.callIndexForCode('PMUL'));
      expect(personalCall.sublist(2, 6), codeBytes('PMUL'));
    });

    test('validates proposer and changed admin set', () {
      final account = AdminAccountState(
        accountHex: '11' * 32,
        institutionCode: 'PMUL',
        kind: 3,
        admins: ['aa' * 32, 'bb' * 32],
        threshold: 2,
        creatorHex: 'aa' * 32,
        createdAt: 1,
        updatedAt: 1,
        status: 1,
      );

      final normalized = AdminSetValidation.validate(
        account: account,
        proposerPubkeyHex: '0x${'aa' * 32}',
        admins: ['0x${'aa' * 32}', '0x${'cc' * 32}'],
        newThreshold: 2,
      );
      expect(normalized.admins, ['aa' * 32, 'cc' * 32]);
      expect(normalized.threshold, 2);
      expect(
        () => AdminSetValidation.validate(
          account: account,
          proposerPubkeyHex: '0x${'aa' * 32}',
          admins: ['aa' * 32, 'bb' * 32],
          newThreshold: 2,
        ),
        throwsStateError,
      );
    });

    test('rejects invalid account kind and institution_code combinations', () {
      AdminAccountState account({
        required String institutionCode,
        required int kind,
      }) {
        return AdminAccountState(
          accountHex: '11' * 32,
          institutionCode: institutionCode,
          kind: kind,
          admins: ['aa' * 32, 'bb' * 32],
          threshold: 2,
          creatorHex: 'aa' * 32,
          createdAt: 1,
          updatedAt: 1,
          status: 1,
        );
      }

      expect(
        () => AdminSetValidation.validate(
          account: account(institutionCode: 'CGOV', kind: 3),
          proposerPubkeyHex: 'aa' * 32,
          admins: ['aa' * 32, 'cc' * 32],
          newThreshold: 2,
        ),
        throwsStateError,
      );
      expect(
        () => AdminSetValidation.validate(
          account: account(institutionCode: 'PMUL', kind: 2),
          proposerPubkeyHex: 'aa' * 32,
          admins: ['aa' * 32, 'cc' * 32],
          newThreshold: 2,
        ),
        throwsStateError,
      );
      expect(
        () => AdminSetValidation.validate(
          account: account(institutionCode: 'UNIN', kind: 1),
          proposerPubkeyHex: 'aa' * 32,
          admins: ['aa' * 32, 'cc' * 32],
          newThreshold: 2,
        ),
        throwsStateError,
      );
      expect(
        AdminSetValidation.validate(
          account: account(institutionCode: 'CGOV', kind: 1),
          proposerPubkeyHex: 'aa' * 32,
          admins: ['aa' * 32, 'cc' * 32],
          newThreshold: 2,
        ).admins,
        ['aa' * 32, 'cc' * 32],
      );
      expect(
        AdminSetValidation.validate(
          account: account(institutionCode: 'UNIN', kind: 2),
          proposerPubkeyHex: 'aa' * 32,
          admins: ['aa' * 32, 'cc' * 32],
          newThreshold: 2,
        ).admins,
        ['aa' * 32, 'cc' * 32],
      );
      expect(
        AdminSetValidation.validate(
          account: account(institutionCode: 'PMUL', kind: 3),
          proposerPubkeyHex: 'aa' * 32,
          admins: ['aa' * 32, 'cc' * 32],
          newThreshold: 2,
        ).admins,
        ['aa' * 32, 'cc' * 32],
      );
    });
  });
}
