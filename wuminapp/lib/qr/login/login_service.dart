import 'dart:convert';

import 'package:wuminapp_mobile/qr/login/login_models.dart';
import 'package:wuminapp_mobile/qr/login/login_replay_guard.dart';
import 'package:wuminapp_mobile/qr/qr_protocols.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

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
  })  : _walletManager = walletManager ?? WalletManager(),
        _replayGuard = replayGuard ?? LoginReplayGuard();

  static const int challengeTtlSeconds = 90;
  static const int maxClockSkewSeconds = 30;
  static const int maxChallengePayloadChars = 4096;
  static const Set<String> allowedSystems = {'cpms', 'sfid'};
  static final RegExp _idPattern = RegExp(r'^[A-Za-z0-9._:-]{4,128}$');

  final WalletManager _walletManager;
  final LoginReplayGuard _replayGuard;

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

    final requestId = _requiredString(data, 'request_id');
    final challenge = _requiredString(data, 'challenge');
    final nonce = _requiredString(data, 'nonce');
    final issuedAt = _requiredInt(data, 'issued_at');
    final expiresAt = _requiredInt(data, 'expires_at');
    final sysPubkey = _requiredString(data, 'sys_pubkey');
    final sysSig = _requiredString(data, 'sys_sig');
    final sysCert = data['sys_cert']?.toString().trim();

    _validateIdField('request_id', requestId);
    _validateOpaqueField('challenge', challenge);
    _validateIdField('nonce', nonce);
    _validateHexField(sysPubkey, 'sys_pubkey');
    _validateHexField(sysSig, 'sys_sig');
    if (system == 'cpms') {
      if (sysCert == null || sysCert.isEmpty) {
        throw const LoginException(
          LoginErrorCode.missingField,
          'CPMS 登录必须包含 sys_cert 字段',
        );
      }
      _validateHexField(sysCert, 'sys_cert');
    }

    final challengeData = LoginChallenge(
      proto: proto,
      system: system,
      requestId: requestId,
      challenge: challenge,
      nonce: nonce,
      issuedAt: issuedAt,
      expiresAt: expiresAt,
      sysPubkey: sysPubkey,
      sysSig: sysSig,
      sysCert: (sysCert != null && sysCert.isNotEmpty) ? sysCert : null,
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

  /// 验证系统签名与信任链。
  ///
  /// - SFID：验证 `sys_sig`，并检查 `sys_pubkey` 是否匹配链上注册的 SFID 公钥。
  /// - CPMS：验证 `sys_sig`，并通过 `sys_cert` 验证 SFID 对该 CPMS 公钥的背书。
  ///
  /// 当前阶段：框架预留，实际验签逻辑在链上公钥缓存模块就绪后接入。
  Future<void> validateSystemSignature(LoginChallenge challenge) async {
    // TODO: 接入 sr25519 验签
    // 1. 重组签名原文：proto|system|request_id|challenge|nonce|issued_at|expires_at
    // 2. 用 challenge.sysPubkey 验证 challenge.sysSig
    // 3. SFID 场景：检查 sysPubkey == 链上缓存的 SFID 公钥
    // 4. CPMS 场景：用 SFID 公钥验证 sysCert 对 sysPubkey 的背书
  }

  // ---------------------------------------------------------------------------
  // 签名原文
  // ---------------------------------------------------------------------------

  /// 构建用户签名原文。
  String buildSignMessage(LoginChallenge challenge) {
    return [
      QrProtocols.login,
      challenge.system,
      challenge.requestId,
      challenge.challenge,
      challenge.nonce,
      challenge.expiresAt.toString(),
    ].join('|');
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
    await _replayGuard.assertNotConsumed(challenge.requestId);

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
      if (active.isColdWallet) {
        throw const LoginException(
          LoginErrorCode.walletMissing,
          '当前为冷钱包，请使用扫码签名',
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
      requestId: challenge.requestId,
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
    await _replayGuard.assertNotConsumed(challenge.requestId);

    final receipt = LoginReceipt(
      proto: QrProtocols.login,
      requestId: challenge.requestId,
      pubkey: pubkeyHex,
      sigAlg: sigAlg,
      signature: signatureHex,
      signedAt: DateTime.now().millisecondsSinceEpoch ~/ 1000,
    );
    await _replayGuard.consume(
      requestId: challenge.requestId,
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

  void _validateIdField(String key, String value) {
    if (!_idPattern.hasMatch(value)) {
      throw LoginException(
        LoginErrorCode.invalidField,
        '二维码字段格式错误：$key',
      );
    }
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
}
