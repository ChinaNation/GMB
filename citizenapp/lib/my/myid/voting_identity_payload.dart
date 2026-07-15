// 公民链上身份确认载荷独立解码器。
//
// 两色识别模型:签名前必须能从 payload 字节独立解码出全部字段并展示给公民,
// 解不开一律拒签。SCALE 布局与链端结构体逐字节一致,字段变更三处必须同步:
//   citizenchain/runtime/misc/citizen-identity/src/lib.rs
//     (VotingIdentityPayload / CandidateIdentityPayload)
//   citizenwallet/lib/signer/payload_decoder.dart(_readCandidateIdentityPayload)
//   本文件
// 注:链上「已存储」的 CandidateIdentity(含 birth_date)另由
//   citizenapp/lib/my/myid/myid_service.dart(_decodeCandidateIdentity)解码。
import 'dart:convert';
import 'dart:typed_data';

import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

/// 链端 register_voting_identity 的最低年龄门槛(周岁)。
const int kMinOnchainCitizenAgeYears = 16;

/// CitizenChain SS58 前缀。
const int _ss58Prefix = 2027;

enum CitizenIdentityConsentLevel { voting, candidate }

class VotingIdentityConsentPayload {
  const VotingIdentityConsentPayload({
    required this.identityLevel,
    required this.cidNumber,
    required this.walletPubkeyHex,
    required this.walletAddress,
    required this.ageYears,
    required this.validFrom,
    required this.validUntil,
    required this.statusNormal,
    required this.provinceCode,
    required this.cityCode,
    required this.townCode,
    this.birthProvinceCode,
    this.birthCityCode,
    this.birthTownCode,
    this.citizenFullName,
    this.citizenSexLabel,
    this.birthDate,
  });

  final CitizenIdentityConsentLevel identityLevel;
  final String cidNumber;

  /// 0x 小写 hex,32 字节公民钱包公钥。
  final String walletPubkeyHex;

  /// SS58(prefix=2027)展示地址。
  final String walletAddress;

  final int ageYears;

  /// YYYYMMDD 整数。
  final int validFrom;
  final int validUntil;

  /// true=NORMAL,false=REVOKED。
  final bool statusNormal;

  final String provinceCode;
  final String cityCode;
  final String townCode;
  final String? birthProvinceCode;
  final String? birthCityCode;
  final String? birthTownCode;
  final String? citizenFullName;
  final String? citizenSexLabel;

  /// 出生日期(YYYYMMDD 整数),仅竞选身份携带。
  final int? birthDate;

  bool get isCandidate =>
      identityLevel == CitizenIdentityConsentLevel.candidate;

  /// 解码 SCALE 公民身份载荷,必须恰好消费完全部字节。
  ///
  /// 任何字段越界、长度非法、年龄不足、日期非法、状态未知都返回 null,
  /// 由调用方按"无法独立验证"拒签。
  static VotingIdentityConsentPayload? decode(Uint8List bytes) {
    return _decodeCandidate(bytes) ?? _decodeVotingRoot(bytes);
  }

  /// 确认页展示条目,字段中文名与 citizenwallet 确认页一致。
  List<(String, String)> get reviewEntries => [
        ('身份类型', isCandidate ? '参选身份' : '投票身份'),
        ('CID编号', cidNumber),
        ('公民钱包账户', walletAddress),
        ('周岁年龄', '$ageYears周岁'),
        (
          '护照有效期',
          '${_formatDateInt(validFrom)} 至 ${_formatDateInt(validUntil)}'
        ),
        ('身份状态', statusNormal ? '正常' : '注销'),
        ('居住地', '$provinceCode / $cityCode / $townCode'),
        if (isCandidate) ...[
          (
            '出生地',
            '$birthProvinceCode / $birthCityCode / $birthTownCode',
          ),
          ('出生日期', birthDate == null ? '' : _formatDateInt(birthDate!)),
          ('公民姓名', citizenFullName ?? ''),
          ('公民性别', citizenSexLabel ?? ''),
        ],
      ];

  static VotingIdentityConsentPayload? _decodeVotingRoot(Uint8List bytes) {
    final decoded = _readVotingIdentityPayload(bytes, 0);
    if (decoded == null || decoded.next != bytes.length) return null;
    return decoded.payload;
  }

  static VotingIdentityConsentPayload? _decodeCandidate(Uint8List bytes) {
    final voting = _readVotingIdentityPayload(bytes, 0);
    if (voting == null) return null;
    var offset = voting.next;

    final (birthProvinceCode, afterBirthProvince) =
        _readUtf8Vec(bytes, offset, maxLen: 16);
    if (birthProvinceCode == null) return null;
    offset = afterBirthProvince;
    final (birthCityCode, afterBirthCity) =
        _readUtf8Vec(bytes, offset, maxLen: 16);
    if (birthCityCode == null) return null;
    offset = afterBirthCity;
    final (birthTownCode, afterBirthTown) =
        _readUtf8Vec(bytes, offset, maxLen: 16);
    if (birthTownCode == null) return null;
    offset = afterBirthTown;
    final (citizenFullName, afterFullName) =
        _readUtf8Vec(bytes, offset, maxLen: 128);
    if (citizenFullName == null) return null;
    offset = afterFullName;
    if (offset >= bytes.length) return null;
    final sex = bytes[offset];
    offset += 1;
    final sexLabel = switch (sex) {
      0 => '男',
      1 => '女',
      _ => null,
    };
    if (sexLabel == null) return null;

    // birth_date: u32 YYYYMMDD(LE),CandidateIdentityPayload 末字段。
    if (offset + 4 > bytes.length) return null;
    final birthDate = _readU32Le(bytes, offset);
    offset += 4;
    if (!_isValidDateInt(birthDate) || offset != bytes.length) return null;

    final base = voting.payload;
    return VotingIdentityConsentPayload(
      identityLevel: CitizenIdentityConsentLevel.candidate,
      cidNumber: base.cidNumber,
      walletPubkeyHex: base.walletPubkeyHex,
      walletAddress: base.walletAddress,
      ageYears: base.ageYears,
      validFrom: base.validFrom,
      validUntil: base.validUntil,
      statusNormal: base.statusNormal,
      provinceCode: base.provinceCode,
      cityCode: base.cityCode,
      townCode: base.townCode,
      birthProvinceCode: birthProvinceCode,
      birthCityCode: birthCityCode,
      birthTownCode: birthTownCode,
      citizenFullName: citizenFullName,
      citizenSexLabel: sexLabel,
      birthDate: birthDate,
    );
  }

