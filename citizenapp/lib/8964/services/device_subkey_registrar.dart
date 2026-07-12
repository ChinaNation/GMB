import 'dart:typed_data';

import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/wallet/core/device_subkey.dart';

/// 对设备绑定证明消息（`signing_message` 的 32 字节摘要）做 sr25519 主钥签名，
/// 返回 `0x` hex 签名。
typedef DeviceBindingSigner = Future<String> Function(Uint8List bindingMessage);
typedef TurnstileTokenProvider = Future<String?> Function();

/// 编排 P-256 设备子钥注册：取子钥公钥 → 构造 `signing_message(OP_SIGN_SQUARE_DEVICE_BIND)`
/// 32B 摘要 → sr25519 主钥签摘要 → 上报后端。
///
/// 于**钱包创建时**调用：用内存里刚派生的 sr25519 keypair 签名（零额外弹窗）。
class DeviceSubkeyRegistrar {
  static TurnstileTokenProvider? turnstileTokenProvider;

  DeviceSubkeyRegistrar({
    DeviceSubkey? deviceSubkey,
    SquareApiClient? apiClient,
    TurnstileTokenProvider? turnstileToken,
  })  : _subkey = deviceSubkey ?? DeviceSubkey(),
        _api = apiClient ?? SquareApiClient(),
        _turnstileToken = turnstileToken ?? turnstileTokenProvider;

  final DeviceSubkey _subkey;
  final SquareApiClient _api;
  final TurnstileTokenProvider? _turnstileToken;

  Future<void> register({
    required int walletIndex,
    required String ownerAccount,
    required DeviceBindingSigner signBinding,
    int? issuedAtMillis,
  }) async {
    final pubkey = await _subkey.publicKeyHex(walletIndex);
    final issuedAt = issuedAtMillis ?? DateTime.now().millisecondsSinceEpoch;
    final message =
        buildDeviceBindingSigningMessage(ownerAccount, pubkey, issuedAt);
    final signatureHex = await signBinding(message);
    final turnstileToken = await _turnstileToken?.call();
    await _api.registerDeviceSubkey(
      ownerAccount: ownerAccount,
      p256PubkeyHex: pubkey,
      issuedAt: issuedAt,
      bindingSignatureHex: signatureHex,
      turnstileToken: turnstileToken,
    );
  }
}
