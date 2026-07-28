import 'package:shared_preferences/shared_preferences.dart';

import 'package:citizenapp/citizen/shared/account_derivation.dart';

/// 「本地鉴权凭证(设备子钥 / 会话 / 通讯录)当前已同步到的身份账户」持久标记。
///
/// CID 换绑重建采用**对账式**触发(死契约 [[cid-rebind-subkeys-must-auto-migrate]]):
/// 每次成功完成本地重建后把标记更新为该身份账户;冷启动 / 进身份页时,把链上实时
/// 解析出的身份账户与本标记比对,不一致即视为「存在待补齐的迁移」并触发重建。
///
/// 相对旧的「一次性换绑意图」:只要「链上真值 != 本标记」这个事实存在就会被发现并补齐,
/// 无论换绑 Finalized 与本地写入之间是否发生过崩溃、意图有没有精确写中某一步——
/// 消除了「Finalized 后崩溃 → 意图丢失 → 永久卡死」的窗口;且标记只在迁移**完全成功**
/// 后更新,换绑正常失败(链上仍旧账户)时标记与链上一致、不会误触发迁移。
class IdentitySyncedAccountStore {
  IdentitySyncedAccountStore({SharedPreferences? preferences})
      : _preferences = preferences;

  static const _key = 'identity_synced_account_v1';

  final SharedPreferences? _preferences;

  Future<SharedPreferences> get _prefs {
    final preferences = _preferences;
    if (preferences != null) return Future.value(preferences);
    return SharedPreferences.getInstance();
  }

  /// 已同步身份账户;无记录 / 损坏返回 null(视作尚无基线)。
  Future<String?> read() async {
    final prefs = await _prefs;
    final raw = prefs.getString(_key);
    if (raw == null || raw.isEmpty || !isAccountIdText(raw)) return null;
    return raw;
  }

  Future<void> write(String accountId) async {
    if (!isAccountIdText(accountId)) {
      throw ArgumentError.value(
        accountId,
        'accountId',
        'account_id 必须为小写 0x + 64 位十六进制',
      );
    }
    final prefs = await _prefs;
    await prefs.setString(_key, accountId);
  }

  Future<void> clear() async {
    final prefs = await _prefs;
    await prefs.remove(_key);
  }
}
