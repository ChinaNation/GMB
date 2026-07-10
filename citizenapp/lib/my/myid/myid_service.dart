import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

import 'identity_badge_snapshot_store.dart';

/// 电子护照只读链上状态。
///
/// 本状态不再表达本机“已登记/待登记”档案流程；公民 App 只承认
/// `CitizenIdentity::VotingIdentityByAccount` 已确认的链上身份。
enum MyIdIdentityStatus {
  notOnchain,
  normal,
  notYetValid,
  expired,
  revoked,
  conflict,
  queryFailed,
}

class MyIdState {
  const MyIdState({
    required this.identityStatus,
    this.identityWalletAccount,
    this.identityCidNumber,
    this.passportValidFrom,
    this.passportValidUntil,
    this.identityLevel,
    this.errorMessage,
  });

  final MyIdIdentityStatus identityStatus;

  /// 链上唯一身份绑定的钱包账户，也就是投票账户。
  final String? identityWalletAccount;
  final String? identityCidNumber;

  /// YYYY-MM-DD，直接来自链上 YYYYMMDD 整数的展示格式。
  final String? passportValidFrom;
  final String? passportValidUntil;

  /// 链上身份档（徽章分色）：仅在护照有效(normal)时为 'voting'/'candidate'，否则 null。
  final String? identityLevel;
  final String? errorMessage;

  bool get hasOnchainIdentity =>
      identityWalletAccount != null && identityCidNumber != null;

  bool get isCertified => identityStatus == MyIdIdentityStatus.normal;
}

class MyIdService {
  MyIdService({
    WalletManager? walletManager,
    ChainRpc? chainRpc,
    IdentityBadgeSnapshotStore? badgeSnapshotStore,
    DateTime Function()? nowProvider,
  })  : _walletManager = walletManager ?? WalletManager(),
        _chainRpc = chainRpc ?? ChainRpc(),
        _badgeSnapshotStore =
            badgeSnapshotStore ?? IdentityBadgeSnapshotStore(),
        _nowProvider = nowProvider ?? DateTime.now;

  final WalletManager _walletManager;
  final ChainRpc _chainRpc;
  final IdentityBadgeSnapshotStore _badgeSnapshotStore;
  final DateTime Function() _nowProvider;

  /// 从本机钱包列表中发现唯一链上身份钱包。
  ///
  /// 这里只读取链上 finalized storage。若本机导入了多个不同公民的身份钱包，
  /// UI 按异常处理，不会显示多个认证身份。
  Future<MyIdState> getState() async {
    List<WalletProfile> wallets;
    try {
      wallets = await _walletManager.getWallets();
    } catch (e) {
      debugPrint('myid wallet list load failed: $e');
      return const MyIdState(
        identityStatus: MyIdIdentityStatus.queryFailed,
        errorMessage: '本地钱包读取失败',
      );
    }
    if (wallets.isEmpty) {
      return const MyIdState(identityStatus: MyIdIdentityStatus.notOnchain);
    }

    final keyToWallet = <String, WalletProfile>{};
    for (final wallet in wallets) {
      try {
        final accountId = Uint8List.fromList(_keyring.decodeAddress(
          wallet.address,
        ));
        final key = _storageMapKey(
          'CitizenIdentity',
          'VotingIdentityByAccount',
          accountId,
        );
        keyToWallet['0x${_hexEncode(key)}'] = wallet;
      } catch (e) {
        debugPrint('myid skip invalid wallet address ${wallet.address}: $e');
      }
    }
    if (keyToWallet.isEmpty) {
      return const MyIdState(identityStatus: MyIdIdentityStatus.notOnchain);
    }

    Map<String, Uint8List?> rows;
    try {
      rows = await _chainRpc.fetchStorageBatch(keyToWallet.keys.toList());
    } catch (e) {
      debugPrint('myid chain identity query failed: $e');
      return const MyIdState(
        identityStatus: MyIdIdentityStatus.queryFailed,
        errorMessage: '链上身份读取失败',
      );
    }

    final identities = <_LocatedVotingIdentity>[];
    for (final entry in rows.entries) {
      final raw = entry.value;
      if (raw == null) continue;
      final wallet = keyToWallet[entry.key];
      if (wallet == null) continue;
      final decoded = _decodeVotingIdentity(raw);
      if (decoded == null) continue;
      identities.add(_LocatedVotingIdentity(wallet: wallet, identity: decoded));
    }

    if (identities.isEmpty) {
      await _persistBadgeSnapshots(wallets);
      return const MyIdState(identityStatus: MyIdIdentityStatus.notOnchain);
    }
    if (identities.length > 1) {
      return const MyIdState(
        identityStatus: MyIdIdentityStatus.conflict,
        errorMessage: '本机检测到多个链上身份钱包',
      );
    }

    final located = identities.single;
    final identity = located.identity;
    final status = _deriveStatus(identity);
    // 护照有效才算认证；再读候选表区分投票/竞选（读失败降级为 voting，不阻塞）。
    final identityLevel = status == MyIdIdentityStatus.normal
        ? await _resolveIdentityLevel(located.wallet.address)
        : null;
    await _persistBadgeSnapshots(
      wallets,
      identityWalletAccount: located.wallet.address,
      identityLevel: identityLevel,
    );
    return MyIdState(
      identityStatus: status,
      identityWalletAccount: located.wallet.address,
      identityCidNumber: identity.cidNumber,
      passportValidFrom: _formatDateInt(identity.passportValidFrom),
      passportValidUntil: _formatDateInt(identity.passportValidUntil),
      identityLevel: identityLevel,
    );
  }

