import 'dart:convert';

import 'package:shared_preferences/shared_preferences.dart';

import 'package:citizenapp/8964/profile/models/citizen_profile.dart';

/// 用户主页资料的本地离线缓存。
///
/// 先渲染缓存 → 后台刷新 → 回刷并写回。只缓存成功拉到的真实资料，
/// 兜底默认值不入缓存（避免把空资料当成真数据回读）。
class CitizenProfileCache {
  const CitizenProfileCache();

  // v2：主页响应新增 identity_level/membership_level/membership_active 字段，
  // bump 前缀作废旧缓存，避免旧形状读出空（见 feedback-dto-field-rename）。
  static const String _keyPrefix = 'square.profile.cache.v2.';

  String _cacheKey(String ownerAccount) => '$_keyPrefix$ownerAccount';

  Future<CitizenProfile?> read(String ownerAccount) async {
    final prefs = await SharedPreferences.getInstance();
    final raw = prefs.getString(_cacheKey(ownerAccount));
    if (raw == null || raw.trim().isEmpty) {
      return null;
    }
    try {
      final decoded = jsonDecode(raw);
      if (decoded is! Map<String, dynamic>) {
        return null;
      }
      return CitizenProfile.fromJson(decoded);
    } on FormatException {
      return null;
    }
  }

  Future<void> write(CitizenProfile profile) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(
      _cacheKey(profile.ownerAccount),
      jsonEncode(profile.toJson()),
    );
  }

  Future<void> clear(String ownerAccount) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove(_cacheKey(ownerAccount));
  }
}
