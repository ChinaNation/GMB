import 'dart:convert';
import 'dart:typed_data';

import 'package:citizenapp/citizen/proposal/admins-change/codec/account_id_codec.dart';
import 'package:citizenapp/citizen/proposal/admins-change/codec/admin_account_codec.dart';
import 'package:citizenapp/citizen/proposal/admins-change/codec/admin_set_change_call_codec.dart';
import 'package:citizenapp/citizen/proposal/admins-change/models/admin_account.dart';
import 'package:citizenapp/citizen/proposal/admins-change/services/admin_set_validation.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  List<int> bytes(String text) =>
      [(utf8.encode(text).length << 2), ...utf8.encode(text)];
  List<int> code(String value) =>
      [...value.codeUnits, ...List.filled(4 - value.length, 0)];
  List<int> u32(int value) => [value, 0, 0, 0];
  List<int> admin(String familyName, String givenName, int accountByte) => [
        ...List.filled(32, accountByte),
        ...bytes(familyName),
        ...bytes(givenName),
      ];
  List<int> publicAdmin(
    String citizenCid,
    String familyName,
    String givenName,
    int accountByte,
  ) =>
      [
        ...List.filled(32, accountByte),
        ...bytes(citizenCid),
        ...bytes(familyName),
        ...bytes(givenName),
      ];

  AdminPerson person(String account, String familyName, String givenName) =>
      AdminPerson(
        admin_account: account,
        family_name: familyName,
        given_name: givenName,
      );

  test('公权机构 AdminAccounts 严格解码管理员四字段', () {
    final value = Uint8List.fromList([
      ...code('CGOV'),
      8,
      ...publicAdmin('GZ000-CTZN6-198805200-2026', '张', '三', 1),
      ...publicAdmin('', '', '', 2),
    ]);
    final decoded = AdminAccountCodec.decodeInstitution(
      cidNumber: 'CID-1',
      data: value,
      institutionKind: 0,
    )!;
    expect(
      decoded.admins.map((admin) => admin.admin_account),
      ['01' * 32, '02' * 32],
    );
    expect(decoded.cidNumber, 'CID-1');
    expect(decoded.isActive, isTrue);
    expect(decoded.admins.first.cid_number, 'GZ000-CTZN6-198805200-2026');
    expect(decoded.admins.last.cid_number, isEmpty);
    expect(decoded.admins.last.family_name, isEmpty);
  });

  test('个人多签继续解码独立账户布局', () {
    final accountId = Uint8List.fromList(List.filled(32, 9));
    final value = Uint8List.fromList([
      ...bytes(''),
      ...code('PMUL'),
      2,
      8,
      ...admin('张', '三', 1),
      ...admin('李', '四', 2),
      ...List.filled(32, 3),
      ...u32(7),
      ...u32(9),
      1,
    ]);
    final decoded = AdminAccountCodec.decodePersonal(accountId, value)!;
    expect(
      decoded.admins.map((admin) => admin.admin_account),
      ['01' * 32, '02' * 32],
    );
    expect(decoded.personalCreatorHex, '03' * 32);
  });

  test('个人多签管理员集合校验仍按钱包账户运行', () {
    final account = AdminAccountState(
      personalAccountHex: '11' * 32,
      institutionCode: 'PMUL',
      kind: 2,
      admins: [
        person('aa' * 32, '张', '三'),
        person('bb' * 32, '李', '四'),
      ],
      threshold: 2,
      personalCreatorHex: 'aa' * 32,
      personalCreatedAt: 1,
      personalUpdatedAt: 1,
      personalStatus: 1,
    );
    final normalized = AdminSetValidation.validate(
      account: account,
      proposerPubkeyHex: 'aa' * 32,
      admins: [
        person('aa' * 32, '张', '三'),
        person('cc' * 32, '管理', '员'),
      ],
      newThreshold: 2,
    );
    expect(
      normalized.admins.map((admin) => admin.admin_account),
      ['aa' * 32, 'cc' * 32],
    );
  });

  test('个人多签管理员更换载荷逐字段编码账户、姓、名', () {
    final accountId = Uint8List.fromList(List.filled(32, 9));
    final callData = PersonalAdminsChangeCallCodec.build(
      institutionCode: 'PMUL',
      adminKind: 2,
      accountId: accountId,
      admins: [
        person('01' * 32, '张', '三'),
        person('02' * 32, '管理', '员'),
      ],
      newThreshold: 2,
    );

    expect(
      callData,
      Uint8List.fromList([
        29,
        0,
        ...code('PMUL'),
        ...accountId,
        8,
        ...admin('张', '三', 1),
        ...admin('管理', '员', 2),
        ...u32(2),
      ]),
    );
  });

  test('机构管理员 storage key 以 CID 为唯一 key', () {
    const cidNumber = 'GD001-CGOV0-123456789-2026';
    final key = AdminAccountIdCodec.institutionAdminStorageKey(
      cidNumber,
      institutionCode: 'CGOV',
    );
    expect(key.length, 32 + 16 + 1 + utf8.encode(cidNumber).length);
    expect(key.sublist(49), utf8.encode(cidNumber));
  });
}
