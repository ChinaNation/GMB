import 'dart:typed_data';

import 'package:citizenapp/citizen/proposal/admins-change/codec/account_id_codec.dart';
import 'package:citizenapp/citizen/proposal/admins-change/models/admin_account.dart';
import 'package:citizenapp/citizen/institution/institution_role_storage_codec.dart';

class AdminAccountCodec {
  AdminAccountCodec._();

  static AdminAccountState? decodeInstitution({
    required String cidNumber,
    required Uint8List data,
    required int institutionKind,
  }) {
    final decoded = InstitutionRoleStorageCodec.decodeAdmins(
      data,
      isPublic: institutionKind == 0,
    );
    if (decoded == null) return null;
    return AdminAccountState(
      cidNumber: cidNumber,
      institutionCode: decoded.institutionCode,
      kind: institutionKind,
      admins: decoded.admins,
      threshold: 0,
    );
  }

  static AdminAccountState? decodePersonal(
    Uint8List personalAccount,
    Uint8List data,
  ) {
    if (data.length < 5) return null;
    var offset = 0;
    // 链端 AdminAccount 前导字段 cid_number: BoundedVec<u8>(个人多签为空);仅消费字节。
    final (cidLen, cidLenSize) = readCompactU32(data, offset);
    offset += cidLenSize + cidLen;
    if (offset + 5 > data.length) return null;
    final institutionCode = String.fromCharCodes(
      data.sublist(offset, offset + 4).where((b) => b != 0),
    );
    offset += 4;
    final kind = data[offset++];
    final decodedAdmins =
        InstitutionRoleStorageCodec.decodeAdminVector(data, offset);
    if (decodedAdmins == null) return null;
    final admins = decodedAdmins.$1;
    offset = decodedAdmins.$2;
    if (offset + 32 + 4 + 4 + 1 != data.length) return null;
    final creatorHex =
        AdminAccountIdCodec.hexEncode(data.sublist(offset, offset + 32));
    offset += 32;
    final createdAt = _readU32(data, offset);
    offset += 4;
    final updatedAt = _readU32(data, offset);
    offset += 4;
    final status = data[offset];
    if (status > 2) return null;
    return AdminAccountState(
      personalAccountHex: AdminAccountIdCodec.hexEncode(personalAccount),
      institutionCode: institutionCode,
      kind: kind,
      admins: admins,
      // runtime 的各管理员 `AdminAccounts` 不保存阈值；
      // 个人多签阈值仍从 internal-vote 的个人阈值 storage 补齐。
      threshold: 0,
      personalCreatorHex: creatorHex,
      personalCreatedAt: createdAt,
      personalUpdatedAt: updatedAt,
      personalStatus: status,
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