  Future<void> _persistBadgeSnapshots(
    List<WalletProfile> wallets, {
    String? identityWalletAccount,
    String? identityLevel,
  }) async {
    try {
      final normalizedIdentityAccount = identityWalletAccount?.trim();
      for (final wallet in wallets) {
        final level = normalizedIdentityAccount != null &&
                normalizedIdentityAccount.isNotEmpty &&
                wallet.address.trim() == normalizedIdentityAccount &&
                (identityLevel == 'voting' || identityLevel == 'candidate')
            ? identityLevel!
            : 'visitor';
        await _badgeSnapshotStore.write(
          walletAccount: wallet.address,
          identityLevel: level,
        );
      }
    } catch (e) {
      // 快照只服务非链页面展示，写失败不能改变本次真实链查询结果。
      debugPrint('myid badge snapshot save failed: $e');
    }
  }

  /// 有候选身份记录=竞选公民，否则投票公民；读失败降级为 voting。
  Future<String> _resolveIdentityLevel(String walletAddress) async {
    try {
      final accountId =
          Uint8List.fromList(_keyring.decodeAddress(walletAddress));
      final candKey = '0x${_hexEncode(_storageMapKey(
        'CitizenIdentity',
        'CandidateIdentityByAccount',
        accountId,
      ))}';
      final rows = await _chainRpc.fetchStorageBatch([candKey]);
      return rows[candKey] != null ? 'candidate' : 'voting';
    } catch (e) {
      debugPrint('myid candidate query failed: $e');
      return 'voting';
    }
  }

  MyIdIdentityStatus _deriveStatus(_VotingIdentity identity) {
    if (identity.citizenStatus == _CitizenStatus.revoked) {
      return MyIdIdentityStatus.revoked;
    }
    final today = _dateInt(_nowProvider());
    if (today < identity.passportValidFrom) {
      return MyIdIdentityStatus.notYetValid;
    }
    if (today > identity.passportValidUntil) {
      return MyIdIdentityStatus.expired;
    }
    return MyIdIdentityStatus.normal;
  }

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
      offset = _readUtf8Vec(data, offset, maxLen: 16).nextOffset;
      offset = _readUtf8Vec(data, offset, maxLen: 16).nextOffset;
      offset = _readUtf8Vec(data, offset, maxLen: 16).nextOffset;
      // BlockNumber 当前为 u32；这里只校验 storage 尾部存在，展示不使用。
      if (offset + 4 > data.length) return null;
      return _VotingIdentity(
        cidNumber: cid.value,
        passportValidFrom: validFrom,
        passportValidUntil: validUntil,
        citizenStatus: status,
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

  static ({String value, int nextOffset}) _readUtf8Vec(
    Uint8List data,
    int offset, {
    required int maxLen,
  }) {
    final (length, lengthSize) = _readCompactU32(data, offset);
    final start = offset + lengthSize;
    final end = start + length;
    if (length <= 0 || length > maxLen || end > data.length) {
      throw const FormatException('BoundedVec 长度不合法');
    }
    final text = utf8.decode(data.sublist(start, end), allowMalformed: false);
    if (text.trim().isEmpty) {
      throw const FormatException('BoundedVec 内容为空');
    }
    return (value: text, nextOffset: end);
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

class _LocatedVotingIdentity {
  const _LocatedVotingIdentity({
    required this.wallet,
    required this.identity,
  });

  final WalletProfile wallet;
  final _VotingIdentity identity;
}

class _VotingIdentity {
  const _VotingIdentity({
    required this.cidNumber,
    required this.passportValidFrom,
    required this.passportValidUntil,
    required this.citizenStatus,
  });

  final String cidNumber;
  final int passportValidFrom;
  final int passportValidUntil;
  final _CitizenStatus citizenStatus;
}

enum _CitizenStatus { normal, revoked }
