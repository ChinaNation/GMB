import 'package:citizenapp/8964/chain/square_chain_service.dart';
import 'package:citizenapp/my/myid/identity_badge_snapshot_store.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 广场身份状态。
///
/// `account_id` 固定使用当前钱包账户；`cid_number` 只能从链上
/// 通过 `CidByAccountId`、`AccountIdByCid`、Active `CidRegistry` 和
/// `VotingIdentityByCid` 闭环读取，App 不允许自行传入链上身份。
class SquareIdentityState {
  const SquareIdentityState({
    required this.accountId,
    this.walletName,
    this.cidNumber,
    this.walletIndex,
    this.ss58Address,
    this.isHotWallet = false,
    this.identityLevel,
  });

  final String accountId;
  final String? walletName;
  final String? cidNumber;
  final int? walletIndex;
  final String? ss58Address;
  final bool isHotWallet;

  /// 链上身份档（徽章分色）：visitor/voting/candidate。
  final String? identityLevel;

  bool get hasWallet => accountId.isNotEmpty;
  bool get isCertified => cidNumber != null && cidNumber!.isNotEmpty;

  /// 竞选身份（candidate）：发布竞选内容的资格（用户 2026-07-16：发帖分类按身份档）。
  bool get isCandidate => identityLevel == 'candidate';

  String get accountLabel {
    if (!hasWallet) return '未选择钱包';
    if (accountId.length <= 14) return accountId;
    return '${accountId.substring(0, 7)}...${accountId.substring(accountId.length - 7)}';
  }
}

class SquareIdentityService {
  const SquareIdentityService({
    this.walletManager,
    this.chainService,
    this.badgeSnapshotStore,
  });

  final WalletManager? walletManager;
  final SquareChainService? chainService;
  final IdentityBadgeSnapshotStore? badgeSnapshotStore;

  /// 加载当前广场身份。
  ///
  /// [readLiveChain] 仅允许发布等主动链流程传 true；广场浏览必须传 false，
  /// 只读账户级徽章快照，不能因此启动 smoldot。
  Future<SquareIdentityState> loadCurrent({bool readLiveChain = true}) async {
    // 发动态身份统一取默认用户钱包（列表中最靠前的热钱包），与聊天同源。
    final wallet = await (walletManager ?? WalletManager()).getDefaultWallet();
    if (wallet == null) {
      return const SquareIdentityState(accountId: '');
    }
    String? cidNumber;
    String identityLevel = 'visitor';
    final snapshotStore = badgeSnapshotStore ?? IdentityBadgeSnapshotStore();
    if (readLiveChain) {
      try {
        final identity = await (chainService ?? SquareChainService())
            .fetchIdentity(wallet.accountId);
        cidNumber = identity.cidNumber;
        identityLevel = identity.identityLevel;
        try {
          await snapshotStore.write(
            accountId: wallet.accountId,
            identityLevel: identityLevel,
          );
        } catch (_) {
          // 快照写失败不影响本次发布流程使用真实链上身份。
        }
      } catch (_) {
        cidNumber = null;
        identityLevel = 'visitor';
      }
    } else {
      final snapshot = await snapshotStore.read(wallet.accountId);
      identityLevel = snapshot?.identityLevel ?? 'visitor';
    }

    return SquareIdentityState(
      accountId: wallet.accountId,
      walletName: wallet.walletName,
      cidNumber: cidNumber,
      walletIndex: wallet.walletIndex,
      ss58Address: wallet.ss58Address,
      isHotWallet: wallet.isHotWallet,
      identityLevel: identityLevel,
    );
  }
}
