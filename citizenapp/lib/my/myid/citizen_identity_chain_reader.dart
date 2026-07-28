import 'dart:convert';
import 'dart:typed_data';

import 'package:polkadart/polkadart.dart' show Hasher;

import 'package:citizenapp/citizen/shared/account_derivation.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';

/// 由永久 CID 定位的链上公民身份快照。
///
/// CID 与钱包的双向绑定、CID 登记状态均已在读取阶段闭环校验;调用方不得再把
/// 裸钱包或单向映射当作已注册 CID。
///
/// [votingIdentity] 为 `null` 表示**匿名已注册**:账户自助占了一个 CID 并双向绑定
/// (`CidRegistry` Active),但链上无 `VotingIdentityByCid`(未经注册局线下升级)。
/// 匿名态只暴露 `cidNumber`,不得据此当投票/竞选公民。非空即投票身份已闭环校验。
class CitizenIdentityChainSnapshot {
  const CitizenIdentityChainSnapshot({
    required this.cidNumber,
    required this.accountId,
    required this.votingIdentity,
    this.candidateIdentity,
  });

  /// 已注册且绑定闭环的匿名 CID(无投票身份)。
  bool get isAnonymous => votingIdentity == null;

  final String cidNumber;
  final Uint8List accountId;
  final Uint8List? votingIdentity;
  final Uint8List? candidateIdentity;
}

/// `citizen-identity` 永久 CID 存储的统一读取器。
class CitizenIdentityChainReader {
  CitizenIdentityChainReader({ChainRpc? chainRpc})
      : _chainRpc = chainRpc ?? ChainRpc();

  final ChainRpc _chainRpc;

  /// 按规范账户 ID 读取身份闭环,区分**纯访客 / 匿名已注册 / 投票 / 竞选**。
  ///
  /// 顺序固定为:`CidByAccountId` → `CidRegistry` Active → `AccountIdByCid`
  /// 反向一致(**绑定闭环**)→ `VotingIdentityByCid`;竞选再读 `CandidateIdentityByCid`。
  /// - `CidByAccountId` 无、或绑定不闭环(反向错配 / 非 Active)→ 返回 `null`
  ///   (**纯访客**,可自助占号)。
  /// - 绑定闭环但无 `VotingIdentityByCid`(或其布局损坏)→ 返回 `isAnonymous` 快照
  ///   (**匿名已注册**,只带 `cidNumber`)。布局损坏归匿名是 fail-closed 的**安全降级**
  ///   (降到匿名而非误升成投票公民)。
  /// - 绑定闭环 + 合法 `VotingIdentityByCid` → **投票**;再有合法 candidate → **竞选**。
  Future<CitizenIdentityChainSnapshot?> readByAccountId(
    String accountIdText,
  ) async {
    if (!isAccountIdText(accountIdText)) {
      throw const FormatException('account_id 必须是小写 0x 加 64 位十六进制');
    }
    final accountId = Uint8List.fromList([
      for (var i = 2; i < accountIdText.length; i += 2)
        int.parse(accountIdText.substring(i, i + 2), radix: 16),
    ]);

    // 同一次身份判断必须锚定同一个 finalized 区块，避免 CID 映射与身份值跨块混读。
    final finalized = await _chainRpc.fetchFinalizedBlock();
    final finalizedHash = hexEncode(finalized.blockHash);

    final cidByAccountIdKey = storageMapKey(
      'CitizenIdentity',
      'CidByAccountId',
      accountId,
    );
    final cidRaw = await _chainRpc.fetchStorageAtBlock(
      hexEncode(cidByAccountIdKey),
      finalizedHash,
    );
    final cidNumber = decodeCidNumber(cidRaw);
    if (cidNumber == null) return null;

    final cidScale = encodeBoundedBytes(utf8.encode(cidNumber));
    final accountIdByCidKey = storageMapKey(
      'CitizenIdentity',
      'AccountIdByCid',
      cidScale,
    );
    final cidRegistryKey = storageMapKey(
      'CitizenIdentity',
      'CidRegistry',
      cidScale,
    );
    final votingKey = storageMapKey(
      'CitizenIdentity',
      'VotingIdentityByCid',
      cidScale,
    );
    final candidateKey = storageMapKey(
      'CitizenIdentity',
      'CandidateIdentityByCid',
      cidScale,
    );
    final keys = <String>[
      hexEncode(accountIdByCidKey),
      hexEncode(cidRegistryKey),
      hexEncode(votingKey),
      hexEncode(candidateKey),
    ];
    final rows = await Future.wait(
      keys.map((key) => _chainRpc.fetchStorageAtBlock(key, finalizedHash)),
    );
    final boundAccountId = rows[0];
    final cidRecord = rows[1];
    final votingIdentity = rows[2];
    final candidateIdentity = rows[3];
    // 绑定闭环:AccountIdByCid 反向一致 + CidRegistry Active。不闭环(反查为空/错配/
    // 非 Active)= 该账户没有有效 CID → 纯访客兜底(可重新占号,链上残留绑定占号时再拒)。
    if (boundAccountId == null ||
        boundAccountId.length != accountId.length ||
        !_sameBytes(boundAccountId, accountId) ||
        !cidRecordIsActive(cidRecord)) {
      return null;
    }

    // CID 闭环成立。VotingIdentity 键不存在或布局损坏 → 匿名已注册(安全降级,只暴露 CID)。
    if (votingIdentity == null ||
        !votingIdentityLayoutIsValid(votingIdentity)) {
      return CitizenIdentityChainSnapshot(
        cidNumber: cidNumber,
        accountId: accountId,
        votingIdentity: null,
        candidateIdentity: null,
      );
    }

    // 合法投票身份;竞选身份布局非法时降级为纯投票(不误升竞选)。
    return CitizenIdentityChainSnapshot(
      cidNumber: cidNumber,
      accountId: accountId,
      votingIdentity: votingIdentity,
      candidateIdentity: candidateIdentity != null &&
              candidateIdentityLayoutIsValid(candidateIdentity)
          ? candidateIdentity
          : null,
    );
  }

