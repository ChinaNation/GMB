import 'package:citizenapp/8964/profile/models/citizen_profile.dart';
import 'package:citizenapp/8964/profile/services/citizen_profile_api.dart';
import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/isar/app_isar.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 钱包名（= 用户昵称）与云端 `display_name` 的同步器。
///
/// **模型（2026-07-23 翻转）：云端为真源，本机为缓存。**
/// 同一助记词可能同时在多台设备上使用；若本机永远赢，A 设备改的名字会被
/// B 设备的旧值反复覆盖。因此本机 `walletName` 降级为缓存，冲突交云端裁决。
///
/// 三条路径：
/// - [onLocalRename]：本机改名后调用 —— 先入待同步队列，再尝试推送到**该钱包
///   自己 accountId** 的 `display_name`（旧实现只推默认钱包，云端根本存不全）。
/// - [syncWalletName]：进钱包页 / 导入后调用 —— 先重放待同步项，再拉云端，
///   云端更新则回写本机。复用通讯录已验证的「拉快照 + 重放待同步」范式。
/// - [resolveRemote]：按 accountId 读云端昵称。
///
/// **不形成回环**：从云端回写本机走 [WalletManager.renameWallet]，不触发推送；
/// 推送只由 UI 的改名入口经 [onLocalRename] 发起。
///
/// 边界：冷钱包没有设备子钥、云端也无其资料，名字保持纯本机，本类对其直接跳过。
///
/// 已知限制：仅 App 侧时语义是「最后到达者赢」。离线久的设备上线后仍可能用旧
/// 编辑覆盖新编辑，须待 Worker 支持 `edited_at` 比较后才成为「最新编辑者赢」。
class NicknamePublisher {
  NicknamePublisher({
    WalletManager? walletManager,
    CitizenProfileApi? api,
    SquareSessionProvider? sessionProvider,
  })  : _wallet = walletManager ?? WalletManager(),
        _api = api ?? CitizenProfileApi(),
        _session = sessionProvider ?? SquareSessionProvider.instance;

  final WalletManager _wallet;
  final CitizenProfileApi _api;
  final SquareSessionProvider _session;

  /// 已同步到的云端 `updated_at`；用于判断云端是否比本机缓存新。
  static String _syncedAtKey(String accountId) =>
      'wallet_name_synced_at:$accountId';

  /// 待推送的本机改名（推送失败时留存，下次同步重放）。
  static String _pendingKey(String accountId) =>
      'wallet_name_pending:$accountId';

  /// 本机改名后调用：入队 → 尝试推送 → 成功则清队并记录云端版本。
  ///
  /// 推送失败（无网 / 未注册子钥）不抛错、不阻塞本机改名，待同步项留在队列里，
  /// 下次 [syncWalletName] 重放。
  Future<void> onLocalRename(WalletProfile wallet, String newName) async {
    final name = newName.trim();
    if (name.isEmpty || !wallet.isHotWallet) return;
    await _writePending(wallet.accountId, name);
    await _flushPending(wallet);
  }

  /// 同步指定钱包的名字：先重放待同步项，再按云端版本回写本机。
  Future<void> syncWalletName(WalletProfile wallet) async {
    if (!wallet.isHotWallet) return;
    await _flushPending(wallet);
    // 仍有待推送项说明本机改动尚未上云，此时绝不能用云端旧值覆盖本机。
    if (await _readPending(wallet.accountId) != null) return;

    final SquareSession? session;
    try {
      session = await _session.ensureSessionFor(wallet);
    } on Exception {
      return;
    }
    final CitizenProfile profile;
    try {
      profile = await _api.fetchProfile(wallet.accountId, session: session);
    } on Exception {
      return;
    }

    final remoteName = profile.displayName.trim();
    if (remoteName.isEmpty) return;
    final syncedAt = await _readInt(_syncedAtKey(wallet.accountId));
    // 只有云端确实更新过才回写，否则本机缓存已是最新，避免无谓写库与列表抖动。
    if (syncedAt != null && profile.updatedAt <= syncedAt) return;
    if (remoteName != wallet.walletName) {
      // 直接改本机，**不经 onLocalRename** —— 否则会把刚拉下来的值再推回去，形成回环。
      await _wallet.renameWallet(wallet.walletIndex, remoteName);
    }
    await _writeInt(_syncedAtKey(wallet.accountId), profile.updatedAt);
  }

  /// 读云端昵称；无资料 / 无网返回 null。
  Future<String?> resolveRemote(
    String accountId, {
    SquareSession? session,
  }) async {
    try {
      final profile = await _api.fetchProfile(accountId, session: session);
      final name = profile.displayName.trim();
      return name.isEmpty ? null : name;
    } on Exception {
      return null;
    }
  }

  /// 尝试把待同步项推上云端；成功则清队并记录云端版本。
  Future<void> _flushPending(WalletProfile wallet) async {
    final pending = await _readPending(wallet.accountId);
    if (pending == null) return;
    try {
      final session = await _session.ensureSessionFor(wallet);
      if (session == null) return;
      final updated = await _api.updateProfile(
        session: session,
        displayName: pending,
      );
      await _clearPending(wallet.accountId);
      await _writeInt(_syncedAtKey(wallet.accountId), updated.updatedAt);
    } on Exception {
      // 保留待同步项，下次进钱包页重放。
    }
  }

  Future<String?> _readPending(String accountId) {
    return WalletIsar.instance.read((isar) async {
      final row = await isar.appKvEntitys.getByKey(_pendingKey(accountId));
      final value = row?.stringValue?.trim();
      return (value == null || value.isEmpty) ? null : value;
    });
  }

  Future<void> _writePending(String accountId, String name) async {
    await WalletIsar.instance.writeTxn((isar) async {
      final key = _pendingKey(accountId);
      final row = await isar.appKvEntitys.getByKey(key) ?? AppKvEntity();
      row
        ..key = key
        ..stringValue = name
        ..intValue = DateTime.now().millisecondsSinceEpoch
        ..boolValue = null;
      await isar.appKvEntitys.putByKey(row);
    });
  }

  Future<void> _clearPending(String accountId) async {
    await WalletIsar.instance.writeTxn((isar) async {
      await isar.appKvEntitys.deleteByKey(_pendingKey(accountId));
    });
  }

  Future<int?> _readInt(String key) {
    return WalletIsar.instance.read((isar) async {
      final row = await isar.appKvEntitys.getByKey(key);
      return row?.intValue;
    });
  }

  Future<void> _writeInt(String key, int value) async {
    await WalletIsar.instance.writeTxn((isar) async {
      final row = await isar.appKvEntitys.getByKey(key) ?? AppKvEntity();
      row
        ..key = key
        ..stringValue = null
        ..intValue = value
        ..boolValue = null;
      await isar.appKvEntitys.putByKey(row);
    });
  }
}
