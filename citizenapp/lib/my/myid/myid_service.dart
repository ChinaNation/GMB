import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import 'package:citizenapp/citizen/public/data/admin_division_store.dart';
import 'package:citizenapp/citizen/public/data/area_path_formatter.dart';
import 'package:citizenapp/citizen/public/data/isar_admin_division_store.dart';
import 'package:citizenapp/citizen/public/data/public_provinces.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

import 'identity_badge_snapshot_store.dart';

/// 电子护照身份档。
///
/// 护照身份钥匙**只认默认用户**(`WalletManager.getDefaultWallet()` = 钱包列表中
/// 最靠前的热钱包,与聊天/发动态同源)。默认用户切换即跟随;链上一人一 CID 一账户
/// 一身份,故不再扫全钱包、不再有多身份冲突。
enum MyIdTier {
  /// 默认用户账户链上无投票身份 → 匿名访客，且没有公民身份信息。
  visitor,

  /// 有 `CitizenIdentity::VotingIdentityByAccount`。
  voting,

  /// 在投票公民之上再有 `CitizenIdentity::CandidateIdentityByAccount`(公开姓名/性别/出生地)。
  candidate,
}

/// 护照有效期/生命周期状态(仅公民档有意义;`queryFailed` 为链读失败兜底)。
enum MyIdStatus { normal, notYetValid, expired, revoked, queryFailed }

/// 电子护照只读链上状态(默认用户维度)。
class MyIdState {
  const MyIdState({
    required this.tier,
    this.status,
    this.votingAccount,
    this.cidNumber,
    this.residenceDistrict,
    this.passportValidFrom,
    this.passportValidUntil,
    this.citizenFullName,
    this.citizenSexLabel,
    this.birthDistrict,
    this.citizenBirthDate,
    this.errorMessage,
  });

  final MyIdTier tier;

  /// 公民档的护照状态;访客为 null,链读失败为 [MyIdStatus.queryFailed]。
  final MyIdStatus? status;

  /// 链上投票绑定账户 = 默认用户钱包地址(SS58)。访客不显示,为 null。
  final String? votingAccount;
  final String? cidNumber;

  /// 预 join 的居住选区「省·市·镇」(service 层查字典拼好,UI 直接展示)。
  final String? residenceDistrict;

  /// YYYY-MM-DD,来自链上 YYYYMMDD 整数。
  final String? passportValidFrom;
  final String? passportValidUntil;

  // ── 竞选公民专属公开字段 ──
  final String? citizenFullName;

  /// 男/女。
  final String? citizenSexLabel;
  final String? birthDistrict;

  /// 出生日期 YYYY-MM-DD,来自链上竞选身份 birth_date(YYYYMMDD 整数)。
  final String? citizenBirthDate;

  final String? errorMessage;

  bool get isCitizen => tier == MyIdTier.voting || tier == MyIdTier.candidate;

  /// 徽章分色信号:visitor/voting/candidate,与 [IdentityBadgeSnapshotStore] 契约一致。
  String get identityLevel => switch (tier) {
        MyIdTier.candidate => 'candidate',
        MyIdTier.voting => 'voting',
        MyIdTier.visitor => 'visitor',
      };
}

class MyIdService {
  MyIdService({
    WalletManager? walletManager,
    ChainRpc? chainRpc,
    AdminDivisionStore? divisionStore,
    IdentityBadgeSnapshotStore? badgeSnapshotStore,
    DateTime Function()? nowProvider,
  })  : _walletManager = walletManager ?? WalletManager(),
        _chainRpc = chainRpc ?? ChainRpc(),
        _divisionStore = divisionStore ?? IsarAdminDivisionStore(),
        _badgeSnapshotStore =
            badgeSnapshotStore ?? IdentityBadgeSnapshotStore(),
        _nowProvider = nowProvider ?? _beijingNow;

  final WalletManager _walletManager;
  final ChainRpc _chainRpc;
  final AdminDivisionStore _divisionStore;
  final IdentityBadgeSnapshotStore _badgeSnapshotStore;
  final DateTime Function() _nowProvider;

  /// 链上护照有效期窗口按 UTC+8 判定(与 runtime `can_vote` 口径一致),
  /// 避免本机时区在跨日边界把"今天"算差一天。
  static DateTime _beijingNow() =>
      DateTime.now().toUtc().add(const Duration(hours: 8));

