import 'dart:convert';
import 'dart:math';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:sr25519/sr25519.dart' as sr25519;

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

/// 交易签名请求（WUMIN_SIGN_V1.0.0）。
///
/// 由在线设备（wuminapp）构建，附带人可读交易摘要 [display]，
/// 离线设备（wumin）可据此展示交易详情并与独立解码结果交叉比对。
class QrSignRequest {
  const QrSignRequest({
    required this.proto,
    required this.requestId,
    required this.account,
    required this.pubkey,
    required this.sigAlg,
    required this.payloadHex,
    required this.issuedAt,
    required this.expiresAt,
    required this.display,
    this.specVersion,
  });

  final String proto;
  final String requestId;
  final String account;
  final String pubkey;
  final String sigAlg;
  final String payloadHex;
  final int issuedAt;
  final int expiresAt;

  /// 人可读交易摘要，由在线设备构建。
  ///
  /// 必须包含 `action`（动作标识）和 `summary`（一句话摘要），
  /// 可包含 `fields`（结构化字段，供离线端交叉比对）。
  final Map<String, dynamic> display;

  /// 链 runtime 的 spec_version，由在线设备从 `state_getRuntimeVersion` 读取。
  ///
  /// 离线设备据此判断自身的 pallet 索引注册表是否与当前链版本匹配。
  /// 为可选字段，旧版在线端不发送时离线端按 decodeFailed 处理。
  final int? specVersion;

  Map<String, dynamic> toJson() {
    final json = <String, dynamic>{
      'proto': proto,
      'type': 'sign_request',
      'request_id': requestId,
      'account': account,
      'pubkey': pubkey,
      'sig_alg': sigAlg,
      'payload_hex': payloadHex,
      'issued_at': issuedAt,
      'expires_at': expiresAt,
      'display': display,
    };
    if (specVersion != null) {
      json['spec_version'] = specVersion;
    }
    return json;
  }
}

/// 交易签名回执（WUMIN_SIGN_V1.0.0）。
///
/// 由离线设备（wumin）签名后生成。
/// [payloadHash] 为 payload_hex 的 SHA-256 摘要，在线设备可校验 payload 一致性。
class QrSignResponse {
  const QrSignResponse({
    required this.proto,
    required this.requestId,
    required this.pubkey,
    required this.sigAlg,
    required this.signature,
    required this.payloadHash,
    required this.signedAt,
  });

  final String proto;
  final String requestId;
  final String pubkey;
  final String sigAlg;
  final String signature;

  /// payload_hex 原始字节的 SHA-256 hex 摘要。
  final String payloadHash;
  final int signedAt;

  Map<String, dynamic> toJson() => {
        'proto': proto,
        'type': 'sign_response',
        'request_id': requestId,
        'pubkey': pubkey,
        'sig_alg': sigAlg,
        'signature': signature,
        'payload_hash': payloadHash,
        'signed_at': signedAt,
      };
}

class QrSigner {
  static const String protocol = 'WUMIN_SIGN_V1.0.0';
  static const int defaultTtlSeconds = 90;
  static const int maxClockSkewSeconds = 30;
  static const int maxPayloadChars = 32768;
  static final RegExp _idPattern = RegExp(r'^[A-Za-z0-9._:-]{16,128}$');
  static final RegExp _addressPattern =
      RegExp(r'^[1-9A-HJ-NP-Za-km-z]{30,80}$');

  /// 生成加密安全的随机 request ID（32 字符 hex）。
  ///
  /// 使用 [Random.secure] 保证不可预测，防止 replay 攻击。
  /// 可选 [prefix] 用于标识请求类型（如 "tx-"、"vote-"）。
  static String generateRequestId({String prefix = ''}) {
    final random = Random.secure();
    final bytes = List<int>.generate(16, (_) => random.nextInt(256));
    final hex = bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
    final id = prefix.isEmpty ? hex : '$prefix$hex';
    return id.length > 128 ? id.substring(0, 128) : id;
  }

