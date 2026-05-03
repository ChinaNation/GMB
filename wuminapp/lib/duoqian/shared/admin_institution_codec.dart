// `AdminsChange::Institutions` value 的最小 SCALE 解码器(req 3 反向索引依赖)。
//
// 链上 [AdminInstitution<AdminList, AccountId, BlockNumber>] SCALE 字节布局:
//   org: u8                                       (1B)
//   kind: AdminSubjectKind                         (1B,Enum 0/1/2)
//   admins: BoundedVec<AccountId, MaxAdmins>       (Compact<u32> + N×AccountId(32B))
//   threshold: u32                                 (4B)
//   creator: AccountId                             (32B)
//   created_at: BlockNumber(u64)                   (8B)
//   updated_at: BlockNumber(u64)                   (8B)
//   status: AdminSubjectStatus                     (1B,Enum 0/1/2)
//
// 反向索引只需 (org, kind, admins) 三字段过滤,后面字段都跳过(只算长度,不解析)。
//
// 链端定义参考:
// - [admins-change/src/lib.rs::AdminInstitution]
// - [admins-change/src/lib.rs::AdminSubjectKind] (Builtin=0 / Sfid=1 / Personal=2)

import 'dart:typed_data';

/// SCALE 解码后的 AdminInstitution 关键字段。
class AdminInstitutionDecoded {
  const AdminInstitutionDecoded({
    required this.org,
    required this.kind,
    required this.adminPubkeysHex,
  });

  /// 治理 org 标识(0=NRC, 1=PRC, 2=PRB, 3=DUOQIAN 等)。
  final int org;

  /// 主体类型:0=BuiltinInstitution / 1=SfidInstitution / 2=PersonalDuoqian。
  final int kind;

  /// 管理员公钥 hex 列表(小写,无 0x 前缀,32 字节 = 64 hex 字符)。
  final List<String> adminPubkeysHex;
}

/// AdminInstitution SCALE 解码工具集 + InstitutionPalletId 反推工具。
class AdminInstitutionCodec {
  AdminInstitutionCodec._();

  /// 链端 `AdminSubjectKind` 枚举值。
  static const int kindBuiltin = 0;
  static const int kindSfid = 1;
  static const int kindPersonal = 2;

  /// 解码 AdminInstitution SCALE bytes;格式不符返回 null(容错,不抛异常)。
  static AdminInstitutionDecoded? tryDecode(Uint8List bytes) {
    try {
      if (bytes.length < 2) return null;
      final org = bytes[0];
      final kind = bytes[1];

      // admins: Compact<u32> 长度前缀 + N × 32B
      var offset = 2;
      final (count, lenBytesRead) = _decodeCompactU32(bytes, offset);
      offset += lenBytesRead;
      if (offset + count * 32 > bytes.length) return null;

      final admins = <String>[];
      for (var i = 0; i < count; i++) {
        admins.add(_hexEncode(bytes.sublist(offset, offset + 32)));
        offset += 32;
      }

      return AdminInstitutionDecoded(
        org: org,
        kind: kind,
        adminPubkeysHex: admins,
      );
    } catch (_) {
      return null;
    }
  }

  /// 从完整 storage key(twox128 prefix(32B) + blake2_128_concat(institution_id)
  /// = hash(16B) + key(48B))末 48 字节提取 institution_id。
  static Uint8List? extractInstitutionIdFromKey(Uint8List storageKey) {
    if (storageKey.length < 48) return null;
    return Uint8List.fromList(
      storageKey.sublist(storageKey.length - 48, storageKey.length),
    );
  }

  /// 个人多签判别 + 提取 personal_address。
  /// institution_id = personal_address(32B) || zeros(16B)。
  /// 末 16 字节非全零则不是合法个人多签 institution_id,返回 null。
  static String? personalAddressFromInstitutionId(Uint8List institutionId) {
    if (institutionId.length != 48) return null;
    for (var i = 32; i < 48; i++) {
      if (institutionId[i] != 0) return null;
    }
    return _hexEncode(institutionId.sublist(0, 32));
  }

  /// SFID 机构判别 + 提取 sfid_id(去除尾部 0x00 padding)。
  /// institution_id = sfid_id_utf8 || zeros padded 48B。
  /// 完全空(全零)返回 null。
  static Uint8List? sfidIdFromInstitutionId(Uint8List institutionId) {
    if (institutionId.length != 48) return null;
    var realLen = 48;
    while (realLen > 0 && institutionId[realLen - 1] == 0) {
      realLen--;
    }
    if (realLen == 0) return null;
    return Uint8List.fromList(institutionId.sublist(0, realLen));
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
