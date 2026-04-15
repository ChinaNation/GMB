import 'dart:math';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:sr25519/sr25519.dart' as sr25519;
import 'package:wuminapp_mobile/qr/qr_protocols.dart';
import 'package:wuminapp_mobile/qr/envelope.dart';
import 'package:wuminapp_mobile/qr/bodies/sign_request_body.dart';
import 'package:wuminapp_mobile/qr/bodies/sign_response_body.dart';

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

/// 交易签名请求/回执的 envelope 便利别名。
typedef SignRequestEnvelope = QrEnvelope<SignRequestBody>;
typedef SignResponseEnvelope = QrEnvelope<SignResponseBody>;

class QrSigner {
  static const int defaultTtlSeconds = 90;
  static const int maxClockSkewSeconds = 30;
  static const int maxPayloadChars = 32768;
  static final RegExp _idPattern = RegExp(r'^[A-Za-z0-9._:-]{16,128}$');
  static final RegExp _addressPattern =
      RegExp(r'^[1-9A-HJ-NP-Za-km-z]{30,80}$');

  /// 生成加密安全的随机 request ID(32 字符 hex)。
  static String generateRequestId({String prefix = ''}) {
    final random = Random.secure();
    final bytes = List<int>.generate(16, (_) => random.nextInt(256));
    final hex = bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
    final id = prefix.isEmpty ? hex : '$prefix$hex';
    return id.length > 128 ? id.substring(0, 128) : id;
  }

  /// 构造 sign_request envelope(wuminapp 热钱包调用)。
  SignRequestEnvelope buildRequest({
    required String requestId,
    required String address,
    required String pubkey,
    required String payloadHex,
    required SignDisplay display,
    required int specVersion,
    int? nowEpochSeconds,
    int ttlSeconds = defaultTtlSeconds,
  }) {
    final now = nowEpochSeconds ?? _now();
    _validateRequestId(requestId);
    _validateAddress(address);
    _validateHexField(pubkey, 'pubkey');
    _validateHexField(payloadHex, 'payload_hex');
    return QrEnvelope<SignRequestBody>(
      kind: QrKind.signRequest,
      id: requestId,
      issuedAt: now,
      expiresAt: now + ttlSeconds,
      body: SignRequestBody(
        address: address,
        pubkey: pubkey,
        sigAlg: 'sr25519',
        payloadHex: payloadHex,
        specVersion: specVersion,
        display: display,
      ),
    );
  }

  String encodeRequest(SignRequestEnvelope request) => request.toRawJson();

  String encodeResponse(SignResponseEnvelope response) => response.toRawJson();

  /// 解析 sign_request envelope(wumin 冷钱包调用)。
  SignRequestEnvelope parseRequest(String raw) {
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

  /// 解析 sign_response envelope(wuminapp 热钱包扫回)。
  SignResponseEnvelope parseResponse(
    String raw, {
    required String expectedRequestId,
    String? expectedPubkey,
    String? expectedPayloadHash,
    String? expectedPayloadHex,
  }) {
    QrEnvelope<QrBody> env;
    try {
      env = QrEnvelope.parse(raw);
    } on FormatException catch (e) {
      throw QrSignException(QrSignErrorCode.invalidFormat, e.message);
    }
    if (env.kind != QrKind.signResponse) {
      throw const QrSignException(
          QrSignErrorCode.invalidField, '二维码类型不是签名回执');
    }
    final body = env.body as SignResponseBody;
    _validateRequestId(env.id!);
    _validateSignedAt(body.signedAt);

    if (env.id != expectedRequestId) {
      throw const QrSignException(
        QrSignErrorCode.mismatchedRequest,
        '签名回执 id 与请求不一致',
      );
    }
    if (expectedPubkey != null) {
      if (_normalizeHex(body.pubkey) != _normalizeHex(expectedPubkey)) {
        throw const QrSignException(
          QrSignErrorCode.mismatchedPubkey,
          '签名回执公钥与当前选中钱包不一致',
        );
      }
    }
    if (expectedPayloadHash != null) {
      if (_normalizeHex(body.payloadHash) != _normalizeHex(expectedPayloadHash)) {
        throw const QrSignException(
          QrSignErrorCode.mismatchedPayloadHash,
          '签名回执 payload_hash 与请求不一致',
        );
      }
    }
    if (expectedPayloadHex != null) {
      if (!verifySr25519Signature(
        pubkeyHex: body.pubkey,
        signatureHex: body.signature,
        payloadHex: expectedPayloadHex,
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

  // ---------------------------------------------------------------------------
  // 内部工具
  // ---------------------------------------------------------------------------

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
    final text = value.startsWith('0x') ? value.substring(2) : value;
    if (text.isEmpty || text.length.isOdd) {
      throw QrSignException(
          QrSignErrorCode.invalidField, '$field 必须是偶数字节 hex');
    }
    if (!RegExp(r'^[0-9a-fA-F]+$').hasMatch(text)) {
      throw QrSignException(
          QrSignErrorCode.invalidField, '$field 必须是合法 hex');
    }
  }

  String _normalizeHex(String value) {
    return value.startsWith('0x')
        ? value.substring(2).toLowerCase()
        : value.toLowerCase();
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
    if (issuedAt > now + maxClockSkewSeconds) {
      throw const QrSignException(
          QrSignErrorCode.invalidField, 'issued_at 超出设备时间范围');
    }
    if (expiresAt < now) {
      throw const QrSignException(
          QrSignErrorCode.expired, '交易签名请求已过期');
    }
  }

  void _validateSignedAt(int signedAt) {
    final now = _now();
    if (signedAt > now + maxClockSkewSeconds) {
      throw const QrSignException(
          QrSignErrorCode.invalidField, 'signed_at 超出设备时间范围');
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
