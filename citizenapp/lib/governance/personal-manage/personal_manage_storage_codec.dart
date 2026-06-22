import 'dart:convert';
import 'dart:typed_data';

import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:citizenapp/governance/shared/institution_code_label.dart';

/// 个人多签账户生命周期快照。
///
/// `PersonalManage::PersonalAccounts` 只保存个人账户生命周期元数据；
/// 管理员真源在 `AdminsChange::AdminAccounts`，动态阈值真源在 `InternalVote`。
class PersonalManageAccountSnapshot {
  const PersonalManageAccountSnapshot({
    required this.creatorHex,
    required this.accountName,
    required this.createdAt,
    required this.statusByte,
  });

  final String creatorHex;
  final Uint8List accountName;
  final int createdAt;
  final int statusByte;
}

/// 管理员与阈值快照。
class PersonalManageAdminSnapshot {
  const PersonalManageAdminSnapshot({
    required this.institutionCode,
    required this.adminsLen,
    required this.admins,
  });

  final String institutionCode;
  final int adminsLen;
  final List<String> admins;
}

/// PersonalManage 专属 storage codec。
class PersonalManageStorageCodec {
  PersonalManageStorageCodec._();

  static Uint8List personalAccountsKey(String personalAccountHex) {
    return storageMapKey(
      'PersonalManage',
      'PersonalAccounts',
      hexDecode(personalAccountHex),
    );
  }

  static Uint8List accountIdFromAccountHex(String accountHex) {
    final account = hexDecode(accountHex);
    if (account.length != 32) {
      throw ArgumentError('account hex 必须为 32 字节');
    }
    return account;
  }

  static Uint8List adminAccountKey(Uint8List accountId) {
    return storageMapKey('AdminsChange', 'AdminAccounts', accountId);
  }

  static Uint8List dynamicThresholdKey({
    required String storageName,
    required String institutionCode,
    required Uint8List accountId,
  }) {
    return storageDoubleMapKey(
      'InternalVote',
      storageName,
      Uint8List.fromList(InstitutionCodeLabel.codeBytes(institutionCode)),
      accountId,
    );
  }

  static PersonalManageAccountSnapshot? decodePersonalAccount(
    Uint8List data,
  ) {
    if (data.length < 32 + 1 + 4 + 1) return null;
    var offset = 0;
    final creatorHex = hexEncode(data.sublist(offset, offset + 32));
    offset += 32;
    final accountName = readBoundedBytes(data, offset);
    if (accountName == null) return null;
    offset = accountName.nextOffset;
    if (offset + 4 + 1 > data.length) return null;
    final createdAt = readU32Le(data, offset);
    offset += 4;
    final statusByte = data[offset];
    return PersonalManageAccountSnapshot(
      creatorHex: creatorHex,
      accountName: accountName.value,
      createdAt: createdAt,
      statusByte: statusByte,
    );
  }

  static PersonalManageAdminSnapshot? decodeAdminAccount(Uint8List data) {
    // institution_code: [u8;4] + kind: u8 = 5 bytes minimum before admins
    if (data.length <= 5) return null;
    var offset = 0;
    final institutionCode =
        InstitutionCodeLabel.codeToString(data.sublist(offset, offset + 4));
    offset += 4;
    offset++; // AdminAccountKind
    final (count, lenSize) = readCompactU32(data, offset);
    offset += lenSize;
    final admins = <String>[];
    for (var i = 0; i < count; i++) {
      if (offset + 32 > data.length) return null;
      admins.add(hexEncode(data.sublist(offset, offset + 32)));
      offset += 32;
    }
    // 中文注释：AdminsChange::AdminAccounts 已不保存 threshold；
    // 后续字段是 creator/created_at/updated_at/status，阈值必须另查 InternalVote。
    if (offset + 32 + 4 + 4 + 1 > data.length) return null;
    return PersonalManageAdminSnapshot(
      institutionCode: institutionCode,
      adminsLen: count,
      admins: admins,
    );
  }

