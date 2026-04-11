// wumin 冷钱包登录 QR 处理器:解析 WUMIN_QR_V1 login_challenge、
// 验证系统签名、构建 login_receipt envelope。

import 'dart:convert';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:sr25519/sr25519.dart' as sr25519;

import 'package:wumin/qr/qr_protocols.dart';
import 'package:wumin/qr/envelope.dart';
import 'package:wumin/qr/signature_message.dart';
import 'package:wumin/qr/bodies/login_challenge_body.dart';
import 'package:wumin/qr/bodies/login_receipt_body.dart';

typedef LoginChallengeEnvelope = QrEnvelope<LoginChallengeBody>;
typedef LoginReceiptEnvelope = QrEnvelope<LoginReceiptBody>;

/// 展示用辅助:从 LoginChallengeEnvelope 获取人可读系统名。
String loginSystemDisplayName(LoginChallengeEnvelope c) {
  switch (c.body.system.toLowerCase()) {
    case 'sfid':
      return 'SFID 身份系统';
    case 'cpms':
      return 'CPMS 机构系统';
    default:
      return c.body.system.toUpperCase();
  }
}

bool isLoginChallengeExpired(LoginChallengeEnvelope c) {
  final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
  return now > (c.expiresAt ?? 0);
}

const _maxTtlSeconds = 120;
const _maxClockSkewSeconds = 30;

/// 解析登录挑战 envelope。
LoginChallengeEnvelope parseLoginChallenge(String raw) {
  QrEnvelope<QrBody> env;
  try {
    env = QrEnvelope.parse(raw);
  } on FormatException catch (e) {
    throw LoginQrException('无法解析登录二维码: ${e.message}');
  }
  if (env.kind != QrKind.loginChallenge) {
    throw const LoginQrException('二维码类型不是登录挑战');
  }
  final body = env.body as LoginChallengeBody;

  final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
  if (now > (env.expiresAt ?? 0) + _maxClockSkewSeconds) {
    throw const LoginQrException('登录二维码已过期');
  }
  if ((env.expiresAt ?? 0) - (env.issuedAt ?? 0) > _maxTtlSeconds) {
    throw const LoginQrException('登录二维码有效期异常');
  }

  return QrEnvelope<LoginChallengeBody>(
    kind: QrKind.loginChallenge,
    id: env.id,
    issuedAt: env.issuedAt,
    expiresAt: env.expiresAt,
    body: body,
  );
}

/// 验证系统签名(确认 QR 确实由 SFID/CPMS 后端签发)。
bool verifySystemSignature(LoginChallengeEnvelope c) {
  final message = buildSignatureMessage(
    kind: QrKind.loginChallenge,
    id: c.id!,
    system: c.body.system,
    expiresAt: c.expiresAt,
    principal: c.body.sysPubkey,
  );
  return _verifySr25519Utf8(
    pubkeyHex: c.body.sysPubkey,
    signatureHex: c.body.sysSig,
    message: message,
  );
}

/// 构建用户签名消息(wumin 钱包用自己的私钥签这串)。
String buildSignMessage(LoginChallengeEnvelope c, String pubkeyHex) {
  return buildSignatureMessage(
    kind: QrKind.loginReceipt,
    id: c.id!,
    system: c.body.system,
    expiresAt: c.expiresAt,
    principal: pubkeyHex,
  );
}

/// 从签名结果构建 login_receipt envelope。
LoginReceiptEnvelope buildLoginReceipt({
  required LoginChallengeEnvelope challenge,
  required String pubkeyHex,
  required String signatureHex,
}) {
  final signMessage = buildSignMessage(challenge, pubkeyHex);
  final payloadHash =
      '0x${sha256.convert(utf8.encode(signMessage)).toString()}';
  return QrEnvelope<LoginReceiptBody>(
    kind: QrKind.loginReceipt,
    id: challenge.id,
    issuedAt: challenge.issuedAt,
    expiresAt: challenge.expiresAt,
    body: LoginReceiptBody(
      system: challenge.body.system,
      pubkey: pubkeyHex,
      sigAlg: 'sr25519',
      signature: signatureHex,
      payloadHash: payloadHash,
      signedAt: DateTime.now().millisecondsSinceEpoch ~/ 1000,
    ),
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

class LoginQrException implements Exception {
  final String message;
  const LoginQrException(this.message);
  @override
  String toString() => message;
}
