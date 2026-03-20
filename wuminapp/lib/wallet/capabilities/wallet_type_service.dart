import 'package:isar/isar.dart';
import 'package:wuminapp_mobile/governance/institution_admin_service.dart';
import 'package:wuminapp_mobile/governance/institution_data.dart';
import 'package:wuminapp_mobile/Isar/wallet_isar.dart';

class WalletTypeService {
  WalletTypeService({InstitutionAdminService? adminService})
      : _adminService = adminService ?? InstitutionAdminService();

  static const String defaultType = '手机钱包';
  static const int _catalogTtlSeconds = 300;
  static const String _kUpdatedAtKey = 'wallet.admin_catalog.updated_at';

  final InstitutionAdminService _adminService;
  Map<String, String>? _memoryRoleMap;
  int? _memoryUpdatedAt;

  Future<String> resolveWalletType(String pubkeyHex) async {
    final normalized = _normalizePubkeyHex(pubkeyHex);
    if (normalized == null) {
      return defaultType;
    }
    await _ensureCatalogFresh();
    final map = await _loadRoleMap();
    return map[normalized] ?? defaultType;
  }

  Future<void> refreshCatalog({bool force = false}) async {
    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    if (!force) {
      final cached = await _loadRoleMap();
      final updatedAt = await _loadUpdatedAt();
      if (cached.isNotEmpty &&
          updatedAt != null &&
          now - updatedAt < _catalogTtlSeconds) {
        return;
      }
    }

    // 从区块链查询所有机构的管理员列表，构建 pubkey → 角色名 映射。
    _adminService.clearCache();
    final next = <String, String>{};
    final allInstitutions = [
      ...kNationalCouncil,
      ...kProvincialCouncils,
      ...kProvincialBanks,
    ];
    for (final inst in allInstitutions) {
      try {
        final admins = await _adminService.fetchAdmins(inst.shenfenId);
        final roleName = '${inst.name}管理员';
        for (final pubkey in admins) {
          final normalized = _normalizePubkeyHex(pubkey);
          if (normalized != null) {
            next[normalized] = roleName;
          }
        }
      } catch (_) {
        // 单个机构查询失败不影响其他机构。
      }
    }

    final isar = await WalletIsar.instance.db();
    await isar.writeTxn(() async {
      await isar.adminRoleCacheEntitys.clear();
      if (next.isNotEmpty) {
        final rows = next.entries
            .map(
              (entry) => AdminRoleCacheEntity()
                ..pubkeyHex = entry.key
                ..roleName = entry.value
                ..updatedAt = now,
            )
            .toList(growable: false);
        await isar.adminRoleCacheEntitys.putAll(rows);
      }
      await isar.appKvEntitys.put(
        AppKvEntity()
          ..key = _kUpdatedAtKey
          ..intValue = now,
      );
    });

    _memoryRoleMap = next;
    _memoryUpdatedAt = now;
  }

  Future<void> _ensureCatalogFresh() async {
    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    final updatedAt = await _loadUpdatedAt();
    final roleMap = await _loadRoleMap();
    if (roleMap.isNotEmpty &&
        updatedAt != null &&
        now - updatedAt < _catalogTtlSeconds) {
      return;
    }
    try {
      await refreshCatalog(force: true);
    } catch (_) {
      // Keep local cache/fallback when chain is unavailable.
    }
  }

  Future<Map<String, String>> _loadRoleMap() async {
    final inMemory = _memoryRoleMap;
    if (inMemory != null) {
      return inMemory;
    }

    final isar = await WalletIsar.instance.db();
    final rows = await isar.adminRoleCacheEntitys.where().anyId().findAll();
    final out = <String, String>{};
    for (final row in rows) {
      out[row.pubkeyHex] = row.roleName;
    }
    _memoryRoleMap = out;
    return out;
  }

  Future<int?> _loadUpdatedAt() async {
    final inMemory = _memoryUpdatedAt;
    if (inMemory != null) {
      return inMemory;
    }

    final isar = await WalletIsar.instance.db();
    final kv =
        await isar.appKvEntitys.where().keyEqualTo(_kUpdatedAtKey).findFirst();
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
