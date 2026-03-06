import 'dart:math';

import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:isar/isar.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_isar.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_secure_keys.dart';

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
  static const FlutterSecureStorage _secureStorage = FlutterSecureStorage();

  static const String _kScope = 'attest';
  static const String _kMetaExpiresAtMillis =
      'wallet.session.attest.expires_at_millis.v1';
  static const String _kMetaPolicy = 'wallet.session.attest.policy.v1';
  static const String _kMetaLastPayload =
      'wallet.session.attest.last_payload.v1';

  static const _kTokenTtlMillis = 15 * 60 * 1000; // 15 min short-lived token
  static const _kRenewThresholdMillis = 2 * 60 * 1000; // renew before expire

  Future<AttestationState> getState() async {
    final isar = await WalletIsar.instance.db();
    final token = await _secureStorage.read(key: _tokenKey());
    final expiresAtMillis = await _getKvInt(isar, _kMetaExpiresAtMillis);
    final policy = await _getKvString(isar, _kMetaPolicy) ??
        'DEFAULT_DERIVATION_PATH_ONLY';
    final payload = await _getKvString(isar, _kMetaLastPayload);
    final hasToken =
        token != null && token.trim().isNotEmpty && expiresAtMillis != null;
    return AttestationState(
      hasToken: hasToken,
      token: token?.trim(),
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
    final isar = await WalletIsar.instance.db();
    await _secureStorage.write(key: _tokenKey(), value: token);
    await isar.writeTxn(() async {
      await _putKvInt(isar, _kMetaExpiresAtMillis, expiresAt);
      await _putKvString(isar, _kMetaPolicy, walletServicePolicy(wallet));
      await _putKvString(isar, _kMetaLastPayload, payload);
    });
    return getState();
  }

  Future<void> clearToken() async {
    final isar = await WalletIsar.instance.db();
    await _secureStorage.delete(key: _tokenKey());
    await isar.writeTxn(() async {
      await _deleteKv(isar, _kMetaExpiresAtMillis);
      await _deleteKv(isar, _kMetaPolicy);
      await _deleteKv(isar, _kMetaLastPayload);
    });
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

  String _tokenKey() => WalletSecureKeys.sessionTokenV1(_kScope);

  Future<int?> _getKvInt(Isar isar, String key) async {
    final row = await isar.appKvEntitys.filter().keyEqualTo(key).findFirst();
    return row?.intValue;
  }

  Future<String?> _getKvString(Isar isar, String key) async {
    final row = await isar.appKvEntitys.filter().keyEqualTo(key).findFirst();
    return row?.stringValue;
  }

  Future<void> _putKvInt(Isar isar, String key, int value) async {
    await isar.appKvEntitys.put(
      AppKvEntity()
        ..key = key
        ..intValue = value
        ..stringValue = null
        ..boolValue = null,
    );
  }

  Future<void> _putKvString(Isar isar, String key, String value) async {
    await isar.appKvEntitys.put(
      AppKvEntity()
        ..key = key
        ..stringValue = value
        ..intValue = null
        ..boolValue = null,
    );
  }

  Future<void> _deleteKv(Isar isar, String key) async {
    final row = await isar.appKvEntitys.filter().keyEqualTo(key).findFirst();
    if (row == null) {
      return;
    }
    await isar.appKvEntitys.delete(row.id);
  }

  String _signChallengeLocally(String challenge, WalletProfile wallet) {
    // MVP placeholder signature flow: keep API shape fixed, replace with real sr25519 signing later.
    final seed = '$challenge|${wallet.pubkeyHex}|${wallet.address}';
    final hash =
        seed.codeUnits.fold<int>(0, (acc, e) => (acc * 131 + e) & 0x7fffffff);
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
