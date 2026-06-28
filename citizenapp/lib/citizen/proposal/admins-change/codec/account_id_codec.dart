import 'dart:typed_data';

import 'package:citizenapp/citizen/shared/institution_code_label.dart';
import 'package:polkadart/polkadart.dart' show Hasher;

class AdminAccountIdCodec {
  AdminAccountIdCodec._();

  static Uint8List fromAccountHex(String accountHex) {
    final account = hexDecode(accountHex);
    if (account.length != 32) {
      throw ArgumentError('账户公钥必须为 32 字节');
    }
    return account;
  }

  static Uint8List fromHex(String accountHex) {
    final bytes = hexDecode(accountHex);
    if (bytes.length != 32) {
      throw ArgumentError('accountId 必须为 32 字节');
    }
    return bytes;
  }

  static Uint8List adminAccountStorageKey(
    Uint8List accountId, {
    required String institutionCode,
    int? adminKind,
  }) {
    if (accountId.length != 32) {
      throw ArgumentError('accountId 必须为 32 字节');
    }
    final palletHash = Hasher.twoxx128.hashString(
      adminKind == null
          ? InstitutionCodeLabel.adminAccountsPalletName(institutionCode)
          : InstitutionCodeLabel.adminAccountsPalletNameForKind(adminKind),
    );
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

  static List<Uint8List> adminAccountStorageKeys(Uint8List accountId) {
    return const [
      'GenesisAdmins',
      'PersonalAdmins',
      'PublicAdmins',
      'PrivateAdmins'
    ]
        .map((palletName) =>
            _adminAccountStorageKeyForPallet(accountId, palletName))
        .toList(growable: false);
  }

  static Uint8List _adminAccountStorageKeyForPallet(
    Uint8List accountId,
    String palletName,
  ) {
    if (accountId.length != 32) {
      throw ArgumentError('accountId 必须为 32 字节');
    }
    final palletHash = Hasher.twoxx128.hashString(palletName);
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

  static Uint8List blake2128Concat(Uint8List data) {
    final hash = Hasher.blake2b128.hash(data);
    final out = Uint8List(hash.length + data.length);
    out.setAll(0, hash);
    out.setAll(hash.length, data);
    return out;
  }

  static String hexEncode(Iterable<int> bytes) =>
      bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();

  static Uint8List hexDecode(String hex) {
    final clean = normalizeHex(hex);
    final out = Uint8List(clean.length ~/ 2);
    for (var i = 0; i < out.length; i++) {
      out[i] = int.parse(clean.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return out;
  }

  static String normalizeHex(String hex) {
    final clean = hex.trim();
    return clean.startsWith('0x')
        ? clean.substring(2).toLowerCase()
        : clean.toLowerCase();
  }
}
