import 'dart:convert';
import 'dart:math';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:sr25519/sr25519.dart' as sr25519;
import 'package:citizenapp/qr/bodies/sign_request_body.dart';
import 'package:citizenapp/qr/bodies/sign_response_body.dart';
import 'package:citizenapp/qr/envelope.dart';
import 'package:citizenapp/qr/qr_protocols.dart';

enum QrSignErrorCode {
  invalidFormat,
  invalidField,
  invalidProtocol,
  expired,
  mismatchedRequest,
  mismatchedAccount,
  mismatchedPubkey,
  mismatchedPayloadHash,
  invalidSignature,
}

class QrSignException implements Exception {
  const QrSignException(this.code, this.message);

  final QrSignErrorCode code;
  final String message;

  @override
  String toString() => message;
}

typedef SignRequestEnvelope = QrEnvelope<SignRequestBody>;
typedef SignResponseEnvelope = QrEnvelope<SignResponseBody>;

class QrSigner {
  static const int defaultTtlSeconds = 90;
  static const int maxPayloadChars = 32768;
  static final RegExp _idPattern = RegExp(r'^[A-Za-z0-9_-]{16,128}$');

  /// 生成加密安全的随机 request id。base64url 比 hex 短,可降低二维码密度。
  static String generateRequestId({String prefix = ''}) {
    final random = Random.secure();
    final bytes = List<int>.generate(16, (_) => random.nextInt(256));
    final token = base64Url.encode(bytes).replaceAll('=', '');
    final id = prefix.isEmpty ? token : '$prefix$token';
    return id.length > 128 ? id.substring(0, 128) : id;
  }

  /// 构造 QR_V1 签名请求。
  SignRequestEnvelope buildRequest({
    required String requestId,
    required String pubkey,
    required String payloadHex,
    required int action,
    int? nowEpochSeconds,
    int ttlSeconds = defaultTtlSeconds,
  }) {
    final now = nowEpochSeconds ?? _now();
    _validateRequestId(requestId);
    _validateHexField(pubkey, 'pubkey');
    _validateHexField(payloadHex, 'payload');
    return QrEnvelope<SignRequestBody>(
      kind: QrKind.signRequest,
      id: requestId,
      issuedAt: now,
      expiresAt: now + ttlSeconds,
      body: SignRequestBody.fromHex(
        action: action,
        pubkeyHex: pubkey,
        payloadHex: payloadHex,
      ),
    );
  }

  String encodeRequest(SignRequestEnvelope request) => request.toRawJson();

  String encodeResponse(SignResponseEnvelope response) => response.toRawJson();

  SignRequestEnvelope parseRequest(String raw) {
    QrEnvelope<QrBody> env;
    try {
      env = QrEnvelope.parse(raw);
    } on FormatException catch (e) {
      throw QrSignException(QrSignErrorCode.invalidFormat, e.message);
    }
    if (env.kind != QrKind.signRequest) {
      throw const QrSignException(QrSignErrorCode.invalidField, '二维码类型不是签名请求');
    }
    final body = env.body as SignRequestBody;
    _validateRequestId(env.id!);
    _validateExpiry(expiresAt: env.expiresAt!);
    return QrEnvelope<SignRequestBody>(
      kind: QrKind.signRequest,
      id: env.id,
      issuedAt: env.issuedAt,
      expiresAt: env.expiresAt,
      body: body,
    );
  }