  static String hexEncode(List<int> bytes) =>
      '0x${bytes.map((byte) => byte.toRadixString(16).padLeft(2, '0')).join()}';

  static Uint8List storageMapKey(
    String palletName,
    String storageName,
    Uint8List keyData,
  ) {
    final palletHash = Hasher.twoxx128.hashString(palletName);
    final storageHash = Hasher.twoxx128.hashString(storageName);
    final keyHash = Hasher.blake2b128.hash(keyData);
    return Uint8List.fromList([
      ...palletHash,
      ...storageHash,
      ...keyHash,
      ...keyData,
    ]);
  }

  static Uint8List encodeBoundedBytes(List<int> value) {
    if (value.isEmpty || value.length > 32) {
      throw const FormatException('CID 长度不合法');
    }
    if (value.length >= 64) {
      throw const FormatException('CID 超出单字节 Compact 长度范围');
    }
    return Uint8List.fromList([value.length << 2, ...value]);
  }

  static String? decodeCidNumber(Uint8List? data) {
    if (data == null) return null;
    try {
      final value = _readBoundedBytes(data, 0, 32);
      if (value.nextOffset != data.length) return null;
      final cid = utf8.decode(value.bytes, allowMalformed: false).trim();
      return cid.isEmpty ? null : cid;
    } catch (_) {
      return null;
    }
  }

