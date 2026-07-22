import 'dart:convert';
import 'dart:typed_data';

import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import 'package:citizenapp/rpc/chain_rpc.dart';

/// 由永久 CID 定位的链上公民身份快照。
///
/// CID 与钱包的双向绑定、CID 登记状态和投票身份均已在读取阶段闭环校验；
/// 调用方不得再把裸钱包或单向映射当作公民身份。
class CitizenIdentityChainSnapshot {
  const CitizenIdentityChainSnapshot({
    required this.cidNumber,
    required this.walletAccountId,
    required this.votingIdentity,
    this.candidateIdentity,
  });

  final String cidNumber;
  final Uint8List walletAccountId;
  final Uint8List votingIdentity;
  final Uint8List? candidateIdentity;
}

/// `citizen-identity` 永久 CID 存储的统一读取器。
class CitizenIdentityChainReader {
  CitizenIdentityChainReader({ChainRpc? chainRpc})
      : _chainRpc = chainRpc ?? ChainRpc();

  final ChainRpc _chainRpc;

  /// 按钱包读取完整身份闭环。
  ///
  /// 顺序固定为：`CidByWalletAccount` → `CidRegistry` Active →
  /// `WalletAccountByCid` 反向一致 → `VotingIdentityByCid`；竞选身份再从
  /// `CandidateIdentityByCid` 读取。任何缺失、吊销或错配都返回 `null`。
  Future<CitizenIdentityChainSnapshot?> readByWallet(
    String walletAddress,
  ) async {
    final accountId =
        Uint8List.fromList(Keyring().decodeAddress(walletAddress));
    if (accountId.length != 32) return null;

    // 同一次身份判断必须锚定同一个 finalized 区块，避免 CID 映射与身份值跨块混读。
    final finalized = await _chainRpc.fetchFinalizedBlock();
    final finalizedHash = hexEncode(finalized.blockHash);

    final cidByWalletKey = storageMapKey(
      'CitizenIdentity',
      'CidByWalletAccount',
      accountId,
    );
    final cidRaw = await _chainRpc.fetchStorageAtBlock(
      hexEncode(cidByWalletKey),
      finalizedHash,
    );
    final cidNumber = decodeCidNumber(cidRaw);
    if (cidNumber == null) return null;

    final cidScale = encodeBoundedBytes(utf8.encode(cidNumber));
    final walletByCidKey = storageMapKey(
      'CitizenIdentity',
      'WalletAccountByCid',
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
      hexEncode(walletByCidKey),
      hexEncode(cidRegistryKey),
      hexEncode(votingKey),
      hexEncode(candidateKey),
    ];
    final rows = await Future.wait(
      keys.map((key) => _chainRpc.fetchStorageAtBlock(key, finalizedHash)),
    );
    final boundWallet = rows[0];
    final cidRecord = rows[1];
    final votingIdentity = rows[2];
    final candidateIdentity = rows[3];
    if (boundWallet == null ||
        boundWallet.length != accountId.length ||
        !_sameBytes(boundWallet, accountId) ||
        !cidRecordIsActive(cidRecord) ||
        votingIdentity == null ||
        !votingIdentityLayoutIsValid(votingIdentity)) {
      return null;
    }

    return CitizenIdentityChainSnapshot(
      cidNumber: cidNumber,
      walletAccountId: accountId,
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
