// wumin 冷钱包登录 QR 处理器：解析 WUMIN_LOGIN_V1.0.0 挑战、验证系统签名、构建 receipt。

import 'dart:convert';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:sr25519/sr25519.dart' as sr25519;

import '../qr/qr_protocols.dart';

/// 登录挑战 QR 解析结果。
class LoginChallenge {
  final String proto;
  final String system;
  final String challenge;
  final int issuedAt;
  final int expiresAt;
  final String sysPubkey;
  final String sysSig;
  final String raw;

  const LoginChallenge({
    required this.proto,
    required this.system,
    required this.challenge,
    required this.issuedAt,
    required this.expiresAt,
    required this.sysPubkey,
    required this.sysSig,
    required this.raw,
  });

  bool get isExpired =>
      DateTime.now().millisecondsSinceEpoch ~/ 1000 > expiresAt;

  String get systemDisplayName {
    switch (system.toLowerCase()) {
      case 'sfid':
        return 'SFID 身份系统';
      case 'cpms':
        return 'CPMS 机构系统';
      default:
        return system.toUpperCase();
    }
  }
}

/// 登录 receipt（签名结果）。
class LoginReceipt {
  final String proto;
  final String system;
  final String challenge;
  final String pubkey;
  final String sigAlg;
  final String signature;
  final String payloadHash;
  final int signedAt;

  const LoginReceipt({
    required this.proto,
    required this.system,
    required this.challenge,
    required this.pubkey,
    required this.sigAlg,
    required this.signature,
    required this.payloadHash,
    required this.signedAt,
  });

  Map<String, dynamic> toJson() => {
        'proto': proto,
        'type': 'login_receipt',
        'system': system,
        'challenge': challenge,
        'pubkey': pubkey,
        'sig_alg': sigAlg,
        'signature': signature,
        'payload_hash': payloadHash,
        'signed_at': signedAt,
      };

  String toJsonString() => jsonEncode(toJson());
}

const _allowedSystems = {'sfid', 'cpms'};
const _maxTtlSeconds = 120;
const _maxClockSkewSeconds = 30;

/// 解析登录挑战 QR 码。
LoginChallenge parseLoginChallenge(String raw) {
  final Map<String, dynamic> data;
  try {
    data = jsonDecode(raw) as Map<String, dynamic>;
  } catch (_) {
    throw const LoginQrException('无法解析登录二维码');
  }

  final proto = _requiredString(data, 'proto');
  if (proto != QrProtocols.login) {
    throw const LoginQrException('不支持的登录协议');
  }

  final type = _requiredString(data, 'type');
  if (type != 'challenge') {
    throw const LoginQrException('二维码类型不是登录挑战');
  }

  final system = _requiredString(data, 'system').toLowerCase();
  if (!_allowedSystems.contains(system)) {
    throw LoginQrException('不支持的系统: $system');
  }

  final challenge = _requiredString(data, 'challenge');
  final issuedAt = _requiredInt(data, 'issued_at');
  final expiresAt = _requiredInt(data, 'expires_at');
  final sysPubkey = _requiredString(data, 'sys_pubkey');
  final sysSig = _requiredString(data, 'sys_sig');

  final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
  if (now > expiresAt + _maxClockSkewSeconds) {
    throw const LoginQrException('登录二维码已过期');
  }
  if (expiresAt - issuedAt > _maxTtlSeconds) {
    throw const LoginQrException('登录二维码有效期异常');
  }

  return LoginChallenge(
    proto: proto,
    system: system,
    challenge: challenge,
    issuedAt: issuedAt,
    expiresAt: expiresAt,
    sysPubkey: sysPubkey,
    sysSig: sysSig,
    raw: raw,
  );
}

/// 验证系统签名（确认 QR ���实由 SFID/CPMS 后端签发）。
bool verifySystemSignature(LoginChallenge c) {
  // 中文注释：签名消息中的 sys_pubkey 必须与 SFID 后端签名时使用的格式完全一致。
  // SFID 后端使用 state.public_key_hex（带 0x 前缀），因此这里也直接使用 QR 中的原始值。
  final message = [
    c.proto,
    c.system,
    c.challenge,
    c.issuedAt.toString(),
    c.expiresAt.toString(),
    c.sysPubkey,
  ].join('|');

  return _verifySr25519Utf8(
    pubkeyHex: c.sysPubkey,
    signatureHex: c.sysSig,
    message: message,
  );
}

/// 构建用户���名消息。
String buildSignMessage(LoginChallenge c) {
  return [
    QrProtocols.login,
    c.system,
    c.challenge,
    c.expiresAt.toString(),
  ].join('|');
}

/// 从签名结果构建 receipt。
LoginReceipt buildLoginReceipt({
  required LoginChallenge challenge,
  required String pubkeyHex,
  required String signatureHex,
}) {
  final signMessage = buildSignMessage(challenge);
  final payloadHash = sha256.convert(utf8.encode(signMessage)).toString();
  return LoginReceipt(
    proto: QrProtocols.login,
    system: challenge.system,
    challenge: challenge.challenge,
    pubkey: pubkeyHex,
    sigAlg: 'sr25519',
    signature: signatureHex,
    payloadHash: payloadHash,
    signedAt: DateTime.now().millisecondsSinceEpoch ~/ 1000,
  );
}

// ---------------------------------------------------------------------------
// 内部工具
// ---------------------------------------------------------------------------

bool _verifySr25519Utf8({
  required String pubkeyHex,
  required String signatureHex,
  required String message,
}) {
  try {
    final pubBytes =
        Uint8List.fromList(_hexToBytes(_normalizeHex(pubkeyHex)));
    final sigBytes =
        Uint8List.fromList(_hexToBytes(_normalizeHex(signatureHex)));
    final msgBytes = Uint8List.fromList(utf8.encode(message));
    final publicKey = sr25519.PublicKey.newPublicKey(pubBytes);
    final signature = sr25519.Signature.fromBytes(sigBytes);
    final (verified, _) =
        sr25519.Sr25519.verify(publicKey, signature, msgBytes);
    return verified;
  } catch (_) {
    return false;
  }
}

String _normalizeHex(String hex) {
  final h = hex.trim().toLowerCase();
  return h.startsWith('0x') ? h.substring(2) : h;
}

List<int> _hexToBytes(String hex) {
  final h = _normalizeHex(hex);
  final result = <int>[];
  for (var i = 0; i < h.length; i += 2) {
    result.add(int.parse(h.substring(i, i + 2), radix: 16));
  }
  return result;
}

String _requiredString(Map<String, dynamic> data, String key) {
  final value = data[key];
  if (value is! String || value.trim().isEmpty) {
    throw LoginQrException('登录二维码缺少字段: $key');
  }
  return value.trim();
}

int _requiredInt(Map<String, dynamic> data, String key) {
  final value = data[key];
  if (value is int) return value;
  if (value is num) return value.toInt();
  throw LoginQrException('登录二维码缺少字段: $key');
}

class LoginQrException implements Exception {
  final String message;
  const LoginQrException(this.message);
  @override
  String toString() => message;
}
