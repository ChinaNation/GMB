import 'dart:convert';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:wuminapp_mobile/qr/login/login_models.dart';
import 'package:wuminapp_mobile/qr/login/login_replay_guard.dart';
import 'package:wuminapp_mobile/qr/qr_protocols.dart';
import 'package:wuminapp_mobile/signer/qr_signer.dart';
import 'package:wuminapp_mobile/signer/system_signature_verifier.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

class LoginExternalSignBundle {
  const LoginExternalSignBundle({
    required this.signMessage,
    required this.request,
    required this.requestJson,
  });

  final String signMessage;
  final QrSignRequest request;
  final String requestJson;
}

/// 登录签名编排服务。
///
/// 职责：
/// - 解析登录挑战码
/// - 校验字段、时效、系统签名
/// - 防重放
/// - 通过 [WalletManager.signUtf8WithWallet] 生成回执（seed 不出 WalletManager）
class LoginService {
  LoginService({
    WalletManager? walletManager,
    LoginReplayGuard? replayGuard,
    LoginSystemSignatureVerifier? systemSignatureVerifier,
    Sr25519MessageVerifier? messageVerifier,
  })  : _walletManager = walletManager ?? WalletManager(),
        _replayGuard = replayGuard ?? LoginReplayGuard(),
        _systemSignatureVerifier =
            systemSignatureVerifier ?? LoginSystemSignatureVerifier(),
        _messageVerifier = messageVerifier ?? Sr25519MessageVerifier();

  static const int challengeTtlSeconds = 90;
  static const int maxClockSkewSeconds = 30;
  static const int maxChallengePayloadChars = 4096;
  static const Set<String> allowedSystems = {'cpms', 'sfid'};

  final WalletManager _walletManager;
  final LoginReplayGuard _replayGuard;
  final LoginSystemSignatureVerifier _systemSignatureVerifier;
  final Sr25519MessageVerifier _messageVerifier;

  // ---------------------------------------------------------------------------
  // 解析
  // ---------------------------------------------------------------------------

