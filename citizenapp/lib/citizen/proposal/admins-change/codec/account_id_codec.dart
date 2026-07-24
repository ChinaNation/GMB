import 'dart:convert';
import 'dart:typed_data';

import 'package:citizenapp/citizen/shared/account_derivation.dart';
import 'package:citizenapp/citizen/shared/institution_code_label.dart';
import 'package:polkadart/polkadart.dart' show Hasher;

class AdminAccountIdCodec {
  AdminAccountIdCodec._();

  static Uint8List fromAccountIdText(String accountId) {
    final account = _decodeAccountId(accountId);
    if (account.length != 32) {
      throw ArgumentError('账户公钥必须为 32 字节');
    }
    return account;
  }

  static Uint8List institutionAdminStorageKey(
    String cidNumber, {
    required String institutionCode,
    int? adminKind,
  }) {
    final cidBytes = utf8.encode(cidNumber);
    if (cidBytes.isEmpty || cidBytes.length > 32) {
      throw ArgumentError('机构 cid_number 必须为 1..32 字节');
    }
    final palletHash = Hasher.twoxx128.hashString(
      adminKind == null
          ? InstitutionCodeLabel.adminAccountsPalletName(institutionCode)
          : InstitutionCodeLabel.adminAccountsPalletNameForKind(adminKind),
    );
    final storageHash = Hasher.twoxx128.hashString('AdminAccounts');
    final keyHash = blake2128Concat(scaleBytes(cidBytes));
    final out =
        Uint8List(palletHash.length + storageHash.length + keyHash.length);
    var offset = 0;
    out.setAll(offset, palletHash);
    offset += palletHash.length;
    out.setAll(offset, storageHash);
    offset += storageHash.length;
    out.setAll(offset, keyHash);
    return out;
  }

  static Uint8List personalAdminStorageKey(Uint8List accountId) {
    if (accountId.length != 32) {
      throw ArgumentError('accountId 必须为 32 字节');
    }
    final palletHash = Hasher.twoxx128.hashString('PersonalAdmins');
    final storageHash = Hasher.twoxx128.hashString('AdminAccounts');
    final keyHash = blake2128Concat(accountId);
    final out =
        Uint8List(palletHash.length + storageHash.length + keyHash.length);
    var offset = 0;
    out.setAll(offset, palletHash);
    offset += palletHash.length;
    out.setAll(offset, storageHash);
    offset += storageHash.length;
    out.setAll(offset, keyHash);
    return out;
  }

  static Uint8List scaleBytes(List<int> bytes) {
    final length = bytes.length;
    if (length >= 64) throw ArgumentError('当前 CID SCALE 长度必须小于 64');
    return Uint8List.fromList([length << 2, ...bytes]);
  }

  static Uint8List blake2128Concat(Uint8List data) {
    final hash = Hasher.blake2b128.hash(data);
    final out = Uint8List(hash.length + data.length);
    out.setAll(0, hash);
    out.setAll(hash.length, data);
    return out;
  }

  static String hexEncode(Iterable<int> bytes) =>
      bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();

  static Uint8List _decodeAccountId(String accountId) {
    if (!isAccountIdText(accountId)) {
      throw const FormatException('account_id 必须为小写 0x + 64 位十六进制');
    }
    final clean = accountId.substring(2);
    final out = Uint8List(clean.length ~/ 2);
    for (var i = 0; i < out.length; i++) {
      out[i] = int.parse(clean.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return out;
  }

  static String requireAccountId(String hex) {
    if (!isAccountIdText(hex)) {
      throw const FormatException('account_id 必须为小写 0x + 64 位十六进制');
    }
    return hex;
  }
}
