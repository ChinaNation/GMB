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
import 'package:citizenapp/signer/signing.dart';

enum QrSignErrorCode {
  invalidFormat,
  invalidField,
  invalidProtocol,
  expired,
  mismatchedRequest,
  mismatchedAccount,
  mismatchedSignerPublicKey,
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

/// 注册局占号/换绑二维码中的完整授权模板。
///
/// 生成端把待绑定账户写成 32 字节零槽；签名端必须严格解码全部字段且确认无尾字节，
/// 再把用户选择的账户原位填入。禁止恢复旧版 `bounded_cid ++ account_id` 末尾拼接。
class CidAccountAuthorizationTemplate {
  CidAccountAuthorizationTemplate._({
    required Uint8List payload,
    required this.genesisHash,
    required this.cidNumber,
    required this.expectedOldAccountId,
    required this.expectedBindingRevision,
    required this.expiresAt,
    required int accountOffset,
  })  : _payload = Uint8List.fromList(payload),
        _accountOffset = accountOffset;

  final Uint8List _payload;
  final int _accountOffset;
  final String genesisHash;
  final String cidNumber;
  final String? expectedOldAccountId;
  final BigInt expectedBindingRevision;
  final BigInt expiresAt;

  /// 把所选账户填回授权结构的零槽，返回与 Runtime 逐字节一致的 SCALE。
  ///
  /// 换绑时新旧账户相同必须拒绝，不能生成可签名载荷。
  Uint8List? materialize(Uint8List accountId) {
    if (accountId.length != 32) return null;
    final oldAccountId = expectedOldAccountId;
    if (oldAccountId != null &&
        oldAccountId == QrSigner._bytesToLowerHex(accountId)) {
      return null;
    }
    final output = Uint8List.fromList(_payload);
    output.setRange(_accountOffset, _accountOffset + 32, accountId);
    return output;
  }
}

class QrSigner {
  static const int defaultTtlSeconds = 90;
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
    required String signerPublicKey,
    required String payloadHex,
    required int action,
    int? nowEpochSeconds,
    int ttlSeconds = defaultTtlSeconds,
  }) {
    final now = nowEpochSeconds ?? _now();
    _validateRequestId(requestId);
    _validateHexField(signerPublicKey, 'signer_public_key');
    _validateHexField(payloadHex, 'payload');
    return QrEnvelope<SignRequestBody>(
      kind: QrKind.signRequest,
      id: requestId,
      issuedAt: now,
      expiresAt: now + ttlSeconds,
      body: SignRequestBody.fromHex(
        action: action,
        signerPublicKeyHex: signerPublicKey,
        payloadHex: payloadHex,
      ),
    );
  }

  String encodeRequest(SignRequestEnvelope request) => request.toRawJson();

  String encodeResponse(SignResponseEnvelope response) => response.toRawJson();

  /// 构造 sign_response envelope。签名响应只携带 u/s,请求绑定由 id 完成。
  SignResponseEnvelope buildResponse({
    required SignRequestEnvelope request,
    required String signatureHex,
    String? signerPublicKeyHexOverride,
  }) {
    _validateHexField(signatureHex, 'signature');
    return QrEnvelope<SignResponseBody>(
      kind: QrKind.signResponse,
      id: request.id,
      issuedAt: request.issuedAt,
      expiresAt: request.expiresAt,
      body: SignResponseBody.fromHex(
        // 占号/换绑:请求 b.u 留空,响应 b.u 用钱包自选账户带回,供 OnChina 取 account_id。
        signerPublicKeyHex:
            signerPublicKeyHexOverride ?? request.body.signerPublicKeyHex,
        signatureHex: signatureHex,
      ),
    );
  }

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
  /// 找回本地 session 中的 action、payload 和 signer_public_key 后验签。
  SignResponseEnvelope parseResponse(
    String raw, {
    required String expectedRequestId,
    String? expectedSignerPublicKey,
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
    if (expectedSignerPublicKey != null &&
        _normalizeHex(body.signerPublicKeyHex) !=
            _normalizeHex(expectedSignerPublicKey)) {
      throw const QrSignException(
        QrSignErrorCode.mismatchedSignerPublicKey,
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
        signerPublicKeyHex: body.signerPublicKeyHex,
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
    required String signerPublicKeyHex,
    required String signatureHex,
    required Uint8List message,
  }) {
    try {
      final pubBytes = Uint8List.fromList(_hexToBytes(signerPublicKeyHex));
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

  /// 按动作码构造签名原文,与 citizenwallet `signingBytesFor` 逐字节一致:
  /// - 公民签名确认(0x10)走 GMB 哈希域 signingMessage;
  /// - 链交易复刻 SignedPayload::using_encoded:
  ///   payload <= 256B 签原文,>256B 签 blake2_256(payload)。
  static Uint8List signingBytesForHex({
    required String payloadHex,
    required int action,
    Uint8List? selfAccountId,
  }) {
    final payload = Uint8List.fromList(_hexToBytes(payloadHex));
    if (action == QrActions.citizenIdentity) {
      return signingMessage(
        opTag: kOpSignCitizenIdentity,
        scalePayload: payload,
      );
    }
    if (QrActions.isSelfAccountDomainAction(action)) {
      if (selfAccountId == null || selfAccountId.length != 32) {
        return Uint8List(0);
      }
      // 两种授权结构的账户槽位置不同：严格解析完整模板并原位填入所选账户。
      // 任何零槽污染、revision 错误、尾字节或换绑新旧账户相同都 fail-closed。
      final authorization = decodeCidAccountAuthorizationTemplate(
        action: action,
        payload: payload,
      );
      final exactPayload = authorization?.materialize(selfAccountId);
      if (exactPayload == null) return Uint8List(0);
      final opTag = action == QrActions.citizenOccupy
          ? kOpSignCidOccupy
          : kOpSignCidAdminRebind;
      return signingMessage(
        opTag: opTag,
        scalePayload: exactPayload,
      );
    }
    if (action == QrActions.squareAccountAction) {
      return signingMessage(
        opTag: kOpSignSquareAction,
        scalePayload: payload,
      );
    }
    if (QrActions.isChainAction(action) && payload.length > 256) {
      return Hasher.blake2b256.hash(payload);
    }
    return payload;
  }

  /// 严格解析注册局占号/换绑的完整授权模板。
  ///
  /// occupy:
  /// `genesis_hash + bounded cid + account_id(32B 零槽) + revision=0 + expires`
  ///
  /// rebind:
  /// `genesis_hash + bounded cid + expected_old_account_id`
  /// `+ new_account_id(32B 零槽) + revision(nonzero) + expires`
  static CidAccountAuthorizationTemplate?
      decodeCidAccountAuthorizationTemplate({
    required int action,
    required Uint8List payload,
  }) {
    if (action == QrActions.citizenOccupy) {
      return _decodeCidOccupyAuthorizationTemplate(payload);
    }
    if (action == QrActions.citizenRebind) {
      return _decodeCidRebindAuthorizationTemplate(payload);
    }
    return null;
  }

  static CidAccountAuthorizationTemplate? _decodeCidOccupyAuthorizationTemplate(
      Uint8List bytes) {
    if (bytes.length < 32) return null;
    var offset = 32;
    final cidRead = _readCanonicalBoundedCid(bytes, offset);
    if (cidRead == null) return null;
    offset = cidRead.$2;
    final accountOffset = offset;
    if (!_hasZeroAccountSlot(bytes, accountOffset)) return null;
    offset += 32;
    if (offset + 16 != bytes.length) return null;
    final revision = _readU64LittleEndian(bytes, offset);
    offset += 8;
    final expiresAt = _readU64LittleEndian(bytes, offset);
    if (revision != BigInt.zero || expiresAt <= BigInt.zero) return null;
    return CidAccountAuthorizationTemplate._(
      payload: bytes,
      genesisHash: _bytesToLowerHex(bytes.sublist(0, 32)),
      cidNumber: cidRead.$1,
      expectedOldAccountId: null,
      expectedBindingRevision: revision,
      expiresAt: expiresAt,
      accountOffset: accountOffset,
    );
  }

  static CidAccountAuthorizationTemplate? _decodeCidRebindAuthorizationTemplate(
      Uint8List bytes) {
    if (bytes.length < 64) return null;
    var offset = 32;
    final cidRead = _readCanonicalBoundedCid(bytes, offset);
    if (cidRead == null) return null;
    offset = cidRead.$2;
    if (offset + 32 > bytes.length) return null;
    final expectedOldAccount = bytes.sublist(offset, offset + 32);
    offset += 32;
    final accountOffset = offset;
    if (!_hasZeroAccountSlot(bytes, accountOffset)) return null;
    offset += 32;
    if (offset + 16 != bytes.length) return null;
    final revision = _readU64LittleEndian(bytes, offset);
    offset += 8;
    final expiresAt = _readU64LittleEndian(bytes, offset);
    if (revision <= BigInt.zero || expiresAt <= BigInt.zero) return null;
    return CidAccountAuthorizationTemplate._(
      payload: bytes,
      genesisHash: _bytesToLowerHex(bytes.sublist(0, 32)),
      cidNumber: cidRead.$1,
      expectedOldAccountId: _bytesToLowerHex(expectedOldAccount),
      expectedBindingRevision: revision,
      expiresAt: expiresAt,
      accountOffset: accountOffset,
    );
  }

  /// CID 上限只有 32 字节，因此其规范 SCALE Compact 长度只能使用单字节 mode 0。
  static (String, int)? _readCanonicalBoundedCid(
    Uint8List bytes,
    int offset,
  ) {
    if (offset >= bytes.length || (bytes[offset] & 0x03) != 0) return null;
    final length = bytes[offset] >> 2;
    if (length == 0 || length > 32) return null;
    final start = offset + 1;
    final end = start + length;
    if (end > bytes.length) return null;
    final raw = bytes.sublist(start, end);
    if (raw.any((byte) => byte < 0x21 || byte > 0x7e)) return null;
    return (String.fromCharCodes(raw), end);
  }

  static bool _hasZeroAccountSlot(Uint8List bytes, int offset) {
    if (offset + 32 > bytes.length) return false;
    for (var index = offset; index < offset + 32; index++) {
      if (bytes[index] != 0) return false;
    }
    return true;
  }

  static BigInt _readU64LittleEndian(Uint8List bytes, int offset) {
    var value = BigInt.zero;
    for (var index = 7; index >= 0; index--) {
      value = (value << 8) | BigInt.from(bytes[offset + index]);
    }
    return value;
  }

  static String _bytesToLowerHex(List<int> bytes) =>
      '0x${bytes.map((byte) => byte.toRadixString(16).padLeft(2, '0')).join()}';

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
    if (expiresAt <= now) {
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