  /// 解析签名响应。QR_V1 响应不再携带 payload hash,生成端必须用 request id
  /// 找回本地 session 中的 action/payload/pubkey 后验签。
  SignResponseEnvelope parseResponse(
    String raw, {
    required String expectedRequestId,
    String? expectedPubkey,
    String? expectedPayloadHash,
    String? expectedPayloadHex,
    int? expectedAction,
  }) {
    QrEnvelope<QrBody> env;
    try {
      env = QrEnvelope.parse(raw);
    } on FormatException catch (e) {
      throw QrSignException(QrSignErrorCode.invalidFormat, e.message);
    }
    if (env.kind != QrKind.signResponse) {
      throw const QrSignException(QrSignErrorCode.invalidField, '二维码类型不是签名响应');
    }
    final body = env.body as SignResponseBody;
    _validateRequestId(env.id!);

    if (env.id != expectedRequestId) {
      throw const QrSignException(
        QrSignErrorCode.mismatchedRequest,
        '签名响应 id 与请求不一致',
      );
    }
    if (expectedPubkey != null &&
        _normalizeHex(body.pubkeyHex) != _normalizeHex(expectedPubkey)) {
      throw const QrSignException(
        QrSignErrorCode.mismatchedPubkey,
        '签名响应公钥与当前选中钱包不一致',
      );
    }
    if (expectedPayloadHash != null && expectedPayloadHex != null) {
      final currentHash = computePayloadHash(expectedPayloadHex);
      if (_normalizeHex(currentHash) != _normalizeHex(expectedPayloadHash)) {
        throw const QrSignException(
          QrSignErrorCode.mismatchedPayloadHash,
          '本地签名 session 的 payload hash 不一致',
        );
      }
    }
    if (expectedPayloadHex != null) {
      final message = signingBytesForHex(
        payloadHex: expectedPayloadHex,
        action: expectedAction ?? 0,
      );
      if (!verifySr25519Signature(
        pubkeyHex: body.pubkeyHex,
        signatureHex: body.signatureHex,
        message: message,
      )) {
        throw const QrSignException(
          QrSignErrorCode.invalidSignature,
          '签名验证失败:签名与 payload 不匹配,请重新签名',
        );
      }
    }
    return QrEnvelope<SignResponseBody>(
      kind: QrKind.signResponse,
      id: env.id,
      issuedAt: env.issuedAt,
      expiresAt: env.expiresAt,
      body: body,
    );
  }

  static String computePayloadHash(String payloadHex) {
    final bytes = _hexToBytes(payloadHex);
    final digest = sha256.convert(bytes);
    return '0x${digest.toString()}';
  }

  static bool verifySr25519Signature({
    required String pubkeyHex,
    required String signatureHex,
    required Uint8List message,
  }) {
    try {
      final pubBytes = Uint8List.fromList(_hexToBytes(pubkeyHex));
      final sigBytes = Uint8List.fromList(_hexToBytes(signatureHex));
      final publicKey = sr25519.PublicKey.newPublicKey(pubBytes);
      final signature = sr25519.Signature.fromBytes(sigBytes);
      final (verified, _) =
          sr25519.Sr25519.verify(publicKey, signature, message);
      return verified;
    } catch (_) {
      return false;
    }
  }

  /// Substrate 交易签名必须复刻 SignedPayload::using_encoded:
  /// payload <= 256B 签原文,>256B 签 blake2_256(payload)。
  static Uint8List signingBytesForHex({
    required String payloadHex,
    required int action,
  }) {
    final payload = Uint8List.fromList(_hexToBytes(payloadHex));
    if (QrActions.isChainAction(action) && payload.length > 256) {
      return Hasher.blake2b256.hash(payload);
    }
    return payload;
  }

  void _validateRequestId(String requestId) {
    if (!_idPattern.hasMatch(requestId)) {
      throw const QrSignException(QrSignErrorCode.invalidField, 'id 格式错误');
    }
  }

  void _validateHexField(String value, String field) {
    if (!value.startsWith('0x')) {
      throw QrSignException(QrSignErrorCode.invalidField, '$field 必须以 0x 开头');
    }
    final text = value.substring(2);
    if (text.isEmpty || text.length.isOdd) {
      throw QrSignException(QrSignErrorCode.invalidField, '$field 必须是偶数字节 hex');
    }
    if (!RegExp(r'^[0-9a-fA-F]+$').hasMatch(text)) {
      throw QrSignException(QrSignErrorCode.invalidField, '$field 必须是合法 hex');
    }
  }

  String _normalizeHex(String value) {
    return value.startsWith('0x')
        ? value.substring(2).toLowerCase()
        : value.toLowerCase();
  }

  void _validateExpiry({required int expiresAt}) {
    final now = _now();
    if (expiresAt < now) {
      throw const QrSignException(QrSignErrorCode.expired, '交易签名请求已过期');
    }
  }

  int _now() => DateTime.now().millisecondsSinceEpoch ~/ 1000;

  static List<int> _hexToBytes(String input) {
    final text = input.startsWith('0x') || input.startsWith('0X')
        ? input.substring(2)
        : input;
    if (text.isEmpty || text.length.isOdd) return const <int>[];
    return List<int>.generate(
      text.length ~/ 2,
      (i) => int.parse(text.substring(i * 2, i * 2 + 2), radix: 16),
      growable: false,
    );
  }
}
