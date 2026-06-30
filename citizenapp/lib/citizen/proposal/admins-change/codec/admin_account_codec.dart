import 'dart:typed_data';

import 'package:citizenapp/citizen/proposal/admins-change/codec/account_id_codec.dart';
import 'package:citizenapp/citizen/proposal/admins-change/models/admin_account.dart';
import 'package:citizenapp/citizen/shared/admin_profile.dart';

class AdminAccountCodec {
  AdminAccountCodec._();

  static AdminAccountState? decode(Uint8List accountId, Uint8List data) {
    if (data.length < 5) return null;
    var offset = 0;
    final institutionCode = String.fromCharCodes(
      data.sublist(offset, offset + 4).where((b) => b != 0),
    );
    offset += 4;
    final kind = data[offset++];
    final (count, countLen) = readCompactU32(data, offset);
    offset += countLen;
    // A2:`AdminAccounts.admins` 为 `Vec<AdminProfile>`(个人多签 kind=2 为裸 `Vec<AccountId>`)。
    final adminsRead = AdminProfile.decodeAdminsVec(
      data,
      offset,
      count,
      isPersonal: kind == 2,
    );
    if (adminsRead == null) return null;
    final (profiles, afterAdmins) = adminsRead;
    offset = afterAdmins;
    if (offset + 32 + 4 + 4 + 1 > data.length) return null;
    final creatorHex =
        AdminAccountIdCodec.hexEncode(data.sublist(offset, offset + 32));
    offset += 32;
    final createdAt = _readU32(data, offset);
    offset += 4;
    final updatedAt = _readU32(data, offset);
    offset += 4;
    final status = data[offset];
    return AdminAccountState(
      accountHex: AdminAccountIdCodec.hexEncode(accountId),
      institutionCode: institutionCode,
      kind: kind,
      profiles: profiles,
      // 中文注释：runtime 的各管理员 `AdminAccounts` 已不再保存阈值；
      // 调用方必须从 internal-vote 动态阈值 storage 或治理固定常量补齐。
      threshold: 0,
      creatorHex: creatorHex,
      createdAt: createdAt,
      updatedAt: updatedAt,
      status: status,
    );
  }

  static (int, int) readCompactU32(Uint8List data, int offset) {
    final first = data[offset];
    final mode = first & 0x03;
    if (mode == 0) return (first >> 2, 1);
    if (mode == 1) {
      return (((data[offset + 1] << 8) | first) >> 2, 2);
    }
    if (mode == 2) {
      final raw = data[offset] |
          (data[offset + 1] << 8) |
          (data[offset + 2] << 16) |
          (data[offset + 3] << 24);
      return (raw >> 2, 4);
    }
    final len = (first >> 2) + 4;
    var value = 0;
    for (var i = 0; i < len && i < 8; i++) {
      value |= data[offset + 1 + i] << (8 * i);
    }
    return (value, len + 1);
  }

  static int _readU32(Uint8List data, int offset) {
    return ByteData.sublistView(data).getUint32(offset, Endian.little);
  }
}
