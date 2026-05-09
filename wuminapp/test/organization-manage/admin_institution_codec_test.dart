// AdminInstitutionCodec golden test:固定字节 → 固定解码结果。
//
// 四类主体覆盖:
// - PersonalDuoqian (kind=2):institution_id 末 16 字节全零
// - InstitutionAccount (kind=3):SubjectId(0x05) 绑定具体机构账户
// - SfidInstitution (kind=1):institution_id 含 sfid_number UTF-8 + 尾部零 padding
// - BuiltinInstitution (kind=0):创世内置主体(NRC/PRC/PRB)

import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/organization-manage/shared/admin_institution_codec.dart';

void main() {
  group('tryDecode', () {
    test('成功解码 BuiltinInstitution(0 admins)', () {
      // org=0, kind=0, admins=Compact(0) = 0x00, 后续字段忽略
      final bytes = Uint8List.fromList([0, 0, 0]);
      final r = AdminInstitutionCodec.tryDecode(bytes)!;
      expect(r.org, 0);
      expect(r.kind, AdminInstitutionCodec.kindBuiltin);
      expect(r.adminPubkeysHex, isEmpty);
    });

    test('成功解码 SfidInstitution 含 2 个 admin', () {
      final admin1 = List.filled(32, 0xAA);
      final admin2 = List.filled(32, 0xBB);
      // org=3, kind=1, admins=Compact(2)=0x08 + 32+32 字节
      final bytes = Uint8List.fromList([
        3, // org = ORG_REN
        AdminInstitutionCodec.kindSfid,
        0x08, // Compact(2): (2<<2) | 0 = 8
        ...admin1,
        ...admin2,
        // 后续字段(threshold u32 + creator 32B + ...)解码器跳过,可省略
      ]);
      final r = AdminInstitutionCodec.tryDecode(bytes)!;
      expect(r.kind, AdminInstitutionCodec.kindSfid);
      expect(r.adminPubkeysHex.length, 2);
      expect(r.adminPubkeysHex[0], 'aa' * 32);
      expect(r.adminPubkeysHex[1], 'bb' * 32);
    });

    test('成功解码 PersonalDuoqian 含 3 个 admin', () {
      final a1 = List.filled(32, 0x11);
      final a2 = List.filled(32, 0x22);
      final a3 = List.filled(32, 0x33);
      final bytes = Uint8List.fromList([
        3,
        AdminInstitutionCodec.kindPersonal,
        0x0C, // Compact(3): (3<<2) | 0 = 12
        ...a1,
        ...a2,
        ...a3,
      ]);
      final r = AdminInstitutionCodec.tryDecode(bytes)!;
      expect(r.kind, AdminInstitutionCodec.kindPersonal);
      expect(r.adminPubkeysHex.length, 3);
    });

    test('成功解码 InstitutionAccount 含 2 个 admin', () {
      final a1 = List.filled(32, 0x44);
      final a2 = List.filled(32, 0x55);
      final bytes = Uint8List.fromList([
        3,
        AdminInstitutionCodec.kindInstitutionAccount,
        0x08,
        ...a1,
        ...a2,
      ]);
      final r = AdminInstitutionCodec.tryDecode(bytes)!;
      expect(r.kind, AdminInstitutionCodec.kindInstitutionAccount);
      expect(r.adminPubkeysHex, ['44' * 32, '55' * 32]);
    });

    test('字节不足返回 null,不抛异常', () {
      expect(AdminInstitutionCodec.tryDecode(Uint8List(0)), isNull);
      expect(AdminInstitutionCodec.tryDecode(Uint8List.fromList([0])), isNull);
    });

    test('admins 数量超过实际字节返回 null', () {
      final bytes = Uint8List.fromList([
        0, 0,
        0x08, // 声明 2 个 admin 但只给 1 个的字节
        ...List.filled(32, 0xCC),
      ]);
      expect(AdminInstitutionCodec.tryDecode(bytes), isNull);
    });

    test('Compact 16 admins (mode=1 两字节长度)', () {
      // Compact(16) = (16<<2) | 1 = 65 = 0x41,但 16 仍可在 mode=0 表示
      // 测试 mode=1 边界:64+ 时进 mode=1
      // 64 = (64<<2)|1 = 257 → 高 6 位=64 低位 mode=1 → 0x01,0x01 (encode_compact 64 → [0x01, 0x01])
      const adminCount = 64;
      final admins = List.generate(adminCount, (_) => List.filled(32, 0xDD));
      // 64 SCALE Compact: low_byte=(64<<2 & 0xFF)|1=0x01, high=64>>6=0x01
      final bytes = <int>[
        0,
        AdminInstitutionCodec.kindPersonal,
        0x01,
        0x01,
      ];
      for (final a in admins) {
        bytes.addAll(a);
      }
      final r = AdminInstitutionCodec.tryDecode(Uint8List.fromList(bytes))!;
      expect(r.adminPubkeysHex.length, adminCount);
    });
  });

  group('extractInstitutionIdFromKey', () {
    test('完整 storage key 末 48 字节 = institution_id', () {
      final key = Uint8List(32 + 16 + 48); // prefix + hash + id
      for (var i = 32 + 16; i < key.length; i++) {
        key[i] = i - (32 + 16); // id 内容 0..47
      }
      final id = AdminInstitutionCodec.extractInstitutionIdFromKey(key)!;
      expect(id.length, 48);
      for (var i = 0; i < 48; i++) {
        expect(id[i], i);
      }
    });

    test('storage key 长度不足返回 null', () {
      expect(
        AdminInstitutionCodec.extractInstitutionIdFromKey(Uint8List(20)),
        isNull,
      );
    });
  });

  group('personalAddressFromInstitutionId', () {
    test('D 协议: kind=0x03 + 32B account + 15B 零 → 返回 32B hex', () {
      // D 协议: byte[0]=0x03 (PersonalDuoqian), byte[1..33]=AccountId, byte[33..48]=15B 零
      final id = Uint8List(48);
      id[0] = 0x03; // SubjectKind::PersonalDuoqian
      for (var i = 1; i < 33; i++) {
        id[i] = 0xAB;
      }
      final addr = AdminInstitutionCodec.personalAddressFromInstitutionId(id);
      expect(addr, 'ab' * 32);
    });

    test('kind tag 不为 0x03 → 返回 null', () {
      final id = Uint8List(48);
      id[0] = 0x01; // Builtin,不是 personal
      for (var i = 1; i < 33; i++) {
        id[i] = 0xAB;
      }
      expect(
        AdminInstitutionCodec.personalAddressFromInstitutionId(id),
        isNull,
      );
    });

    test('末 15 字节(byte[33..48])非全零 → 返回 null', () {
      final id = Uint8List(48);
      id[0] = 0x03;
      id[40] = 1;
      expect(
        AdminInstitutionCodec.personalAddressFromInstitutionId(id),
        isNull,
      );
    });

    test('长度非 48 → 返回 null', () {
      expect(
        AdminInstitutionCodec.personalAddressFromInstitutionId(Uint8List(32)),
        isNull,
      );
    });
  });

  group('institutionAccountFromSubjectId', () {
    test('ADR-015: kind=0x05 + 32B account + 15B 零 → 返回 32B hex', () {
      final id = Uint8List(48);
      id[0] = 0x05;
      for (var i = 1; i < 33; i++) {
        id[i] = 0xCD;
      }
      final addr = AdminInstitutionCodec.institutionAccountFromSubjectId(id);
      expect(addr, 'cd' * 32);
    });

    test('kind tag 不为 0x05 → 返回 null', () {
      final id = Uint8List(48);
      id[0] = 0x02;
      expect(AdminInstitutionCodec.institutionAccountFromSubjectId(id), isNull);
    });

    test('末 15 字节非全零 → 返回 null', () {
      final id = Uint8List(48);
      id[0] = 0x05;
      id[47] = 1;
      expect(AdminInstitutionCodec.institutionAccountFromSubjectId(id), isNull);
    });
  });

  group('sfidNumberFromInstitutionId', () {
    test('D 协议: kind=0x02 + sfid_number + 右零填充 → 提取 sfid_number', () {
      // D 协议: byte[0]=0x02 (SfidInstitution), byte[1..]=sfid_number 字节(右填零)
      const sfidStr = 'SFR-LN001-CB0C-Z001-20260222';
      final sfidBytes = sfidStr.codeUnits;
      final id = Uint8List(48);
      id[0] = 0x02; // SubjectKind::SfidInstitution
      id.setAll(1, sfidBytes);
      final extracted = AdminInstitutionCodec.sfidNumberFromInstitutionId(id)!;
      expect(extracted.length, sfidBytes.length);
      expect(String.fromCharCodes(extracted), sfidStr);
    });

    test('kind tag 不为 0x02 → 返回 null', () {
      final id = Uint8List(48);
      id[0] = 0x03; // Personal,不是 sfid
      id[1] = 0x41;
      expect(
        AdminInstitutionCodec.sfidNumberFromInstitutionId(id),
        isNull,
      );
    });

    test('全零 institution_id → 返回 null', () {
      expect(
        AdminInstitutionCodec.sfidNumberFromInstitutionId(Uint8List(48)),
        isNull,
      );
    });

    test('长度非 48 → 返回 null', () {
      expect(
        AdminInstitutionCodec.sfidNumberFromInstitutionId(Uint8List(32)),
        isNull,
      );
    });
  });
}
