import 'dart:convert';

import 'package:shared_preferences/shared_preferences.dart';

/// 单个永久 CID 的公开链上身份徽章快照。
///
/// 这里只保存 `visitor` / `voting` / `candidate` 展示信号，不保存护照详情、
/// 私钥或签名材料。快照用于非链页面展示，不得作为发布、投票或权限判断依据。
class IdentityBadgeSnapshot {
  const IdentityBadgeSnapshot({
    required this.cidNumber,
    required this.identityLevel,
    required this.updatedAtMillis,
  });

  final String cidNumber;
  final String identityLevel;
  final int updatedAtMillis;
}

/// 按永久 CID 隔离的身份徽章持久快照。
///
/// 钱包账户只负责取得最新链上快照；换绑后新账户继续读写同一个 CID 键。
class IdentityBadgeSnapshotStore {
  IdentityBadgeSnapshotStore({
    SharedPreferences? preferences,
    DateTime Function()? nowProvider,
  })  : _preferences = preferences,
        _nowProvider = nowProvider ?? DateTime.now;

  static const _keyPrefix = 'identity_badge_snapshot_by_cid:';
  static const _allowedLevels = {'visitor', 'voting', 'candidate'};

  final SharedPreferences? _preferences;
  final DateTime Function() _nowProvider;

  Future<SharedPreferences> get _prefs {
    final preferences = _preferences;
    if (preferences != null) return Future.value(preferences);
    return SharedPreferences.getInstance();
  }

  Future<IdentityBadgeSnapshot?> read(String cidNumber) async {
    final normalizedCidNumber = cidNumber.trim();
    if (normalizedCidNumber.isEmpty) return null;

    final preferences = await _prefs;
    final key = _key(normalizedCidNumber);
    final raw = preferences.getString(key);
    if (raw == null || raw.isEmpty) return null;

    try {
      final decoded = jsonDecode(raw);
      if (decoded is! Map<String, dynamic> ||
          decoded.length != 3 ||
          decoded['cid_number'] != normalizedCidNumber ||
          decoded['identity_level'] is! String ||
          !_allowedLevels.contains(decoded['identity_level']) ||
          decoded['updated_at_millis'] is! int) {
        throw const FormatException('身份徽章快照字段无效');
      }
      return IdentityBadgeSnapshot(
        cidNumber: normalizedCidNumber,
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
    required String cidNumber,
    required String identityLevel,
  }) async {
    final normalizedCidNumber = cidNumber.trim();
    if (normalizedCidNumber.isEmpty) {
      throw ArgumentError.value(
        cidNumber,
        'cidNumber',
        'cid_number 不能为空',
      );
    }
    if (!_allowedLevels.contains(identityLevel)) {
      throw ArgumentError.value(
        identityLevel,
        'identityLevel',
        '身份档必须是 visitor、voting 或 candidate',
      );
    }

    final payload = jsonEncode({
      'cid_number': normalizedCidNumber,
      'identity_level': identityLevel,
      'updated_at_millis': _nowProvider().millisecondsSinceEpoch,
    });
    final preferences = await _prefs;
    await preferences.setString(_key(normalizedCidNumber), payload);
  }

  Future<void> remove(String cidNumber) async {
    final normalizedCidNumber = cidNumber.trim();
    if (normalizedCidNumber.isEmpty) return;
    final preferences = await _prefs;
    await preferences.remove(_key(normalizedCidNumber));
  }

  String _key(String cidNumber) => '$_keyPrefix$cidNumber';
}
