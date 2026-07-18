import 'package:citizenapp/8964/services/square_api_client.dart'
    show SquareApiClient, SquareApiException;
import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/my/creator/creator_api.dart';
import 'package:citizenapp/my/creator/models/creator_overview.dart';
import 'package:citizenapp/my/creator/models/creator_plan.dart';
import 'package:citizenapp/wallet/core/device_subkey.dart' show bytesToHex;
import 'package:citizenapp/wallet/core/secure_seed_store.dart'
    show SecureSeedException;
import 'package:citizenapp/wallet/core/seed_sign_error.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 创作者页展示态：门禁（需先成为平台会员）/ 已开通（含计划与概览）。

class CreatorPageData {
  const CreatorPageData._({required this.gated, this.plan, this.overview});

  /// true = 非平台会员（或无热钱包/会话）→ 展示门禁态。
  final bool gated;
  final CreatorPlan? plan;
  final CreatorOverview? overview;

  factory CreatorPageData.gated() => const CreatorPageData._(gated: true);

  factory CreatorPageData.active({
    required CreatorPlan plan,
    required CreatorOverview overview,
  }) =>
      CreatorPageData._(gated: false, plan: plan, overview: overview);
}

class CreatorException implements Exception {
  const CreatorException(this.message);
  final String message;
  @override
  String toString() => message;
}

/// 创作者管理编排：门禁校验（链上平台会员态，经 Cloudflare 读）+ 读写档位 + 概览。
///
/// - 门禁数据源复用现有 `SquareApiClient.fetchMembership`（平台会员=链上态镜像）。
/// - 保存档位复用现有广场账户动作统一签名（0x1D），主钥签名读硬件金库触发生物识别；
///   离链写入不新增任何签名协议。
class CreatorService {
  CreatorService({
    CreatorApi? api,
    WalletManager? walletManager,
    SquareSessionProvider? sessionProvider,
    SquareApiClient? squareApiClient,
  })  : _api = api ?? CreatorApiHttp(),
        _wallet = walletManager ?? WalletManager(),
        _session = sessionProvider ?? SquareSessionProvider.instance,
        _square = squareApiClient ?? SquareApiClient();

  final CreatorApi _api;
  final WalletManager _wallet;
  final SquareSessionProvider _session;
  final SquareApiClient _square;

  /// 首屏加载：无热钱包/会话或非平台会员 → 门禁态；否则并行拉档位 + 概览。
  Future<CreatorPageData> load() async {
    final session = await _session.ensureSession();
    if (session == null) return CreatorPageData.gated();

    // 门禁：创作者必须是当前有效平台会员（平台会员态=链上态镜像）。
    final membership = await _square.fetchMembership(session);
    if (!membership.active) return CreatorPageData.gated();

    final results = await Future.wait([
      _api.fetchMyPlan(session),
      _api.fetchOverview(session),
    ]);
    final plan = results[0] as CreatorPlan?;
    final overview = results[1] as CreatorOverview;
    return CreatorPageData.active(
      plan: plan ?? CreatorPlan.empty(session.ownerAccount),
      overview: overview,
    );
  }

  /// 覆盖式保存档位。★核心操作：主钥签名（生物识别）经统一 0x1D 动作往返写 Cloudflare。
  Future<CreatorPlan> saveTiers(List<CreatorTier> tiers) async {
    if (tiers.length > CreatorPlan.maxTiers) {
      throw const CreatorException('最多 ${CreatorPlan.maxTiers} 个会员档');
    }
    final wallet = await _wallet.getDefaultWallet();
    if (wallet == null || !wallet.isHotWallet) {
      throw const CreatorException('请先在「我的 → 我的钱包」创建热钱包');
    }
    final session = await _session.ensureSession();
    if (session == null) {
      throw const CreatorException('会话不可用，请稍后重试');
    }
    try {
      return await _api.saveMyPlan(
        session: session,
        ownerAccount: wallet.address,
        tiers: tiers,
        signAction: (message) async {
          // 主钥对 0x1D 摘要签名：读硬件金库 → 弹一次生物识别 → 64B sr25519。
          final signature =
              await _wallet.signWithWallet(wallet.walletIndex, message);
          return '0x${bytesToHex(signature)}';
        },
      );
    } on SecureSeedException catch (e) {
      // 生物识别取消 / 无锁屏等：单源文案，杜绝静默失败。
      throw CreatorException(seedSignErrorMessage(e));
    } on WalletAuthException catch (e) {
      throw CreatorException(e.message);
    } on CreatorApiException catch (e) {
      throw CreatorException(e.message);
    } on SquareApiException catch (e) {
      throw CreatorException(e.message);
    } on Exception catch (e) {
      throw CreatorException('保存失败：$e');
    }
  }
}
