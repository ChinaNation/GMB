import 'package:flutter/foundation.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/my/myid/myid_api.dart';

/// 电子护照绑定状态（与后端 bind_status 响应对齐）。
enum MyIdBindStatus { unset, pending, bound }

class MyIdState {
  const MyIdState({
    required this.bindStatus,
    this.walletAddress,
    this.walletPubkeyHex,
    this.walletIndex,
    this.sfidCode,
    this.identityStatus,
    this.validFrom,
    this.validUntil,
    this.statusUpdatedAt,
    this.isColdWallet = false,
    this.updatedAtMillis,
  });

  final MyIdBindStatus bindStatus;
  final String? walletAddress;
  final String? walletPubkeyHex;
  final int? walletIndex;
  final String? sfidCode;
  final String? identityStatus;
  final String? validFrom;
  final String? validUntil;
  final int? statusUpdatedAt;
  final bool isColdWallet;
  final int? updatedAtMillis;
}

class MyIdService {
  final MyIdApi _api = MyIdApi();

  static const _kBindStatus = 'myid.bind_status';
  static const _kAddress = 'myid.wallet_address';
  static const _kPubkeyHex = 'myid.wallet_pubkey_hex';
  static const _kWalletIndex = 'myid.wallet_index';
  static const _kSfidCode = 'myid.sfid_code';
  static const _kIdentityStatus = 'myid.identity_status';
  static const _kValidFrom = 'myid.valid_from';
  static const _kValidUntil = 'myid.valid_until';
  static const _kStatusUpdatedAt = 'myid.status_updated_at';
  static const _kIsColdWallet = 'myid.is_cold_wallet';
  static const _kUpdatedAt = 'myid.updated_at';

  Future<MyIdState> getState() async {
    final prefs = await SharedPreferences.getInstance();
    final rawBindStatus = prefs.getString(_kBindStatus) ?? 'unset';
    final bindStatus = switch (rawBindStatus) {
      'pending' => MyIdBindStatus.pending,
      'bound' => MyIdBindStatus.bound,
      _ => MyIdBindStatus.unset,
    };
    return MyIdState(
      bindStatus: bindStatus,
      walletAddress: prefs.getString(_kAddress),
      walletPubkeyHex: prefs.getString(_kPubkeyHex),
      walletIndex: prefs.getInt(_kWalletIndex),
      sfidCode: prefs.getString(_kSfidCode),
      identityStatus: prefs.getString(_kIdentityStatus),
      validFrom: prefs.getString(_kValidFrom),
      validUntil: prefs.getString(_kValidUntil),
      statusUpdatedAt: prefs.getInt(_kStatusUpdatedAt),
      isColdWallet: prefs.getBool(_kIsColdWallet) ?? false,
      updatedAtMillis: prefs.getInt(_kUpdatedAt),
    );
  }

  /// 选择电子护照使用的钱包。
  ///
  /// 中文注释：CPMS 阶段只需要扫描钱包地址;真正的钱包签名与已绑定确认
  /// 统一放到 SFID 绑定阶段,所以这里不联网注册、不写 bound。
  Future<MyIdState> selectWallet({
    required String walletAddress,
    required String walletPubkeyHex,
    required int walletIndex,
    required bool isColdWallet,
  }) async {
    final now = DateTime.now().millisecondsSinceEpoch;
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_kBindStatus, 'pending');
    await prefs.setString(_kAddress, walletAddress);
    await prefs.setString(_kPubkeyHex, walletPubkeyHex.trim());
    await prefs.setInt(_kWalletIndex, walletIndex);
    await prefs.setBool(_kIsColdWallet, isColdWallet);
    await prefs.setInt(_kUpdatedAt, now);
    debugPrint('myid wallet selected: address=$walletAddress');
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
      switch (remote.bindStatus) {
        case 'bound':
          await prefs.setString(_kBindStatus, 'bound');
          await _setStringIfPresent(prefs, _kAddress, remote.walletAddress);
          await _setOptionalString(prefs, _kSfidCode, remote.sfidCode);
          await _setOptionalString(
            prefs,
            _kIdentityStatus,
            remote.identityStatus,
          );
          await _setOptionalString(prefs, _kValidFrom, remote.validFrom);
          await _setOptionalString(prefs, _kValidUntil, remote.validUntil);
          await _setOptionalInt(
            prefs,
            _kStatusUpdatedAt,
            remote.statusUpdatedAt,
          );
        case 'pending':
          await prefs.setString(_kBindStatus, 'pending');
          await _setStringIfPresent(prefs, _kAddress, remote.walletAddress);
          await _setOptionalString(prefs, _kSfidCode, remote.sfidCode);
          await _setOptionalString(
            prefs,
            _kIdentityStatus,
            remote.identityStatus,
          );
          await _setOptionalString(prefs, _kValidFrom, remote.validFrom);
          await _setOptionalString(prefs, _kValidUntil, remote.validUntil);
          await _setOptionalInt(
            prefs,
            _kStatusUpdatedAt,
            remote.statusUpdatedAt,
          );
        default:
          if (localState.bindStatus == MyIdBindStatus.bound) {
            // 中文注释：只有曾经由 SFID 确认 bound 的状态,才允许远端 unset 清空。
            await prefs.setString(_kBindStatus, 'unset');
            await prefs.remove(_kAddress);
            await prefs.remove(_kPubkeyHex);
            await prefs.remove(_kWalletIndex);
            await prefs.remove(_kSfidCode);
            await prefs.remove(_kIdentityStatus);
            await prefs.remove(_kValidFrom);
            await prefs.remove(_kValidUntil);
            await prefs.remove(_kStatusUpdatedAt);
            await prefs.remove(_kIsColdWallet);
          }
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
    await prefs.remove(_kBindStatus);
    await prefs.remove(_kAddress);
    await prefs.remove(_kPubkeyHex);
    await prefs.remove(_kWalletIndex);
    await prefs.remove(_kSfidCode);
    await prefs.remove(_kIdentityStatus);
    await prefs.remove(_kValidFrom);
    await prefs.remove(_kValidUntil);
    await prefs.remove(_kStatusUpdatedAt);
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

  Future<void> _setOptionalInt(
    SharedPreferences prefs,
    String key,
    int? value,
  ) async {
    if (value == null) {
      await prefs.remove(key);
      return;
    }
    await prefs.setInt(key, value);
  }
}
