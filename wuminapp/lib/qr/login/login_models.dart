import 'package:wuminapp_mobile/qr/qr_protocols.dart';

/// 登录挑战码数据模型（系统 → 手机）。
class LoginChallenge {
  const LoginChallenge({
    required this.proto,
    required this.system,
    required this.requestId,
    required this.challenge,
    required this.nonce,
    required this.issuedAt,
    required this.expiresAt,
    required this.sysPubkey,
    required this.sysSig,
    this.sysCert,
    required this.raw,
  });

  /// 协议标识，固定 `WUMINAPP_LOGIN_V1`。
  final String proto;

  /// 目标系统：`sfid` 或 `cpms`。
  final String system;

  /// 请求 ID，一次性，防重放。
  final String requestId;

  /// 随机挑战值。
  final String challenge;

  /// 随机数，增加签名随机性。
  final String nonce;

  /// 签发时间（秒级 epoch）。
  final int issuedAt;

  /// 过期时间（秒级 epoch），TTL 固定 90 秒。
  final int expiresAt;

  /// 系统公钥（0x + hex）。
  ///
  /// - SFID：SFID 服务器自身公钥，手机可通过区块链验证。
  /// - CPMS：该 CPMS 实例自身公钥。
  final String sysPubkey;

  /// 系统对挑战字段的签名（0x + hex）。
  ///
  /// 签名原文：`proto|system|request_id|challenge|nonce|issued_at|expires_at`
  final String sysSig;

  /// SFID 对 CPMS 公钥的背书签名（0x + hex）。
  ///
  /// 仅 `system == cpms` 时必填。
  /// 手机通过验证 `sysCert` 确认该 CPMS 实例被 SFID 信任。
  final String? sysCert;

  /// 原始扫码字符串。
  final String raw;

  bool get isExpired => _nowEpochSeconds() > expiresAt;
  int get ttlSeconds => expiresAt - _nowEpochSeconds();

  /// 是否需要 SFID 证书链验证（CPMS 场景）。
  bool get requiresCert => system == 'cpms';

  static int _nowEpochSeconds() =>
      DateTime.now().millisecondsSinceEpoch ~/ 1000;
}

/// 登录回执码数据模型（手机 → 系统）。
class LoginReceipt {
  const LoginReceipt({
    required this.proto,
    required this.requestId,
    required this.pubkey,
    required this.sigAlg,
    required this.signature,
    required this.signedAt,
  });

  final String proto;
  final String requestId;
  final String pubkey;
  final String sigAlg;
  final String signature;
  final int signedAt;

  Map<String, dynamic> toJson() {
    return {
      'proto': proto,
      'request_id': requestId,
      'pubkey': pubkey,
      'sig_alg': sigAlg,
      'signature': signature,
      'signed_at': signedAt,
    };
  }
}

/// 登录模块错误码。
class LoginErrorCode {
  LoginErrorCode._();

  static const String invalidFormat = 'L1001';
  static const String invalidProtocol = 'L1002';
  static const String invalidSystem = 'L1003';
  static const String missingField = 'L1004';
  static const String invalidField = 'L1005';
  static const String expired = 'L1101';
  static const String replay = 'L1102';
  static const String invalidTtl = 'L1103';
  static const String invalidSystemSignature = 'L1201';
  static const String untrustedSystem = 'L1202';
  static const String walletMissing = 'L1301';
  static const String walletNotFound = 'L1302';
  static const String walletMismatch = 'L1303';
  static const String biometricUnavailable = 'L1401';
  static const String biometricRejected = 'L1402';
}

/// 登录模块异常。
class LoginException implements Exception {
  const LoginException(this.code, this.message);

  final String code;
  final String message;

  @override
  String toString() => '[$code] $message';
}