  static ({
    VotingIdentityConsentPayload payload,
    int next,
  })? _readVotingIdentityPayload(Uint8List bytes, int offset) {
    final (cidNumber, afterCid) = _readUtf8Vec(bytes, offset, maxLen: 32);
    if (cidNumber == null) return null;
    offset = afterCid;
    if (offset + 32 + 1 + 4 + 4 + 1 > bytes.length) return null;

    final walletBytes = bytes.sublist(offset, offset + 32);
    offset += 32;

    final age = bytes[offset];
    offset += 1;
    if (age < kMinOnchainCitizenAgeYears) return null;

    final validFrom = _readU32Le(bytes, offset);
    offset += 4;
    final validUntil = _readU32Le(bytes, offset);
    offset += 4;
    if (!_isValidDateInt(validFrom) || !_isValidDateInt(validUntil)) {
      return null;
    }
    if (validUntil < validFrom) return null;

    final status = bytes[offset];
    offset += 1;
    if (status > 1) return null;

    final (provinceCode, afterProvince) =
        _readUtf8Vec(bytes, offset, maxLen: 16);
    if (provinceCode == null) return null;
    offset = afterProvince;
    final (cityCode, afterCity) = _readUtf8Vec(bytes, offset, maxLen: 16);
    if (cityCode == null) return null;
    offset = afterCity;
    final (townCode, afterTown) = _readUtf8Vec(bytes, offset, maxLen: 16);
    if (townCode == null) return null;
    offset = afterTown;

    return (
      payload: VotingIdentityConsentPayload(
        identityLevel: CitizenIdentityConsentLevel.voting,
        cidNumber: cidNumber,
        walletPubkeyHex: _bytesToLowerHex(walletBytes),
        walletAddress:
            Keyring().encodeAddress(walletBytes.toList(), _ss58Prefix),
        ageYears: age,
        validFrom: validFrom,
        validUntil: validUntil,
        statusNormal: status == 0,
        provinceCode: provinceCode,
        cityCode: cityCode,
        townCode: townCode,
      ),
      next: offset,
    );
  }

  static (String?, int) _readUtf8Vec(
    Uint8List bytes,
    int offset, {
    required int maxLen,
  }) {
    if (offset >= bytes.length) return (null, offset);
    final (len, lenSize) = _decodeCompactU32(bytes, offset);
    if (lenSize == 0) return (null, offset);
    offset += lenSize;
    if (len <= 0 || len > maxLen || offset + len > bytes.length) {
      return (null, offset);
    }
    final text = utf8.decode(
      bytes.sublist(offset, offset + len),
      allowMalformed: false,
    );
    if (text.trim().isEmpty) return (null, offset);
    return (text, offset + len);
  }

  /// 解码 SCALE Compact<u32>,返回 (值, 消耗字节数);big-int 模式不会出现在
  /// 本载荷的长度前缀里,按非法处理。
  static (int, int) _decodeCompactU32(Uint8List bytes, int offset) {
    if (offset >= bytes.length) return (0, 0);
    final first = bytes[offset];
    switch (first & 0x03) {
      case 0:
        return (first >> 2, 1);
      case 1:
        if (offset + 2 > bytes.length) return (0, 0);
        return ((first | (bytes[offset + 1] << 8)) >> 2, 2);
      case 2:
        if (offset + 4 > bytes.length) return (0, 0);
        final value = first |
            (bytes[offset + 1] << 8) |
            (bytes[offset + 2] << 16) |
            (bytes[offset + 3] << 24);
        return (value >> 2, 4);
      default:
        return (0, 0);
    }
  }

  static int _readU32Le(Uint8List bytes, int offset) {
    return bytes[offset] |
        (bytes[offset + 1] << 8) |
        (bytes[offset + 2] << 16) |
        (bytes[offset + 3] << 24);
  }

  static bool _isValidDateInt(int value) {
    final year = value ~/ 10000;
    final month = (value ~/ 100) % 100;
    final day = value % 100;
    return year >= 1900 &&
        year <= 9999 &&
        month >= 1 &&
        month <= 12 &&
        day >= 1 &&
        day <= 31;
  }

  static String _formatDateInt(int value) {
    final year = value ~/ 10000;
    final month = (value ~/ 100) % 100;
    final day = value % 100;
    return '${year.toString().padLeft(4, '0')}-'
        '${month.toString().padLeft(2, '0')}-'
        '${day.toString().padLeft(2, '0')}';
  }

  static String _bytesToLowerHex(Uint8List bytes) {
    return '0x${bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
  }
}
