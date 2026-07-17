import 'dart:convert';
import 'dart:typed_data';

import 'institution_role_models.dart';

/// entity 岗位/任职与 admins 钱包集合的严格 SCALE 解码器。
class InstitutionRoleStorageCodec {
  InstitutionRoleStorageCodec._();

  static InstitutionAdminsStorage? decodeAdmins(Uint8List data) {
    var offset = 0;
    // 机构 CID 是 `AdminAccounts` 的 storage key，不在 value 中重复保存。
    // value 唯一布局为 institution_code:[u8;4]
    // + admins:BoundedVec<(admin_name:BoundedVec<u8>, admin_account:AccountId)>。
    if (offset + 4 > data.length) return null;
    final code = String.fromCharCodes(
        data.sublist(offset, offset + 4).where((b) => b != 0));
    offset += 4;
    final count = _readCompact(data, offset);
    if (count == null) return null;
    offset += count.$2;
    final admins = <InstitutionAdminPerson>[];
    for (var i = 0; i < count.$1; i++) {
      final name = _readBytes(data, offset, minLength: 1, maxLength: 128);
      if (name == null) return null;
      offset = name.$2;
      if (offset + 32 > data.length) return null;
      try {
        admins.add(InstitutionAdminPerson(
          adminName: utf8.decode(name.$1),
          adminAccount: _hex(data.sublist(offset, offset + 32)),
        ));
      } on FormatException {
        return null;
      }
      offset += 32;
    }
    if (offset != data.length) return null;
    return InstitutionAdminsStorage(
      institutionCode: code,
      admins: admins,
    );
  }

  static InstitutionRole? decodeRole(Uint8List data) {
    try {
      var offset = 0;
      final cid = _readBytes(data, offset, minLength: 1, maxLength: 32);
      if (cid == null) return null;
      offset = cid.$2;
      final code = _readBytes(data, offset, minLength: 1, maxLength: 64);
      if (code == null) return null;
      offset = code.$2;
      final name = _readBytes(data, offset, minLength: 1, maxLength: 128);
      if (name == null) return null;
      offset = name.$2;
      if (offset + 2 != data.length ||
          data[offset] > 1 ||
          data[offset + 1] > 1) {
        return null;
      }
      return InstitutionRole(
        cidNumber: utf8.decode(cid.$1),
        roleCode: utf8.decode(code.$1),
        roleName: utf8.decode(name.$1),
        termRequired: data[offset] == 1,
        status: InstitutionRoleStatus.values[data[offset + 1]],
      );
    } on FormatException {
      return null;
    }
  }

  static List<InstitutionAdminAssignment>? decodeAssignments(Uint8List data) {
    var offset = 0;
    final count = _readCompact(data, offset);
    if (count == null) return null;
    offset += count.$2;
    final out = <InstitutionAdminAssignment>[];
    for (var i = 0; i < count.$1; i++) {
      final cid = _readBytes(data, offset, minLength: 1, maxLength: 32);
      if (cid == null) return null;
      offset = cid.$2;
      if (offset + 32 > data.length) return null;
      final account = _hex(data.sublist(offset, offset + 32));
      offset += 32;
      final code = _readBytes(data, offset, minLength: 1, maxLength: 64);
      if (code == null) return null;
      offset = code.$2;
      if (offset + 9 > data.length) return null;
      final termStart =
          ByteData.sublistView(data).getUint32(offset, Endian.little);
      offset += 4;
      final termEnd =
          ByteData.sublistView(data).getUint32(offset, Endian.little);
      offset += 4;
      final source = data[offset++];
      final sourceRef = _readBytes(data, offset, maxLength: 128);
      if (sourceRef == null) return null;
      offset = sourceRef.$2;
      if (offset >= data.length ||
          source >= InstitutionAssignmentSource.values.length) {
        return null;
      }
      final status = data[offset++];
      if (status > 1) return null;
      try {
        out.add(InstitutionAdminAssignment(
          cidNumber: utf8.decode(cid.$1),
          adminAccount: account,
          roleCode: utf8.decode(code.$1),
          termStart: termStart,
          termEnd: termEnd,
          source: InstitutionAssignmentSource.values[source],
          sourceRef: utf8.decode(sourceRef.$1),
          active: status == 0,
        ));
      } on FormatException {
        return null;
      }
    }
    return offset == data.length ? out : null;
  }

  static (Uint8List, int)? _readBytes(
    Uint8List data,
    int offset, {
    int minLength = 0,
    required int maxLength,
  }) {
    final compact = _readCompact(data, offset);
    if (compact == null) return null;
    if (compact.$1 < minLength || compact.$1 > maxLength) return null;
    final start = offset + compact.$2;
    final end = start + compact.$1;
    if (end > data.length) return null;
    return (Uint8List.sublistView(data, start, end), end);
  }

  static (int, int)? _readCompact(Uint8List data, int offset) {
    if (offset >= data.length) return null;
    final first = data[offset];
    final mode = first & 3;
    if (mode == 0) return (first >> 2, 1);
    if (mode == 1 && offset + 2 <= data.length) {
      return ((((data[offset + 1] << 8) | first) >> 2), 2);
    }
    if (mode == 2 && offset + 4 <= data.length) {
      final raw = ByteData.sublistView(data).getUint32(offset, Endian.little);
      return (raw >> 2, 4);
    }
    return null;
  }

  static String _hex(List<int> bytes) =>
      bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
}
