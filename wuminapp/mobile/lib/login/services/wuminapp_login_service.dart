import 'dart:convert';
import 'dart:typed_data';

import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:wuminapp_mobile/login/models/login_models.dart';
import 'package:wuminapp_mobile/login/models/login_exception.dart';
import 'package:wuminapp_mobile/login/services/login_replay_guard.dart';
import 'package:wuminapp_mobile/login/services/login_whitelist_policy.dart';
import 'package:wuminapp_mobile/services/wallet_service.dart';

class WuminLoginService {
  WuminLoginService({
    WalletService? walletService,
    LoginReplayGuard? replayGuard,
    LoginWhitelistPolicy? whitelistPolicy,
  })  : _walletService = walletService ?? WalletService(),
        _replayGuard = replayGuard ?? LoginReplayGuard(),
        _whitelistPolicy = whitelistPolicy ?? LoginWhitelistPolicy();

  static const String protocol = 'WUMINAPP_LOGIN_V1';
  static const Set<String> allowedSystems = {
    'cpms',
    'sfid',
    'citizenchain',
  };

  final WalletService _walletService;
  final LoginReplayGuard _replayGuard;
  final LoginWhitelistPolicy _whitelistPolicy;

  WuminLoginChallenge parseChallenge(String raw) {
    final text = raw.trim();
    final decoded = jsonDecode(text);
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
    final origin = _requiredString(data, 'origin');

    final challengeData = WuminLoginChallenge(
      proto: proto,
      system: system,
      requestId: requestId,
      challenge: challenge,
      nonce: nonce,
      issuedAt: issuedAt,
      expiresAt: expiresAt,
      aud: aud,
      origin: origin,
      raw: raw,
    );

    if (challengeData.isExpired) {
      throw const LoginException(LoginErrorCode.expired, '登录挑战已过期，请刷新后重试');
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
        ? await _walletService.getLatestWalletSecret()
        : await _walletService.getWalletSecretByIndex(walletIndex);
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
    final mnemonic = walletSecret.mnemonic;
    final pair = await Keyring.sr25519.fromMnemonic(mnemonic);
    pair.ss58Format = wallet.ss58;

    final localPubkeyHex = _toHex(pair.bytes().toList(growable: false));
    if (localPubkeyHex.toLowerCase() != wallet.pubkeyHex.toLowerCase()) {
      throw const LoginException(
        LoginErrorCode.walletMismatch,
        '本地签名密钥与当前钱包不一致，请重新导入钱包',
      );
    }

    final signMessage = _buildSignMessage(challenge);
    final message = Uint8List.fromList(utf8.encode(signMessage));
    final signature = pair.sign(message);

    final receipt = WuminLoginReceipt(
      proto: protocol,
      requestId: challenge.requestId,
      account: wallet.address,
      pubkey: '0x${wallet.pubkeyHex}',
      sigAlg: 'sr25519',
      signature: '0x${_toHex(signature.toList(growable: false))}',
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
      challenge.origin,
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

  String _toHex(List<int> bytes) {
    const chars = '0123456789abcdef';
    final buf = StringBuffer();
    for (final b in bytes) {
      buf
        ..write(chars[(b >> 4) & 0x0f])
        ..write(chars[b & 0x0f]);
    }
    return buf.toString();
  }
}
