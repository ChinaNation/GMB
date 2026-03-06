import 'dart:convert';

import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/wallet/capabilities/api_client.dart';

class WalletTypeService {
  WalletTypeService({ApiClient? apiClient})
      : _apiClient = apiClient ?? ApiClient();

  static const String defaultType = '手机钱包';
  static const String _kRoleMap = 'wallet.admin_catalog.role_map';
  static const String _kRoleMapUpdatedAt = 'wallet.admin_catalog.updated_at';
  static const int _catalogTtlSeconds = 300;

  final ApiClient _apiClient;
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

    final catalog = await _apiClient.fetchAdminCatalog();
    final next = <String, String>{};
    for (final entry in catalog.entries) {
      final normalized = _normalizePubkeyHex(entry.pubkeyHex);
      if (normalized == null) {
        continue;
      }
      next[normalized] = entry.roleName.trim();
    }

    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_kRoleMap, jsonEncode(next));
    await prefs.setInt(_kRoleMapUpdatedAt, now);
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
      // Keep local cache/fallback when backend or chain is unavailable.
    }
  }

  Future<Map<String, String>> _loadRoleMap() async {
    final inMemory = _memoryRoleMap;
    if (inMemory != null) {
      return inMemory;
    }

    final prefs = await SharedPreferences.getInstance();
    final raw = prefs.getString(_kRoleMap);
    if (raw == null || raw.isEmpty) {
      _memoryRoleMap = <String, String>{};
      return _memoryRoleMap!;
    }
    final decoded = jsonDecode(raw);
    if (decoded is! Map) {
      _memoryRoleMap = <String, String>{};
      return _memoryRoleMap!;
    }
    final out = <String, String>{};
    for (final entry in decoded.entries) {
      final key = _normalizePubkeyHex(entry.key.toString());
      final value = entry.value?.toString().trim() ?? '';
      if (key == null || value.isEmpty) {
        continue;
      }
      out[key] = value;
    }
    _memoryRoleMap = out;
    return out;
  }

  Future<int?> _loadUpdatedAt() async {
    final inMemory = _memoryUpdatedAt;
    if (inMemory != null) {
      return inMemory;
    }
    final prefs = await SharedPreferences.getInstance();
    final value = prefs.getInt(_kRoleMapUpdatedAt);
    _memoryUpdatedAt = value;
    return value;
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
