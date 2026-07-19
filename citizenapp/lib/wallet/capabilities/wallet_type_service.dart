import 'package:isar_community/isar.dart';
import 'package:citizenapp/citizen/shared/admin_accounts_scan_service.dart';
import 'package:citizenapp/citizen/shared/institution_code_label.dart';
import 'package:citizenapp/isar/app_isar.dart';

class WalletLabelService {
  WalletLabelService({AdminAccountsScanService? scanService})
      : _scanService = scanService ??
            AdminAccountsScanService(
              palletNames: const [
                'PublicAdmins',
                'PrivateAdmins',
                'PersonalAdmins',
              ],
            );

  static const String defaultType = '手机钱包';
  static const int _catalogTtlSeconds = 300;
  static const String _kUpdatedAtKey = 'wallet.admin_chain_catalog.updated_at';

  final AdminAccountsScanService _scanService;
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

    final scan = await _scanService.scanAll();
    if (scan.partialFailure) {
      throw StateError('链上管理员目录扫描不完整，拒绝覆盖完整缓存');
    }

    // 同一管理员可能同时属于多个机构。链上扫描完成后按机构码去重并稳定排序，
    // 缓存只保存派生展示标签，不参与任何权限判断。
    final labelsByAdmin = <String, Set<String>>{};
    for (final account in scan.accounts) {
      final label = InstitutionCodeLabel.codeLabel(account.institutionCode);
      for (final admin in account.admins) {
        final normalized = _normalizePubkeyHex(admin.admin_account);
        if (normalized != null) {
          labelsByAdmin.putIfAbsent(normalized, () => <String>{}).add(label);
        }
      }
    }
    final next = <String, String>{};
    for (final entry in labelsByAdmin.entries) {
      final labels = entry.value.toList(growable: false)..sort();
      next[entry.key] = labels.join(' / ');
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
      // 链暂不可用时只保留最近一次链派生缓存；缓存永远不参与权限判断。
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
