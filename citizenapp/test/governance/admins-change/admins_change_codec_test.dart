import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/citizen/proposal/admins-change/codec/admin_set_change_call_codec.dart';
import 'package:citizenapp/citizen/proposal/admins-change/codec/admin_account_codec.dart';
import 'package:citizenapp/citizen/proposal/admins-change/codec/account_id_codec.dart';
import 'package:citizenapp/citizen/proposal/admins-change/models/admin_account.dart';
import 'package:citizenapp/citizen/proposal/admins-change/services/admin_set_validation.dart';
import 'package:citizenapp/citizen/shared/admin_profile.dart';

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

    // A2 金标:AdminAccounts.admins 为 Vec<AdminProfile>(account + cid + name + admin_role
    // + term_start + term_end + source),逐字节与链端 admin-primitives::AdminProfile 对齐。
    List<int> boundedUtf8(String s) {
      final b = utf8.encode(s);
      return [b.length << 2, ...b]; // len<64 → 单字节 Compact(mode 0)
    }

    test('decodes full AdminAccount value with AdminProfile (A2)', () {
      final accountId = AdminAccountIdCodec.fromAccountHex('11' * 32);
      final data = Uint8List.fromList([
        ...codeBytes('PRC'), // institution_code
        0, // kind = PublicInstitution(非个人多签 → Vec<AdminProfile>)
        0x08, // Compact count = 2
        // admin 0:实名资料齐全
        ...List<int>.filled(32, 0xaa),
        ...boundedUtf8('CID-A'),
        ...boundedUtf8('张三'),
        ...boundedUtf8('主任'),
        ...u32Le(100), ...u32Le(200), 1, // term + source=registry(1)
        // admin 1:空 meta(如创世)
        ...List<int>.filled(32, 0xbb),
        ...boundedUtf8(''),
        ...boundedUtf8(''),
        ...boundedUtf8(''),
        ...u32Le(0), ...u32Le(0), 0, // source=genesis(0)
        // trailer:creator + created_at + updated_at + status
        ...List<int>.filled(32, 0xcc),
        ...u32Le(7), ...u32Le(9), 1,
      ]);

      final decoded = AdminAccountCodec.decode(accountId, data)!;
      expect(decoded.admins, ['aa' * 32, 'bb' * 32]); // getter 抽 account
      expect(decoded.profiles.length, 2);
      expect(decoded.profiles[0].cidNumber, 'CID-A');
      expect(decoded.profiles[0].name, '张三');
      expect(decoded.profiles[0].adminRole, '主任');
      expect(decoded.profiles[0].termStartDay, 100);
      expect(decoded.profiles[0].termEndDay, 200);
      expect(decoded.profiles[0].source, AdminProfileSource.registry);
      expect(decoded.profiles[1].source, AdminProfileSource.genesis);
      expect(decoded.threshold, 0);
      expect(decoded.creatorHex, 'cc' * 32);
      expect(decoded.statusLabel, '已激活');
    });

    test('decodes personal-multisig AdminAccount as bare accounts (kind=2)',
        () {
      final accountId = AdminAccountIdCodec.fromAccountHex('11' * 32);
      final data = Uint8List.fromList([
        ...codeBytes('PMUL'),
        2, // kind = PersonalMultisig → 裸 Vec<AccountId>
        0x08, // count = 2
        ...List<int>.filled(32, 0xaa),
        ...List<int>.filled(32, 0xbb),
        ...List<int>.filled(32, 0xcc), // creator
        ...u32Le(7), ...u32Le(9), 1,
      ]);
      final decoded = AdminAccountCodec.decode(accountId, data)!;
      expect(decoded.admins, ['aa' * 32, 'bb' * 32]);
      expect(decoded.profiles.every((p) => p.cidNumber.isEmpty), isTrue);
      expect(decoded.creatorHex, 'cc' * 32);
    });

    test('builds propose_admin_set_change call data', () {
      final accountId = Uint8List.fromList(List<int>.filled(32, 0x11));
      final call = AdminSetChangeCallCodec.build(
        institutionCode: 'NRC',
        adminKind: 0,
        accountId: accountId,
        admins: ['22' * 32, '33' * 32],
        newThreshold: 13,
      );

      expect(call[0], AdminSetChangeCallCodec.palletIndexForKind(0));
      expect(call[1], AdminSetChangeCallCodec.callIndexForKind(0));
      expect(call.sublist(2, 6), codeBytes('NRC'));
      expect(call.sublist(6, 38), List<int>.filled(32, 0x11));
      expect(call[38], 0x08);
      expect(call.sublist(call.length - 4), u32Le(13));
      expect(call.length, 2 + 4 + 32 + 1 + 64 + 4);

      final personalCall = AdminSetChangeCallCodec.build(
        institutionCode: 'PMUL',
        adminKind: 2,
        accountId: accountId,
        admins: ['22' * 32, '33' * 32],
        newThreshold: 2,
      );
      expect(personalCall[0], AdminSetChangeCallCodec.palletIndexForKind(2));
      expect(personalCall[1], AdminSetChangeCallCodec.callIndexForKind(2));
      expect(personalCall.sublist(2, 6), codeBytes('PMUL'));
    });

    test('rejects FRG generic admin set change call data', () {
      final accountId = Uint8List.fromList(List<int>.filled(32, 0x11));

      expect(
        () => AdminSetChangeCallCodec.build(
          institutionCode: 'FRG',
          adminKind: 0,
          accountId: accountId,
          admins: ['22' * 32, '33' * 32],
          newThreshold: 2,
        ),
        throwsArgumentError,
      );
    });

    test('validates proposer and changed admin set', () {
      final account = AdminAccountState(
        accountHex: '11' * 32,
        institutionCode: 'PMUL',
        kind: 2,
        profiles: [
          AdminProfile(account: 'aa' * 32),
          AdminProfile(account: 'bb' * 32)
        ],
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
          profiles: [
            AdminProfile(account: 'aa' * 32),
            AdminProfile(account: 'bb' * 32)
          ],
          threshold: 2,
          creatorHex: 'aa' * 32,
          createdAt: 1,
          updatedAt: 1,
          status: 1,
        );
      }

      expect(
        () => AdminSetValidation.validate(
          account: account(institutionCode: 'CGOV', kind: 2),
          proposerPubkeyHex: 'aa' * 32,
          admins: ['aa' * 32, 'cc' * 32],
          newThreshold: 2,
        ),
        throwsStateError,
      );
      expect(
        () => AdminSetValidation.validate(
          account: account(institutionCode: 'PMUL', kind: 1),
          proposerPubkeyHex: 'aa' * 32,
          admins: ['aa' * 32, 'cc' * 32],
          newThreshold: 2,
        ),
        throwsStateError,
      );
      expect(
        AdminSetValidation.validate(
          account: account(institutionCode: 'UNIN', kind: 1),
          proposerPubkeyHex: 'aa' * 32,
          admins: ['aa' * 32, 'cc' * 32],
          newThreshold: 2,
        ).admins,
        ['aa' * 32, 'cc' * 32],
      );
      expect(
        AdminSetValidation.validate(
          account: account(institutionCode: 'CGOV', kind: 0),
          proposerPubkeyHex: 'aa' * 32,
          admins: ['aa' * 32, 'cc' * 32],
          newThreshold: 2,
        ).admins,
        ['aa' * 32, 'cc' * 32],
      );
      expect(
        AdminSetValidation.validate(
          account: account(institutionCode: 'UNIN', kind: 1),
          proposerPubkeyHex: 'aa' * 32,
          admins: ['aa' * 32, 'cc' * 32],
          newThreshold: 2,
        ).admins,
        ['aa' * 32, 'cc' * 32],
      );
      expect(
        AdminSetValidation.validate(
          account: account(institutionCode: 'PMUL', kind: 2),
          proposerPubkeyHex: 'aa' * 32,
          admins: ['aa' * 32, 'cc' * 32],
          newThreshold: 2,
        ).admins,
        ['aa' * 32, 'cc' * 32],
      );
    });

    test('validates fixed governance counts and thresholds', () {
      String key(int byte) => byte.toRadixString(16).padLeft(2, '0') * 32;

      AdminAccountState fixedAccount({
        required String institutionCode,
        required int count,
      }) {
        return AdminAccountState(
          accountHex: '11' * 32,
          institutionCode: institutionCode,
          kind: 0,
          profiles: List<AdminProfile>.generate(
            count,
            (i) => AdminProfile(account: i == 0 ? 'aa' * 32 : key(0x20 + i)),
          ),
          threshold: 1,
          creatorHex: 'aa' * 32,
          createdAt: 1,
          updatedAt: 1,
          status: 1,
        );
      }

      List<String> newAdmins(int count) => [
            'aa' * 32,
            for (var i = 1; i < count; i++) key(0x60 + i),
          ];

      expect(
        AdminSetValidation.validate(
          account: fixedAccount(institutionCode: 'FRG', count: 5),
          proposerPubkeyHex: 'aa' * 32,
          admins: newAdmins(5),
          newThreshold: 3,
        ).threshold,
        3,
      );
      expect(
        AdminSetValidation.validate(
          account: fixedAccount(institutionCode: 'NJD', count: 13),
          proposerPubkeyHex: 'aa' * 32,
          admins: newAdmins(13),
          newThreshold: 8,
        ).threshold,
        8,
      );
      expect(
        () => AdminSetValidation.validate(
          account: fixedAccount(institutionCode: 'FRG', count: 5),
          proposerPubkeyHex: 'aa' * 32,
          admins: newAdmins(4),
          newThreshold: 3,
        ),
        throwsStateError,
      );
      expect(
        () => AdminSetValidation.validate(
          account: fixedAccount(institutionCode: 'NJD', count: 13),
          proposerPubkeyHex: 'aa' * 32,
          admins: newAdmins(13),
          newThreshold: 7,
        ),
        throwsStateError,
      );
    });
  });
}
