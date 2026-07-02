import 'package:flutter/foundation.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:citizenapp/my/myid/myid_api.dart';

/// 本地电子护照档案状态。
enum MyIdArchiveStatus { unset, pending, registered }

class MyIdState {
  const MyIdState({
    required this.archiveStatus,
    this.walletAddress,
    this.walletPubkeyHex,
    this.walletIndex,
    this.cidNumber,
    this.passportNo,
    this.citizenStatus,
    this.votingEligible,
    this.voteStatus,
    this.identityStatus,
    this.passportValidFrom,
    this.passportValidUntil,
    this.statusUpdatedAt,
    this.isColdWallet = false,
    this.updatedAtMillis,
  });

  final MyIdArchiveStatus archiveStatus;
  final String? walletAddress;
  final String? walletPubkeyHex;
  final int? walletIndex;
  final String? cidNumber;
  final String? passportNo;
  final String? citizenStatus;
  final bool? votingEligible;
  final String? voteStatus;
  final String? identityStatus;
  final String? passportValidFrom;
  final String? passportValidUntil;
  final int? statusUpdatedAt;
  final bool isColdWallet;
  final int? updatedAtMillis;
}

class MyIdService {
  final MyIdApi _api = MyIdApi();

  static const _kArchiveStatus = 'myid.archive_status';
  static const _kAddress = 'myid.wallet_address';
  static const _kPubkeyHex = 'myid.wallet_pubkey_hex';
  static const _kWalletIndex = 'myid.wallet_index';
  static const _kCidNumber = 'myid.cid_number';
  static const _kPassportNo = 'myid.passport_no';
  static const _kCitizenStatus = 'myid.citizen_status';
  static const _kVotingEligible = 'myid.voting_eligible';
  static const _kVoteStatus = 'myid.vote_status';
  static const _kIdentityStatus = 'myid.identity_status';
  static const _kPassportValidFrom = 'myid.passport_valid_from';
  static const _kPassportValidUntil = 'myid.passport_valid_until';
  static const _kStatusUpdatedAt = 'myid.status_updated_at';
  static const _kIsColdWallet = 'myid.is_cold_wallet';
  static const _kUpdatedAt = 'myid.updated_at';

  Future<MyIdState> getState() async {
    final prefs = await SharedPreferences.getInstance();
    final rawArchiveStatus = prefs.getString(_kArchiveStatus) ?? 'unset';
    final archiveStatus = switch (rawArchiveStatus) {
      'pending' => MyIdArchiveStatus.pending,
      'registered' => MyIdArchiveStatus.registered,
      _ => MyIdArchiveStatus.unset,
    };
    return MyIdState(
      archiveStatus: archiveStatus,
      walletAddress: prefs.getString(_kAddress),
      walletPubkeyHex: prefs.getString(_kPubkeyHex),
      walletIndex: prefs.getInt(_kWalletIndex),
      cidNumber: prefs.getString(_kCidNumber),
      passportNo: prefs.getString(_kPassportNo),
      citizenStatus: prefs.getString(_kCitizenStatus),
      votingEligible: prefs.getBool(_kVotingEligible),
      voteStatus: prefs.getString(_kVoteStatus),
      identityStatus: prefs.getString(_kIdentityStatus),
      passportValidFrom: prefs.getString(_kPassportValidFrom),
      passportValidUntil: prefs.getString(_kPassportValidUntil),
      statusUpdatedAt: prefs.getInt(_kStatusUpdatedAt),
      isColdWallet: prefs.getBool(_kIsColdWallet) ?? false,
      updatedAtMillis: prefs.getInt(_kUpdatedAt),
    );
  }

  /// 选择电子护照使用的钱包。
  ///
  /// 这里仅选择本机电子护照钱包,不联网注册、不写已登记态。
  Future<MyIdState> selectWallet({
    required String walletAddress,
    required String walletPubkeyHex,
    required int walletIndex,
    required bool isColdWallet,
  }) async {
    final now = DateTime.now().millisecondsSinceEpoch;
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_kArchiveStatus, 'pending');
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
      if (remote.found) {
        await prefs.setString(_kArchiveStatus, 'registered');
        await _setStringIfPresent(prefs, _kAddress, remote.walletAddress);
        await _setOptionalString(prefs, _kCidNumber, remote.cidNumber);
        await _setOptionalString(prefs, _kPassportNo, remote.passportNo);
        await _setOptionalString(
          prefs,
          _kCitizenStatus,
          remote.citizenStatus,
        );
        await _setOptionalBool(
          prefs,
          _kVotingEligible,
          remote.votingEligible,
        );
        await _setOptionalString(prefs, _kVoteStatus, remote.voteStatus);
        await _setOptionalString(
          prefs,
          _kIdentityStatus,
          remote.identityStatus,
        );
        await _setOptionalString(
          prefs,
          _kPassportValidFrom,
          remote.passportValidFrom,
        );
        await _setOptionalString(
          prefs,
          _kPassportValidUntil,
          remote.passportValidUntil,
        );
        await _setOptionalInt(
          prefs,
          _kStatusUpdatedAt,
          remote.statusUpdatedAt,
        );
      } else if (localState.archiveStatus == MyIdArchiveStatus.registered) {
        // 只有曾经由后端确认有档案的状态,才允许远端未找到时清空。
        await prefs.setString(_kArchiveStatus, 'unset');
        await prefs.remove(_kAddress);
        await prefs.remove(_kPubkeyHex);
        await prefs.remove(_kWalletIndex);
        await prefs.remove(_kCidNumber);
        await prefs.remove(_kPassportNo);
        await prefs.remove(_kCitizenStatus);
        await prefs.remove(_kVotingEligible);
        await prefs.remove(_kVoteStatus);
        await prefs.remove(_kIdentityStatus);
        await prefs.remove(_kPassportValidFrom);
        await prefs.remove(_kPassportValidUntil);
        await prefs.remove(_kStatusUpdatedAt);
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

  /// 清除本地电子护照档案状态。
  Future<MyIdState> clear() async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove(_kArchiveStatus);
    await prefs.remove(_kAddress);
    await prefs.remove(_kPubkeyHex);
    await prefs.remove(_kWalletIndex);
    await prefs.remove(_kCidNumber);
    await prefs.remove(_kPassportNo);
    await prefs.remove(_kCitizenStatus);
    await prefs.remove(_kVotingEligible);
    await prefs.remove(_kVoteStatus);
    await prefs.remove(_kIdentityStatus);
    await prefs.remove(_kPassportValidFrom);
    await prefs.remove(_kPassportValidUntil);
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

  Future<void> _setOptionalBool(
    SharedPreferences prefs,
    String key,
    bool? value,
  ) async {
    if (value == null) {
      await prefs.remove(key);
      return;
    }
    await prefs.setBool(key, value);
  }
}
