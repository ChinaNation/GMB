import 'dart:convert';

import 'package:wuminapp_mobile/login/models/login_models.dart';
import 'package:wuminapp_mobile/login/models/login_exception.dart';
import 'package:wuminapp_mobile/login/services/login_replay_guard.dart';
import 'package:wuminapp_mobile/login/services/login_whitelist_policy.dart';
import 'package:wuminapp_mobile/signer/local_signer.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

class SignService {
  SignService({
    WalletManager? walletManager,
    LoginReplayGuard? replayGuard,
    LoginWhitelistPolicy? whitelistPolicy,
    LocalSigner? localSigner,
  })  : _walletManager = walletManager ?? WalletManager(),
        _replayGuard = replayGuard ?? LoginReplayGuard(),
        _whitelistPolicy = whitelistPolicy ?? LoginWhitelistPolicy(),
        _localSigner = localSigner ?? LocalSigner();

  static const String protocol = 'WUMINAPP_LOGIN_V1';
  static const int challengeTtlSeconds = 90;
  static const int maxClockSkewSeconds = 30;
  static const int maxChallengePayloadChars = 4096;
  static const Set<String> allowedSystems = {
    'cpms',
    'sfid',
  };
  static final RegExp _idPattern = RegExp(r'^[A-Za-z0-9._:-]{4,128}$');
  static final RegExp _audPattern = RegExp(r'^[a-z0-9._:-]{3,64}$');

  final WalletManager _walletManager;
  final LoginReplayGuard _replayGuard;
  final LoginWhitelistPolicy _whitelistPolicy;
  final LocalSigner _localSigner;

  WuminLoginChallenge parseChallenge(String raw) {
    final text = raw.trim();
    if (text.isEmpty || text.length > maxChallengePayloadChars) {
      throw const LoginException(
        LoginErrorCode.invalidFormat,
        '二维码格式错误：登录挑战长度无效',
      );
    }

    dynamic decoded;
    try {
      decoded = jsonDecode(text);
    } catch (_) {
      throw const LoginException(
        LoginErrorCode.invalidFormat,
        '二维码格式错误：必须为 JSON 对象',
      );
    }
    if (decoded is! Map) {
      throw const LoginException(
        LoginErrorCode.invalidFormat,
        '二维码格式错误：必须为 JSON 对象',
      );
    }
    final data = decoded.map((k, v) => MapEntry(k.toString(), v));

    final proto = _requiredString(data, 'proto');
    if (proto != protocol) {
      throw LoginException(LoginErrorCode.invalidProtocol, '不支持的协议：$proto');
    }

    final system = _requiredString(data, 'system').toLowerCase();
    if (!allowedSystems.contains(system)) {
      throw LoginException(LoginErrorCode.invalidSystem, '不支持的系统：$system');
    }

    final requestId = _requiredString(data, 'request_id');
    final challenge = _requiredString(data, 'challenge');
    final nonce = _requiredString(data, 'nonce');
    final issuedAt = _requiredInt(data, 'issued_at');
    final expiresAt = _requiredInt(data, 'expires_at');
    final aud = _requiredString(data, 'aud');
    _validateIdField('request_id', requestId);
    _validateOpaqueField('challenge', challenge);
    _validateIdField('nonce', nonce);
    _validateAudField(aud);

    final challengeData = WuminLoginChallenge(
      proto: proto,
      system: system,
      requestId: requestId,
      challenge: challenge,
      nonce: nonce,
      issuedAt: issuedAt,
      expiresAt: expiresAt,
      aud: aud,
      raw: raw,
    );

    if (challengeData.isExpired) {
      throw const LoginException(LoginErrorCode.expired, '登录挑战已过期，请刷新后重试');
    }
    if ((challengeData.expiresAt - challengeData.issuedAt) !=
        challengeTtlSeconds) {
      throw const LoginException(
        LoginErrorCode.invalidTtl,
        '登录挑战有效期必须为 90 秒',
      );
    }
    final now = _nowEpochSeconds();
    if (challengeData.issuedAt > now + maxClockSkewSeconds) {
      throw const LoginException(
        LoginErrorCode.invalidField,
        '二维码字段格式错误：issued_at 超出设备时间范围',
      );
    }
    if (challengeData.expiresAt <= challengeData.issuedAt) {
      throw const LoginException(
        LoginErrorCode.invalidField,
        '二维码字段格式错误：expires_at 必须晚于 issued_at',
      );
    }
    return challengeData;
  }

