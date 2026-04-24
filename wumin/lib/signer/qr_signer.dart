import 'dart:convert';
import 'dart:math';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:sr25519/sr25519.dart' as sr25519;
import 'package:wumin/qr/qr_protocols.dart';
import 'package:wumin/qr/envelope.dart';
import 'package:wumin/qr/bodies/sign_request_body.dart';
import 'package:wumin/qr/bodies/sign_response_body.dart';

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
  static final RegExp _idPattern = RegExp(r'^[A-Za-z0-9._:-]{16,128}$');
  static final RegExp _addressPattern =
      RegExp(r'^[1-9A-HJ-NP-Za-km-z]{30,80}$');

  static String generateRequestId({String prefix = ''}) {
    final random = Random.secure();
    final bytes = List<int>.generate(16, (_) => random.nextInt(256));
    final hex = bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
    final id = prefix.isEmpty ? hex : '$prefix$hex';
    return id.length > 128 ? id.substring(0, 128) : id;
  }

  /// 解析 sign_request envelope(wumin 冷钱包从 wuminapp 扫到的内容)。
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
        final kindWire = preview['kind'];
        if (kindWire is String && kindWire != QrKind.signRequest.wire) {
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
      throw const QrSignException(
          QrSignErrorCode.invalidField, '二维码类型不是签名请求');
    }
    final body = env.body as SignRequestBody;
    _validateRequestId(env.id!);
    _validateAddress(body.address);
    _validateExpiry(issuedAt: env.issuedAt!, expiresAt: env.expiresAt!);
    return QrEnvelope<SignRequestBody>(
      kind: QrKind.signRequest,
      id: env.id,
      issuedAt: env.issuedAt,
      expiresAt: env.expiresAt,
      body: body,
    );
  }

  /// 构造 sign_response envelope(wumin 冷钱包签名完成后生成)。
  SignResponseEnvelope buildResponse({
    required SignRequestEnvelope request,
    required String signatureHex,
    int? nowEpochSeconds,
  }) {
    final now = nowEpochSeconds ?? _now();
    final requestBody = request.body;
    final payloadHash = computePayloadHash(requestBody.payloadHex);
    _validateHexField(signatureHex, 'signature');
    return QrEnvelope<SignResponseBody>(
      kind: QrKind.signResponse,
      id: request.id,
      issuedAt: request.issuedAt,
      expiresAt: request.expiresAt,
      body: SignResponseBody(
        pubkey: requestBody.pubkey,
        sigAlg: 'sr25519',
        signature: signatureHex,
        payloadHash: payloadHash,
        signedAt: now,
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
    required String payloadHex,
  }) {
    try {
      final pubBytes = Uint8List.fromList(_hexToBytes(pubkeyHex));
      final sigBytes = Uint8List.fromList(_hexToBytes(signatureHex));
      final msgBytes = Uint8List.fromList(_hexToBytes(payloadHex));
      final publicKey = sr25519.PublicKey.newPublicKey(pubBytes);
      final signature = sr25519.Signature.fromBytes(sigBytes);
      final (verified, _) =
          sr25519.Sr25519.verify(publicKey, signature, msgBytes);
      return verified;
    } catch (_) {
      return false;
    }
  }

  void _validateRequestId(String requestId) {
    if (!_idPattern.hasMatch(requestId)) {
      throw const QrSignException(
          QrSignErrorCode.invalidField, 'id 格式错误');
    }
  }

  void _validateAddress(String address) {
    if (!_addressPattern.hasMatch(address)) {
      throw const QrSignException(
          QrSignErrorCode.invalidField, 'address 格式错误');
    }
  }

  void _validateHexField(String value, String field) {
    // feedback_pubkey_format_rule 铁律: 内部统一 0x 小写 hex。
    // 拒绝裸 hex, 强制 0x 前缀。SignRequestBody.fromJson 同要求。
    if (!value.startsWith('0x')) {
      throw QrSignException(
          QrSignErrorCode.invalidField, '$field 必须以 0x 开头');
    }
    final text = value.substring(2);
    if (text.isEmpty || text.length.isOdd) {
      throw QrSignException(
          QrSignErrorCode.invalidField, '$field 必须是偶数字节 hex');
    }
    if (!RegExp(r'^[0-9a-fA-F]+$').hasMatch(text)) {
      throw QrSignException(
          QrSignErrorCode.invalidField, '$field 必须是合法 hex');
    }
  }

  void _validateExpiry({
    required int issuedAt,
    required int expiresAt,
  }) {
    final now = _now();
    if (expiresAt <= issuedAt) {
      throw const QrSignException(
          QrSignErrorCode.invalidField, 'expires_at 必须晚于 issued_at');
    }
    if (expiresAt - issuedAt > maxTtlSeconds) {
      throw const QrSignException(
          QrSignErrorCode.invalidField, 'TTL 超过 300 秒');
    }
    if (issuedAt > now + maxClockSkewSeconds) {
      throw const QrSignException(
          QrSignErrorCode.invalidField, 'issued_at 超出设备时间范围');
    }
    if (expiresAt < now) {
      throw const QrSignException(
          QrSignErrorCode.expired, '交易签名请求已过期');
    }
  }

  int _now() => DateTime.now().millisecondsSinceEpoch ~/ 1000;

  static List<int> _hexToBytes(String input) {
    final text =
        input.startsWith('0x') ? input.substring(2) : input;
    if (text.isEmpty || text.length.isOdd) return const <int>[];
    return List<int>.generate(
      text.length ~/ 2,
      (i) => int.parse(text.substring(i * 2, i * 2 + 2), radix: 16),
      growable: false,
    );
  }
}
