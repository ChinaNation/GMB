import 'dart:convert';

enum QrSignErrorCode {
  invalidFormat,
  invalidField,
  invalidProtocol,
  expired,
  mismatchedRequest,
  mismatchedAccount,
}

class QrSignException implements Exception {
  const QrSignException(this.code, this.message);

  final QrSignErrorCode code;
  final String message;

  @override
  String toString() => message;
}

enum QrSignScope {
  login,
  onchainTx,
}

extension on QrSignScope {
  String get wireValue {
    switch (this) {
      case QrSignScope.login:
        return 'login';
      case QrSignScope.onchainTx:
        return 'onchain_tx';
    }
  }
}

class QrSignRequest {
  const QrSignRequest({
    required this.proto,
    required this.scope,
    required this.requestId,
    required this.account,
    required this.pubkey,
    required this.sigAlg,
    required this.payloadHex,
    required this.issuedAt,
    required this.expiresAt,
  });

  final String proto;
  final QrSignScope scope;
  final String requestId;
  final String account;
  final String pubkey;
  final String sigAlg;
  final String payloadHex;
  final int issuedAt;
  final int expiresAt;

  Map<String, dynamic> toJson() => {
        'proto': proto,
        'type': 'sign_request',
        'scope': scope.wireValue,
        'request_id': requestId,
        'account': account,
        'pubkey': pubkey,
        'sig_alg': sigAlg,
        'payload_hex': payloadHex,
        'issued_at': issuedAt,
        'expires_at': expiresAt,
      };
}

class QrSignResponse {
  const QrSignResponse({
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

  Map<String, dynamic> toJson() => {
        'proto': proto,
        'type': 'sign_response',
        'request_id': requestId,
        'pubkey': pubkey,
        'sig_alg': sigAlg,
        'signature': signature,
        'signed_at': signedAt,
      };
}

class QrSigner {
  static const String protocol = 'WUMINAPP_QR_SIGN_V1';
  static const int defaultTtlSeconds = 90;
  static const int maxClockSkewSeconds = 30;
  static const int maxPayloadChars = 32768;
  static final RegExp _idPattern = RegExp(r'^[A-Za-z0-9._:-]{4,128}$');
  static final RegExp _addressPattern =
      RegExp(r'^[1-9A-HJ-NP-Za-km-z]{30,80}$');

  QrSignRequest buildRequest({
    required QrSignScope scope,
    required String requestId,
    required String account,
    required String pubkey,
    required String payloadHex,
    String sigAlg = 'sr25519',
    int? nowEpochSeconds,
    int ttlSeconds = defaultTtlSeconds,
  }) {
    final now = nowEpochSeconds ?? _now();
    return QrSignRequest(
      proto: protocol,
      scope: scope,
      requestId: requestId,
      account: account,
      pubkey: pubkey,
      sigAlg: sigAlg,
      payloadHex: payloadHex,
      issuedAt: now,
      expiresAt: now + ttlSeconds,
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
          QrSignErrorCode.invalidProtocol, '不支持的扫码签名协议');
    }
    final type = _requiredString(data, 'type');
    if (type != 'sign_request') {
      throw const QrSignException(QrSignErrorCode.invalidField, '二维码类型不是签名请求');
    }
    final scope = _parseScope(_requiredString(data, 'scope'));
    final requestId = _requiredString(data, 'request_id');
    final account = _requiredString(data, 'account');
    final pubkey = _requiredString(data, 'pubkey');
    final sigAlg = _requiredString(data, 'sig_alg');
    final payloadHex = _requiredString(data, 'payload_hex');
    final issuedAt = _requiredInt(data, 'issued_at');
    final expiresAt = _requiredInt(data, 'expires_at');

    _validateRequestId(requestId);
    _validateAddress(account);
    _validateHexField(pubkey, 'pubkey');
    _validateHexField(payloadHex, 'payload_hex');
    _validateSigAlg(sigAlg);
    _validateExpiry(issuedAt: issuedAt, expiresAt: expiresAt);

    return QrSignRequest(
      proto: proto,
      scope: scope,
      requestId: requestId,
      account: account,
      pubkey: pubkey,
      sigAlg: sigAlg,
      payloadHex: payloadHex,
      issuedAt: issuedAt,
      expiresAt: expiresAt,
    );
  }

  QrSignResponse parseResponse(
    String raw, {
    required String expectedRequestId,
  }) {
    final data = _parseJson(raw);
    final proto = _requiredString(data, 'proto');
    if (proto != protocol) {
      throw const QrSignException(
          QrSignErrorCode.invalidProtocol, '不支持的扫码签名协议');
    }
    final type = _requiredString(data, 'type');
    if (type != 'sign_response') {
      throw const QrSignException(QrSignErrorCode.invalidField, '二维码类型不是签名回执');
    }
    final requestId = _requiredString(data, 'request_id');
    final pubkey = _requiredString(data, 'pubkey');
    final sigAlg = _requiredString(data, 'sig_alg');
    final signature = _requiredString(data, 'signature');
    final signedAt = _requiredInt(data, 'signed_at');

    _validateRequestId(requestId);
    _validateHexField(pubkey, 'pubkey');
    _validateHexField(signature, 'signature');
    _validateSigAlg(sigAlg);
    _validateSignedAt(signedAt);

    if (requestId != expectedRequestId) {
      throw const QrSignException(
        QrSignErrorCode.mismatchedRequest,
        '签名回执 request_id 与请求不一致',
      );
    }

    return QrSignResponse(
      proto: proto,
      requestId: requestId,
      pubkey: pubkey,
      sigAlg: sigAlg,
      signature: signature,
      signedAt: signedAt,
    );
  }

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

  QrSignScope _parseScope(String raw) {
    switch (raw.toLowerCase()) {
      case 'login':
        return QrSignScope.login;
      case 'onchain_tx':
        return QrSignScope.onchainTx;
      default:
        throw const QrSignException(QrSignErrorCode.invalidField, '不支持的签名场景');
    }
  }

  String _requiredString(Map<String, dynamic> data, String key) {
    final value = data[key]?.toString().trim();
    if (value == null || value.isEmpty) {
      throw QrSignException(QrSignErrorCode.invalidField, '扫码数据缺少字段：$key');
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
    throw QrSignException(QrSignErrorCode.invalidField, '扫码数据字段格式错误：$key');
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
      throw QrSignException(QrSignErrorCode.invalidField, '$field 必须是偶数字节 hex');
    }
    if (!RegExp(r'^[0-9a-fA-F]+$').hasMatch(text)) {
      throw QrSignException(QrSignErrorCode.invalidField, '$field 必须是合法 hex');
    }
  }

  void _validateSigAlg(String sigAlg) {
    if (sigAlg.toLowerCase() != 'sr25519') {
      throw const QrSignException(QrSignErrorCode.invalidField, '仅支持 sr25519');
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
      throw const QrSignException(QrSignErrorCode.expired, '扫码签名请求已过期');
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
}
