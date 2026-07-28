import '../qr/qr_protocols.dart';
import '../isar/wallet_isar.dart';
import '../wallet/wallet_manager.dart';
import 'action_labels.dart';
import 'field_labels.dart';
import 'payload_decoder.dart';
import 'qr_signer.dart';

enum OfflineSignErrorCode {
  accountNotFound,
  accountMismatch,
  invalidPayload,
  contentMismatch,
  expired,
  replayed,
}

class OfflineSignException implements Exception {
  const OfflineSignException(this.code, this.message);

  final OfflineSignErrorCode code;
  final String message;

  @override
  String toString() => message;
}

/// 离线签名验证结果。
class OfflineSignVerification {
  const OfflineSignVerification({
    required this.decoded,
    required this.status,
    required this.actionLabel,
    this.rejectReason,
  });

  final DecodedPayload? decoded;
  final SignDecisionStatus status;
  final String? actionLabel;
  final String? rejectReason;

  bool get canSign => status == SignDecisionStatus.normal;
}

/// 公民钱包扫码签名只允许两种终态。
///
/// normal = 绿色,允许签名；reject = 红色,禁止签名。
/// 不再保留“动作不匹配/解码失败”等独立状态,原因统一放入 rejectReason。
enum SignDecisionStatus { normal, reject }

/// 离线签名执行服务。
class OfflineSignService {
  OfflineSignService({
    WalletManager? walletManager,
    QrSigner? signer,
  })  : _walletManager = walletManager ?? WalletManager(),
        _signer = signer ?? QrSigner();

  final WalletManager _walletManager;
  final QrSigner _signer;

  SignRequestEnvelope parseRequest(String raw) {
    return _signer.parseRequest(raw);
  }

  OfflineSignVerification verifyPayload(SignRequestEnvelope request) {
    final body = request.body;
    final qrActionLabel = actionLabelForQrAction(body.action);
    if (qrActionLabel == null) {
      return const OfflineSignVerification(
        decoded: null,
        status: SignDecisionStatus.reject,
        actionLabel: null,
        rejectReason: '未登记的签名动作，已拒绝签名',
      );
    }

    // Runtime 升级只在 QR 中携带 32B 待签摘要,原始 WASM call_data 留在生成端 session。
    if (QrActions.isRuntimeHashOnly(body.action)) {
      if (body.payloadBytes.length == 32) {
        return OfflineSignVerification(
          decoded: null,
          status: SignDecisionStatus.normal,
          actionLabel: qrActionLabel,
        );
      }
      return OfflineSignVerification(
        decoded: null,
        status: SignDecisionStatus.reject,
        actionLabel: qrActionLabel,
        rejectReason: 'Runtime 升级签名载荷必须是 32 字节哈希，已拒绝签名',
      );
    }

    final decoded = PayloadDecoder.decode(body.payloadHex);

    if (decoded == null) {
      return OfflineSignVerification(
        decoded: null,
        status: SignDecisionStatus.reject,
        actionLabel: qrActionLabel,
        rejectReason: body.payloadBytes.length == 32 &&
                QrActions.isChainAction(body.action)
            ? '普通链交易不能只签 32 字节哈希，已拒绝签名'
            : '签名载荷无法解码，已拒绝签名',
      );
    }

    final decodedActionLabel = actionLabelForDecodedAction(decoded.action);
    if (decodedActionLabel == null) {
      return OfflineSignVerification(
        decoded: decoded,
        status: SignDecisionStatus.reject,
        actionLabel: qrActionLabel,
        rejectReason: '签名动作缺少中文名称，已拒绝签名',
      );
    }

    final decodedAction = QrActions.fromDecodedAction(decoded.action);
    if (decodedAction == 0) {
      return OfflineSignVerification(
        decoded: decoded,
        status: SignDecisionStatus.reject,
        actionLabel: qrActionLabel,
        rejectReason: '签名动作未登记，已拒绝签名',
      );
    }
    if (decodedAction != body.action) {
      return OfflineSignVerification(
        decoded: decoded,
        status: SignDecisionStatus.reject,
        actionLabel: qrActionLabel,
        rejectReason: '签名动作和载荷内容不匹配，已拒绝签名',
      );
    }

    String? missingField;
    for (final fieldKey in decoded.reviewFields.keys) {
      if (!hasFieldLabel(fieldKey)) {
        missingField = fieldKey;
        break;
      }
    }
    if (missingField != null) {
      return OfflineSignVerification(
        decoded: decoded,
        status: SignDecisionStatus.reject,
        actionLabel: decodedActionLabel,
        rejectReason: '签名字段缺少中文名称，已拒绝签名',
      );
    }

    return OfflineSignVerification(
      decoded: decoded,
      status: SignDecisionStatus.normal,
      actionLabel: decodedActionLabel,
    );
  }

