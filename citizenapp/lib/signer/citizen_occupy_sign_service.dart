import 'dart:typed_data';

import 'package:citizenapp/qr/qr_protocols.dart';
import 'package:citizenapp/signer/qr_signer.dart';
import 'package:citizenapp/wallet/core/device_subkey.dart' show bytesToHex;
import 'package:citizenapp/wallet/core/wallet_manager.dart';

class CitizenOccupySignException implements Exception {
  const CitizenOccupySignException(this.message);
  final String message;
  @override
  String toString() => message;
}

/// 注册局代办占号/换绑的已校验待签态。请求 b.u 留空,绑定账户由用户自选。
class CitizenOccupySignPrep {
  const CitizenOccupySignPrep({
    required this.request,
    required this.actionLabel,
    required this.cidNumber,
    required this.isOccupy,
    required this.genesisHash,
    required this.expectedOldAccountId,
    required this.expectedBindingRevision,
    required this.expiresAt,
    required this.account,
  });

  final SignRequestEnvelope request;
  final String actionLabel;
  final String cidNumber;
  final bool isOccupy;
  final String genesisHash;
  final String? expectedOldAccountId;
  final BigInt expectedBindingRevision;
  final BigInt expiresAt;
  final Account account;
}

/// 注册局占号/换绑签名服务。
///
/// 请求 `b.u` 留空；`d` 必须是包含创世哈希、CID、账户零槽、绑定 revision 和过期时间的
/// 完整 Runtime 授权模板。服务严格解码、核对外层 `e == 内层 expires_at`，再把用户选择
/// 的本机账户原位填入零槽签名；响应 `b.u` 用该账户带回。
class CitizenOccupySignService {
  CitizenOccupySignService({QrSigner? signer}) : _signer = signer ?? QrSigner();
  final QrSigner _signer;

  /// [selectedAccount] = 用户自选或账户卡扫码入口锁定的绑定账户(占即绑一账户)。
  Future<CitizenOccupySignPrep> prepare(
    String raw,
    Account selectedAccount,
  ) async {
    final SignRequestEnvelope request;
    try {
      request = _signer.parseRequest(raw);
    } on QrSignException catch (error) {
      throw CitizenOccupySignException(error.message);
    }
    final action = request.body.action;
    if (!QrActions.isSelfAccountDomainAction(action)) {
      throw const CitizenOccupySignException('该二维码不是注册局占号/换绑请求');
    }
    final actionLabel = QrActions.actionLabelForCode(action);
    if (actionLabel == null) {
      throw const CitizenOccupySignException('未登记的签名动作，已拒绝签名');
    }
    final authorization = QrSigner.decodeCidAccountAuthorizationTemplate(
      action: action,
      payload: Uint8List.fromList(request.body.payloadBytes),
    );
    if (authorization == null) {
      throw const CitizenOccupySignException('签名内容无法完整中文展示，已拒绝签名');
    }
    final outerExpiresAt = request.expiresAt;
    if (outerExpiresAt == null ||
        BigInt.from(outerExpiresAt) != authorization.expiresAt) {
      throw const CitizenOccupySignException(
        '二维码过期时间与授权载荷不一致，已拒绝签名',
      );
    }
    final accountId = _accountIdBytes(selectedAccount.accountId);
    if (accountId == null) {
      throw const CitizenOccupySignException('所选账户 account_id 格式错误');
    }
    if (authorization.materialize(accountId) == null) {
      throw const CitizenOccupySignException('换绑新账户不得与当前绑定账户相同');
    }
    return CitizenOccupySignPrep(
      request: request,
      actionLabel: actionLabel,
      cidNumber: authorization.cidNumber,
      isOccupy: action == QrActions.citizenOccupy,
      genesisHash: authorization.genesisHash,
      expectedOldAccountId: authorization.expectedOldAccountId,
      expectedBindingRevision: authorization.expectedBindingRevision,
      expiresAt: authorization.expiresAt,
      account: selectedAccount,
    );
  }

  Future<String> sign(
    CitizenOccupySignPrep prep,
    WalletManager walletManager,
  ) async {
    final accountId = _accountIdBytes(prep.account.accountId);
    if (accountId == null) {
      throw const CitizenOccupySignException('所选账户 account_id 格式错误');
    }
    final bytes = QrSigner.signingBytesForHex(
      payloadHex: prep.request.body.payloadHex,
      action: prep.request.body.action,
      selfAccountId: accountId,
    );
    if (bytes.isEmpty) {
      throw const CitizenOccupySignException('签名负载为空,无法签名');
    }
    final signature = await walletManager.signForAccountId(
      prep.account.accountId,
      bytes,
    );
    return _signer.encodeResponse(_signer.buildResponse(
      request: prep.request,
      signatureHex: '0x${bytesToHex(signature)}',
      signerPublicKeyHexOverride: prep.account.accountId,
    ));
  }

  static Uint8List? _accountIdBytes(String accountIdHex) {
    if (!RegExp(r'^0x[0-9a-f]{64}$').hasMatch(accountIdHex)) return null;
    final hex = accountIdHex.substring(2);
    final out = Uint8List(32);
    for (var i = 0; i < out.length; i++) {
      out[i] = int.parse(hex.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return out;
  }
}