  QrSignRequest buildRequest({
    required String requestId,
    required String account,
    required String pubkey,
    required String payloadHex,
    required Map<String, dynamic> display,
    String sigAlg = 'sr25519',
    int? nowEpochSeconds,
    int ttlSeconds = defaultTtlSeconds,
    int? specVersion,
  }) {
    final now = nowEpochSeconds ?? _now();
    return QrSignRequest(
      proto: protocol,
      requestId: requestId,
      account: account,
      pubkey: pubkey,
      sigAlg: sigAlg,
      payloadHex: payloadHex,
      issuedAt: now,
      expiresAt: now + ttlSeconds,
      display: display,
      specVersion: specVersion,
    );
  }

  String encodeRequest(QrSignRequest request) {
    return jsonEncode(request.toJson());
  }

  String encodeResponse(QrSignResponse response) {
    return jsonEncode(response.toJson());
  }

  QrSignRequest parseRequest(String raw) {
    final data = _parseJson(raw);
    final proto = _requiredString(data, 'proto');
    if (proto != protocol) {
      throw const QrSignException(
          QrSignErrorCode.invalidProtocol, '不支持的交易签名协议');
    }
    final type = _requiredString(data, 'type');
    if (type != 'sign_request') {
      throw const QrSignException(
          QrSignErrorCode.invalidField, '二维码类型不是签名请求');
    }
    final requestId = _requiredString(data, 'request_id');
    final account = _requiredString(data, 'account');
    final pubkey = _requiredString(data, 'pubkey');
    final sigAlg = _requiredString(data, 'sig_alg');
    final payloadHex = _requiredString(data, 'payload_hex');
    final issuedAt = _requiredInt(data, 'issued_at');
    final expiresAt = _requiredInt(data, 'expires_at');
    final display = _requiredMap(data, 'display');

    _validateRequestId(requestId);
    _validateAddress(account);
    _validateHexField(pubkey, 'pubkey');
    _validateHexField(payloadHex, 'payload_hex');
    _validateSigAlg(sigAlg);
    _validateExpiry(issuedAt: issuedAt, expiresAt: expiresAt);
    _validateDisplay(display);

    // spec_version 为可选字段，旧版在线端可能不发送。
    final specVersionRaw = data['spec_version'];
    int? specVersion;
    if (specVersionRaw is int) {
      specVersion = specVersionRaw;
    } else if (specVersionRaw is String) {
      specVersion = int.tryParse(specVersionRaw);
    }

    return QrSignRequest(
      proto: proto,
      requestId: requestId,
      account: account,
      pubkey: pubkey,
      sigAlg: sigAlg,
      payloadHex: payloadHex,
      issuedAt: issuedAt,
      expiresAt: expiresAt,
      display: display,
      specVersion: specVersion,
    );
  }

  QrSignResponse parseResponse(
    String raw, {
    required String expectedRequestId,
    String? expectedPubkey,
    String? expectedPayloadHash,
    String? expectedPayloadHex,
  }) {
    final data = _parseJson(raw);
    final proto = _requiredString(data, 'proto');
    if (proto != protocol) {
      throw const QrSignException(
          QrSignErrorCode.invalidProtocol, '不支持的交易签名协议');
    }
    final type = _requiredString(data, 'type');
    if (type != 'sign_response') {
      throw const QrSignException(
          QrSignErrorCode.invalidField, '二维码类型不是签名回执');
    }
    final requestId = _requiredString(data, 'request_id');
    final pubkey = _requiredString(data, 'pubkey');
    final sigAlg = _requiredString(data, 'sig_alg');
    final signature = _requiredString(data, 'signature');
    final payloadHash = _requiredString(data, 'payload_hash');
    final signedAt = _requiredInt(data, 'signed_at');

    _validateRequestId(requestId);
    _validateHexField(pubkey, 'pubkey');
    _validateHexField(signature, 'signature');
    _validateHexField(payloadHash, 'payload_hash');
    _validateSigAlg(sigAlg);
    _validateSignedAt(signedAt);

    if (requestId != expectedRequestId) {
      throw const QrSignException(
        QrSignErrorCode.mismatchedRequest,
        '签名回执 request_id 与请求不一致',
      );
    }

    if (expectedPubkey != null) {
      final actual = _normalizeHex(pubkey);
      final expected = _normalizeHex(expectedPubkey);
      if (actual != expected) {
        throw const QrSignException(
          QrSignErrorCode.mismatchedPubkey,
          '签名回执公钥与当前选中钱包不一致',
        );
      }
    }

    if (expectedPayloadHash != null) {
      final actual = _normalizeHex(payloadHash);
      final expected = _normalizeHex(expectedPayloadHash);
      if (actual != expected) {
        throw const QrSignException(
          QrSignErrorCode.mismatchedPayloadHash,
          '签名回执 payload_hash 与请求不一致',
        );
      }
    }

    if (expectedPayloadHex != null) {
      if (!verifySr25519Signature(
        pubkeyHex: pubkey,
        signatureHex: signature,
        payloadHex: expectedPayloadHex,
      )) {
        throw const QrSignException(
          QrSignErrorCode.invalidSignature,
          '签名验证失败：签名与 payload 不匹配，请重新签名',
        );
      }
    }

    return QrSignResponse(
      proto: proto,
      requestId: requestId,
      pubkey: pubkey,
      sigAlg: sigAlg,
      signature: signature,
      payloadHash: payloadHash,
      signedAt: signedAt,
    );
  }