  Future<void> validateTrust(WuminLoginChallenge challenge) async {
    await _whitelistPolicy.assertAllowed(challenge);
  }

  Future<Map<String, dynamic>> buildReceiptPayload(String rawChallenge) async {
    final challenge = parseChallenge(rawChallenge);
    await validateTrust(challenge);
    return buildReceiptPayloadForChallenge(challenge);
  }

  Future<Map<String, dynamic>> buildReceiptPayloadForChallenge(
    WuminLoginChallenge challenge, {
    int? walletIndex,
  }) async {
    await validateTrust(challenge);
    await _replayGuard.assertNotConsumed(challenge.requestId);
    final walletSecret = walletIndex == null
        ? await _walletManager.getLatestWalletSecret()
        : await _walletManager.getWalletSecretByIndex(walletIndex);
    if (walletSecret == null) {
      if (walletIndex == null) {
        throw const LoginException(LoginErrorCode.walletMissing, '请先创建或导入钱包');
      }
      throw LoginException(
        LoginErrorCode.walletNotFound,
        '未找到指定钱包（walletIndex=$walletIndex）',
      );
    }

    final wallet = walletSecret.profile;
    final signMessage = _buildSignMessage(challenge);
    late LocalSignResult signed;
    try {
      signed = await _localSigner.signUtf8(
        walletSecret: walletSecret,
        message: signMessage,
      );
    } on LocalSignerException catch (e) {
      if (e.code == LocalSignerErrorCode.walletMismatch) {
        throw const LoginException(
          LoginErrorCode.walletMismatch,
          '本地签名密钥与当前钱包不一致，请重新导入钱包',
        );
      }
      throw LoginException(LoginErrorCode.invalidField, e.message);
    }

    final receipt = WuminLoginReceipt(
      proto: protocol,
      requestId: challenge.requestId,
      account: wallet.address,
      pubkey: signed.pubkeyHex,
      sigAlg: signed.sigAlg,
      signature: signed.signatureHex,
      signedAt: DateTime.now().millisecondsSinceEpoch ~/ 1000,
    );
    await _replayGuard.consume(
      requestId: challenge.requestId,
      expiresAt: challenge.expiresAt,
    );
    return receipt.toJson();
  }

  String buildSignPreview(String rawChallenge) {
    final challenge = parseChallenge(rawChallenge);
    return _buildSignMessage(challenge);
  }

  String buildSignPreviewForChallenge(WuminLoginChallenge challenge) {
    return _buildSignMessage(challenge);
  }

  String _buildSignMessage(WuminLoginChallenge challenge) {
    return [
      protocol,
      challenge.system,
      challenge.aud,
      challenge.requestId,
      challenge.challenge,
      challenge.nonce,
      challenge.expiresAt.toString(),
    ].join('|');
  }

  String _requiredString(Map<String, dynamic> data, String key) {
    final value = data[key]?.toString().trim();
    if (value == null || value.isEmpty) {
      throw LoginException(LoginErrorCode.missingField, '二维码缺少字段：$key');
    }
    return value;
  }

  int _requiredInt(Map<String, dynamic> data, String key) {
    final value = data[key];
    if (value is int) {
      return value;
    }
    if (value is String) {
      final parsed = int.tryParse(value);
      if (parsed != null) {
        return parsed;
      }
    }
    throw LoginException(LoginErrorCode.invalidField, '二维码字段格式错误：$key');
  }

  void _validateIdField(String key, String value) {
    if (!_idPattern.hasMatch(value)) {
      throw LoginException(LoginErrorCode.invalidField, '二维码字段格式错误：$key');
    }
  }

  void _validateOpaqueField(String key, String value) {
    if (value.length < 4 || value.length > 512) {
      throw LoginException(LoginErrorCode.invalidField, '二维码字段格式错误：$key');
    }
    if (RegExp(r'\s').hasMatch(value)) {
      throw LoginException(LoginErrorCode.invalidField, '二维码字段格式错误：$key');
    }
  }

  void _validateAudField(String aud) {
    if (!_audPattern.hasMatch(aud.toLowerCase())) {
      throw const LoginException(
        LoginErrorCode.invalidField,
        '二维码字段格式错误：aud',
      );
    }
  }

  int _nowEpochSeconds() => DateTime.now().millisecondsSinceEpoch ~/ 1000;
}
