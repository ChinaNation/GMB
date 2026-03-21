import 'dart:convert';

import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/qr/login/login_models.dart';

/// 登录请求防重放守卫。
///
/// 基于 `challenge` 做一次性消费，过期条目自动清理。
class LoginReplayGuard {
  static const String _kUsedChallenges = 'login.used_challenges';

  Future<bool> isConsumed(String challenge) async {
    final all = await _load();
    return all.containsKey(challenge);
  }

  Future<void> consume({
    required String challenge,
    required int expiresAt,
  }) async {
    final all = await _load();
    final now = _nowEpochSeconds();
    all.removeWhere((_, exp) => exp < now);
    all[challenge] = expiresAt;
    await _save(all);
  }

  Future<void> assertNotConsumed(String challenge) async {
    final consumed = await isConsumed(challenge);
    if (consumed) {
      throw const LoginException(
        LoginErrorCode.replay,
        '登录挑战已使用，请刷新二维码后重试',
      );
    }
  }

  Future<Map<String, int>> _load() async {
    final prefs = await SharedPreferences.getInstance();
    final raw = prefs.getString(_kUsedChallenges);
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
    await prefs.setString(_kUsedChallenges, jsonEncode(data));
  }

  int _nowEpochSeconds() => DateTime.now().millisecondsSinceEpoch ~/ 1000;
}
