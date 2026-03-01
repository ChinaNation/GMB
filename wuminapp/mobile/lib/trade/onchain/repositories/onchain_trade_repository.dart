import 'dart:convert';

import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/trade/onchain/models/onchain_trade_models.dart';

abstract class OnchainTradeRepository {
  Future<void> save(OnchainTxRecord record);

  Future<void> upsert(OnchainTxRecord record);

  Future<List<OnchainTxRecord>> listRecent();
}

class LocalOnchainTradeRepository implements OnchainTradeRepository {
  static const String _kOnchainRecords = 'trade.onchain.records';
  List<OnchainTxRecord>? _cache;

  @override
  Future<void> save(OnchainTxRecord record) async {
    await upsert(record);
  }

  @override
  Future<void> upsert(OnchainTxRecord record) async {
    final records = List<OnchainTxRecord>.from(await _load());
    final index = records.indexWhere((it) => it.txHash == record.txHash);
    if (index >= 0) {
      records[index] = record;
    } else {
      records.insert(0, record);
    }
    records.sort((a, b) => b.createdAt.compareTo(a.createdAt));
    await _save(records);
  }

  @override
  Future<List<OnchainTxRecord>> listRecent() async {
    final records = await _load();
    return List<OnchainTxRecord>.unmodifiable(records);
  }

  Future<List<OnchainTxRecord>> _load() async {
    if (_cache != null) {
      return _cache!;
    }
    final prefs = await SharedPreferences.getInstance();
    final raw = prefs.getString(_kOnchainRecords);
    if (raw == null || raw.isEmpty) {
      _cache = <OnchainTxRecord>[];
      return _cache!;
    }

    final decoded = jsonDecode(raw);
    if (decoded is! List) {
      _cache = <OnchainTxRecord>[];
      return _cache!;
    }

    final records = <OnchainTxRecord>[];
    for (final item in decoded) {
      if (item is Map<String, dynamic>) {
        records.add(OnchainTxRecord.fromJson(item));
      } else if (item is Map) {
        records.add(
          OnchainTxRecord.fromJson(
            item.map((k, v) => MapEntry(k.toString(), v)),
          ),
        );
      }
    }
    records.sort((a, b) => b.createdAt.compareTo(a.createdAt));
    _cache = records;
    return _cache!;
  }

  Future<void> _save(List<OnchainTxRecord> records) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(
      _kOnchainRecords,
      jsonEncode(records.map((it) => it.toJson()).toList(growable: false)),
    );
    _cache = records;
  }
}

class InMemoryOnchainTradeRepository implements OnchainTradeRepository {
  final List<OnchainTxRecord> _records = <OnchainTxRecord>[];

  @override
  Future<void> save(OnchainTxRecord record) async {
    await upsert(record);
  }

  @override
  Future<void> upsert(OnchainTxRecord record) async {
    final index = _records.indexWhere((it) => it.txHash == record.txHash);
    if (index >= 0) {
      _records[index] = record;
      return;
    }
    _records.insert(0, record);
  }

  @override
  Future<List<OnchainTxRecord>> listRecent() async {
    return List<OnchainTxRecord>.unmodifiable(_records);
  }
}
