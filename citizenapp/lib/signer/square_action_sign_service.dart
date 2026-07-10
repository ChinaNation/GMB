import 'dart:typed_data';

import 'package:citizenapp/qr/qr_protocols.dart';
import 'package:citizenapp/signer/qr_signer.dart';
import 'package:citizenapp/signer/square_action_payload.dart';
import 'package:citizenapp/wallet/core/device_subkey.dart' show bytesToHex;
import 'package:citizenapp/wallet/core/wallet_manager.dart';

enum SquareActionSignError {
  invalidRequest,
  unsupportedAction,
  undecodable,
  accountNotLocal,
  coldWalletUnsupported,
}

class SquareActionSignException implements Exception {
  const SquareActionSignException(this.error, this.message);

  final SquareActionSignError error;
  final String message;

  @override
  String toString() => message;
}

/// 扫到的广场账户动作签名请求，经校验/解码/定位钱包后的待签态。
class SquareActionSignPrep {
  const SquareActionSignPrep({
    required this.request,
    required this.decoded,
    required this.wallet,
  });

  final SignRequestEnvelope request;
  final SquareActionPayload decoded;
  final WalletProfile wallet;
}

/// 广场账户动作「签名响应方」（官网无私钥，CitizenApp 扫一扫代签）。
///
/// 流程：扫 signRequest → 解析/两色解码 → 按 QR `u` 定位 owner 钱包（拒本机没有/冷钱包）
/// → 用户核对动作 → **owner 主钥**对 signing_message(0x1D) 签名（生物识别）→ 出 signResponse。
class SquareActionSignService {
  SquareActionSignService({QrSigner? signer}) : _signer = signer ?? QrSigner();

  final QrSigner _signer;

  /// 解析 + 两色解码 + 定位钱包（不签名、不弹生物识别）。失败抛 [SquareActionSignException]。
  Future<SquareActionSignPrep> prepare(String raw, WalletManager walletManager) async {
    final SignRequestEnvelope request;
    try {
      request = _signer.parseRequest(raw);
    } on QrSignException catch (e) {
      throw SquareActionSignException(SquareActionSignError.invalidRequest, e.message);
    }
    final body = request.body;
    if (body.action != QrActions.squareAccountAction) {
      throw const SquareActionSignException(
        SquareActionSignError.unsupportedAction,
        '暂不支持该签名类型',
      );
    }
    final decoded = decodeSquareActionPayload(body.payloadHex);
    if (decoded == null) {
      throw const SquareActionSignException(
        SquareActionSignError.undecodable,
        '无法解析签名内容，禁止签名',
      );
    }
    final wallet = await _resolveWalletByPubkey(walletManager, body.pubkeyBytes);
    if (wallet == null) {
      throw const SquareActionSignException(
        SquareActionSignError.accountNotLocal,
        '此签名请求的账户不在本机',
      );
    }
    if (wallet.isColdWallet) {
      throw const SquareActionSignException(
        SquareActionSignError.coldWalletUnsupported,
        '冷钱包无法在此签名',
      );
    }
    return SquareActionSignPrep(request: request, decoded: decoded, wallet: wallet);
  }

  /// 主钥签名（读硬件金库、弹生物识别）→ 构造 signResponse envelope JSON。
  Future<String> sign(SquareActionSignPrep prep, WalletManager walletManager) async {
    final signBytes = QrSigner.signingBytesForHex(
      payloadHex: prep.request.body.payloadHex,
      action: prep.request.body.action,
    );
    final signature = await walletManager.signWithWallet(prep.wallet.walletIndex, signBytes);
    final response = _signer.buildResponse(
      request: prep.request,
      signatureHex: '0x${bytesToHex(signature)}',
    );
    return _signer.encodeResponse(response);
  }

  Future<WalletProfile?> _resolveWalletByPubkey(
    WalletManager walletManager,
    Uint8List pubkey,
  ) async {
    final target = bytesToHex(pubkey);
    for (final wallet in await walletManager.getWallets()) {
      if (_normalizeHex(wallet.pubkeyHex) == target) {
        return wallet;
      }
    }
    return null;
  }

  static String _normalizeHex(String hex) {
    final text = hex.startsWith('0x') || hex.startsWith('0X') ? hex.substring(2) : hex;
    return text.toLowerCase();
  }
}
