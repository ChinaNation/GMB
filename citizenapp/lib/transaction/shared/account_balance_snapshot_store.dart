import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:citizenapp/citizen/shared/account_derivation.dart';
import 'package:citizenapp/isar/app_isar.dart';

/// 账户余额展示快照。
///
/// 该快照只用于页面展示，提交交易前的余额校验必须重新读链，
/// 不能使用这里的缓存值。
class AccountBalanceSnapshotStore {
  AccountBalanceSnapshotStore._();

  static final AccountBalanceSnapshotStore instance =
      AccountBalanceSnapshotStore._();

  static const Duration displayTtl = Duration(seconds: 45);
  static const String _prefix = 'chain.account.balance.';

  Future<AccountBalanceSnapshot?> read(String accountId) {
    return WalletIsar.instance.read((isar) async {
      final entity = await isar.appKvEntitys.getByKey(key(accountId));
      return AccountBalanceSnapshot.fromJsonString(entity?.stringValue);
    });
  }

  Future<AccountBalanceSnapshot?> readFresh(
    String accountId, {
    Duration ttl = displayTtl,
  }) async {
    final snapshot = await read(accountId);
    if (snapshot == null || !snapshot.isFresh(ttl)) return null;
    return snapshot;
  }

  Future<void> put({
    required String accountId,
    required double balanceYuan,
    String source = 'chain',
  }) async {
    final now = DateTime.now().millisecondsSinceEpoch;
    final snapshot = AccountBalanceSnapshot(
      accountId: _requireAccountId(accountId),
      balanceYuan: balanceYuan,
      updatedAtMillis: now,
      source: source,
    );
    await WalletIsar.instance.writeTxn((isar) async {
      final entity =
          await isar.appKvEntitys.getByKey(key(accountId)) ?? AppKvEntity();
      entity
        ..key = key(accountId)
        ..stringValue = jsonEncode(snapshot.toJson())
        ..intValue = now
        ..boolValue = null;
      await isar.appKvEntitys.putByKey(entity);
    });
  }

  Future<double?> fetchForDisplay({
    required String accountId,
    required Future<double> Function() fetchChainBalance,
    Duration ttl = displayTtl,
  }) async {
    final local = await readFresh(accountId, ttl: ttl);
    if (local != null) return local.balanceYuan;
    final balance = await fetchChainBalance();
    await put(accountId: accountId, balanceYuan: balance);
    return balance;
  }

  static String key(String accountId) =>
      '$_prefix${_requireAccountId(accountId)}';

  static String _requireAccountId(String accountId) {
    if (!isAccountIdText(accountId)) {
      throw const FormatException('account_id 必须为小写 0x + 64 位十六进制');
    }
    return accountId;
  }
}

class AccountBalanceSnapshot {
  const AccountBalanceSnapshot({
    required this.accountId,
    required this.balanceYuan,
    required this.updatedAtMillis,
    required this.source,
  });

  final String accountId;
  final double balanceYuan;
  final int updatedAtMillis;
  final String source;

  bool isFresh(Duration ttl) {
    return DateTime.now().millisecondsSinceEpoch - updatedAtMillis <
        ttl.inMilliseconds;
  }

  Map<String, Object?> toJson() => {
        'account_id': accountId,
        'balance_yuan': balanceYuan,
        'updated_at_millis': updatedAtMillis,
        'source': source,
      };

  static AccountBalanceSnapshot? fromJsonString(String? raw) {
    if (raw == null || raw.isEmpty) return null;
    try {
      final decoded = jsonDecode(raw);
      if (decoded is! Map<String, dynamic>) return null;
      final accountId = decoded['account_id']?.toString();
      final balance = _toDouble(decoded['balance_yuan']);
      final updatedAtMillis = _toInt(decoded['updated_at_millis']);
      final source = decoded['source']?.toString() ?? 'chain';
      if (accountId == null ||
          !isAccountIdText(accountId) ||
          balance == null ||
          updatedAtMillis == null) {
        return null;
      }
      return AccountBalanceSnapshot(
        accountId: accountId,
        balanceYuan: balance,
        updatedAtMillis: updatedAtMillis,
        source: source,
      );
    } catch (e) {
      // 缓存解析失败按"无快照"降级，但要留痕以区分"缓存损坏"与"缓存不存在"。
      debugPrint('[BalanceSnapshot] 快照 JSON 解析失败: $e');
      return null;
    }
  }

  static int? _toInt(Object? value) {
    if (value == null) return null;
    if (value is int) return value;
    return int.tryParse(value.toString());
  }

  static double? _toDouble(Object? value) {
    if (value == null) return null;
    if (value is double) return value;
    if (value is int) return value.toDouble();
    return double.tryParse(value.toString());
  }
}
