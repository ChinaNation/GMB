import 'package:flutter/foundation.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/wallet/capabilities/api_client.dart';

/// 投票账户绑定状态（与后端 /api/v1/app/vote-account/status 对齐）。
enum SfidBindStatus { unset, pending, bound }

class SfidBindState {
  const SfidBindState({
    required this.status,
    this.walletAddress,
    this.walletPubkeyHex,
    this.isColdWallet = false,
    this.updatedAtMillis,
  });

  final SfidBindStatus status;
  final String? walletAddress;
  final String? walletPubkeyHex;
  final bool isColdWallet;
  final int? updatedAtMillis;
}

class SfidBindingService {
  final ApiClient _apiClient = ApiClient();

  static const _kStatus = 'sfid.bind.status';
  static const _kAddress = 'sfid.bind.address';
  static const _kPubkeyHex = 'sfid.bind.pubkey_hex';
  static const _kIsColdWallet = 'sfid.bind.is_cold_wallet';
  static const _kUpdatedAt = 'sfid.bind.updated_at';

  Future<SfidBindState> getState() async {
    final prefs = await SharedPreferences.getInstance();
    final rawStatus = prefs.getString(_kStatus) ?? 'unset';
    final status = switch (rawStatus) {
      'pending' => SfidBindStatus.pending,
      'bound' => SfidBindStatus.bound,
      _ => SfidBindStatus.unset,
    };
    return SfidBindState(
      status: status,
      walletAddress: prefs.getString(_kAddress),
      walletPubkeyHex: prefs.getString(_kPubkeyHex),
      isColdWallet: prefs.getBool(_kIsColdWallet) ?? false,
      updatedAtMillis: prefs.getInt(_kUpdatedAt),
    );
  }

  /// 注册投票账户（带签名证明私钥所有权）。
  ///
  /// 调用后端 POST /api/v1/app/vote-account/register，成功后本地状态变为 pending。
  Future<SfidBindState> registerVoteAccount({
    required String walletAddress,
    required String walletPubkeyHex,
    required bool isColdWallet,
    required String signatureHex,
    required String signMessage,
  }) async {
    await _apiClient.registerVoteAccount(
      address: walletAddress,
      pubkeyHex: walletPubkeyHex,
      signatureHex: signatureHex,
      signMessage: signMessage,
    );
    final now = DateTime.now().millisecondsSinceEpoch;
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_kStatus, 'pending');
    await prefs.setString(_kAddress, walletAddress);
    await prefs.setString(_kPubkeyHex, walletPubkeyHex.trim());
    await prefs.setBool(_kIsColdWallet, isColdWallet);
    await prefs.setInt(_kUpdatedAt, now);
    debugPrint('vote account registered: address=$walletAddress');
    return getState();
  }

  /// 从后端同步投票账户状态。
  ///
  /// 在 initState / onResume 时调用，静默更新本地缓存。
  Future<SfidBindState> syncFromBackend() async {
    final localState = await getState();
    if (localState.walletAddress == null ||
        localState.walletAddress!.isEmpty) {
      return localState;
    }
    try {
      final remote = await _apiClient.queryVoteAccountStatus(
        localState.walletAddress!,
      );
      final prefs = await SharedPreferences.getInstance();
      final now = DateTime.now().millisecondsSinceEpoch;
      switch (remote.status) {
        case 'bound':
          await prefs.setString(_kStatus, 'bound');
        case 'pending':
          await prefs.setString(_kStatus, 'pending');
        default:
          // 后端返回 "unset"：绑定已被解除
          await prefs.setString(_kStatus, 'unset');
          await prefs.remove(_kAddress);
          await prefs.remove(_kPubkeyHex);
          await prefs.remove(_kIsColdWallet);
      }
      await prefs.setInt(_kUpdatedAt, now);
      return getState();
    } catch (e) {
      // 静默失败：网络不可用时不影响本地状态
      debugPrint('syncFromBackend failed: $e');
      return localState;
    }
  }

  /// 清除本地绑定状态。
  Future<SfidBindState> clear() async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove(_kStatus);
    await prefs.remove(_kAddress);
    await prefs.remove(_kPubkeyHex);
    await prefs.remove(_kIsColdWallet);
    await prefs.remove(_kUpdatedAt);
    return getState();
  }
}
