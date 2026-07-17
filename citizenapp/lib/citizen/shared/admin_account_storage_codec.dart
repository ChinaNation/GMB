// 分类管理员模块 `AdminAccounts` 的严格 SCALE 解码器。
//
// - PublicAdmins / PrivateAdmins: key=CID，value=institution_code + admins。
// - PersonalAdmins: key=personal_account，value=个人多签 AdminAccount 完整结构。
//
// 机构 CID 只存在 storage key，机构 value 不再保存 CID、kind、生命周期或创建资料。

import 'dart:convert';
import 'dart:typed_data';

import 'package:citizenapp/citizen/institution/institution_role_storage_codec.dart';
import 'package:citizenapp/citizen/shared/institution_code_label.dart';

class AdminAccountStorageDecoded {
  const AdminAccountStorageDecoded({
    required this.institutionCode,
    required this.kind,
    required this.adminsHex,
  });

  final String institutionCode;
  final int kind;
  final List<String> adminsHex;
}

class AdminAccountStorageCodec {
  AdminAccountStorageCodec._();

  static const int kindPublicInstitution = 0;
  static const int kindPrivateInstitution = 1;
  static const int kindPersonal = 2;

  /// [kind] 必须由扫描到的 pallet 名确定，不能再从机构 value 猜测。
  static AdminAccountStorageDecoded? tryDecode(
    Uint8List bytes, {
    required int kind,
  }) {
    try {
      if (kind != kindPersonal) {
        final decoded = InstitutionRoleStorageCodec.decodeAdmins(bytes);
        if (decoded == null) return null;
        return AdminAccountStorageDecoded(
          institutionCode: decoded.institutionCode,
          kind: kind,
          adminsHex: decoded.admins.map((admin) => admin.adminAccount).toList(),
        );
      }

      var offset = 0;
      final (cidLen, cidLenBytes) = _decodeCompactU32(bytes, offset);
      offset += cidLenBytes + cidLen;
      if (offset + 5 > bytes.length) return null;
      final institutionCode = InstitutionCodeLabel.codeToString(
        bytes.sublist(offset, offset + 4),
      );
      offset += 4;
      final storedKind = bytes[offset++];
      if (storedKind != kindPersonal) return null;

      final (count, lenBytesRead) = _decodeCompactU32(bytes, offset);
      offset += lenBytesRead;
      if (offset + count * 32 > bytes.length) return null;
      final admins = <String>[];
      for (var i = 0; i < count; i++) {
        admins.add(_hexEncode(bytes.sublist(offset, offset + 32)));
        offset += 32;
      }
      // 个人多签 value 后续仍有 creator/区块号/status；扫描只消费 admins。
      if (offset + 32 + 4 + 4 + 1 != bytes.length) return null;
      return AdminAccountStorageDecoded(
        institutionCode: institutionCode,
        kind: kindPersonal,
        adminsHex: admins,
      );
    } catch (_) {
      return null;
    }
  }

  /// 从机构 AdminAccounts 完整 key 提取 CID。key 尾部是
  /// `blake2_128(cid_scale) + Compact<len> + cid_bytes`。
  static String? extractCidNumberFromKey(Uint8List storageKey) {
    const valueOffset = 32 + 16;
    if (storageKey.length <= valueOffset) return null;
    try {
      final (length, compactSize) = _decodeCompactU32(storageKey, valueOffset);
      final start = valueOffset + compactSize;
      if (length <= 0 || start + length != storageKey.length) return null;
      return utf8.decode(storageKey.sublist(start), allowMalformed: false);
    } catch (_) {
      return null;
    }
  }

  /// PersonalAdmins 的 key 仍是 32B personal_account。
  static Uint8List? extractPersonalAccountFromKey(Uint8List storageKey) {
    const expectedLength = 32 + 16 + 32;
    if (storageKey.length != expectedLength) return null;
    return Uint8List.fromList(storageKey.sublist(storageKey.length - 32));
  }

  static String? accountHexFromAccountId(Uint8List accountId) {
    if (accountId.length != 32) return null;
    return _hexEncode(accountId);
  }

  static (int, int) _decodeCompactU32(Uint8List data, int offset) {
    if (offset >= data.length) throw const FormatException('Compact 越界');
    final mode = data[offset] & 0x03;
    if (mode == 0) return (data[offset] >> 2, 1);
    if (mode == 1 && offset + 2 <= data.length) {
      return (((data[offset + 1] << 8) | data[offset]) >> 2, 2);
    }
    if (mode == 2 && offset + 4 <= data.length) {
      final raw = ByteData.sublistView(data).getUint32(offset, Endian.little);
      return (raw >> 2, 4);
    }
    throw const FormatException('不支持的 Compact 编码');
  }

  static String _hexEncode(List<int> bytes) =>
      bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
}
