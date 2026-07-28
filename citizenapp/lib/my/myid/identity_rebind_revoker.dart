import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/wallet/core/device_subkey.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 换绑后吊销**旧身份账户**的云端隐私/鉴权数据。
///
/// 用旧账户的 **P-256 设备子钥静默建旧会话**(子钥按 walletIndex 存,与新账户同一物理
/// 子钥、后端仍登记为旧账户,故可静默登录旧账户),再调吊销端点删旧通讯录密文 / Chat
/// 材料 / 设备子钥 / 会话。无生物识别、可重试、只吊销自己——补上换绑「私钥泄漏止损」
/// 缺口(死契约 [[cid-rebind-subkeys-must-auto-migrate]] 的安全侧):换绑常因种子泄漏触发,
/// 删除旧账户云端敏感副本后,即便旧私钥泄漏也无法重建旧会话解密旧通讯录。
class IdentityRebindRevoker {
  IdentityRebindRevoker({
    SquareApiClient? apiClient,
    DeviceSubkey? deviceSubkey,
    WalletManager? walletManager,
  })  : _api = apiClient ?? SquareApiClient(),
        _deviceSubkey = deviceSubkey ?? DeviceSubkey(),
        _walletManager = walletManager ?? WalletManager();

  final SquareApiClient _api;
  final DeviceSubkey _deviceSubkey;
  final WalletManager _walletManager;

  Future<void> revokeOldAccount(String oldAccountId) async {
    final wallet = await _walletManager.getDefaultWallet();
    if (wallet == null) return;
    final session = await _api.ensureSession(
      accountId: oldAccountId,
      signLoginPayload: (loginMessage) async =>
          '0x${await _deviceSubkey.signRawHex(wallet.walletIndex, loginMessage)}',
    );
    await _api.revokeRebindOldAccount(session: session);
  }
}