  /// 读取默认用户的电子护照状态。
  ///
  /// 只读默认用户那一个账户的链上 finalized 身份:
  /// 无 `VotingIdentityByAccount` → 访客;有 → 投票公民,再有
  /// `CandidateIdentityByAccount` → 竞选公民。
  Future<MyIdState> getState() async {
    WalletProfile? wallet;
    try {
      wallet = await _walletManager.getDefaultWallet();
    } catch (e) {
      debugPrint('myid default wallet load failed: $e');
      return const MyIdState(
        tier: MyIdTier.visitor,
        status: MyIdStatus.queryFailed,
        errorMessage: '默认用户读取失败',
      );
    }
    if (wallet == null) {
      // 无热钱包 → 无默认用户 → 访客(引导创建钱包)。
      return const MyIdState(tier: MyIdTier.visitor, errorMessage: '请先创建钱包');
    }

    final Uint8List accountId;
    try {
      accountId = Uint8List.fromList(_keyring.decodeAddress(wallet.address));
    } catch (e) {
      debugPrint('myid decode default wallet address failed: $e');
      return const MyIdState(
        tier: MyIdTier.visitor,
        status: MyIdStatus.queryFailed,
        errorMessage: '默认用户地址无效',
      );
    }

    final votingKey =
        '0x${_hexEncode(_storageMapKey('CitizenIdentity', 'VotingIdentityByAccount', accountId))}';
    final candidateKey =
        '0x${_hexEncode(_storageMapKey('CitizenIdentity', 'CandidateIdentityByAccount', accountId))}';

    Map<String, Uint8List?> rows;
    try {
      rows = await _chainRpc.fetchStorageBatch([votingKey, candidateKey]);
    } catch (e) {
      debugPrint('myid chain identity query failed: $e');
      // 链读失败不静默降级访客、不覆盖徽章快照,交由 UI 提示重试。
      return const MyIdState(
        tier: MyIdTier.visitor,
        status: MyIdStatus.queryFailed,
        errorMessage: '链上身份读取失败',
      );
    }

    final votingRaw = rows[votingKey];
    if (votingRaw == null) {
      await _persistBadgeSnapshot(wallet.address, 'visitor');
      return const MyIdState(tier: MyIdTier.visitor);
    }

    final voting = _decodeVotingIdentity(votingRaw);
    if (voting == null) {
      // 有记录但解不开 = 数据异常,不静默当访客。
      return const MyIdState(
        tier: MyIdTier.visitor,
        status: MyIdStatus.queryFailed,
        errorMessage: '身份数据解析失败',
      );
    }

    final status = _deriveStatus(voting);
    final residence = await _resolveDistrict(
      voting.resProvince,
      voting.resCity,
      voting.resTown,
    );

    final candidateRaw = rows[candidateKey];
    final candidate =
        candidateRaw == null ? null : _decodeCandidateIdentity(candidateRaw);
    final tier = candidate != null ? MyIdTier.candidate : MyIdTier.voting;
    await _persistBadgeSnapshot(
      wallet.address,
      tier == MyIdTier.candidate ? 'candidate' : 'voting',
    );

    final birth = candidate == null
        ? null
        : await _resolveDistrict(
            candidate.birthProvince,
            candidate.birthCity,
            candidate.birthTown,
          );

    return MyIdState(
      tier: tier,
      status: status,
      votingAccount: wallet.address,
      cidNumber: voting.cidNumber,
      residenceDistrict: residence,
      passportValidFrom: _formatDateInt(voting.passportValidFrom),
      passportValidUntil: _formatDateInt(voting.passportValidUntil),
      citizenFullName: candidate?.fullName,
      citizenSexLabel:
          candidate == null ? null : (candidate.sex == 1 ? '女' : '男'),
      birthDistrict: birth,
      citizenBirthDate:
          candidate == null ? null : _formatDateInt(candidate.birthDate),
    );
  }