  static int? decodeDynamicThreshold(Uint8List? data) {
    if (data == null || data.length < 4) return null;
    return readU32Le(data, 0);
  }

  static Uint8List storageMapKey(
    String palletName,
    String storageName,
    Uint8List keyData,
  ) {
    final palletHash = Hasher.twoxx128.hashString(palletName);
    final storageHash = Hasher.twoxx128.hashString(storageName);
    final keyHash = blake2128Concat(keyData);
    final result =
        Uint8List(palletHash.length + storageHash.length + keyHash.length);
    var offset = 0;
    result.setAll(offset, palletHash);
    offset += palletHash.length;
    result.setAll(offset, storageHash);
    offset += storageHash.length;
    result.setAll(offset, keyHash);
    return result;
  }

  static Uint8List storageDoubleMapKey(
    String palletName,
    String storageName,
    Uint8List key1Data,
    Uint8List key2Data,
  ) {
    final palletHash = Hasher.twoxx128.hashString(palletName);
    final storageHash = Hasher.twoxx128.hashString(storageName);
    final key1Hash = blake2128Concat(key1Data);
    final key2Hash = blake2128Concat(key2Data);
    final result = Uint8List(
      palletHash.length +
          storageHash.length +
          key1Hash.length +
          key2Hash.length,
    );
    var offset = 0;
    result.setAll(offset, palletHash);
    offset += palletHash.length;
    result.setAll(offset, storageHash);
    offset += storageHash.length;
    result.setAll(offset, key1Hash);
    offset += key1Hash.length;
    result.setAll(offset, key2Hash);
    return result;
  }

  static Uint8List blake2128Concat(Uint8List data) {
    final hash = Hasher.blake2b128.hash(data);
    final result = Uint8List(hash.length + data.length);
    result.setAll(0, hash);
    result.setAll(hash.length, data);
    return result;
  }

  static ({Uint8List value, int nextOffset})? readBoundedBytes(
    Uint8List data,
    int offset,
  ) {
    if (offset >= data.length) return null;
    final (len, lenSize) = readCompactU32(data, offset);
    offset += lenSize;
    if (offset + len > data.length) return null;
    return (
      value: Uint8List.fromList(data.sublist(offset, offset + len)),
      nextOffset: offset + len,
    );
  }

  static (int, int) readCompactU32(Uint8List data, int offset) {
    if (offset >= data.length) {
      throw const FormatException('Compact<u32> offset 越界');
    }
    final first = data[offset];
    final mode = first & 0x03;
    if (mode == 0) return (first >> 2, 1);
    if (mode == 1) {
      if (offset + 1 >= data.length) {
        throw const FormatException('Compact<u32> mode1 长度不足');
      }
      return ((first >> 2) | (data[offset + 1] << 6), 2);
    }
    if (mode == 2) {
      if (offset + 3 >= data.length) {
        throw const FormatException('Compact<u32> mode2 长度不足');
      }
      return (
        (first >> 2) |
            (data[offset + 1] << 6) |
            (data[offset + 2] << 14) |
            (data[offset + 3] << 22),
        4,
      );
    }
    throw const FormatException('Compact<u32> big-integer 模式暂不支持');
  }

  static int readU32Le(Uint8List data, int offset) {
    final bd = ByteData.sublistView(data);
    return bd.getUint32(offset, Endian.little);
  }

  static String accountNameText(Uint8List bytes) {
    return utf8.decode(bytes, allowMalformed: true);
  }

  static String hexEncode(List<int> bytes) =>
      bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();

  static Uint8List hexDecode(String hex) {
    final h = hex.startsWith('0x') ? hex.substring(2) : hex;
    final result = Uint8List(h.length ~/ 2);
    for (var i = 0; i < result.length; i++) {
      result[i] = int.parse(h.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return result;
  }
}
