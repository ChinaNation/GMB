import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:citizenapp/isar/wallet_isar.dart';

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

  Future<AccountBalanceSnapshot?> read(String accountHex) {
    return WalletIsar.instance.read((isar) async {
      final entity = await isar.appKvEntitys.getByKey(key(accountHex));
      return AccountBalanceSnapshot.fromJsonString(entity?.stringValue);
    });
  }

  Future<AccountBalanceSnapshot?> readFresh(
    String accountHex, {
    Duration ttl = displayTtl,
  }) async {
    final snapshot = await read(accountHex);
    if (snapshot == null || !snapshot.isFresh(ttl)) return null;
    return snapshot;
  }

  Future<void> put({
    required String accountHex,
    required double balanceYuan,
    String source = 'chain',
  }) async {
    final now = DateTime.now().millisecondsSinceEpoch;
    final snapshot = AccountBalanceSnapshot(
      accountHex: _normalizeHex(accountHex),
      balanceYuan: balanceYuan,
      updatedAtMillis: now,
      source: source,
    );
    await WalletIsar.instance.writeTxn((isar) async {
      final entity =
          await isar.appKvEntitys.getByKey(key(accountHex)) ?? AppKvEntity();
      entity
        ..key = key(accountHex)
        ..stringValue = jsonEncode(snapshot.toJson())
        ..intValue = now
        ..boolValue = null;
      await isar.appKvEntitys.putByKey(entity);
    });
  }

  Future<double?> fetchForDisplay({
    required String accountHex,
    required Future<double> Function() fetchChainBalance,
    Duration ttl = displayTtl,
  }) async {
    final local = await readFresh(accountHex, ttl: ttl);
    if (local != null) return local.balanceYuan;
    final balance = await fetchChainBalance();
    await put(accountHex: accountHex, balanceYuan: balance);
    return balance;
  }

  static String key(String accountHex) =>
      '$_prefix${_normalizeHex(accountHex)}';

  static String _normalizeHex(String hex) {
    final clean = hex.startsWith('0x') ? hex.substring(2) : hex;
    return clean.toLowerCase();
  }
}

class AccountBalanceSnapshot {
  const AccountBalanceSnapshot({
    required this.accountHex,
    required this.balanceYuan,
    required this.updatedAtMillis,
    required this.source,
  });

  final String accountHex;
  final double balanceYuan;
  final int updatedAtMillis;
  final String source;

  bool isFresh(Duration ttl) {
    return DateTime.now().millisecondsSinceEpoch - updatedAtMillis <
        ttl.inMilliseconds;
  }

  Map<String, Object?> toJson() => {
        'account_hex': accountHex,
        'balance_yuan': balanceYuan,
        'updated_at_millis': updatedAtMillis,
        'source': source,
      };

  static AccountBalanceSnapshot? fromJsonString(String? raw) {
    if (raw == null || raw.isEmpty) return null;
    try {
      final decoded = jsonDecode(raw);
      if (decoded is! Map<String, dynamic>) return null;
      final accountHex = decoded['account_hex']?.toString();
      final balance = _toDouble(decoded['balance_yuan']);
      final updatedAtMillis = _toInt(decoded['updated_at_millis']);
      final source = decoded['source']?.toString() ?? 'chain';
      if (accountHex == null ||
          accountHex.isEmpty ||
          balance == null ||
          updatedAtMillis == null) {
        return null;
      }
      return AccountBalanceSnapshot(
        accountHex: accountHex,
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
