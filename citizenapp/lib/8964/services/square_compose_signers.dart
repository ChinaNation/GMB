import 'dart:typed_data';

import 'package:flutter/material.dart';

import 'package:citizenapp/8964/chain/square_chain_service.dart';
import 'package:citizenapp/8964/services/square_identity_state.dart';
import 'package:citizenapp/8964/services/square_publish_service.dart';
import 'package:citizenapp/qr/pages/qr_sign_session_page.dart';
import 'package:citizenapp/qr/qr_protocols.dart';
import 'package:citizenapp/signer/qr_signer.dart';
import 'package:citizenapp/wallet/core/device_subkey.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 广场发布签名器（发动态 / 发文章共用）。
///
/// 登录挑战 = 后端会话握手 → **P-256 硬件设备子钥静默签名**（不读 seed、不弹）。
/// 发布上链 = 动钱动权 → 读硬件金库时弹一次生物识别。冷钱包走 QR 冷签兜底，但广场
/// 身份恒为默认热钱包，冷签分支实际不触发。
class SquareComposeSigners {
  SquareComposeSigners({
    required this.context,
    required this.identity,
    DeviceSubkey? deviceSubkey,
  })  : hotWalletManager = identity.isHotWallet ? WalletManager() : null,
        _deviceSubkey = deviceSubkey ?? DeviceSubkey();

  final BuildContext context;
  final SquareIdentityState identity;
  final WalletManager? hotWalletManager;
  final DeviceSubkey _deviceSubkey;

  Future<String> signLogin(Uint8List loginMessage) async {
    final walletIndex = identity.walletIndex;
    if (walletIndex == null) {
      throw const SquarePublishException('当前钱包信息不完整');
    }
    if (hotWalletManager == null) {
      // 广场身份恒为默认热钱包；冷钱包不参与后端会话握手。
      throw const SquarePublishException('冷钱包不支持广场登录');
    }
    // 会话握手 = 非用户动权 → P-256 硬件子钥静默签名 signing_message(0x1B) 摘要，后端 ES256 验。
    final raw = await _deviceSubkey.signRawHex(walletIndex, loginMessage);
    return '0x$raw';
  }

  Future<Uint8List> signChain(Uint8List payload) {
    // 发布上链 = 动钱动权 → 读硬件金库 seed 时弹一次生物识别验证。
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
