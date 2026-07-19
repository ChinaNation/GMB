// AdminAccountStorageCodec golden test:固定字节 -> 固定解码结果。
//
// 覆盖链上三类管理员 pallet `AdminAccounts` 的目标布局：
// - 机构 value 只保存 institution_code + admins，CID 来自 storage key；
// - 个人多签 value 保持独立账户布局，personal_account 来自 storage key。

import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/citizen/shared/admin_account_storage_codec.dart';

void main() {
  List<int> codeBytes(String code) {
    final out = List<int>.filled(4, 0);
    final raw = code.codeUnits;
    for (var i = 0; i < out.length && i < raw.length; i++) {
      out[i] = raw[i];
    }
    return out;
  }

  List<int> scaleBytes(String cid) {
    final raw = utf8.encode(cid);
    return [raw.length << 2, ...raw];
  }

  List<int> adminBytes(
    String familyName,
    String givenName,
    List<int> account,
  ) =>
      [
        ...account,
        ...scaleBytes(familyName),
        ...scaleBytes(givenName),
      ];

  group('tryDecode', () {
    test('成功解码 PublicInstitution(0 admins)', () {
      final bytes = Uint8List.fromList([
        ...codeBytes('NRC'),
        0,
      ]);
      final r = AdminAccountStorageCodec.tryDecode(
        bytes,
        kind: AdminAccountStorageCodec.kindPublicInstitution,
      )!;
      expect(r.institutionCode, 'NRC');
      expect(r.kind, AdminAccountStorageCodec.kindPublicInstitution);
      expect(r.admins, isEmpty);
    });

    test('成功解码 Personal 含 3 个 admin(空前导 cid=0x00)', () {
      final a1 = List.filled(32, 0x11);
      final a2 = List.filled(32, 0x22);
      final a3 = List.filled(32, 0x33);
      final bytes = Uint8List.fromList([
        0, // 个人多签结构中的 cid_number 为空
        ...codeBytes('PMUL'),
        AdminAccountStorageCodec.kindPersonal,
        0x0C, // Compact(3): (3<<2) | 0 = 12
        ...adminBytes('张', '三', a1),
        ...adminBytes('李', '四', a2),
        ...adminBytes('管理', '员', a3),
        ...List.filled(32, 0x44),
        ...List.filled(8, 0),
        1,
      ]);
      final r = AdminAccountStorageCodec.tryDecode(
        bytes,
        kind: AdminAccountStorageCodec.kindPersonal,
      )!;
      expect(r.institutionCode, 'PMUL');
      expect(r.kind, AdminAccountStorageCodec.kindPersonal);
      expect(
        r.admins.map((admin) => admin.admin_account),
        ['11' * 32, '22' * 32, '33' * 32],
      );
    });

    test('成功解码 PublicInstitution 含 2 个 admin', () {
      final a1 = List.filled(32, 0x44);
      final a2 = List.filled(32, 0x55);
      final bytes = Uint8List.fromList([
        ...codeBytes('CGOV'),
        0x08, // Compact(2)
        ...adminBytes('张', '三', a1),
        ...adminBytes('李', '四', a2),
      ]);
      final r = AdminAccountStorageCodec.tryDecode(
        bytes,
        kind: AdminAccountStorageCodec.kindPublicInstitution,
      )!;
      expect(r.institutionCode, 'CGOV');
      expect(r.kind, AdminAccountStorageCodec.kindPublicInstitution);
      expect(
        r.admins.map((admin) => admin.admin_account),
        ['44' * 32, '55' * 32],
      );
    });

    test('字节不足返回 null,不抛异常', () {
      expect(
        AdminAccountStorageCodec.tryDecode(
          Uint8List(0),
          kind: AdminAccountStorageCodec.kindPublicInstitution,
        ),
        isNull,
      );
      expect(
        AdminAccountStorageCodec.tryDecode(
          Uint8List.fromList([0]),
          kind: AdminAccountStorageCodec.kindPersonal,
        ),
        isNull,
      );
    });

    test('admins 数量超过实际字节返回 null', () {
      final bytes = Uint8List.fromList([
        ...codeBytes('NRC'),
        0x08, // 声明 2 个 admin 但只给 1 个完整管理员的字节。
        ...adminBytes('管理', '员', List.filled(32, 0xCC)),
      ]);
      expect(
        AdminAccountStorageCodec.tryDecode(
          bytes,
          kind: AdminAccountStorageCodec.kindPublicInstitution,
        ),
        isNull,
      );
    });

    test('Compact 64 admins (mode=1 两字节长度)', () {
      const adminsLen = 64;
      final admins = List.generate(adminsLen, (_) => List.filled(32, 0xDD));
      final bytes = <int>[
        0,
        ...codeBytes('PMUL'),
        AdminAccountStorageCodec.kindPersonal,
        0x01,
        0x01,
      ];
      for (var i = 0; i < admins.length; i++) {
        final account = List<int>.from(admins[i]);
        account[0] = i;
        bytes.addAll(adminBytes('管理', '员', account));
      }
      bytes.addAll(List.filled(32, 0xEE));
      bytes.addAll(List.filled(8, 0));
      bytes.add(1);
      final r = AdminAccountStorageCodec.tryDecode(
        Uint8List.fromList(bytes),
        kind: AdminAccountStorageCodec.kindPersonal,
      )!;
      expect(r.admins.length, adminsLen);
    });
  });

  group('storage key 主键提取', () {
    test('机构 storage key 提取 CID', () {
      const cid = 'GD001-CGOV0-123456789-2026';
      final key = Uint8List.fromList([
        ...List.filled(32, 0),
        ...List.filled(16, 1),
        ...scaleBytes(cid),
      ]);
      expect(AdminAccountStorageCodec.extractCidNumberFromKey(key), cid);
    });

    test('个人多签 storage key 末 32 字节 = personal_account', () {
      final key = Uint8List(32 + 16 + 32);
      for (var i = 48; i < key.length; i++) {
        key[i] = i - 48;
      }
      final accountId =
          AdminAccountStorageCodec.extractPersonalAccountFromKey(key)!;
      expect(
        AdminAccountStorageCodec.accountHexFromAccountId(accountId),
        '000102030405060708090a0b0c0d0e0f'
        '101112131415161718191a1b1c1d1e1f',
      );
    });

    test('storage key 长度不足返回 null', () {
      expect(
        AdminAccountStorageCodec.extractCidNumberFromKey(Uint8List(20)),
        isNull,
      );
      expect(
        AdminAccountStorageCodec.extractPersonalAccountFromKey(Uint8List(20)),
        isNull,
      );
      expect(
        AdminAccountStorageCodec.accountHexFromAccountId(Uint8List(31)),
        isNull,
      );
    });
  });
}
