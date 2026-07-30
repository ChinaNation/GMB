import 'package:citizenapp/8964/chain/square_chain_service.dart';
import 'package:citizenapp/8964/profile/models/profile_presentation.dart';
import 'package:citizenapp/8964/profile/services/citizen_profile_cache.dart';
import 'package:citizenapp/my/myid/identity_account_cache.dart';
import 'package:citizenapp/my/myid/identity_badge_snapshot_store.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 广场身份状态。
///
/// `account_id` 固定使用 CID 当前绑定账户；`cid_number` 只能从链上
/// 通过 `CidByAccountId`、`AccountIdByCid`、Active `CidRegistry` 和
/// `VotingIdentityByCid` 闭环读取，App 不允许自行传入链上身份。
class SquareIdentityState {
  const SquareIdentityState({
    required this.accountId,
    this.displayName,
    this.cidNumber,
    this.walletIndex,
    this.ss58Address,
    this.isHotWallet = false,
    this.identityLevel,
  });

  final String accountId;
  final String? displayName;
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

  /// 发帖页展示公开昵称；资料尚未缓存时按 CID（无 CID 才按账户）稳定兜底。
  String get resolvedDisplayName =>
      ProfilePresentation.forIdentityKey(cidNumber ?? accountId)
          .resolveDisplayName(publicName: displayName);

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
    this.identityAccountCache,
    this.profileCache,
  });

  final WalletManager? walletManager;
  final SquareChainService? chainService;
  final IdentityBadgeSnapshotStore? badgeSnapshotStore;
  final IdentityAccountCache? identityAccountCache;
  final CitizenProfileCache? profileCache;

  /// 加载当前广场身份。
  ///
  /// 身份主键 = CID 号:`accountId`/`ss58`/`cid` 跟随**身份账户**(CID 绑定账户,
  /// 常态=账户0),`walletIndex` 保持钱包级(设备子钥按 walletIndex 存)、公开昵称
  /// 只从 CID 资料缓存读取。[readLiveChain] 仅允许发布等主动链流程传 true;广场浏览必须传 false,
  /// 只读 CID 级徽章快照，不能因此启动 smoldot（身份账户解析同样按此不链读）。
  Future<SquareIdentityState> loadCurrent({bool readLiveChain = true}) async {
    final wallet = await (walletManager ?? WalletManager()).getDefaultWallet();
    if (wallet == null) {
      return const SquareIdentityState(accountId: '');
    }
    // 身份账户（CID 当前绑定账户）；禁止链读时只使用已命中的身份缓存，
    // 未命中则账户0仅作为访客展示账户，绝不据此授予 CID 权限。
    final identity =
        await (identityAccountCache ?? IdentityAccountCache.instance)
            .resolve(allowChainRead: readLiveChain);
    final identityAccountId = identity?.accountId ?? wallet.accountId;
    final identitySs58 = identity?.ss58Address ?? wallet.ss58Address;

    String? cidNumber = identity?.snapshot?.cidNumber;
    String identityLevel = 'visitor';
    final snapshotStore = badgeSnapshotStore ?? IdentityBadgeSnapshotStore();
    if (readLiveChain) {
      try {
        final chainIdentity = await (chainService ?? SquareChainService())
            .fetchIdentity(identityAccountId);
        final liveCidNumber = chainIdentity.cidNumber?.trim();
        cidNumber = liveCidNumber == null || liveCidNumber.isEmpty
            ? null
            : liveCidNumber;
        identityLevel = chainIdentity.identityLevel;
        if (cidNumber != null) {
          try {
            await snapshotStore.write(
              cidNumber: cidNumber,
              identityLevel: identityLevel,
            );
          } catch (_) {
            // 快照写失败不影响本次发布流程使用真实链上身份。
          }
        }
      } catch (_) {
        cidNumber = null;
        identityLevel = 'visitor';
      }
    } else {
      final snapshot =
          cidNumber == null ? null : await snapshotStore.read(cidNumber);
      identityLevel = snapshot?.identityLevel ?? 'visitor';
    }

    String? displayName;
    final profileCid = cidNumber?.trim() ?? '';
    if (profileCid.isNotEmpty) {
      final profile =
          await (profileCache ?? const CitizenProfileCache()).read(profileCid);
      final cachedName = profile?.displayName.trim() ?? '';
      if (cachedName.isNotEmpty) {
        displayName = cachedName;
      }
    }

    return SquareIdentityState(
      accountId: identityAccountId,
      displayName: displayName,
      cidNumber: cidNumber,
      walletIndex: wallet.walletIndex,
      ss58Address: identitySs58,
      isHotWallet: wallet.isHotWallet,
      identityLevel: identityLevel,
    );
  }
}