  /// 解析登录挑战码 JSON 字符串。
  LoginChallenge parseChallenge(String raw) {
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
    if (proto != QrProtocols.login) {
      throw LoginException(
        LoginErrorCode.invalidProtocol,
        '不支持的协议：$proto',
      );
    }

    final system = _requiredString(data, 'system').toLowerCase();
    if (!allowedSystems.contains(system)) {
      throw LoginException(
        LoginErrorCode.invalidSystem,
        '不支持的系统：$system',
      );
    }

    final challenge = _requiredString(data, 'challenge');
    final issuedAt = _requiredInt(data, 'issued_at');
    final expiresAt = _requiredInt(data, 'expires_at');
    final sysPubkey = _requiredString(data, 'sys_pubkey');
    final sysSig = _requiredString(data, 'sys_sig');

    _validateOpaqueField('challenge', challenge);
    _validateHexField(sysPubkey, 'sys_pubkey');
    _validateHexField(sysSig, 'sys_sig');

    final challengeData = LoginChallenge(
      proto: proto,
      system: system,
      challenge: challenge,
      issuedAt: issuedAt,
      expiresAt: expiresAt,
      sysPubkey: sysPubkey,
      sysSig: sysSig,
      raw: raw,
    );

    // 时效校验。
    if (challengeData.isExpired) {
      throw const LoginException(
        LoginErrorCode.expired,
        '登录挑战已过期，请刷新后重试',
      );
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

  // ---------------------------------------------------------------------------
  // 系统签名验证
  // ---------------------------------------------------------------------------

  /// 验证系统签名。
  Future<void> validateSystemSignature(LoginChallenge challenge) async {
    await _systemSignatureVerifier.verify(challenge);
  }

  // ---------------------------------------------------------------------------
  // 签名原文
  // ---------------------------------------------------------------------------

  /// 构建用户签名原文。
  String buildSignMessage(LoginChallenge challenge) {
    return [
      QrProtocols.login,
      challenge.system,
      challenge.challenge,
      challenge.expiresAt.toString(),
    ].join('|');
  }

  /// 计算签名原文的 SHA-256 hex 摘要。
  static String _computePayloadHash(String signMessage) {
    final bytes = utf8.encode(signMessage);
    final digest = sha256.convert(bytes);
    return digest.toString();
  }

  /// 为冷钱包登录构造外部签名请求（通过交易签名协议中继）。
  Future<LoginExternalSignBundle> buildExternalSignRequest(
    LoginChallenge challenge, {
    required WalletProfile wallet,
  }) async {
    await validateSystemSignature(challenge);
    await _replayGuard.assertNotConsumed(challenge.challenge);
    if (challenge.isExpired) {
      throw const LoginException(
        LoginErrorCode.expired,
        '登录挑战已过期，请重新扫码',
      );
    }

    final signMessage = buildSignMessage(challenge);
    final payloadHex = '0x${_hexEncode(utf8.encode(signMessage))}';
    final now = _nowEpochSeconds();
    final remaining = challenge.expiresAt - now;
    final ttlSeconds = remaining > 1 ? remaining : 1;
    final request = QrSigner().buildRequest(
      requestId: challenge.challenge,
      account: wallet.address,
      pubkey: '0x${wallet.pubkeyHex}',
      payloadHex: payloadHex,
      display: {
        'action': 'login',
        'summary': '登录 ${challenge.system.toUpperCase()} 系统',
        'fields': {
          'system': challenge.system,
        },
      },
      nowEpochSeconds: now,
      ttlSeconds: ttlSeconds,
    );
    return LoginExternalSignBundle(
      signMessage: signMessage,
      request: request,
      requestJson: QrSigner().encodeRequest(request),
    );
  }

  // ---------------------------------------------------------------------------
  // 回执生成
  // ---------------------------------------------------------------------------

  /// 生成登录回执 payload。
  Future<Map<String, dynamic>> buildReceiptPayload(
    LoginChallenge challenge, {
    int? walletIndex,
  }) async {
    await validateSystemSignature(challenge);
    await _replayGuard.assertNotConsumed(challenge.challenge);

    // 确定要使用的钱包 index
    final int targetIndex;
    if (walletIndex != null) {
      targetIndex = walletIndex;
    } else {
      final active = await _walletManager.getWallet();
      if (active == null) {
        throw const LoginException(
          LoginErrorCode.walletMissing,
          '请先创建或导入钱包',
        );
      }
      targetIndex = active.walletIndex;
    }

    final signMessage = buildSignMessage(challenge);
    late WalletSignResult signed;
    try {
      signed = await _walletManager.signUtf8WithWallet(
        targetIndex,
        signMessage,
      );
    } on WalletAuthException catch (e) {
      if (e.message.contains('不一致')) {
        throw const LoginException(
          LoginErrorCode.walletMismatch,
          '本地签名密钥与当前钱包不一致，请重新导入钱包',
        );
      }
      throw LoginException(LoginErrorCode.invalidField, e.message);
    }

    final receipt = LoginReceipt(
      proto: QrProtocols.login,
      system: challenge.system,
      challenge: challenge.challenge,
      pubkey: signed.pubkeyHex,
      sigAlg: signed.sigAlg,
      signature: signed.signatureHex,
      payloadHash: _computePayloadHash(signMessage),
      signedAt: DateTime.now().millisecondsSinceEpoch ~/ 1000,
    );
    await _replayGuard.consume(
      challenge: challenge.challenge,
      expiresAt: challenge.expiresAt,
    );
    return receipt.toJson();
  }

  /// 冷钱包登录：接受外部签名结果构建回执。
  ///
  /// 调用方通过 QrSigner 协议获取签名后，将结果传入此方法。
  Future<Map<String, dynamic>> buildReceiptFromSignature({
    required LoginChallenge challenge,
    required String pubkeyHex,
    required String signatureHex,
    String sigAlg = 'sr25519',
  }) async {
    await validateSystemSignature(challenge);
    await _replayGuard.assertNotConsumed(challenge.challenge);
    final signMessage = buildSignMessage(challenge);
    final verified = _messageVerifier.verify(
      pubkeyHex: pubkeyHex,
      message: Uint8List.fromList(utf8.encode(signMessage)),
      signatureHex: signatureHex,
    );
    if (!verified) {
      throw const LoginException(
        LoginErrorCode.invalidField,
        '冷钱包签名结果校验失败，请重新扫码签名',
      );
    }

    final receipt = LoginReceipt(
      proto: QrProtocols.login,
      system: challenge.system,
      challenge: challenge.challenge,
      pubkey: pubkeyHex,
      sigAlg: sigAlg,
      signature: signatureHex,
      payloadHash: _computePayloadHash(signMessage),
      signedAt: DateTime.now().millisecondsSinceEpoch ~/ 1000,
    );
    await _replayGuard.consume(
      challenge: challenge.challenge,
      expiresAt: challenge.expiresAt,
    );
    return receipt.toJson();
  }

  // ---------------------------------------------------------------------------
  // 内部工具
  // ---------------------------------------------------------------------------

  String _requiredString(Map<String, dynamic> data, String key) {
    final value = data[key]?.toString().trim();
    if (value == null || value.isEmpty) {
      throw LoginException(
        LoginErrorCode.missingField,
        '二维码缺少字段：$key',
      );
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
    throw LoginException(
      LoginErrorCode.invalidField,
      '二维码字段格式错误：$key',
    );
  }

  void _validateOpaqueField(String key, String value) {
    if (value.length < 4 || value.length > 512) {
      throw LoginException(
        LoginErrorCode.invalidField,
        '二维码字段格式错误：$key',
      );
    }
    if (RegExp(r'\s').hasMatch(value)) {
      throw LoginException(
        LoginErrorCode.invalidField,
        '二维码字段格式错误：$key',
      );
    }
  }

  void _validateHexField(String value, String field) {
    final text = value.startsWith('0x') ? value.substring(2) : value;
    if (text.isEmpty || text.length.isOdd) {
      throw LoginException(
        LoginErrorCode.invalidField,
        '$field 必须是偶数字节 hex',
      );
    }
    if (!RegExp(r'^[0-9a-fA-F]+$').hasMatch(text)) {
      throw LoginException(
        LoginErrorCode.invalidField,
        '$field 必须是合法 hex',
      );
    }
  }

  int _nowEpochSeconds() => DateTime.now().millisecondsSinceEpoch ~/ 1000;

  String _hexEncode(List<int> bytes) {
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
