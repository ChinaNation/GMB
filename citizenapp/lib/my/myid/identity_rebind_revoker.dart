import 'dart:convert';

import 'package:shared_preferences/shared_preferences.dart';

import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/citizen/shared/account_derivation.dart'
    show isAccountIdText;
import 'package:citizenapp/wallet/core/device_subkey.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 一笔尚未完成云端旧账户清理的换绑授权。
///
/// 旧账户签名是公开授权证明而非私钥，可安全落 SharedPreferences。记录必须在提交换绑
/// extrinsic 前写入、Worker 确认幂等清理完成后才删除，消除 finalized 后进程退出造成的
/// 永久残留窗口。
class PendingRebindCleanup {
  const PendingRebindCleanup({
    required this.cidNumber,
    required this.oldAccountId,
    required this.newAccountId,
    required this.oldAccountSignature,
  });

  final String cidNumber;
  final String oldAccountId;
  final String newAccountId;
  final String oldAccountSignature;

  Map<String, String> toJson() => <String, String>{
        'cid_number': cidNumber,
        'old_account_id': oldAccountId,
        'new_account_id': newAccountId,
        'old_account_signature': oldAccountSignature,
      };

  static PendingRebindCleanup? tryParse(Object? raw) {
    if (raw is! Map<String, dynamic>) return null;
    final cidNumber = raw['cid_number'];
    final oldAccountId = raw['old_account_id'];
    final newAccountId = raw['new_account_id'];
    final oldAccountSignature = raw['old_account_signature'];
    if (cidNumber is! String ||
        utf8.encode(cidNumber).isEmpty ||
        utf8.encode(cidNumber).length > 32 ||
        oldAccountId is! String ||
        !isAccountIdText(oldAccountId) ||
        newAccountId is! String ||
        !isAccountIdText(newAccountId) ||
        oldAccountId == newAccountId ||
        oldAccountSignature is! String ||
        !RegExp(r'^0x[0-9a-f]{128}$').hasMatch(oldAccountSignature)) {
      return null;
    }
    return PendingRebindCleanup(
      cidNumber: cidNumber,
      oldAccountId: oldAccountId,
      newAccountId: newAccountId,
      oldAccountSignature: oldAccountSignature,
    );
  }
}

/// 换绑后由**新身份账户会话**代为吊销旧账户的云端鉴权数据。
///
/// 服务端同时校验新账户当前 CID 绑定和旧账户对 `(cid_number, new_account_id)` 的换绑
/// 授权，故不能借“旧账户已经解绑”越权删除他人账户。客户端持久化授权并持续重试；删除
/// 范围只含旧账户 Chat 密钥包/设备、登录挑战、设备子钥和会话，不碰 CID 数据、实时 DO
/// 或 CID 级设备绑定 nonce。
class IdentityRebindRevoker {
  IdentityRebindRevoker({
    SquareApiClient? apiClient,
    DeviceSubkey? deviceSubkey,
    WalletManager? walletManager,
    SharedPreferences? preferences,
  })  : _api = apiClient ?? SquareApiClient(),
        _deviceSubkey = deviceSubkey ?? DeviceSubkey(),
        _walletManager = walletManager ?? WalletManager(),
        _preferences = preferences;

  final SquareApiClient _api;
  final DeviceSubkey _deviceSubkey;
  final WalletManager _walletManager;
  final SharedPreferences? _preferences;

  static const _pendingKey = 'identity_rebind_cleanup_pending';

  Future<SharedPreferences> get _prefs {
    final preferences = _preferences;
    if (preferences != null) {
      return Future<SharedPreferences>.value(preferences);
    }
    return SharedPreferences.getInstance();
  }