  /// 计算 payload_hex 原始字节的 SHA-256 hex 摘要。
  static String computePayloadHash(String payloadHex) {
    final bytes = _hexToBytes(payloadHex);
    final digest = sha256.convert(bytes);
    return digest.toString();
  }

  /// 验证 sr25519 签名是否与 payload 和公钥匹配。
  ///
  /// 用于在线端接收签名回执后、提交上链前提前发现无效签名。
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

  Map<String, dynamic> _parseJson(String raw) {
    final text = raw.trim();
    if (text.isEmpty || text.length > maxPayloadChars) {
      throw const QrSignException(
        QrSignErrorCode.invalidFormat,
        '扫码数据格式错误：内容为空或超出长度限制',
      );
    }
    dynamic decoded;
    try {
      decoded = jsonDecode(text);
    } catch (_) {
      throw const QrSignException(
        QrSignErrorCode.invalidFormat,
        '扫码数据格式错误：必须为 JSON 对象',
      );
    }
    if (decoded is! Map) {
      throw const QrSignException(
        QrSignErrorCode.invalidFormat,
        '扫码数据格式错误：必须为 JSON 对象',
      );
    }
    return decoded.map((k, v) => MapEntry(k.toString(), v));
  }

  String _requiredString(Map<String, dynamic> data, String key) {
    final value = data[key]?.toString().trim();
    if (value == null || value.isEmpty) {
      throw QrSignException(
          QrSignErrorCode.invalidField, '扫码数据缺少字段：$key');
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
    throw QrSignException(
        QrSignErrorCode.invalidField, '扫码数据字段格式错误：$key');
  }

  Map<String, dynamic> _requiredMap(Map<String, dynamic> data, String key) {
    final value = data[key];
    if (value is Map) {
      return value.map((k, v) => MapEntry(k.toString(), v));
    }
    throw QrSignException(
        QrSignErrorCode.invalidField, '扫码数据缺少字段：$key');
  }

  void _validateRequestId(String requestId) {
    if (!_idPattern.hasMatch(requestId)) {
      throw const QrSignException(
          QrSignErrorCode.invalidField, 'request_id 格式错误');
    }
  }

  void _validateAddress(String address) {
    if (!_addressPattern.hasMatch(address)) {
      throw const QrSignException(
          QrSignErrorCode.invalidField, 'account 地址格式错误');
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

  void _validateDisplay(Map<String, dynamic> display) {
    final action = display['action'];
    if (action == null || action.toString().trim().isEmpty) {
      throw const QrSignException(
          QrSignErrorCode.invalidField, 'display.action 不能为空');
    }
    final summary = display['summary'];
    if (summary == null || summary.toString().trim().isEmpty) {
      throw const QrSignException(
          QrSignErrorCode.invalidField, 'display.summary 不能为空');
    }
  }

  String _normalizeHex(String value) {
    return value.startsWith('0x')
        ? value.substring(2).toLowerCase()
        : value.toLowerCase();
  }

  void _validateSigAlg(String sigAlg) {
    if (sigAlg.toLowerCase() != 'sr25519') {
      throw const QrSignException(
          QrSignErrorCode.invalidField, '仅支持 sr25519');
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
