import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/material.dart';

import 'package:citizenapp/8964/chain/square_chain_service.dart';
import 'package:citizenapp/8964/services/square_identity_state.dart';
import 'package:citizenapp/8964/services/square_publish_service.dart';
import 'package:citizenapp/qr/pages/qr_sign_session_page.dart';
import 'package:citizenapp/qr/qr_protocols.dart';
import 'package:citizenapp/signer/qr_signer.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 广场发布签名器（发动态 / 发文章共用）。
///
/// 默认热钱包静默签名（`signWithWallet`，无弹窗）；冷钱包走 QR 冷签兜底，
/// 但广场身份恒为默认热钱包，冷签分支实际不触发。
class SquareComposeSigners {
  SquareComposeSigners({required this.context, required this.identity})
      : hotWalletManager = identity.isHotWallet ? WalletManager() : null;

  final BuildContext context;
  final SquareIdentityState identity;
  final WalletManager? hotWalletManager;

  Future<String> signLogin(String signingPayload) async {
    final signature = await _sign(
      payload: Uint8List.fromList(utf8.encode(signingPayload)),
      action: QrActions.login,
      requestPrefix: 'square-login-',
    );
    return '0x${_hexEncode(signature)}';
  }

  Future<Uint8List> signChain(Uint8List payload) {
    return _sign(
      payload: payload,
      action: QrActions.chain(
        SquareChainService.palletIndex,
        SquareChainService.publishSquarePostCallIndex,
      ),
      requestPrefix: 'square-post-',
    );
  }

  Future<Uint8List> _sign({
    required Uint8List payload,
    required int action,
    required String requestPrefix,
  }) async {
    final walletIndex = identity.walletIndex;
    final pubkeyHex = identity.pubkeyHex;
    if (walletIndex == null || pubkeyHex == null) {
      throw const SquarePublishException('当前钱包信息不完整');
    }
    final hotWallet = hotWalletManager;
    if (hotWallet != null) {
      return hotWallet.signWithWallet(walletIndex, payload);
    }

    final qrSigner = QrSigner();
    final request = qrSigner.buildRequest(
      requestId: QrSigner.generateRequestId(prefix: requestPrefix),
      pubkey: '0x$pubkeyHex',
      payloadHex: '0x${_hexEncode(payload)}',
      action: action,
    );
    final requestJson = qrSigner.encodeRequest(request);
    if (!context.mounted) throw const SquarePublishException('页面已关闭');
    final response = await Navigator.push<SignResponseEnvelope>(
      context,
      MaterialPageRoute(
        builder: (_) => QrSignSessionPage(
          request: request,
          requestJson: requestJson,
          expectedPubkey: '0x$pubkeyHex',
        ),
      ),
    );
    if (response == null) throw const SquarePublishException('签名已取消');
    return Uint8List.fromList(_hexDecode(response.body.signatureHex));
  }

  static String _hexEncode(List<int> bytes) =>
      bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();

  static List<int> _hexDecode(String input) {
    final text = input.startsWith('0x') ? input.substring(2) : input;
    final out = <int>[];
    for (var i = 0; i < text.length; i += 2) {
      out.add(int.parse(text.substring(i, i + 2), radix: 16));
    }
    return out;
  }
}
