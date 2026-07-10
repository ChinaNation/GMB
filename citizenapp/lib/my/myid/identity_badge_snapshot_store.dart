import 'dart:convert';

import 'package:shared_preferences/shared_preferences.dart';

/// 单个钱包账户的公开链上身份徽章快照。
///
/// 这里只保存 `visitor` / `voting` / `candidate` 展示信号，不保存护照详情、
/// 私钥或签名材料。快照用于非链页面展示，不得作为发布、投票或权限判断依据。
class IdentityBadgeSnapshot {
  const IdentityBadgeSnapshot({
    required this.walletAccount,
    required this.identityLevel,
    required this.updatedAtMillis,
  });

  final String walletAccount;
  final String identityLevel;
  final int updatedAtMillis;
}

/// 按钱包账户隔离的身份徽章持久快照。
class IdentityBadgeSnapshotStore {
  IdentityBadgeSnapshotStore({
    SharedPreferences? preferences,
    DateTime Function()? nowProvider,
  })  : _preferences = preferences,
        _nowProvider = nowProvider ?? DateTime.now;

  static const _schemaVersion = 1;
  static const _keyPrefix = 'identity_badge_snapshot_v1:';
  static const _allowedLevels = {'visitor', 'voting', 'candidate'};

  final SharedPreferences? _preferences;
  final DateTime Function() _nowProvider;

  Future<SharedPreferences> get _prefs {
    final preferences = _preferences;
    if (preferences != null) return Future.value(preferences);
    return SharedPreferences.getInstance();
  }

  Future<IdentityBadgeSnapshot?> read(String walletAccount) async {
    final normalizedAccount = walletAccount.trim();
    if (normalizedAccount.isEmpty) return null;

    final preferences = await _prefs;
    final key = _key(normalizedAccount);
    final raw = preferences.getString(key);
    if (raw == null || raw.isEmpty) return null;

    try {
      final decoded = jsonDecode(raw);
      if (decoded is! Map<String, dynamic> ||
          decoded['schema_version'] != _schemaVersion ||
          decoded['wallet_account'] != normalizedAccount ||
          decoded['identity_level'] is! String ||
          !_allowedLevels.contains(decoded['identity_level']) ||
          decoded['updated_at_millis'] is! int) {
        throw const FormatException('身份徽章快照字段无效');
      }
      return IdentityBadgeSnapshot(
        walletAccount: normalizedAccount,
        identityLevel: decoded['identity_level'] as String,
        updatedAtMillis: decoded['updated_at_millis'] as int,
      );
    } catch (_) {
      // 损坏快照不能冒充链上身份；清除后按无快照展示。
      await preferences.remove(key);
      return null;
    }
  }

  Future<void> write({
    required String walletAccount,
    required String identityLevel,
  }) async {
    final normalizedAccount = walletAccount.trim();
    if (normalizedAccount.isEmpty) {
      throw ArgumentError.value(walletAccount, 'walletAccount', '钱包账户不能为空');
    }
    if (!_allowedLevels.contains(identityLevel)) {
      throw ArgumentError.value(
        identityLevel,
        'identityLevel',
        '身份档必须是 visitor、voting 或 candidate',
      );
    }

    final payload = jsonEncode({
      'schema_version': _schemaVersion,
      'wallet_account': normalizedAccount,
      'identity_level': identityLevel,
      'updated_at_millis': _nowProvider().millisecondsSinceEpoch,
    });
    final preferences = await _prefs;
    await preferences.setString(_key(normalizedAccount), payload);
  }

  Future<void> remove(String walletAccount) async {
    final normalizedAccount = walletAccount.trim();
    if (normalizedAccount.isEmpty) return;
    final preferences = await _prefs;
    await preferences.remove(_key(normalizedAccount));
  }

  String _key(String walletAccount) => '$_keyPrefix$walletAccount';
}
