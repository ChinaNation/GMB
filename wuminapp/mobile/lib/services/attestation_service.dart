import 'dart:math';

import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/services/wallet_service.dart';

class AttestationState {
  const AttestationState({
    required this.hasToken,
    required this.token,
    required this.expiresAtMillis,
    required this.policy,
    required this.lastRequestPayload,
  });

  final bool hasToken;
  final String? token;
  final int? expiresAtMillis;
  final String policy;
  final String? lastRequestPayload;

  bool get isValid =>
      hasToken &&
      token != null &&
      expiresAtMillis != null &&
      expiresAtMillis! > DateTime.now().millisecondsSinceEpoch;
}

class AttestationService {
  static const _kToken = 'attest.token';
  static const _kExpiresAtMillis = 'attest.expires_at_millis';
  static const _kPolicy = 'attest.policy';
  static const _kLastPayload = 'attest.last_payload';
  static const _kTokenTtlMillis = 15 * 60 * 1000; // 15 min short-lived token
  static const _kRenewThresholdMillis = 2 * 60 * 1000; // renew before expire

  Future<AttestationState> getState() async {
    final prefs = await SharedPreferences.getInstance();
    final token = prefs.getString(_kToken);
    final expiresAtMillis = prefs.getInt(_kExpiresAtMillis);
    final policy = prefs.getString(_kPolicy) ?? 'DEFAULT_DERIVATION_PATH_ONLY';
    final payload = prefs.getString(_kLastPayload);
    final hasToken = token != null && expiresAtMillis != null;
    return AttestationState(
      hasToken: hasToken,
      token: token,
      expiresAtMillis: expiresAtMillis,
      policy: policy,
      lastRequestPayload: payload,
    );
  }

  Future<AttestationState> ensureValidToken(WalletProfile wallet) async {
    final state = await getState();
    if (!state.hasToken || state.expiresAtMillis == null) {
      return applyOfficialProof(wallet);
    }
    final now = DateTime.now().millisecondsSinceEpoch;
    if (state.expiresAtMillis! - now <= _kRenewThresholdMillis) {
      return applyOfficialProof(wallet);
    }
    return state;
  }

  Future<AttestationState> applyOfficialProof(WalletProfile wallet) async {
    final payload = _buildPayload(wallet);
    final token = _issueToken();
    final expiresAt = DateTime.now().millisecondsSinceEpoch + _kTokenTtlMillis;
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_kToken, token);
    await prefs.setInt(_kExpiresAtMillis, expiresAt);
    await prefs.setString(_kPolicy, walletServicePolicy(wallet));
    await prefs.setString(_kLastPayload, payload);
    return getState();
  }

  Future<void> clearToken() async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove(_kToken);
    await prefs.remove(_kExpiresAtMillis);
    await prefs.remove(_kPolicy);
    await prefs.remove(_kLastPayload);
  }

  Future<SfidBindDraft> buildSfidBindDraft({
    required WalletProfile wallet,
    required String sfidCode,
  }) async {
    final state = await getState();
    if (!state.isValid || state.token == null) {
      throw Exception('未拿到官方证明，不能绑定 SFID');
    }
    final challenge = _issueChallenge();
    final signature = _signChallengeLocally(challenge, wallet);
    return SfidBindDraft(
      sfidCode: sfidCode,
      attestationToken: state.token!,
      challenge: challenge,
      challengeSignature: signature,
    );
  }

  String walletServicePolicy(WalletProfile wallet) {
    return 'alg=${wallet.alg};ss58=${wallet.ss58};path=default-only';
  }

  String _buildPayload(WalletProfile wallet) {
    final policy = walletServicePolicy(wallet);
    return '{'
        '"pubkey":"${wallet.pubkeyHex}",'
        '"alg":"${wallet.alg}",'
        '"ss58":${wallet.ss58},'
        '"policy":"$policy",'
        '"device_integrity":"ios_dev_mode_attested_placeholder"'
        '}';
  }

  String _issueToken() {
    final now = DateTime.now().millisecondsSinceEpoch;
    final rand = Random(now).nextInt(1 << 32).toRadixString(16);
    return 'attest_${now}_$rand';
  }

  String _issueChallenge() {
    final now = DateTime.now().millisecondsSinceEpoch;
    return 'challenge_$now';
  }

  String _signChallengeLocally(String challenge, WalletProfile wallet) {
    // MVP placeholder signature flow: keep API shape fixed, replace with real sr25519 signing later.
    final seed = '$challenge|${wallet.pubkeyHex}|${wallet.address}';
    final hash = seed.codeUnits.fold<int>(0, (acc, e) => (acc * 131 + e) & 0x7fffffff);
    return 'sig_${hash.toRadixString(16)}';
  }
}

class SfidBindDraft {
  const SfidBindDraft({
    required this.sfidCode,
    required this.attestationToken,
    required this.challenge,
    required this.challengeSignature,
  });

  final String sfidCode;
  final String attestationToken;
  final String challenge;
  final String challengeSignature;
}
