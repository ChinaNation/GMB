import '../qr/qr_protocols.dart';
import '../wallet/wallet_manager.dart';
import 'action_labels.dart';
import 'field_labels.dart';
import 'payload_decoder.dart';
import 'qr_signer.dart';

enum OfflineSignErrorCode {
  walletNotFound,
  coldWalletUnsupported,
  walletMismatch,
  invalidPayload,
  contentMismatch,
  expired,
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
    required int walletIndex,
    required String raw,
  }) async {
    final request = parseRequest(raw);
    return signParsedRequest(walletIndex: walletIndex, request: request);
  }

  Future<SignResponseEnvelope> signParsedRequest({
    required int walletIndex,
    required SignRequestEnvelope request,
  }) async {
    final body = request.body;
    // 签名时再次校验过期
    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    if ((request.expiresAt ?? 0) < now) {
      throw const OfflineSignException(
        OfflineSignErrorCode.expired,
        '签名请求已过期,请重新扫描',
      );
    }

    final wallet = await _walletManager.getWalletByIndex(walletIndex);
    if (wallet == null) {
      throw const OfflineSignException(
        OfflineSignErrorCode.walletNotFound,
        '未找到指定钱包',
      );
    }
    if (wallet.isColdWallet) {
      throw const OfflineSignException(
        OfflineSignErrorCode.coldWalletUnsupported,
        '当前钱包为冷钱包,无法作为离线签名设备',
      );
    }

    if (_normalizeHex(wallet.pubkeyHex) != _normalizeHex(body.pubkeyHex)) {
      throw const OfflineSignException(
        OfflineSignErrorCode.walletMismatch,
        '签名请求中的公钥与当前钱包不一致',
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

    final signature = await _walletManager.signWithWallet(
      wallet.walletIndex,
      payloadBytes,
    );

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

String _normalizeHex(String input) {
  final trimmed = input.trim();
  if (trimmed.startsWith('0x') || trimmed.startsWith('0X')) {
    return trimmed.substring(2).toLowerCase();
  }
  return trimmed.toLowerCase();
}
