import 'dart:typed_data';

import 'package:citizenapp/my/myid/voting_identity_payload.dart';
import 'package:citizenapp/qr/qr_protocols.dart';
import 'package:citizenapp/signer/qr_signer.dart';
import 'package:citizenapp/wallet/core/device_subkey.dart' show bytesToHex;
import 'package:citizenapp/wallet/core/wallet_manager.dart';

class CitizenIdentitySignException implements Exception {
  const CitizenIdentitySignException(this.message);
  final String message;
  @override
  String toString() => message;
}

/// 公民身份签名的已校验待签态；三个扫码入口共用，避免页面各自实现协议。
class CitizenIdentitySignPrep {
  const CitizenIdentitySignPrep({
    required this.request,
    required this.actionLabel,
    required this.decoded,
    required this.account,
  });

  final SignRequestEnvelope request;
  final String actionLabel;
  final VotingIdentityConsentPayload decoded;
  final Account account;
}

/// 公民签名统一服务：完整解码、请求/载荷/本机钱包三方公钥一致后才允许签名。
class CitizenIdentitySignService {
  CitizenIdentitySignService({QrSigner? signer})
      : _signer = signer ?? QrSigner();
  final QrSigner _signer;

  Future<CitizenIdentitySignPrep> prepare(
    String raw,
    WalletManager walletManager, {
    Account? requiredAccount,
  }) async {
    final SignRequestEnvelope request;
    try {
      request = _signer.parseRequest(raw);
    } on QrSignException catch (error) {
      throw CitizenIdentitySignException(error.message);
    }
    if (request.body.action != QrActions.citizenIdentity) {
      throw const CitizenIdentitySignException('该二维码不是公民签名确认请求');
    }
    final actionLabel = QrActions.actionLabelForCode(request.body.action);
    if (actionLabel == null) {
      throw const CitizenIdentitySignException('未登记的签名动作，已拒绝签名');
    }
    final decoded = VotingIdentityConsentPayload.decode(
      Uint8List.fromList(request.body.payloadBytes),
    );
    if (decoded == null) {
      throw const CitizenIdentitySignException('签名内容无法完整中文展示，已拒绝签名');
    }
    final requestPublicKey = _normalizeHex(request.body.signerPublicKeyHex);
    if (_normalizeHex(decoded.accountId) != requestPublicKey) {
      throw const CitizenIdentitySignException('身份载荷钱包与签名请求不一致');
    }
    final account = requiredAccount ??
        await walletManager.getAccountByAccountId(
          request.body.signerPublicKeyHex.toLowerCase(),
        );
    if (account == null ||
        _normalizeHex(account.accountId) != requestPublicKey) {
      throw const CitizenIdentitySignException('此签名请求的账户不在本机');
    }
    return CitizenIdentitySignPrep(
      request: request,
      actionLabel: actionLabel,
      decoded: decoded,
      account: account,
    );
  }

  Future<String> sign(
    CitizenIdentitySignPrep prep,
    WalletManager walletManager,
  ) async {
    final bytes = QrSigner.signingBytesForHex(
      payloadHex: prep.request.body.payloadHex,
      action: prep.request.body.action,
    );
    final signature = await walletManager.signForAccountId(
      prep.account.accountId,
      bytes,
    );
    return _signer.encodeResponse(_signer.buildResponse(
      request: prep.request,
      signatureHex: '0x${bytesToHex(signature)}',
    ));
  }

  static String _normalizeHex(String value) {
    final text = value.startsWith('0x') || value.startsWith('0X')
        ? value.substring(2)
        : value;
    return text.toLowerCase();
  }
}
