import 'dart:typed_data';

import 'package:polkadart/scale_codec.dart' show ByteOutput, CompactBigIntCodec;
import 'package:citizenapp/citizen/proposal/admins-change/codec/account_id_codec.dart';
import 'package:citizenapp/citizen/shared/institution_code_label.dart';

class AdminSetChangeCallCodec {
  AdminSetChangeCallCodec._();

  static const int palletIndex = 12;
  static const int proposeAdminSetChangeCallIndex = 0;

  static Uint8List build({
    required String institutionCode,
    required Uint8List accountId,
    required List<String> admins,
    required int newThreshold,
  }) {
    if (accountId.length != 32) {
      throw ArgumentError('accountId 必须为 32 字节');
    }
    if (newThreshold <= 0) {
      throw ArgumentError('newThreshold 必须大于 0');
    }
    final output = ByteOutput();
    output.pushByte(palletIndex);
    output.pushByte(proposeAdminSetChangeCallIndex);
    // institution_code: [u8;4]
    output.write(Uint8List.fromList(InstitutionCodeLabel.codeBytes(institutionCode)));
    output.write(accountId);
    output
        .write(CompactBigIntCodec.codec.encode(BigInt.from(admins.length)));
    for (final admin in admins) {
      final bytes = AdminAccountIdCodec.hexDecode(admin);
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