  Future<SignResponseEnvelope> signRequestRaw({
    required String accountId,
    required String raw,
  }) async {
    final request = parseRequest(raw);
    return signParsedRequest(accountId: accountId, request: request);
  }

  /// 按账户签名。签名主体是账户（accountId），QR 请求里的
  /// `signerPublicKeyHex` 即指定该由哪个账户签，故此处只需按账户定位并逐字比对。
  Future<SignResponseEnvelope> signParsedRequest({
    required String accountId,
    required SignRequestEnvelope request,
  }) async {
    final body = request.body;
    // 签名时再次校验过期
    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    if ((request.expiresAt ?? 0) <= now) {
      throw const OfflineSignException(
        OfflineSignErrorCode.expired,
        '签名请求已过期,请重新扫描',
      );
    }

    final account = await _walletManager.getAccountByAccountId(accountId);
    if (account == null) {
      throw const OfflineSignException(
        OfflineSignErrorCode.accountNotFound,
        '未找到指定账户',
      );
    }

    // 当前 sr25519 的 AccountId32 与 signer public key 字节相同；请求 getter
    // 已输出规范 AccountId 文本，因此这里只允许完全相等，不做兼容归一化。
    if (account.accountId != body.signerPublicKeyHex) {
      throw const OfflineSignException(
        OfflineSignErrorCode.accountMismatch,
        '签名请求中的公钥与所选账户不一致',
      );
    }

    final verification = verifyPayload(request);
    // 两色识别模型:只有 normal 绿色态才允许签名;reject 红色态绝不签名。
    switch (verification.status) {
      case SignDecisionStatus.normal:
        break;
      case SignDecisionStatus.reject:
        throw OfflineSignException(
          OfflineSignErrorCode.contentMismatch,
          verification.rejectReason ?? '签名请求已拒绝',
        );
    }

    final payloadBytes = QrSigner.signingBytesFor(body);
    if (payloadBytes.isEmpty) {
      throw const OfflineSignException(
        OfflineSignErrorCode.invalidPayload,
        '签名负载为空,无法签名',
      );
    }

    final requestId = request.id!;
    final claimed = await SignedQrRequestStore.claim(
      requestId: requestId,
      expiresAt: request.expiresAt!,
    );
    if (!claimed) {
      throw const OfflineSignException(
        OfflineSignErrorCode.replayed,
        '该签名请求已处理或已过期，请生成新请求',
      );
    }

    late final List<int> signature;
    try {
      signature = await _walletManager.signForAccount(
        account.accountId,
        payloadBytes,
      );
    } catch (_) {
      await SignedQrRequestStore.release(requestId);
      rethrow;
    }

    return _signer.buildResponse(
      request: request,
      signatureHex: '0x${_toHex(signature)}',
    );
  }

  String _toHex(List<int> bytes) {
    const chars = '0123456789abcdef';
    final buffer = StringBuffer();
    for (final byte in bytes) {
      buffer
        ..write(chars[(byte >> 4) & 0x0f])
        ..write(chars[byte & 0x0f]);
    }
    return buffer.toString();
  }
}
