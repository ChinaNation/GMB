// 分类管理员模块 `AdminAccounts` value 的最小 SCALE 解码器(req 3 反向索引依赖)。
//
// 链上 [AdminAccount<AdminList, AccountId, BlockNumber>] SCALE 字节布局:
//   institution_code: [u8;4]                      (4B)
//   kind: AdminAccountKind                         (1B,Enum 0/1/2/3)
//   admins: BoundedVec<AccountId, MaxAdmins>       (Compact<u32> + N×AccountId(32B))
//   creator: AccountId                             (32B)
//   created_at: BlockNumber(u64)                   (8B)
//   updated_at: BlockNumber(u64)                   (8B)
//   status: AdminAccountStatus                     (1B,Enum 0/1/2)
//
// 反向索引只需 (institutionCode, kind, admins) 三字段过滤,后面字段都跳过。
//
// 链端定义参考:
// - [admin-primitives/src/lib.rs::AdminAccount]
// - [admin-primitives/src/lib.rs::AdminAccountKind]
//   (Genesis=0 / Public=1 / Private=2 / Personal=3)
//
// 中文注释：本文件统一放在 `lib/governance/shared/`，供机构多签、个人多签
// 和治理提案展示复用；业务模块不得各自复制管理员账户解码逻辑。

import 'dart:typed_data';

import 'package:citizenapp/citizen/shared/institution_code_label.dart';

/// SCALE 解码后的 AdminAccount 关键字段。
class AdminAccountStorageDecoded {
  const AdminAccountStorageDecoded({
    required this.institutionCode,
    required this.kind,
    required this.adminsHex,
  });

  /// 4 字节机构码字符串（"NRC"/"PRC"/"PRB"/"PMUL"/"CGOV" 等）。
  final String institutionCode;

  /// 管理员账户类型:0=Genesis / 1=Public / 2=Private / 3=Personal。
  final int kind;

  /// 管理员公钥 hex 列表(小写,无 0x 前缀,32 字节 = 64 hex 字符)。
  final List<String> adminsHex;
}

/// AdminAccount SCALE 解码工具集 + AccountId 提取工具。
class AdminAccountStorageCodec {
  AdminAccountStorageCodec._();

  /// 链端 `AdminAccountKind` 枚举值。
  static const int kindGenesis = 0;
  static const int kindPublicInstitution = 1;
  static const int kindPrivateInstitution = 2;
  static const int kindPersonal = 3;

  /// 解码 AdminAccount SCALE bytes;格式不符返回 null(容错,不抛异常)。
  static AdminAccountStorageDecoded? tryDecode(Uint8List bytes) {
    try {
      // institution_code: [u8;4] + kind: u8 = 5 bytes minimum before admins
      if (bytes.length < 5) return null;
      final institutionCode =
          InstitutionCodeLabel.codeToString(bytes.sublist(0, 4));
      final kind = bytes[4];

      // admins: Compact<u32> 长度前缀 + N × 32B
      var offset = 5;
      final (count, lenBytesRead) = _decodeCompactU32(bytes, offset);
      offset += lenBytesRead;
      if (offset + count * 32 > bytes.length) return null;

      final admins = <String>[];
      for (var i = 0; i < count; i++) {
        admins.add(_hexEncode(bytes.sublist(offset, offset + 32)));
        offset += 32;
      }

      return AdminAccountStorageDecoded(
        institutionCode: institutionCode,
        kind: kind,
        adminsHex: admins,
      );
    } catch (_) {
      return null;
    }
  }

  /// 从完整 storage key(twox128 prefix(32B) + blake2_128_concat(account_id)
  /// = hash(16B) + key(32B))末 32 字节提取 AccountId。
  static Uint8List? extractAccountIdFromKey(Uint8List storageKey) {
    if (storageKey.length < 32) return null;
    return Uint8List.fromList(storageKey.sublist(storageKey.length - 32));
  }

  /// 把 32B AccountId 转成小写 hex；格式不符返回 null。
  static String? accountHexFromAccountId(Uint8List accountId) {
    if (accountId.length != 32) return null;
    return _hexEncode(accountId);
  }

  // ──── 内部工具 ────

  /// SCALE Compact<u32> 解码,返回 (value, bytesRead)。
  /// mode 0 = 1B / 1 = 2B / 2 = 4B / 3 = big int(本场景不会触发,fallback 取 1B 长度)。
  static (int, int) _decodeCompactU32(Uint8List data, int offset) {
    final mode = data[offset] & 0x03;
    if (mode == 0) return (data[offset] >> 2, 1);
    if (mode == 1) {
      return ((data[offset] >> 2) | (data[offset + 1] << 6), 2);
    }
    if (mode == 2) {
      return (
        (data[offset] >> 2) |
            (data[offset + 1] << 6) |
            (data[offset + 2] << 14) |
            (data[offset + 3] << 22),
        4
      );
    }
    // mode 3: BigInt 模式,本场景 admins 数量不会触发(MaxAdmins ≤ 64)。
    final len = ((data[offset] >> 2) + 4) & 0xFF;
    var value = 0;
    for (var i = 0; i < len && i < 8; i++) {
      value |= data[offset + 1 + i] << (i * 8);
    }
    return (value, len + 1);
  }

  static String _hexEncode(Uint8List bytes) =>
      bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
}