  /// 把三段行政区码预 join 成「省·市·镇」展示串;省码空则返回空串。
  ///
  /// [formatAreaPath] 内部字典缺失会回退显 code(绝不崩、绝不空);再包一层
  /// 兜底防字典异常,避免选区展示阻断整卡。
  Future<String> _resolveDistrict(
    String province,
    String city,
    String town,
  ) async {
    if (province.isEmpty) return '';
    try {
      return await formatAreaPath(
        _divisionStore,
        provinceName: provinceDisplayNameByCode(province),
        provinceCode: province,
        cityCode: city,
        townCode: town,
      );
    } catch (e) {
      debugPrint('myid area path resolve failed: $e');
      return [provinceDisplayNameByCode(province), city, town]
          .where((s) => s.isNotEmpty)
          .join(' · ');
    }
  }

  /// 写默认用户的身份徽章快照,供非链页面(个人页/广场)展示,不作权限依据。
  Future<void> _persistBadgeSnapshot(String walletAccount, String level) async {
    try {
      await _badgeSnapshotStore.write(
        walletAccount: walletAccount,
        identityLevel: level,
      );
    } catch (e) {
      // 快照只服务展示,写失败不改变本次真实链查询结果。
      debugPrint('myid badge snapshot save failed: $e');
    }
  }

  MyIdStatus _deriveStatus(_VotingIdentity identity) {
    if (identity.citizenStatus == _CitizenStatus.revoked) {
      return MyIdStatus.revoked;
    }
    final today = _dateInt(_nowProvider());
    if (today < identity.passportValidFrom) return MyIdStatus.notYetValid;
    if (today > identity.passportValidUntil) return MyIdStatus.expired;
    return MyIdStatus.normal;
  }

  /// 解码链上 `VotingIdentity<BlockNumber>`,字段序与
  /// `citizenchain/runtime/misc/citizen-identity/src/lib.rs` 逐字节一致:
  /// cid_number + valid_from(u32) + valid_until(u32) + status(u8)
  /// + residence_省/市/镇码 + updated_at(u32)。
  _VotingIdentity? _decodeVotingIdentity(Uint8List data) {
    try {
      var offset = 0;
      final cid = _readUtf8Vec(data, offset, maxLen: 32);
      offset = cid.nextOffset;
      if (offset + 4 + 4 + 1 > data.length) return null;
      final validFrom = _readU32Le(data, offset);
      offset += 4;
      final validUntil = _readU32Le(data, offset);
      offset += 4;
      if (!_isValidDateInt(validFrom) || !_isValidDateInt(validUntil)) {
        return null;
      }
      final status = switch (data[offset]) {
        0 => _CitizenStatus.normal,
        1 => _CitizenStatus.revoked,
        _ => null,
      };
      if (status == null) return null;
      offset += 1;
      // 居住 3 码允许空(区划码可能只到市;空段绝不能把整条身份误判为不存在)。
      final prov = _readUtf8VecAllowEmpty(data, offset, maxLen: 16);
      offset = prov.nextOffset;
      final city = _readUtf8VecAllowEmpty(data, offset, maxLen: 16);
      offset = city.nextOffset;
      final town = _readUtf8VecAllowEmpty(data, offset, maxLen: 16);
      offset = town.nextOffset;
      // updated_at(BlockNumber=u32):只校验尾部存在,展示不使用。
      if (offset + 4 > data.length) return null;
      return _VotingIdentity(
        cidNumber: cid.value,
        passportValidFrom: validFrom,
        passportValidUntil: validUntil,
        citizenStatus: status,
        resProvince: prov.value,
        resCity: city.value,
        resTown: town.value,
      );
    } catch (_) {
      return null;
    }
  }

  /// 解码链上 `CandidateIdentity<BlockNumber>`(增量存储,不含 voting 基础字段):
  /// birth_省/市/镇码 + citizen_full_name + citizen_sex(u8,0男1女)
  /// + birth_date(u32 YYYYMMDD) + updated_at(u32)。
  _CandidateIdentity? _decodeCandidateIdentity(Uint8List data) {
    try {
      var offset = 0;
      final prov = _readUtf8VecAllowEmpty(data, offset, maxLen: 16);
      offset = prov.nextOffset;
      final city = _readUtf8VecAllowEmpty(data, offset, maxLen: 16);
      offset = city.nextOffset;
      final town = _readUtf8VecAllowEmpty(data, offset, maxLen: 16);
      offset = town.nextOffset;
      final name = _readUtf8VecAllowEmpty(data, offset, maxLen: 128);
      offset = name.nextOffset;
      if (offset + 1 > data.length) return null;
      final sex = data[offset];
      offset += 1;
      if (sex != 0 && sex != 1) return null;
      // birth_date(u32 YYYYMMDD) + 尾部 updated_at(u32)。
      if (offset + 4 > data.length) return null;
      final birthDate = _readU32Le(data, offset);
      offset += 4;
      if (!_isValidDateInt(birthDate)) return null;
      if (offset + 4 > data.length) return null;
      return _CandidateIdentity(
        birthProvince: prov.value,
        birthCity: city.value,
        birthTown: town.value,
        fullName: name.value,
        sex: sex,
        birthDate: birthDate,
      );
    } catch (_) {
      return null;
    }
  }

