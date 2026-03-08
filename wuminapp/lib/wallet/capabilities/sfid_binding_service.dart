import 'package:flutter/foundation.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/wallet/capabilities/api_client.dart';

enum SfidBindStatus { unbound, pending, bound }

class SfidBindState {
  const SfidBindState({
    required this.status,
    this.walletAddress,
    this.updatedAtMillis,
  });

  final SfidBindStatus status;
  final String? walletAddress;
  final int? updatedAtMillis;
}

class SfidBindingService {
  final ApiClient _apiClient = ApiClient();

  static const _kStatus = 'sfid.bind.status';
  static const _kAddress = 'sfid.bind.address';
  static const _kUpdatedAt = 'sfid.bind.updated_at';

  Future<SfidBindState> getState() async {
    final prefs = await SharedPreferences.getInstance();
    final rawStatus = prefs.getString(_kStatus) ?? 'unbound';
    final status = switch (rawStatus) {
      'pending' => SfidBindStatus.pending,
      'bound' => SfidBindStatus.bound,
      _ => SfidBindStatus.unbound,
    };
    return SfidBindState(
      status: status,
      walletAddress: prefs.getString(_kAddress),
      updatedAtMillis: prefs.getInt(_kUpdatedAt),
    );
  }

  Future<SfidBindState> submitBinding(
    String walletAddress,
    String walletPubkeyHex,
  ) async {
    await _apiClient.requestChainBindByPubkey(walletPubkeyHex);
    final now = DateTime.now().millisecondsSinceEpoch;
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_kStatus, 'pending');
    await prefs.setString(_kAddress, walletAddress);
    await prefs.setInt(_kUpdatedAt, now);
    debugPrint('chain bind request sent: pubkey=$walletPubkeyHex');
    return getState();
  }

  Future<SfidBindState> markBound() async {
    final now = DateTime.now().millisecondsSinceEpoch;
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_kStatus, 'bound');
    await prefs.setInt(_kUpdatedAt, now);
    return getState();
  }
}
