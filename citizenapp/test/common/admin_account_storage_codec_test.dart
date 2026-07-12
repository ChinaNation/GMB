// AdminAccountStorageCodec golden test:固定字节 -> 固定解码结果。
//
// 覆盖链上三类管理员 pallet `AdminAccounts` 的最小解码路径：
// - PublicInstitution (kind=0)
// - Personal (kind=2)
// - storage key 末 32B AccountId 提取

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

  // 前导 cid_number: Compact<u32> 长度(<64 → 单字节 mode0)+ UTF8 字节;空 = [0x00]。
  List<int> cidPrefix(String cid) {
    final raw = cid.codeUnits;
    return [raw.length << 2, ...raw];
  }

  group('tryDecode', () {
    test('成功解码 PublicInstitution(0 admins,非空前导 cid)', () {
      // cid_number=NRC01, institution_code=NRC, kind=0, admins=Compact(0)=0x00。
      final bytes = Uint8List.fromList([
        ...cidPrefix('NRC01'),
        ...codeBytes('NRC'),
        AdminAccountStorageCodec.kindPublicInstitution,
        0,
      ]);
      final r = AdminAccountStorageCodec.tryDecode(bytes)!;
      expect(r.institutionCode, 'NRC');
      expect(r.kind, AdminAccountStorageCodec.kindPublicInstitution);
      expect(r.adminsHex, isEmpty);
    });

    test('成功解码 Personal 含 3 个 admin(空前导 cid=0x00)', () {
      final a1 = List.filled(32, 0x11);
      final a2 = List.filled(32, 0x22);
      final a3 = List.filled(32, 0x33);
      final bytes = Uint8List.fromList([
        ...cidPrefix(''), // 个人多签前导 cid 为空
        ...codeBytes('PMUL'),
        AdminAccountStorageCodec.kindPersonal,
        0x0C, // Compact(3): (3<<2) | 0 = 12
        ...a1,
        ...a2,
        ...a3,
      ]);
      final r = AdminAccountStorageCodec.tryDecode(bytes)!;
      expect(r.institutionCode, 'PMUL');
      expect(r.kind, AdminAccountStorageCodec.kindPersonal);
      expect(r.adminsHex, ['11' * 32, '22' * 32, '33' * 32]);
    });

    test('成功解码 PublicInstitution 含 2 个 admin', () {
      final a1 = List.filled(32, 0x44);
      final a2 = List.filled(32, 0x55);
      final bytes = Uint8List.fromList([
        ...cidPrefix('CGOV9'),
        ...codeBytes('CGOV'),
        AdminAccountStorageCodec.kindPublicInstitution,
        0x08, // Compact(2)
        ...a1,
        ...a2,
      ]);
      final r = AdminAccountStorageCodec.tryDecode(bytes)!;
      expect(r.institutionCode, 'CGOV');
      expect(r.kind, AdminAccountStorageCodec.kindPublicInstitution);
      expect(r.adminsHex, ['44' * 32, '55' * 32]);
    });

    test('字节不足返回 null,不抛异常', () {
      expect(AdminAccountStorageCodec.tryDecode(Uint8List(0)), isNull);
      // 仅前导 cid 空字节、无 institution_code/kind → null。
      expect(
          AdminAccountStorageCodec.tryDecode(Uint8List.fromList([0])), isNull);
    });

    test('admins 数量超过实际字节返回 null', () {
      final bytes = Uint8List.fromList([
        ...cidPrefix(''),
        ...codeBytes('NRC'),
        AdminAccountStorageCodec.kindPublicInstitution,
        0x08, // 声明 2 个 admin 但只给 1 个的字节。
        ...List.filled(32, 0xCC),
      ]);
      expect(AdminAccountStorageCodec.tryDecode(bytes), isNull);
    });

    test('Compact 64 admins (mode=1 两字节长度)', () {
      const adminsLen = 64;
      final admins = List.generate(adminsLen, (_) => List.filled(32, 0xDD));
      final bytes = <int>[
        ...cidPrefix(''),
        ...codeBytes('PMUL'),
        AdminAccountStorageCodec.kindPersonal,
        0x01,
        0x01,
      ];
      for (final a in admins) {
        bytes.addAll(a);
      }
      final r = AdminAccountStorageCodec.tryDecode(Uint8List.fromList(bytes))!;
      expect(r.adminsHex.length, adminsLen);
    });
  });

  group('extractAccountIdFromKey', () {
    test('完整 storage key 末 32 字节 = AccountId', () {
      final key = Uint8List(32 + 16 + 32); // prefix + hash + AccountId
      for (var i = 32 + 16; i < key.length; i++) {
        key[i] = i - (32 + 16);
      }
      final accountId = AdminAccountStorageCodec.extractAccountIdFromKey(key)!;
      expect(accountId.length, 32);
      for (var i = 0; i < 32; i++) {
        expect(accountId[i], i);
      }
      expect(
        AdminAccountStorageCodec.accountHexFromAccountId(accountId),
        '000102030405060708090a0b0c0d0e0f'
        '101112131415161718191a1b1c1d1e1f',
      );
    });

    test('storage key 长度不足返回 null', () {
      expect(
        AdminAccountStorageCodec.extractAccountIdFromKey(Uint8List(20)),
        isNull,
      );
      expect(
        AdminAccountStorageCodec.accountHexFromAccountId(Uint8List(31)),
        isNull,
      );
    });
  });
}
