import 'dart:convert';
import 'dart:math';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:pointycastle/digests/blake2b.dart';
import 'package:sr25519/sr25519.dart' as sr25519;
import 'package:citizenwallet/qr/qr_protocols.dart';
import 'package:citizenwallet/qr/envelope.dart';
import 'package:citizenwallet/qr/bodies/sign_request_body.dart';
import 'package:citizenwallet/qr/bodies/sign_response_body.dart';

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
  static const int maxTtlSeconds = 300;
  static const int maxClockSkewSeconds = 30;
  static const int maxPayloadChars = 32768;
  static final RegExp _idPattern = RegExp(r'^[A-Za-z0-9_-]{16,128}$');

  static String generateRequestId({String prefix = ''}) {
    final random = Random.secure();
    final bytes = List<int>.generate(16, (_) => random.nextInt(256));
    final token = base64Url.encode(bytes).replaceAll('=', '');
    final id = prefix.isEmpty ? token : '$prefix$token';
    return id.length > 128 ? id.substring(0, 128) : id;
  }

  /// 解析 sign_request envelope(CitizenWallet 公民钱包从 CitizenApp 扫到的内容)。
  SignRequestEnvelope parseRequest(String raw) {
    if (raw.isEmpty || raw.length > maxPayloadChars) {
      throw const QrSignException(
        QrSignErrorCode.invalidFormat,
        '扫码数据格式错误:内容为空或超出长度限制',
      );
    }
    // 预检 kind:在完整 body 解析之前拦截非 sign_request,
    // 避免 body 结构不匹配导致的 FormatException 掩盖真实错误。
    try {
      final preview = jsonDecode(raw);
      if (preview is Map<String, dynamic>) {
        final kindWire = preview['k'];
        final kind = QrKind.fromWire(kindWire);
        if (kind != QrKind.signRequest) {
          throw const QrSignException(
              QrSignErrorCode.invalidField, '二维码类型不是签名请求');
        }
      }
    } on QrSignException {
      rethrow;
    } catch (_) {
      // JSON 解析失败等情况交给下面的 QrEnvelope.parse 统一报错
    }
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

  /// 构造 sign_response envelope(CitizenWallet 公民钱包签名完成后生成)。
  SignResponseEnvelope buildResponse({
    required SignRequestEnvelope request,
    required String signatureHex,
    int? nowEpochSeconds,
  }) {
    final requestBody = request.body;
    _validateHexField(signatureHex, 'signature');
    return QrEnvelope<SignResponseBody>(
      kind: QrKind.signResponse,
      id: request.id,
      issuedAt: request.issuedAt,
      expiresAt: request.expiresAt,
      body: SignResponseBody.fromHex(
        pubkeyHex: requestBody.pubkeyHex,
        signatureHex: signatureHex,
      ),
    );
  }

  String encodeResponse(SignResponseEnvelope response) => response.toRawJson();
  String encodeRequest(SignRequestEnvelope request) => request.toRawJson();

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
  /// payload <= 256B 时签原文,>256B 时签 blake2_256(payload)。
  static Uint8List signingBytesFor(SignRequestBody body) {
    final payload = body.payloadBytes;
    if (QrActions.isChainAction(body.action) && payload.length > 256) {
      final digest = Blake2bDigest(digestSize: 32)
        ..update(payload, 0, payload.length);
      final out = Uint8List(32);
      digest.doFinal(out, 0);
      return out;
    }
    return payload;
  }

  void _validateRequestId(String requestId) {
    if (!_idPattern.hasMatch(requestId)) {
      throw const QrSignException(QrSignErrorCode.invalidField, 'id 格式错误');
    }
  }

  void _validateHexField(String value, String field) {
    // sign_request 机读字段统一使用 0x hex,拒绝裸 hex。
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

  void _validateExpiry({
    required int expiresAt,
  }) {
    final now = _now();
    if (expiresAt < now) {
      throw const QrSignException(QrSignErrorCode.expired, '交易签名请求已过期');
    }
  }

  int _now() => DateTime.now().millisecondsSinceEpoch ~/ 1000;

  static List<int> _hexToBytes(String input) {
    final text = input.startsWith('0x') ? input.substring(2) : input;
    if (text.isEmpty || text.length.isOdd) return const <int>[];
    return List<int>.generate(
      text.length ~/ 2,
      (i) => int.parse(text.substring(i * 2, i * 2 + 2), radix: 16),
      growable: false,
    );
  }
}
