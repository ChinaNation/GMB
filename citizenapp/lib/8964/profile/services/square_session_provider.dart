import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/my/myid/identity_account_cache.dart';
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
    IdentityAccountCache? identityAccountCache,
  })  : _client = client ?? SquareApiClient(),
        _walletManager = walletManager ?? WalletManager(),
        _deviceSubkey = deviceSubkey ?? DeviceSubkey(),
        _identityAccountCache = identityAccountCache;

  static final SquareSessionProvider instance = SquareSessionProvider();

  final SquareApiClient _client;
  final WalletManager _walletManager;
  final DeviceSubkey _deviceSubkey;
  final IdentityAccountCache? _identityAccountCache;

  IdentityAccountCache get _identityCache =>
      _identityAccountCache ?? IdentityAccountCache.instance;

  /// 返回当前**身份账户**的可用 session；无热钱包返回 null（调用方按不可用处理）。
  ///
  /// **身份主键 = CID 号**:会话 `accountId` 取 CID 绑定的身份账户([IdentityAccountCache]),
  /// 而设备子钥签名仍用钱包 `walletIndex`(P-256 子钥按 walletIndex 存,与 accountId
  /// **解耦**)。会话只有这一条身份级入口，不再存在钱包名同步专用会话。
  Future<SquareSession?> ensureSession() async {
    final wallet = await _walletManager.getDefaultWallet();
    if (wallet == null || !wallet.isHotWallet) {
      return null;
    }
    final identityAccountId =
        await _identityCache.accountId() ?? wallet.accountId;
    return _client.ensureSession(
      accountId: identityAccountId,
      signLoginPayload: (loginMessage) async {
        // 会话握手 = 非用户动权 → P-256 硬件子钥静默签名(后端 ES256 验,不读 seed)。
        final raw =
            await _deviceSubkey.signRawHex(wallet.walletIndex, loginMessage);
        return '0x$raw';
      },
    );
  }
}
