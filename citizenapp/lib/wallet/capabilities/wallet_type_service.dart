import 'package:isar_community/isar.dart';
import 'package:citizenapp/wallet/capabilities/api_client.dart';
import 'package:citizenapp/isar/app_isar.dart';

class WalletLabelService {
  WalletLabelService({ApiClient? apiClient})
      : _apiClient = apiClient ?? ApiClient();

  static const String defaultType = '手机钱包';
  static const int _catalogTtlSeconds = 300;
  static const String _kUpdatedAtKey = 'wallet.admin_catalog.updated_at';

  final ApiClient _apiClient;
  Map<String, String>? _memoryAdminGroupMap;
  int? _memoryUpdatedAt;

  Future<String> resolveWalletLabel(String pubkeyHex) async {
    final normalized = _normalizePubkeyHex(pubkeyHex);
    if (normalized == null) {
      return defaultType;
    }
    await _ensureCatalogFresh();
    final map = await _loadAdminGroupMap();
    return map[normalized] ?? defaultType;
  }

  Future<void> refreshCatalog({bool force = false}) async {
    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    if (!force) {
      final cached = await _loadAdminGroupMap();
      final updatedAt = await _loadUpdatedAt();
      if (cached.isNotEmpty &&
          updatedAt != null &&
          now - updatedAt < _catalogTtlSeconds) {
        return;
      }
    }

    final catalog = await _apiClient.fetchAdminCatalog();
    final next = <String, String>{};
    for (final entry in catalog.entries) {
      final normalized = _normalizePubkeyHex(entry.pubkeyHex);
      if (normalized == null) {
        continue;
      }
      next[normalized] = entry.adminGroupName.trim();
    }

    await WalletIsar.instance.writeTxn((isar) async {
      await isar.adminGroupCacheEntitys.clear();
      if (next.isNotEmpty) {
        final rows = next.entries
            .map(
              (entry) => AdminGroupCacheEntity()
                ..pubkeyHex = entry.key
                ..adminGroupName = entry.value
                ..updatedAt = now,
            )
            .toList(growable: false);
        await isar.adminGroupCacheEntitys.putAll(rows);
      }
      await isar.appKvEntitys.put(
        AppKvEntity()
          ..key = _kUpdatedAtKey
          ..intValue = now,
      );
    });

    _memoryAdminGroupMap = next;
    _memoryUpdatedAt = now;
  }

  Future<void> _ensureCatalogFresh() async {
    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    final updatedAt = await _loadUpdatedAt();
    final adminGroupMap = await _loadAdminGroupMap();
    if (adminGroupMap.isNotEmpty &&
        updatedAt != null &&
        now - updatedAt < _catalogTtlSeconds) {
      return;
    }
    try {
      await refreshCatalog(force: true);
    } catch (_) {
      // Keep local cache/fallback when backend or chain is unavailable.
    }
  }

  Future<Map<String, String>> _loadAdminGroupMap() async {
    final inMemory = _memoryAdminGroupMap;
    if (inMemory != null) {
      return inMemory;
    }

    final rows = await WalletIsar.instance.read((isar) {
      return isar.adminGroupCacheEntitys.where().anyId().findAll();
    });
    final out = <String, String>{};
    for (final row in rows) {
      out[row.pubkeyHex] = row.adminGroupName;
    }
    _memoryAdminGroupMap = out;
    return out;
  }

  Future<int?> _loadUpdatedAt() async {
    final inMemory = _memoryUpdatedAt;
    if (inMemory != null) {
      return inMemory;
    }

    final kv = await WalletIsar.instance.read((isar) {
      return isar.appKvEntitys.where().keyEqualTo(_kUpdatedAtKey).findFirst();
    });
    _memoryUpdatedAt = kv?.intValue;
    return _memoryUpdatedAt;
  }

  String? _normalizePubkeyHex(String input) {
    var v = input.trim().toLowerCase();
    if (v.startsWith('0x')) {
      v = v.substring(2);
    }
    final ok = RegExp(r'^[0-9a-f]{64}$').hasMatch(v);
    if (!ok) {
      return null;
    }
    return v;
  }
}
