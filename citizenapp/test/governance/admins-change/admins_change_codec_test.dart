import 'dart:convert';
import 'dart:typed_data';

import 'package:citizenapp/citizen/proposal/admins-change/codec/account_id_codec.dart';
import 'package:citizenapp/citizen/proposal/admins-change/codec/admin_account_codec.dart';
import 'package:citizenapp/citizen/proposal/admins-change/models/admin_account.dart';
import 'package:citizenapp/citizen/proposal/admins-change/services/admin_set_validation.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  List<int> bytes(String text) =>
      [(utf8.encode(text).length << 2), ...utf8.encode(text)];
  List<int> code(String value) =>
      [...value.codeUnits, ...List.filled(4 - value.length, 0)];
  List<int> u32(int value) => [value, 0, 0, 0];

  test('机构 AdminAccounts 只解码钱包集合', () {
    final accountId = Uint8List.fromList(List.filled(32, 9));
    final value = Uint8List.fromList([
      ...bytes('CID-1'),
      ...code('CGOV'),
      8,
      ...List.filled(32, 1),
      ...List.filled(32, 2),
      1,
    ]);
    final decoded = AdminAccountCodec.decode(
      accountId,
      value,
      personalMultisig: false,
    )!;
    expect(decoded.admins, ['01' * 32, '02' * 32]);
    expect(decoded.creatorHex, isEmpty);
    expect(decoded.isActive, isTrue);
  });

  test('个人多签继续解码独立旧账户布局', () {
    final accountId = Uint8List.fromList(List.filled(32, 9));
    final value = Uint8List.fromList([
      ...bytes(''),
      ...code('PMUL'),
      2,
      8,
      ...List.filled(32, 1),
      ...List.filled(32, 2),
      ...List.filled(32, 3),
      ...u32(7),
      ...u32(9),
      1,
    ]);
    final decoded = AdminAccountCodec.decode(
      accountId,
      value,
      personalMultisig: true,
    )!;
    expect(decoded.admins, ['01' * 32, '02' * 32]);
    expect(decoded.creatorHex, '03' * 32);
  });

  test('个人多签管理员集合校验仍按钱包账户运行', () {
    final account = AdminAccountState(
      accountHex: '11' * 32,
      institutionCode: 'PMUL',
      kind: 2,
      admins: ['aa' * 32, 'bb' * 32],
      threshold: 2,
      creatorHex: 'aa' * 32,
      createdAt: 1,
      updatedAt: 1,
      status: 1,
    );
    final normalized = AdminSetValidation.validate(
      account: account,
      proposerPubkeyHex: 'aa' * 32,
      admins: ['aa' * 32, 'cc' * 32],
      newThreshold: 2,
    );
    expect(normalized.admins, ['aa' * 32, 'cc' * 32]);
  });

  test('分类管理员 storage key 长度固定', () {
    final key = AdminAccountIdCodec.adminAccountStorageKey(
      Uint8List.fromList(List.filled(32, 1)),
      institutionCode: 'CGOV',
    );
    expect(key.length, 80);
  });
}
