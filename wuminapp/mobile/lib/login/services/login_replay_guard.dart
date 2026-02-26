import 'dart:convert';

import 'package:shared_preferences/shared_preferences.dart';

class LoginReplayGuard {
  static const String _kUsedRequestIds = 'login.used_request_ids';

  Future<bool> isConsumed(String requestId) async {
    final all = await _load();
    return all.containsKey(requestId);
  }

  Future<void> consume({
    required String requestId,
    required int expiresAt,
  }) async {
    final all = await _load();
    final now = _nowEpochSeconds();
    all.removeWhere((_, exp) => exp < now);
    all[requestId] = expiresAt;
    await _save(all);
  }

  Future<void> assertNotConsumed(String requestId) async {
    final consumed = await isConsumed(requestId);
    if (consumed) {
      throw Exception('登录挑战已使用，请刷新二维码后重试');
    }
  }

  Future<Map<String, int>> _load() async {
    final prefs = await SharedPreferences.getInstance();
    final raw = prefs.getString(_kUsedRequestIds);
    if (raw == null || raw.isEmpty) {
      return {};
    }
    final decoded = jsonDecode(raw);
    if (decoded is! Map) {
      return {};
    }

    final out = <String, int>{};
    for (final entry in decoded.entries) {
      final key = entry.key.toString();
      final value = entry.value;
      if (value is int) {
        out[key] = value;
      } else if (value is String) {
        final parsed = int.tryParse(value);
        if (parsed != null) {
          out[key] = parsed;
        }
      }
    }
    return out;
  }

  Future<void> _save(Map<String, int> data) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_kUsedRequestIds, jsonEncode(data));
  }

  int _nowEpochSeconds() => DateTime.now().millisecondsSinceEpoch ~/ 1000;
}