  /// 写入或覆盖同一笔待清理授权。调用方保证发生在换绑 extrinsic 提交之前。
  Future<void> stagePendingCleanup({
    required String cidNumber,
    required String oldAccountId,
    required String newAccountId,
    required String oldAccountSignature,
  }) async {
    final record = PendingRebindCleanup.tryParse(<String, dynamic>{
      'cid_number': cidNumber,
      'old_account_id': oldAccountId,
      'new_account_id': newAccountId,
      'old_account_signature': oldAccountSignature.toLowerCase(),
    });
    if (record == null) {
      throw ArgumentError('换绑待清理授权格式不合法');
    }
    final pending = await readPendingCleanup();
    if (pending != null &&
        (pending.cidNumber != cidNumber ||
            pending.oldAccountId != oldAccountId ||
            pending.newAccountId != newAccountId)) {
      throw StateError('上一次换绑安全清理尚未完成，禁止覆盖待清理授权');
    }
    final prefs = await _prefs;
    await prefs.setString(_pendingKey, jsonEncode(record.toJson()));
  }

  /// 待清理记录；损坏值 fail-closed 抛错并保留原值，禁止把解析失败当作已完成。
  Future<PendingRebindCleanup?> readPendingCleanup() async {
    final prefs = await _prefs;
    final raw = prefs.getString(_pendingKey);
    if (raw == null || raw.isEmpty) return null;
    try {
      final parsed = PendingRebindCleanup.tryParse(jsonDecode(raw));
      if (parsed == null) throw const FormatException('字段不合法');
      return parsed;
    } catch (_) {
      throw StateError('换绑安全清理记录损坏');
    }
  }

  /// 新换绑开始前的串行门禁。同一 CID/旧/新三元组允许重试；不同目标必须先完成旧清理。
  Future<void> ensureCanStartRebind({
    required String cidNumber,
    required String oldAccountId,
    required String newAccountId,
  }) async {
    final raw = (await _prefs).getString(_pendingKey);
    if (raw == null || raw.isEmpty) return;
    final pending = await readPendingCleanup();
    if (pending == null) return;
    if (pending.cidNumber == cidNumber &&
        pending.oldAccountId == oldAccountId &&
        pending.newAccountId == newAccountId) {
      return;
    }
    throw StateError('上一次换绑安全清理尚未完成，暂不能再次换绑');
  }

  /// 当前链上绑定已经是待清理记录的新账户时，用新账户设备子钥静默建会话并代吊销。
  ///
  /// 返回 false 表示链上尚未切到记录的新账户；不会误用旧账户建会话，也不会删除记录。
  Future<bool> retryPendingCleanup({
    required String cidNumber,
    required String currentAccountId,
  }) async {
    final pending = await readPendingCleanup();
    if (pending == null) return false;
    if (pending.cidNumber != cidNumber) {
      throw StateError('待清理授权 CID 与当前身份不一致');
    }
    if (pending.newAccountId != currentAccountId) return false;

    final wallet = await _walletManager.getDefaultWallet();
    if (wallet == null) {
      throw StateError('无热钱包，无法建立新账户会话完成换绑清理');
    }
    final session = await _api.ensureSession(
      accountId: currentAccountId,
      signLoginPayload: (loginMessage) async =>
          '0x${await _deviceSubkey.signRawHex(wallet.walletIndex, loginMessage)}',
    );
    if (session.accountId != currentAccountId ||
        session.cidNumber != pending.cidNumber) {
      throw StateError('新账户会话与待清理换绑授权不一致');
    }
    await _api.revokeRebindOldAccount(
      session: session,
      oldAccountId: pending.oldAccountId,
      oldAccountSignature: pending.oldAccountSignature,
    );

    // 仅在 HTTP 清理确认成功后删除同一记录；若并发写入了别的授权，禁止误清新记录。
    final prefs = await _prefs;
    final latest = await readPendingCleanup();
    if (latest != null &&
        latest.cidNumber == pending.cidNumber &&
        latest.oldAccountId == pending.oldAccountId &&
        latest.newAccountId == pending.newAccountId &&
        latest.oldAccountSignature == pending.oldAccountSignature) {
      await prefs.remove(_pendingKey);
    }
    return true;
  }
}
