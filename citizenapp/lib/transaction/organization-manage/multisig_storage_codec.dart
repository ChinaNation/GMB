import 'dart:convert';
import 'dart:typed_data';

import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:citizenapp/citizen/shared/institution_code_label.dart';

/// `OrganizationManage::AccountRegisteredCid` 反查结果。
class RegisteredInstitutionRef {
  const RegisteredInstitutionRef({
    required this.cidNumber,
    required this.accountName,
  });

  final Uint8List cidNumber;
  final Uint8List accountName;

  String get cidNumberText => utf8.decode(cidNumber, allowMalformed: true);
  String get accountNameText => utf8.decode(accountName, allowMalformed: true);
}

/// 管理员与阈值快照。
class AdminSnapshot {
  const AdminSnapshot({
    required this.institutionCode,
    required this.adminsLen,
    required this.threshold,
    required this.admins,
    required this.statusByte,
  });

  final String institutionCode;
  final int adminsLen;
  final int? threshold;
  final List<String> admins;
  final int statusByte;
}

/// 机构账户快照。
class InstitutionAccountSnapshot {
  const InstitutionAccountSnapshot({
    required this.accountHex,
    required this.statusByte,
  });

  final String accountHex;
  final int statusByte;
}

/// P0-3 当前链上 storage 真源 codec。
class MultisigStorageCodec {
  MultisigStorageCodec._();

  static Uint8List accountIdFromAccountHex(String accountHex) {
    final account = hexDecode(accountHex);
    if (account.length != 32) {
      throw ArgumentError('account hex 必须为 32 字节');
    }
    return account;
  }

  static Uint8List adminAccountKey({
    required String institutionCode,
    required Uint8List accountId,
  }) {
    return storageMapKey(
      InstitutionCodeLabel.adminAccountsPalletName(institutionCode),
      'AdminAccounts',
      accountId,
    );
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

  static Uint8List accountRegisteredCidKey(String accountHex) {
    return storageMapKey(
      'OrganizationManage',
      'AccountRegisteredCid',
      hexDecode(accountHex),
    );
  }

  static Uint8List institutionKey(Uint8List cidNumber) {
    return storageMapKey('OrganizationManage', 'Institutions', cidNumber);
  }

  static Uint8List institutionAccountKey(
    Uint8List cidNumber,
    Uint8List accountName,
  ) {
    return storageDoubleMapKey(
      'OrganizationManage',
      'InstitutionAccounts',
      cidNumber,
      accountName,
    );
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
    final result = Uint8List(palletHash.length +
        storageHash.length +
        key1Hash.length +
        key2Hash.length);
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

  static RegisteredInstitutionRef? decodeRegisteredInstitution(Uint8List data) {
    var offset = 0;
    final cid = readBoundedBytes(data, offset);
    if (cid == null) return null;
    offset = cid.nextOffset;
    final accountName = readBoundedBytes(data, offset);
    if (accountName == null) return null;
    return RegisteredInstitutionRef(
      cidNumber: cid.value,
      accountName: accountName.value,
    );
  }

  static AdminSnapshot? decodeAdminAccount(Uint8List data) {
    if (data.length <= 5) return null;
    var offset = 0;
    // institution_code: [u8;4]
    final code = InstitutionCodeLabel.codeToString(
      data.sublist(offset, offset + 4),
    );
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
    // 中文注释：分类管理员模块的 AdminAccounts 后续字段是 creator/时间/status，
    // 动态阈值不在这里保存，必须按 institution_code + account 从 InternalVote 查询。
    if (offset + 32 + 4 + 4 + 1 > data.length) return null;
    return AdminSnapshot(
      institutionCode: code,
      adminsLen: count,
      threshold: null,
      admins: admins,
      statusByte: 0,
    );
  }

  static AdminSnapshot? decodeInstitutionInfo(Uint8List data) {
    var offset = 0;
    final name = readBoundedBytes(data, offset);
    if (name == null) return null;
    offset = name.nextOffset;
    // 中文注释：必须和 runtime 的 InstitutionInfo 字段顺序保持一致：
    // cid_full_name -> main_account -> fee_account -> institution_code。
    if (offset + 32 + 32 + 4 + 4 + 4 > data.length) return null;
    offset += 32; // main_account
    offset += 32; // fee_account
    final code = InstitutionCodeLabel.codeToString(
      data.sublist(offset, offset + 4),
    );
    offset += 4;
    final adminsLen = readU32Le(data, offset);
    offset += 4;
    final threshold = readU32Le(data, offset);
    offset += 4;

    final (adminLen, lenSize) = readCompactU32(data, offset);
    offset += lenSize;
    final admins = <String>[];
    for (var i = 0; i < adminLen; i++) {
      if (offset + 32 > data.length) return null;
      admins.add(hexEncode(data.sublist(offset, offset + 32)));
      offset += 32;
    }
    if (offset + 32 + 4 + 1 > data.length) {
      return AdminSnapshot(
        institutionCode: code,
        adminsLen: adminsLen,
        threshold: threshold,
        admins: admins,
        statusByte: 0,
      );
    }
    offset += 32; // creator
    offset += 4; // created_at: BlockNumber(u32)
    final statusByte = data[offset];
    return AdminSnapshot(
      institutionCode: code,
      adminsLen: adminsLen,
      threshold: threshold,
      admins: admins,
      statusByte: statusByte,
    );
  }

  static InstitutionAccountSnapshot? decodeInstitutionAccount(Uint8List data) {
    if (data.length < 32 + 16 + 1) return null;
    final accountHex = hexEncode(data.sublist(0, 32));
    final statusByte = data[32 + 16];
    return InstitutionAccountSnapshot(
      accountHex: accountHex,
      statusByte: statusByte,
    );
  }

  static int? decodeDynamicThreshold(Uint8List? data) {
    if (data == null || data.length < 4) return null;
    return readU32Le(data, 0);
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
