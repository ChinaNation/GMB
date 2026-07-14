import 'dart:convert';
import 'dart:typed_data';

import 'institution_role_models.dart';

/// entity 岗位/任职与 admins 钱包集合的严格 SCALE 解码器。
class InstitutionRoleStorageCodec {
  InstitutionRoleStorageCodec._();

  static InstitutionAdminAccountStorage? decodeAdminAccount(Uint8List data) {
    var offset = 0;
    final cid = _readBytes(data, offset);
    if (cid == null) return null;
    offset = cid.$2;
    if (offset + 4 > data.length) return null;
    final code = String.fromCharCodes(
        data.sublist(offset, offset + 4).where((b) => b != 0));
    offset += 4;
    final count = _readCompact(data, offset);
    if (count == null) return null;
    offset += count.$2;
    final admins = <String>[];
    for (var i = 0; i < count.$1; i++) {
      if (offset + 32 > data.length) return null;
      admins.add(_hex(data.sublist(offset, offset + 32)));
      offset += 32;
    }
    if (offset + 1 != data.length) return null;
    return InstitutionAdminAccountStorage(
      cidNumber: utf8.decode(cid.$1, allowMalformed: true),
      institutionCode: code,
      admins: admins,
      status: data[offset],
    );
  }

  static InstitutionRole? decodeRole(Uint8List data) {
    var offset = 0;
    final cid = _readBytes(data, offset);
    if (cid == null) return null;
    offset = cid.$2;
    final code = _readBytes(data, offset);
    if (code == null) return null;
    offset = code.$2;
    final name = _readBytes(data, offset);
    if (name == null) return null;
    offset = name.$2;
    if (offset + 2 != data.length || data[offset + 1] > 1) return null;
    return InstitutionRole(
      cidNumber: utf8.decode(cid.$1, allowMalformed: true),
      roleCode: utf8.decode(code.$1, allowMalformed: true),
      roleName: utf8.decode(name.$1, allowMalformed: true),
      termRequired: data[offset] != 0,
      status: InstitutionRoleStatus.values[data[offset + 1]],
    );
  }

  static List<InstitutionAdminAssignment>? decodeAssignments(Uint8List data) {
    var offset = 0;
    final count = _readCompact(data, offset);
    if (count == null) return null;
    offset += count.$2;
    final out = <InstitutionAdminAssignment>[];
    for (var i = 0; i < count.$1; i++) {
      final cid = _readBytes(data, offset);
      if (cid == null) return null;
      offset = cid.$2;
      if (offset + 32 > data.length) return null;
      final account = _hex(data.sublist(offset, offset + 32));
      offset += 32;
      final code = _readBytes(data, offset);
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
      final sourceRef = _readBytes(data, offset);
      if (sourceRef == null) return null;
      offset = sourceRef.$2;
      if (offset >= data.length ||
          source >= InstitutionAssignmentSource.values.length) {
        return null;
      }
      final status = data[offset++];
      if (status > 1) return null;
      out.add(InstitutionAdminAssignment(
        cidNumber: utf8.decode(cid.$1, allowMalformed: true),
        adminAccount: account,
        roleCode: utf8.decode(code.$1, allowMalformed: true),
        termStart: termStart,
        termEnd: termEnd,
        source: InstitutionAssignmentSource.values[source],
        sourceRef: utf8.decode(sourceRef.$1, allowMalformed: true),
        active: status == 0,
      ));
    }
    return offset == data.length ? out : null;
  }

  static (Uint8List, int)? _readBytes(Uint8List data, int offset) {
    final compact = _readCompact(data, offset);
    if (compact == null) return null;
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
