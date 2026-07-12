import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/8964/services/device_subkey_registrar.dart';
import 'package:citizenapp/wallet/core/device_subkey.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 广场登录态提供器（全 App 共享单例）。
///
/// 后端会话握手用**默认热钱包的 P-256 硬件设备子钥静默签名**（不读 seed、不弹
/// 生物识别）换取 session token，由 [SquareApiClient] 内部按 owner 缓存复用。
///
/// 子钥注册只在**钱包创建时**静默完成（[WalletManager] 用内存 keypair 签，见
/// `subkeyRegistrar`）；后台会话流程**绝不读 seed、绝不弹窗、绝不懒注册**——未注册
/// 的钱包（如旧格式钱包）会话直接失败按公开只读处理，用户重建钱包即注册。
class SquareSessionProvider {
  SquareSessionProvider({
    SquareApiClient? client,
    WalletManager? walletManager,
    DeviceSubkey? deviceSubkey,
  })  : _client = client ?? SquareApiClient(),
        _walletManager = walletManager ?? WalletManager(),
        _deviceSubkey = deviceSubkey ?? DeviceSubkey();

  static final SquareSessionProvider instance = SquareSessionProvider();

  final SquareApiClient _client;
  final WalletManager _walletManager;
  final DeviceSubkey _deviceSubkey;

  /// 返回默认热钱包的可用 session；无热钱包返回 null（调用方按公开只读处理）。
  Future<SquareSession?> ensureSession() async {
    final wallet = await _walletManager.getDefaultWallet();
    if (wallet == null) {
      return null;
    }
    return _client.ensureSession(
      ownerAccount: wallet.address,
      signLoginPayload: (loginMessage) async {
        // 会话握手 = 非用户动权 → P-256 硬件子钥静默签名 signing_message 摘要（后端 ES256 验，不读 seed）。
        final raw =
            await _deviceSubkey.signRawHex(wallet.walletIndex, loginMessage);
        return '0x$raw';
      },
      onDeviceNotRegistered: () => DeviceSubkeyRegistrar(
        apiClient: _client,
        deviceSubkey: _deviceSubkey,
      ).register(
        walletIndex: wallet.walletIndex,
        ownerAccount: wallet.address,
        signBinding: (message) async {
          final signature = await _walletManager.signWithWallet(
            wallet.walletIndex,
            message,
          );
          return '0x${bytesToHex(signature)}';
        },
      ),
    );
  }
}
