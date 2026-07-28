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
    required this.wallet,
  });

  final SignRequestEnvelope request;
  final String actionLabel;
  final String cidNumber;
  final bool isOccupy;
  final WalletProfile wallet;
}

/// 注册局占号/换绑签名服务。
///
/// 与 [CitizenIdentitySignService] 的三方一致校验**相反**:占号/换绑请求 `b.u` 留空、
/// `d = append_bounded(cid)` 不含账户,由用户自选本机账户绑定到该 CID;只从 `d` 读
/// CID 两色展示,签名字节 = `signingMessage(op, d ++ 选中账户32)`,响应 `b.u` 用选中账户带回。
class CitizenOccupySignService {
  CitizenOccupySignService({QrSigner? signer}) : _signer = signer ?? QrSigner();
  final QrSigner _signer;

  /// [selectedWallet] = 用户自选的绑定账户(占即绑一账户)。
  Future<CitizenOccupySignPrep> prepare(
    String raw,
    WalletProfile selectedWallet,
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
    final cid = _readBoundedCid(Uint8List.fromList(request.body.payloadBytes));
    if (cid == null) {
      throw const CitizenOccupySignException('签名内容无法完整中文展示，已拒绝签名');
    }
    if (selectedWallet.isColdWallet) {
      throw const CitizenOccupySignException('公民 App 不能替离线钱包签名');
    }
    return CitizenOccupySignPrep(
      request: request,
      actionLabel: actionLabel,
      cidNumber: cid,
      isOccupy: action == QrActions.citizenOccupy,
      wallet: selectedWallet,
    );
  }

  Future<String> sign(
    CitizenOccupySignPrep prep,
    WalletManager walletManager,
  ) async {
    final bytes = QrSigner.signingBytesForHex(
      payloadHex: prep.request.body.payloadHex,
      action: prep.request.body.action,
      selfAccountId: _accountIdBytes(prep.wallet.accountId),
    );
    if (bytes.isEmpty) {
      throw const CitizenOccupySignException('签名负载为空,无法签名');
    }
    final signature =
        await walletManager.signWithWallet(prep.wallet.walletIndex, bytes);
    return _signer.encodeResponse(_signer.buildResponse(
      request: prep.request,
      signatureHex: '0x${bytesToHex(signature)}',
      signerPublicKeyHexOverride: prep.wallet.accountId,
    ));
  }

  /// `d = append_bounded(cid) = Compact(len) ++ cid`,读满全部字节得 CID。
  static String? _readBoundedCid(Uint8List bytes) {
    if (bytes.isEmpty) return null;
    final (len, next) = _readCompact(bytes, 0);
    if (len == 0 || len > 32 || next == 0) return null;
    final end = next + len;
    if (end != bytes.length) return null;
    final raw = bytes.sublist(next, end);
    if (raw.any((b) => b < 0x21 || b > 0x7e)) return null;
    return String.fromCharCodes(raw);
  }

  /// SCALE compact-u32 解码,返回 `(value, nextOffset)`;失败时 nextOffset=0。
  static (int, int) _readCompact(Uint8List bytes, int offset) {
    if (offset >= bytes.length) return (0, 0);
    final mode = bytes[offset] & 0x03;
    if (mode == 0) return (bytes[offset] >> 2, offset + 1);
    if (mode == 1) {
      if (offset + 2 > bytes.length) return (0, 0);
      return ((bytes[offset] | (bytes[offset + 1] << 8)) >> 2, offset + 2);
    }
    if (mode == 2) {
      if (offset + 4 > bytes.length) return (0, 0);
      final v = (bytes[offset] |
              (bytes[offset + 1] << 8) |
              (bytes[offset + 2] << 16) |
              (bytes[offset + 3] << 24)) >>
          2;
      return (v, offset + 4);
    }
    return (0, 0); // big-integer mode 不用于 CID 长度
  }

  static Uint8List _accountIdBytes(String accountIdHex) {
    final hex = accountIdHex.startsWith('0x') || accountIdHex.startsWith('0X')
        ? accountIdHex.substring(2)
        : accountIdHex;
    final out = Uint8List(hex.length ~/ 2);
    for (var i = 0; i < out.length; i++) {
      out[i] = int.parse(hex.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return out;
  }
}
