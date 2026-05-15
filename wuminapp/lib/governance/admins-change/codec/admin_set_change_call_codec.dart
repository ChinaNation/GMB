import 'dart:typed_data';

import 'package:polkadart/scale_codec.dart' show ByteOutput, CompactBigIntCodec;
import 'package:wuminapp_mobile/governance/admins-change/codec/subject_id_codec.dart';

class AdminSetChangeCallCodec {
  AdminSetChangeCallCodec._();

  static const int palletIndex = 12;
  static const int proposeAdminSetChangeCallIndex = 0;

  static Uint8List build({
    required int org,
    required Uint8List subjectId,
    required List<String> newAdmins,
    required int newThreshold,
  }) {
    if (subjectId.length != 48) {
      throw ArgumentError('subjectId 必须为 48 字节');
    }
    if (newThreshold <= 0) {
      throw ArgumentError('newThreshold 必须大于 0');
    }
    final output = ByteOutput();
    output.pushByte(palletIndex);
    output.pushByte(proposeAdminSetChangeCallIndex);
    output.pushByte(org);
    output.write(subjectId);
    output
        .write(CompactBigIntCodec.codec.encode(BigInt.from(newAdmins.length)));
    for (final admin in newAdmins) {
      final bytes = AdminSubjectIdCodec.hexDecode(admin);
      if (bytes.length != 32) {
        throw ArgumentError('管理员公钥必须为 32 字节');
      }
      output.write(bytes);
    }
    final thresholdBytes = Uint8List(4);
    ByteData.sublistView(thresholdBytes)
        .setUint32(0, newThreshold, Endian.little);
    output.write(thresholdBytes);
    return output.toBytes();
  }
}