  /// 解码 `CidRecord` 到 status 字段；只接受 `Active = 0`。
  static bool cidRecordIsActive(Uint8List? data) {
    if (data == null) return false;
    try {
      var offset = _readBoundedBytes(data, 0, 32).nextOffset;
      offset += 32; // commitment
      if (offset > data.length) return false;
      offset = _readBoundedBytes(data, offset, 16).nextOffset;
      offset = _readBoundedBytes(data, offset, 16).nextOffset;
      if (offset + 1 + 4 + 1 > data.length || data[offset] != 0) {
        return false;
      }
      offset += 1 + 4;
      // Active 记录必须没有撤销块号；状态与 revoked_at 自相矛盾时 fail-closed。
      return data[offset] == 0 && offset + 1 == data.length;
    } catch (_) {
      return false;
    }
  }

  /// 校验 `VotingIdentity<BlockNumber>` 的最终 SCALE 布局，不接受截断或尾随字节。
  static bool votingIdentityLayoutIsValid(Uint8List data) {
    try {
      if (data.length < 9) return false;
      final validFrom = _readU32Le(data, 0);
      final validUntil = _readU32Le(data, 4);
      if (!_isValidDateInt(validFrom) || !_isValidDateInt(validUntil)) {
        return false;
      }
      if (data[8] != 0 && data[8] != 1) return false;
      var offset = 9;
      offset = _readBoundedBytes(
        data,
        offset,
        16,
        allowEmpty: true,
      ).nextOffset;
      offset = _readBoundedBytes(
        data,
        offset,
        16,
        allowEmpty: true,
      ).nextOffset;
      offset = _readBoundedBytes(
        data,
        offset,
        16,
        allowEmpty: true,
      ).nextOffset;
      return offset + 4 == data.length;
    } catch (_) {
      return false;
    }
  }

  /// 校验 `CandidateIdentity<BlockNumber>` 的最终 SCALE 布局。
  static bool candidateIdentityLayoutIsValid(Uint8List data) {
    try {
      var offset = 0;
      for (var index = 0; index < 3; index++) {
        offset = _readBoundedBytes(
          data,
          offset,
          16,
          allowEmpty: true,
        ).nextOffset;
      }
      final familyName = _readBoundedBytes(data, offset, 128);
      offset = familyName.nextOffset;
      final givenName = _readBoundedBytes(data, offset, 128);
      offset = givenName.nextOffset;
      if (offset + 1 + 4 + 4 != data.length) return false;
      if (data[offset] != 0 && data[offset] != 1) return false;
      final birthDate = _readU32Le(data, offset + 1);
      return _isValidDateInt(birthDate);
    } catch (_) {
      return false;
    }
  }

  static ({Uint8List bytes, int nextOffset}) _readBoundedBytes(
    Uint8List data,
    int offset,
    int maxLength, {
    bool allowEmpty = false,
  }) {
    if (offset >= data.length) throw const FormatException('Compact 越界');
    final first = data[offset];
    if ((first & 0x03) != 0) {
      throw const FormatException('当前身份键只允许短 Compact 长度');
    }
    final length = first >> 2;
    final start = offset + 1;
    final end = start + length;
    if ((!allowEmpty && length == 0) ||
        length > maxLength ||
        end > data.length) {
      throw const FormatException('BoundedVec 长度不合法');
    }
    return (bytes: Uint8List.sublistView(data, start, end), nextOffset: end);
  }

  static int _readU32Le(Uint8List data, int offset) =>
      data[offset] |
      (data[offset + 1] << 8) |
      (data[offset + 2] << 16) |
      (data[offset + 3] << 24);

  static bool _isValidDateInt(int value) {
    final year = value ~/ 10000;
    final month = (value % 10000) ~/ 100;
    final day = value % 100;
    if (year < 1900 || month < 1 || month > 12 || day < 1 || day > 31) {
      return false;
    }
    final date = DateTime.utc(year, month, day);
    return date.year == year && date.month == month && date.day == day;
  }

  static bool _sameBytes(Uint8List left, Uint8List right) {
    if (left.length != right.length) return false;
    for (var index = 0; index < left.length; index++) {
      if (left[index] != right[index]) return false;
    }
    return true;
  }
}
