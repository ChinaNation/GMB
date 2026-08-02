import 'dart:convert';

import 'package:shared_preferences/shared_preferences.dart';

import 'package:citizenapp/8964/profile/models/citizen_profile.dart';

/// 用户主页资料的本地离线缓存。
///
/// 先渲染缓存 → 后台刷新 → 回刷并写回。只缓存成功拉到的真实资料，
/// 兜底默认值不入缓存（避免把空资料当成真数据回读）。
class CitizenProfileCache {
  const CitizenProfileCache();

  // 主页寻址以身份主键 cid_number 为唯一缓存键；不读取按账户寻址的历史形状。
  static const String _keyPrefix = 'square.profile.cache.';

  String _cacheKey(String cidNumber) => '$_keyPrefix$cidNumber';

  Future<CitizenProfile?> read(String cidNumber) async {
    final prefs = await SharedPreferences.getInstance();
    final raw = prefs.getString(_cacheKey(cidNumber));
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
    // 缓存主键 = 身份主键 cid_number；缺失身份（cid 为空）的资料不入缓存，
    // 避免把无主键的兜底资料当真数据回读。
    final cidNumber = profile.cidNumber?.trim();
    if (cidNumber == null || cidNumber.isEmpty) return;
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(
      _cacheKey(cidNumber),
      jsonEncode(profile.toJson()),
    );
  }

  Future<void> clear(String cidNumber) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove(_cacheKey(cidNumber));
  }
}
