import 'dart:convert';
import 'dart:typed_data';

import 'package:polkadart/polkadart.dart' show Hasher;

/// `OrganizationManage::AddressRegisteredSfid` 反查结果。
class RegisteredInstitutionRef {
  const RegisteredInstitutionRef({
    required this.sfidNumber,
    required this.accountName,
  });

  final Uint8List sfidNumber;
  final Uint8List accountName;

  String get sfidNumberText => utf8.decode(sfidNumber, allowMalformed: true);
  String get accountNameText => utf8.decode(accountName, allowMalformed: true);
}

/// 管理员与阈值快照。
class DuoqianAdminSnapshot {
  const DuoqianAdminSnapshot({
    required this.adminCount,
    required this.threshold,
    required this.adminPubkeys,
    required this.statusByte,
  });

  final int adminCount;
  final int threshold;
  final List<String> adminPubkeys;
  final int statusByte;
}

/// 个人多签账户生命周期快照。
///
/// 第 3 步破坏式改造后，`PersonalManage::PersonalDuoqians` 不再镜像
/// admins / threshold；管理员真源统一在 `AdminsChange::Subjects`。
/// 重新创世前总审计后，creator/account_name/created_at/status 统一存放在本表。
class PersonalDuoqianSnapshot {
  const PersonalDuoqianSnapshot({
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

/// 机构账户快照。
class InstitutionAccountSnapshot {
  const InstitutionAccountSnapshot({
    required this.addressHex,
    required this.statusByte,
  });

  final String addressHex;
  final int statusByte;
}

/// P0-3 当前链上 storage 真源 codec。
class DuoqianStorageCodec {
  DuoqianStorageCodec._();

  static const int subjectKindBuiltin = 0x01;
  static const int subjectKindSfidInstitution = 0x02;
  static const int subjectKindPersonalDuoqian = 0x03;
  static const int subjectKindInstitutionAccount = 0x05;

  static Uint8List subjectIdFromBuiltin(String sfidNumber) {
    final raw = Uint8List.fromList(utf8.encode(sfidNumber));
    return _buildSubjectId(subjectKindBuiltin, raw);
  }

  static Uint8List subjectIdFromSfidBytes(Uint8List sfidNumber) {
    return _buildSubjectId(subjectKindSfidInstitution, sfidNumber);
  }

  static Uint8List subjectIdFromAccountHex(String accountHex) {
    final account = hexDecode(accountHex);
    if (account.length != 32) {
      throw ArgumentError('account hex 必须为 32 字节');
    }
    return _buildSubjectId(subjectKindPersonalDuoqian, account);
  }

  static Uint8List subjectIdFromInstitutionAccountHex(String accountHex) {
    final account = hexDecode(accountHex);
    if (account.length != 32) {
      throw ArgumentError('account hex 必须为 32 字节');
    }
    return _buildSubjectId(subjectKindInstitutionAccount, account);
  }

  static Uint8List adminSubjectKey(Uint8List subjectId) {
    return storageMapKey('AdminsChange', 'Subjects', subjectId);
  }

  static Uint8List addressRegisteredSfidKey(String duoqianAddressHex) {
    return storageMapKey(
      'OrganizationManage',
      'AddressRegisteredSfid',
      hexDecode(duoqianAddressHex),
    );
  }

  static Uint8List institutionKey(Uint8List sfidNumber) {
    return storageMapKey('OrganizationManage', 'Institutions', sfidNumber);
  }

  static Uint8List institutionAccountKey(
    Uint8List sfidNumber,
    Uint8List accountName,
  ) {
    return storageDoubleMapKey(
      'OrganizationManage',
      'InstitutionAccounts',
      sfidNumber,
      accountName,
    );
  }

  static Uint8List personalDuoqiansKey(String personalAddressHex) {
    return storageMapKey(
      'PersonalManage',
      'PersonalDuoqians',
      hexDecode(personalAddressHex),
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
    final sfid = readBoundedBytes(data, offset);
    if (sfid == null) return null;
    offset = sfid.nextOffset;
    final accountName = readBoundedBytes(data, offset);
    if (accountName == null) return null;
    return RegisteredInstitutionRef(
      sfidNumber: sfid.value,
      accountName: accountName.value,
    );
  }

  static DuoqianAdminSnapshot? decodeAdminSubject(Uint8List data) {
    if (data.length <= 2) return null;
    var offset = 2; // org + AdminSubjectKind
    final (count, lenSize) = readCompactU32(data, offset);
    offset += lenSize;
    final admins = <String>[];
    for (var i = 0; i < count; i++) {
      if (offset + 32 > data.length) return null;
      admins.add(hexEncode(data.sublist(offset, offset + 32)));
      offset += 32;
    }
    if (offset + 4 > data.length) return null;
    final threshold = readU32Le(data, offset);
    return DuoqianAdminSnapshot(
      adminCount: count,
      threshold: threshold,
      adminPubkeys: admins,
      statusByte: 0,
    );
  }

  static DuoqianAdminSnapshot? decodeInstitutionInfo(Uint8List data) {
    var offset = 0;
    final name = readBoundedBytes(data, offset);
    if (name == null) return null;
    offset = name.nextOffset;
    if (offset + 32 + 32 + 4 + 4 > data.length) return null;
    offset += 32; // main_address
    offset += 32; // fee_address
    final adminCount = readU32Le(data, offset);
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
      return DuoqianAdminSnapshot(
        adminCount: adminCount,
        threshold: threshold,
        adminPubkeys: admins,
        statusByte: 0,
      );
    }
    offset += 32; // creator
    offset += 4; // created_at: BlockNumber(u32)
    final statusByte = data[offset];
    return DuoqianAdminSnapshot(
      adminCount: adminCount,
      threshold: threshold,
      adminPubkeys: admins,
      statusByte: statusByte,
    );
  }

  static InstitutionAccountSnapshot? decodeInstitutionAccount(Uint8List data) {
    if (data.length < 32 + 16 + 1) return null;
    final addressHex = hexEncode(data.sublist(0, 32));
    final statusByte = data[32 + 16];
    return InstitutionAccountSnapshot(
      addressHex: addressHex,
      statusByte: statusByte,
    );
  }

  static PersonalDuoqianSnapshot? decodePersonalDuoqian(Uint8List data) {
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
    return PersonalDuoqianSnapshot(
      creatorHex: creatorHex,
      accountName: accountName.value,
      createdAt: createdAt,
      statusByte: statusByte,
    );
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

  static Uint8List _buildSubjectId(int kind, Uint8List payload) {
    if (payload.isEmpty || payload.length > 47) {
      throw ArgumentError('subject payload 长度需在 1..=47 字节');
    }
    final out = Uint8List(48);
    out[0] = kind;
    out.setAll(1, payload);
    return out;
  }
}