  static final Keyring _keyring = Keyring();

  static Uint8List _storageMapKey(
    String palletName,
    String storageName,
    Uint8List keyData,
  ) {
    final palletHash = Hasher.twoxx128.hashString(palletName);
    final storageHash = Hasher.twoxx128.hashString(storageName);
    final keyHash = _blake2128Concat(keyData);
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

  static Uint8List _blake2128Concat(Uint8List data) {
    final hash = Hasher.blake2b128.hash(data);
    final result = Uint8List(hash.length + data.length);
    result.setAll(0, hash);
    result.setAll(hash.length, data);
    return result;
  }

  /// 读 `BoundedVec<u8>`(Compact 长度 + 字节),内容必须非空;空/超长抛异常。
  /// 仅 cid_number 用(空 cid = 无效身份信号)。
  static ({String value, int nextOffset}) _readUtf8Vec(
    Uint8List data,
    int offset, {
    required int maxLen,
  }) {
    final result = _readUtf8VecAllowEmpty(data, offset, maxLen: maxLen);
    if (result.value.trim().isEmpty) {
      throw const FormatException('BoundedVec 内容为空');
    }
    return result;
  }

  /// 读 `BoundedVec<u8>`,允许空(长度 0 返回空串)。区划码/姓名用。
  static ({String value, int nextOffset}) _readUtf8VecAllowEmpty(
    Uint8List data,
    int offset, {
    required int maxLen,
  }) {
    final (length, lengthSize) = _readCompactU32(data, offset);
    final start = offset + lengthSize;
    final end = start + length;
    if (length < 0 || length > maxLen || end > data.length) {
      throw const FormatException('BoundedVec 长度不合法');
    }
    if (length == 0) return (value: '', nextOffset: start);
    return (
      value: utf8.decode(data.sublist(start, end), allowMalformed: false),
      nextOffset: end,
    );
  }

  static (int, int) _readCompactU32(Uint8List data, int offset) {
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

  static int _readU32Le(Uint8List data, int offset) {
    return data[offset] |
        (data[offset + 1] << 8) |
        (data[offset + 2] << 16) |
        (data[offset + 3] << 24);
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

  static int _dateInt(DateTime value) =>
      value.year * 10000 + value.month * 100 + value.day;

  static String _formatDateInt(int value) {
    final year = value ~/ 10000;
    final month = (value ~/ 100) % 100;
    final day = value % 100;
    return '${year.toString().padLeft(4, '0')}-'
        '${month.toString().padLeft(2, '0')}-'
        '${day.toString().padLeft(2, '0')}';
  }

  static String _hexEncode(List<int> bytes) =>
      bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
}

class _VotingIdentity {
  const _VotingIdentity({
    required this.cidNumber,
    required this.passportValidFrom,
    required this.passportValidUntil,
    required this.citizenStatus,
    required this.resProvince,
    required this.resCity,
    required this.resTown,
  });

  final String cidNumber;
  final int passportValidFrom;
  final int passportValidUntil;
  final _CitizenStatus citizenStatus;
  final String resProvince;
  final String resCity;
  final String resTown;
}

class _CandidateIdentity {
  const _CandidateIdentity({
    required this.birthProvince,
    required this.birthCity,
    required this.birthTown,
    required this.fullName,
    required this.sex,
    required this.birthDate,
  });

  final String birthProvince;
  final String birthCity;
  final String birthTown;
  final String fullName;
  final int sex;

  /// 出生日期(YYYYMMDD 整数),竞选身份专属。
  final int birthDate;
}

enum _CitizenStatus { normal, revoked }
