// CitizenWallet 公民钱包登录 QR 处理器:解析 QR_V1 登录签名请求、
// 验证系统签名、构建 QR_V1 签名响应。

import 'dart:convert';
import 'dart:typed_data';

import 'package:sr25519/sr25519.dart' as sr25519;

import 'package:citizenwallet/qr/qr_protocols.dart';
import 'package:citizenwallet/qr/envelope.dart';
import 'package:citizenwallet/qr/signature_message.dart';
import 'package:citizenwallet/qr/bodies/sign_request_body.dart';
import 'package:citizenwallet/qr/bodies/sign_response_body.dart';

typedef LoginSignRequestEnvelope = QrEnvelope<SignRequestBody>;
typedef LoginSignResponseEnvelope = QrEnvelope<SignResponseBody>;

/// 展示用辅助:从登录签名请求中获取人可读系统名。
String loginSystemDisplayName(LoginSignRequestEnvelope c) {
  switch (_loginData(c).system.toLowerCase()) {
    case 'cid':
      return 'CID 身份系统';
    default:
      return _loginData(c).system.toUpperCase();
  }
}

bool isLoginSignRequestExpired(LoginSignRequestEnvelope c) {
  final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
  return now > (c.expiresAt ?? 0);
}

const _maxClockSkewSeconds = 30;

/// 解析登录签名请求 envelope。
LoginSignRequestEnvelope parseLoginSignRequest(String raw) {
  QrEnvelope<QrBody> env;
  try {
    env = QrEnvelope.parse(raw);
  } on FormatException catch (e) {
    throw LoginQrException('无法解析登录二维码: ${e.message}');
  }
  if (env.kind != QrKind.signRequest) {
    throw const LoginQrException('二维码类型不是签名请求');
  }
  final body = env.body as SignRequestBody;
  if (body.action != QrActions.login) {
    throw const LoginQrException('二维码不是登录签名请求');
  }

  final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
  if (now > (env.expiresAt ?? 0) + _maxClockSkewSeconds) {
    throw const LoginQrException('登录二维码已过期');
  }
  _loginData(QrEnvelope<SignRequestBody>(
    kind: QrKind.signRequest,
    id: env.id,
    issuedAt: env.issuedAt,
    expiresAt: env.expiresAt,
    body: body,
  ));

  return QrEnvelope<SignRequestBody>(
    kind: QrKind.signRequest,
    id: env.id,
    issuedAt: env.issuedAt,
    expiresAt: env.expiresAt,
    body: body,
  );
}

/// 验证系统签名(确认 QR 确实由 CID 后端签发)。
bool verifySystemSignature(LoginSignRequestEnvelope c) {
  final data = _loginData(c);
  final message = buildSignatureMessage(
    kind: QrKind.signRequest,
    id: c.id!,
    system: data.system,
    expiresAt: c.expiresAt,
    principal: c.body.pubkeyHex,
  );
  return _verifySr25519Utf8(
    pubkeyHex: c.body.pubkeyHex,
    signatureHex: data.sysSig,
    message: message,
  );
}

/// 构建用户签名消息(CitizenWallet 钱包用自己的私钥签这串)。
String buildSignMessage(LoginSignRequestEnvelope c, String pubkeyHex) {
  final data = _loginData(c);
  return buildSignatureMessage(
    kind: QrKind.signResponse,
    id: c.id!,
    system: data.system,
    expiresAt: c.expiresAt,
    principal: pubkeyHex,
  );
}

/// 从签名结果构建 QR_V1 k=2 登录签名响应 envelope。
LoginSignResponseEnvelope buildLoginSignResponse({
  required LoginSignRequestEnvelope request,
  required String pubkeyHex,
  required String signatureHex,
}) {
  return QrEnvelope<SignResponseBody>(
    kind: QrKind.signResponse,
    id: request.id,
    issuedAt: request.issuedAt,
    expiresAt: request.expiresAt,
    body: SignResponseBody.fromHex(
      pubkeyHex: pubkeyHex,
      signatureHex: signatureHex,
    ),
  );
}

// ---------------------------------------------------------------------------
// 内部工具
// ---------------------------------------------------------------------------

class _LoginRequestData {
  const _LoginRequestData({required this.system, required this.sysSig});

  final String system;
  final String sysSig;
}

_LoginRequestData _loginData(LoginSignRequestEnvelope c) {
  final text = utf8.decode(c.body.payloadBytes, allowMalformed: false);
  final parts = text.split('|');
  if (parts.length != 2 || parts[0] != 'cid') {
    throw const LoginQrException('登录二维码载荷无效');
  }
  if (!parts[1].startsWith('0x')) {
    throw const LoginQrException('登录二维码系统签名无效');
  }
  return _LoginRequestData(system: parts[0], sysSig: parts[1]);
}

bool _verifySr25519Utf8({
  required String pubkeyHex,
  required String signatureHex,
  required String message,
}) {
  try {
    final pubBytes = Uint8List.fromList(_hexToBytes(_normalizeHex(pubkeyHex)));
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
