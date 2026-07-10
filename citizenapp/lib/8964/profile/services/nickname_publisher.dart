import 'package:citizenapp/8964/profile/services/citizen_profile_api.dart';
import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 昵称发布器：把「默认钱包名称（= 用户昵称，单一真源）」发布到后端
/// `display_name`，让**他人**在你的主页也看到同一个昵称。
///
/// 钱包账户即用户、钱包名称即昵称，是同一个字段：本机钱包名是真源，后端
/// `display_name` 只是它的公开镜像。任一改名入口（编辑资料 / 我的钱包重命名）
/// 落盘后调此方法把默认钱包名推到后端，两侧永不分叉。best-effort：无会话或
/// 网络失败不阻塞本机改名，下次编辑 / 加载会再同步。
class NicknamePublisher {
  NicknamePublisher({
    WalletManager? walletManager,
    CitizenProfileApi? api,
    SquareSessionProvider? sessionProvider,
  })  : _wallet = walletManager ?? WalletManager(),
        _api = api ?? CitizenProfileApi(),
        _session = sessionProvider ?? SquareSessionProvider.instance;

  final WalletManager _wallet;
  final CitizenProfileApi _api;
  final SquareSessionProvider _session;

  /// 读当前默认钱包名并发布到后端 `display_name`。
  ///
  /// 总是发布**默认钱包**的名字（= 当前身份昵称），因此在「我的钱包」里改任一
  /// 钱包后调用都安全：改的是默认钱包就推新名，改的是非默认钱包就重发默认名
  /// （幂等无副作用）。会话由默认热钱包静默签名换取，不弹生物识别。
  Future<void> publishDefault() async {
    try {
      final wallet = await _wallet.getDefaultWallet();
      final name = wallet?.walletName.trim() ?? '';
      if (name.isEmpty) return;
      final session = await _session.ensureSession();
      if (session == null) return;
      await _api.updateProfile(session: session, displayName: name);
    } on Exception {
      // 后端发布失败不阻塞本机改名；下次编辑资料 / 加载主页会再同步。
    }
  }
}
