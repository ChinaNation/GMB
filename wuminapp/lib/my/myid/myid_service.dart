import 'package:flutter/foundation.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/my/myid/myid_api.dart';

/// 电子护照绑定状态（与后端状态响应对齐）。
enum MyIdStatus { unset, pending, bound }

class MyIdState {
  const MyIdState({
    required this.status,
    this.walletAddress,
    this.walletPubkeyHex,
    this.sfidCode,
    this.identityStatus,
    this.isColdWallet = false,
    this.updatedAtMillis,
  });

  final MyIdStatus status;
  final String? walletAddress;
  final String? walletPubkeyHex;
  final String? sfidCode;
  final String? identityStatus;
  final bool isColdWallet;
  final int? updatedAtMillis;
}

class MyIdService {
  final MyIdApi _api = MyIdApi();

  // 中文注释：存储 key 暂时沿用旧 sfid.bind.*，避免用户升级后丢失已登记状态。
  static const _kStatus = 'sfid.bind.status';
  static const _kAddress = 'sfid.bind.address';
  static const _kPubkeyHex = 'sfid.bind.pubkey_hex';
  static const _kSfidCode = 'sfid.bind.sfid_code';
  static const _kIdentityStatus = 'sfid.bind.identity_status';
  static const _kIsColdWallet = 'sfid.bind.is_cold_wallet';
  static const _kUpdatedAt = 'sfid.bind.updated_at';

  Future<MyIdState> getState() async {
    final prefs = await SharedPreferences.getInstance();
    final rawStatus = prefs.getString(_kStatus) ?? 'unset';
    final status = switch (rawStatus) {
      'pending' => MyIdStatus.pending,
      'bound' => MyIdStatus.bound,
      _ => MyIdStatus.unset,
    };
    return MyIdState(
      status: status,
      walletAddress: prefs.getString(_kAddress),
      walletPubkeyHex: prefs.getString(_kPubkeyHex),
      sfidCode: prefs.getString(_kSfidCode),
      identityStatus: prefs.getString(_kIdentityStatus),
      isColdWallet: prefs.getBool(_kIsColdWallet) ?? false,
      updatedAtMillis: prefs.getInt(_kUpdatedAt),
    );
  }

  /// 注册电子护照账户（带签名证明私钥所有权）。
  ///
  /// 调用后端注册接口成功后，本地状态变为 pending。
  Future<MyIdState> registerMyId({
    required String walletAddress,
    required String walletPubkeyHex,
    required bool isColdWallet,
    required String signatureHex,
    required String signMessage,
  }) async {
    await _api.registerMyId(
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
    debugPrint('myid registered: address=$walletAddress');
    return getState();
  }

  /// 从后端同步电子护照状态。
  ///
  /// 在 initState / onResume 时调用，静默更新本地缓存。
  Future<MyIdState> syncFromBackend() async {
    final localState = await getState();
    if (localState.walletAddress == null || localState.walletAddress!.isEmpty) {
      return localState;
    }
    try {
      final remote = await _api.queryMyIdStatus(
        localState.walletAddress!,
      );
      final prefs = await SharedPreferences.getInstance();
      final now = DateTime.now().millisecondsSinceEpoch;
      switch (remote.status) {
        case 'bound':
          await prefs.setString(_kStatus, 'bound');
          await _setStringIfPresent(prefs, _kAddress, remote.address);
          await _setOptionalString(prefs, _kSfidCode, remote.sfidCode);
          await _setOptionalString(
            prefs,
            _kIdentityStatus,
            remote.identityStatus,
          );
        case 'pending':
          await prefs.setString(_kStatus, 'pending');
          await _setStringIfPresent(prefs, _kAddress, remote.address);
          await _setOptionalString(prefs, _kSfidCode, remote.sfidCode);
          await _setOptionalString(
            prefs,
            _kIdentityStatus,
            remote.identityStatus,
          );
        default:
          // 后端返回 "unset"：绑定已被解除
          await prefs.setString(_kStatus, 'unset');
          await prefs.remove(_kAddress);
          await prefs.remove(_kPubkeyHex);
          await prefs.remove(_kSfidCode);
          await prefs.remove(_kIdentityStatus);
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
  Future<MyIdState> clear() async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove(_kStatus);
    await prefs.remove(_kAddress);
    await prefs.remove(_kPubkeyHex);
    await prefs.remove(_kSfidCode);
    await prefs.remove(_kIdentityStatus);
    await prefs.remove(_kIsColdWallet);
    await prefs.remove(_kUpdatedAt);
    return getState();
  }

  Future<void> _setOptionalString(
    SharedPreferences prefs,
    String key,
    String? value,
  ) async {
    final normalized = value?.trim();
    if (normalized == null || normalized.isEmpty) {
      await prefs.remove(key);
      return;
    }
    await prefs.setString(key, normalized);
  }

  Future<void> _setStringIfPresent(
    SharedPreferences prefs,
    String key,
    String? value,
  ) async {
    final normalized = value?.trim();
    if (normalized == null || normalized.isEmpty) {
      return;
    }
    await prefs.setString(key, normalized);
  }
}
