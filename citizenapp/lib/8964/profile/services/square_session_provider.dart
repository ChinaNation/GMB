import 'dart:convert';
import 'dart:typed_data';

import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 广场登录态提供器（全 App 共享单例）。
///
/// 默认热钱包对登录挑战串**静默签名**（无弹窗、无扫码，[WalletManager.signWithWalletNoAuth]）
/// 换取 session token，并由 [SquareApiClient] 内部按 owner 缓存复用；关注/取关等写操作
/// 复用同一 token，不逐次签名。冷钱包不可能是默认用户，此处只会用到热钱包。
class SquareSessionProvider {
  SquareSessionProvider({SquareApiClient? client, WalletManager? walletManager})
      : _client = client ?? SquareApiClient(),
        _walletManager = walletManager ?? WalletManager();

  static final SquareSessionProvider instance = SquareSessionProvider();

  final SquareApiClient _client;
  final WalletManager _walletManager;

  /// 返回默认热钱包的可用 session；无热钱包返回 null（调用方按公开只读处理）。
  Future<SquareSession?> ensureSession() async {
    final wallet = await _walletManager.getDefaultWallet();
    if (wallet == null) {
      return null;
    }
    return _client.ensureSession(
      ownerAccount: wallet.address,
      signLoginPayload: (payload) async {
        final signature = await _walletManager.signWithWalletNoAuth(
          wallet.walletIndex,
          Uint8List.fromList(utf8.encode(payload)),
        );
        return '0x${_hexEncode(signature)}';
      },
    );
  }

  static String _hexEncode(List<int> bytes) =>
      bytes.map((byte) => byte.toRadixString(16).padLeft(2, '0')).join();
}
