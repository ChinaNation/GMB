import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/governance/admins-change/codec/admin_set_change_call_codec.dart';
import 'package:citizenapp/governance/admins-change/codec/admin_account_codec.dart';
import 'package:citizenapp/governance/admins-change/codec/account_id_codec.dart';
import 'package:citizenapp/governance/admins-change/admin_set_change_qr_adapter.dart';
import 'package:citizenapp/governance/admins-change/models/admin_account.dart';
import 'package:citizenapp/governance/admins-change/services/admin_set_validation.dart';

void main() {
  List<int> u32Le(int value) => [
        value & 0xff,
        (value >> 8) & 0xff,
        (value >> 16) & 0xff,
        (value >> 24) & 0xff,
      ];

  group('admins_change codec', () {
    test('builds AdminsChange::AdminAccounts storage key', () {
      final accountId = AdminAccountIdCodec.fromAccountHex('11' * 32);
      final key = AdminAccountIdCodec.adminAccountStorageKey(accountId);

      expect(accountId.length, 32);
      expect(key.length, 16 + 16 + 16 + 32);
    });

    test('decodes full AdminAccount value', () {
      final accountId = AdminAccountIdCodec.fromAccountHex('11' * 32);
      final data = Uint8List.fromList([
        0,
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
        org: 0,
        accountId: accountId,
        admins: ['22' * 32, '33' * 32],
        newThreshold: 13,
      );

      expect(call[0], AdminSetChangeCallCodec.palletIndex);
      expect(call[1], AdminSetChangeCallCodec.proposeAdminSetChangeCallIndex);
      expect(call[2], 0);
      expect(call.sublist(3, 35), List<int>.filled(32, 0x11));
      expect(call[35], 0x08);
      expect(call.sublist(call.length - 4), u32Le(13));
      expect(call.length, 2 + 1 + 32 + 1 + 64 + 4);
    });

    test('validates proposer and changed admin set', () {
      final account = AdminAccountState(
        accountHex: '11' * 32,
        org: 3,
        kind: 1,
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

    test('rejects invalid account kind and org combinations', () {
      AdminAccountState account({required int org, required int kind}) {
        return AdminAccountState(
          accountHex: '11' * 32,
          org: org,
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
          account: account(org: 4, kind: 1),
          proposerPubkeyHex: 'aa' * 32,
          admins: ['aa' * 32, 'cc' * 32],
          newThreshold: 2,
        ),
        throwsStateError,
      );
      expect(
        () => AdminSetValidation.validate(
          account: account(org: 3, kind: 2),
          proposerPubkeyHex: 'aa' * 32,
          admins: ['aa' * 32, 'cc' * 32],
          newThreshold: 2,
        ),
        throwsStateError,
      );
      expect(
        () => AdminSetValidation.validate(
          account: account(org: 3, kind: 3),
          proposerPubkeyHex: 'aa' * 32,
          admins: ['aa' * 32, 'cc' * 32],
          newThreshold: 2,
        ),
        throwsStateError,
      );
      expect(
        AdminSetValidation.validate(
          account: account(org: 5, kind: 2),
          proposerPubkeyHex: 'aa' * 32,
          admins: ['aa' * 32, 'cc' * 32],
          newThreshold: 2,
        ).admins,
        ['aa' * 32, 'cc' * 32],
      );
    });

    test('builds QR display fields matching cold wallet decoder keys', () {
      final account = AdminAccountState(
        accountHex: '11' * 32,
        org: 5,
        kind: 2,
        admins: ['aa' * 32, 'bb' * 32],
        threshold: 2,
        creatorHex: 'aa' * 32,
        createdAt: 1,
        updatedAt: 1,
        status: 1,
      );

      final display = AdminSetChangeQrAdapter.buildDisplay(
        account: account,
        admins: ['0x${'aa' * 32}', 'cc' * 32],
        newThreshold: 2,
      );
      final fields = {
        for (final field in display.fields) field.key: field.value
      };

      expect(fields['org'], '其他机构账户');
      expect(fields['account'], '0x${'11' * 32}');
      expect(fields['admins'], '0x${'aa' * 32},0x${'cc' * 32}');
      expect(fields['new_threshold'], '2/2');
      expect(fields.containsKey('account_id'), isFalse);
      expect(fields.containsKey('admins_len'), isFalse);
      expect(fields.containsKey('threshold'), isFalse);
    });
  });
}
