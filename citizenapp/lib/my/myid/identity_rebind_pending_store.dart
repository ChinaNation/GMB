import 'dart:convert';

import 'package:shared_preferences/shared_preferences.dart';
import 'package:citizenapp/citizen/shared/account_derivation.dart';

/// 一次「进行中」的 CID 换绑续跑意图(旧身份账户 → 新身份账户)。
///
/// CID 换绑链上 Finalized 后,本地须把设备子钥 / 会话 / 通讯录**自动更换到新账户**
/// (死契约 [[cid-rebind-subkeys-must-auto-migrate]])。本地重建可能因网络抖动中断,
/// 故换绑落链前先持久化本意图;全部重建成功再清除。App 下次进身份页时据此续跑,
/// 直到成功——保证「不允许不更换或失败」。
class IdentityRebindPending {
  const IdentityRebindPending({
    required this.oldAccountId,
    required this.newAccountId,
  });

  final String oldAccountId;
  final String newAccountId;
}

/// 换绑续跑意图的持久存储。同一时刻只可能有一个进行中的换绑(换绑操作的是当前
/// 身份),故用单键覆盖;不保存签名材料或私钥。
class IdentityRebindPendingStore {
  IdentityRebindPendingStore({SharedPreferences? preferences})
      : _preferences = preferences;

  static const _schemaVersion = 1;
  static const _key = 'identity_rebind_pending_v1';

  final SharedPreferences? _preferences;

  Future<SharedPreferences> get _prefs {
    final preferences = _preferences;
    if (preferences != null) return Future.value(preferences);
    return SharedPreferences.getInstance();
  }

  Future<IdentityRebindPending?> read() async {
    final preferences = await _prefs;
    final raw = preferences.getString(_key);
    if (raw == null || raw.isEmpty) return null;
    try {
      final decoded = jsonDecode(raw);
      if (decoded is! Map<String, dynamic> ||
          decoded['schema_version'] != _schemaVersion ||
          decoded['old_account_id'] is! String ||
          decoded['new_account_id'] is! String ||
          !isAccountIdText(decoded['old_account_id'] as String) ||
          !isAccountIdText(decoded['new_account_id'] as String) ||
          decoded['old_account_id'] == decoded['new_account_id']) {
        throw const FormatException('换绑续跑意图字段无效');
      }
      return IdentityRebindPending(
        oldAccountId: decoded['old_account_id'] as String,
        newAccountId: decoded['new_account_id'] as String,
      );
    } catch (_) {
      // 损坏意图不能触发误迁移;清除后视作无进行中换绑。
      await preferences.remove(_key);
      return null;
    }
  }

  Future<void> write({
    required String oldAccountId,
    required String newAccountId,
  }) async {
    if (!isAccountIdText(oldAccountId) || !isAccountIdText(newAccountId)) {
      throw ArgumentError('换绑续跑意图的账户 ID 必须为小写 0x + 64 位十六进制');
    }
    if (oldAccountId == newAccountId) {
      throw ArgumentError('换绑续跑意图的新旧账户不能相同');
    }
    final payload = jsonEncode({
      'schema_version': _schemaVersion,
      'old_account_id': oldAccountId,
      'new_account_id': newAccountId,
    });
    final preferences = await _prefs;
    await preferences.setString(_key, payload);
  }

  Future<void> clear() async {
    final preferences = await _prefs;
    await preferences.remove(_key);
  }
}
