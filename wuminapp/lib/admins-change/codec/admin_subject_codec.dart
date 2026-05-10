import 'dart:typed_data';

import 'package:wuminapp_mobile/admins-change/codec/subject_id_codec.dart';
import 'package:wuminapp_mobile/admins-change/models/admin_subject.dart';

class AdminSubjectCodec {
  AdminSubjectCodec._();

  static AdminSubjectState? decode(Uint8List subjectId, Uint8List data) {
    if (data.length < 2) return null;
    var offset = 0;
    final org = data[offset++];
    final kind = data[offset++];
    final (count, countLen) = readCompactU32(data, offset);
    offset += countLen;
    final admins = <String>[];
    for (var i = 0; i < count; i++) {
      if (offset + 32 > data.length) return null;
      admins.add(
          AdminSubjectIdCodec.hexEncode(data.sublist(offset, offset + 32)));
      offset += 32;
    }
    if (offset + 4 + 32 + 8 + 8 + 1 > data.length) return null;
    final threshold = _readU32(data, offset);
    offset += 4;
    final creatorHex =
        AdminSubjectIdCodec.hexEncode(data.sublist(offset, offset + 32));
    offset += 32;
    final createdAt = _readU64(data, offset);
    offset += 8;
    final updatedAt = _readU64(data, offset);
    offset += 8;
    final status = data[offset];
    return AdminSubjectState(
      subjectIdHex: AdminSubjectIdCodec.hexEncode(subjectId),
      org: org,
      kind: kind,
      admins: admins,
      threshold: threshold,
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

  static int _readU64(Uint8List data, int offset) {
    return ByteData.sublistView(data).getUint64(offset, Endian.little);
  }
}
