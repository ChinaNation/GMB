import 'dart:convert';
import 'dart:typed_data';

import 'package:polkadart/polkadart.dart' show Hasher;

class AdminSubjectIdCodec {
  AdminSubjectIdCodec._();

  static const int builtinInstitution = 0x01;
  static const int sfidInstitution = 0x02;
  static const int personalDuoqian = 0x03;
  static const int institutionAccount = 0x05;

  static Uint8List fromBuiltinSfid(String sfidNumber) {
    return _build(
        builtinInstitution, Uint8List.fromList(utf8.encode(sfidNumber)));
  }

  static Uint8List fromAccountHex(int kind, String accountHex) {
    final account = hexDecode(accountHex);
    if (account.length != 32) {
      throw ArgumentError('账户公钥必须为 32 字节');
    }
    return _build(kind, account);
  }

  static Uint8List fromHex(String subjectIdHex) {
    final bytes = hexDecode(subjectIdHex);
    if (bytes.length != 48) {
      throw ArgumentError('subjectId 必须为 48 字节');
    }
    return bytes;
  }

  static Uint8List adminSubjectStorageKey(Uint8List subjectId) {
    final palletHash = Hasher.twoxx128.hashString('AdminsChange');
    final storageHash = Hasher.twoxx128.hashString('Subjects');
    final keyHash = blake2128Concat(subjectId);
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

  static Uint8List _build(int kind, Uint8List payload) {
    if (payload.isEmpty || payload.length > 47) {
      throw ArgumentError('SubjectId payload 长度必须在 1..=47 字节');
    }
    final out = Uint8List(48);
    out[0] = kind;
    out.setAll(1, payload);
    return out;
  }
}
