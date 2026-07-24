import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/wallet/core/device_subkey.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 广场登录态提供器（全 App 共享单例）。
///
/// 后端会话握手用**默认热钱包的 P-256 硬件设备子钥静默签名**（不读 seed、不弹
/// 生物识别）换取 session token，由 [SquareApiClient] 内部按 accountId 缓存复用。
///
/// 子钥注册只在**创建 / 导入钱包时**静默完成（[WalletManager] 用内存 keypair 签，见
/// `subkeyRegistrar`）；后台会话流程**绝不读 seed、绝不弹窗、绝不懒注册**——拿不到
/// session（无热钱包 / 未注册）时广场与聊天按**不可用**处理，绝不在此补注册。
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

  /// 返回默认热钱包的可用 session；无热钱包返回 null（调用方按不可用处理，不放行浏览）。
  Future<SquareSession?> ensureSession() async {
    final wallet = await _walletManager.getDefaultWallet();
    if (wallet == null) {
      return null;
    }
    return ensureSessionFor(wallet);
  }

  /// 返回**指定钱包**账户的可用 session。
  ///
  /// 钱包名同步需要把每个钱包的名字推到**它自己 accountId** 的 display_name
  /// （只推默认钱包会让云端存不全），故需按钱包换会话，而非固定默认钱包。
  ///
  /// 冷钱包没有设备子钥、云端也无其资料，直接返回 null。
  ///
  /// 与 [ensureSession] 同一条死契约：**绝不懒注册、绝不弹 Turnstile、绝不读 seed**。
  /// 未注册设备子钥的钱包在此直接失败按不可用处理，注册只在 WalletManager
  /// 创建 / 导入钱包时静默完成（`subkeyRegistrar`）。冷启动广场并发拉
  /// feed/membership/identity 都走这里，越界懒注册会把主线程顶死成 ANR。
  Future<SquareSession?> ensureSessionFor(WalletProfile wallet) async {
    if (!wallet.isHotWallet) {
      return null;
    }
    return _client.ensureSession(
      accountId: wallet.accountId,
      signLoginPayload: (loginMessage) async {
        // 会话握手 = 非用户动权 → P-256 硬件子钥静默签名 signing_message 摘要（后端 ES256 验，不读 seed）。
        final raw =
            await _deviceSubkey.signRawHex(wallet.walletIndex, loginMessage);
        return '0x$raw';
      },
    );
  }
}
