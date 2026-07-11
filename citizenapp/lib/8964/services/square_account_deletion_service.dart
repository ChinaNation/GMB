import 'package:citizenapp/8964/profile/services/citizen_profile_cache.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/chat/storage/chat_store.dart';
import 'package:citizenapp/wallet/core/device_subkey.dart';

/// 注销用户编排：签名验删服务端全部数据 → 清本地零残留。
///
/// 顺序钉死：**先服务端硬删**（op_tag 0x1D 主钥签名；失败即上抛、绝不清本地，
/// 保证「服务端没删就别动本地」的一致性），**成功后再清本地**（资料缓存 / 会话缓存 /
/// Chat 私信历史 / 原生 P-256 设备子钥）。钱包与链上身份不受影响。
class SquareAccountDeletionService {
  SquareAccountDeletionService({
    SquareApiClient? apiClient,
    CitizenProfileCache? profileCache,
    DeviceSubkey? deviceSubkey,
    ChatStore? chatStore,
  })  : _api = apiClient ?? SquareApiClient(),
        _profileCache = profileCache ?? const CitizenProfileCache(),
        _deviceSubkey = deviceSubkey ?? DeviceSubkey(),
        _chatStore = chatStore ?? ChatStore();

  final SquareApiClient _api;
  final CitizenProfileCache _profileCache;
  final DeviceSubkey _deviceSubkey;
  final ChatStore _chatStore;

  /// [signAction] 对 signing_message(0x1D) 摘要用 sr25519 主钥签名（弹生物识别）。
  Future<void> deleteAccount({
    required String ownerAccount,
    required int walletIndex,
    required SquareActionSigner signAction,
  }) async {
    // 1. 服务端硬删（失败上抛 → 本地一律不动，UI/数据保持一致）。
    await _api.deleteAccount(
        ownerAccount: ownerAccount, signAction: signAction);
    // 2. 服务端确认后清本地，做到零残留。
    await _profileCache.clear(ownerAccount);
    _api.clearSession(ownerAccount);
    await _chatStore.clearAllForOwner(ownerAccount);
    // 服务端 square_device_subkeys 已 purge，删本机原生子钥迫使下次干净重注册。
    await _deviceSubkey.delete(walletIndex);
  }
}
