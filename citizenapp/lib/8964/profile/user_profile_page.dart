import 'dart:async';

import 'package:flutter/material.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/pages/square_article_detail_page.dart';
import 'package:citizenapp/8964/pages/square_post_detail_page.dart';
import 'package:citizenapp/8964/profile/follows_list_page.dart';
import 'package:citizenapp/8964/profile/models/citizen_profile.dart';
import 'package:citizenapp/8964/profile/models/profile_presentation.dart';
import 'package:citizenapp/8964/profile/profile_edit_page.dart';
import 'package:citizenapp/8964/profile/services/citizen_profile_api.dart';
import 'package:citizenapp/8964/profile/services/citizen_profile_cache.dart';
import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/profile/user_qr_page.dart';
import 'package:citizenapp/8964/profile/widgets/collapsible_header.dart';
import 'package:citizenapp/8964/profile/widgets/creator_subscribe_button.dart';
import 'package:citizenapp/8964/profile/widgets/profile_action_icons.dart';
import 'package:citizenapp/8964/profile/widgets/profile_category_tabs.dart';
import 'package:citizenapp/8964/profile/widgets/profile_header_card.dart';
import 'package:citizenapp/8964/profile/widgets/profile_kebab_menu.dart';
import 'package:citizenapp/8964/profile/widgets/profile_posts_list.dart';
import 'package:citizenapp/8964/services/square_account_deletion_service.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/chat/open_direct_chat.dart';
import 'package:citizenapp/my/myid/identity_account_cache.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/wallet/core/device_subkey.dart' show bytesToHex;
import 'package:citizenapp/wallet/core/secure_seed_store.dart';
import 'package:citizenapp/wallet/core/seed_sign_error.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 推特式用户主页。
///
/// 折叠虚化头部 + 圆角方形头像/背景（R2）+ 认证勾 + 展示名/地址/签名/计数 +
/// 三图标（本人 通知/聊天/关注 · 他人 关注/消息）+ ⋮（二维码/编辑资料）+
/// 帖子/竞选/照片/视频/文章五 Tab。身份主键 = CID 号（cid_number）；cache-first
/// 加载，关注复用登录 session 静默签名，公开资料只进 R2、不上链。链上订阅/私信/
/// 二维码需要的钱包账户 account_id 从已拉取的 profile.account_id（当前绑定账户）取。
class UserProfilePage extends StatefulWidget {
  const UserProfilePage({
    super.key,
    required this.cidNumber,
    required this.isSelf,
    this.initialProfile,
    this.api,
    this.cache,
    this.sessionProvider,
    this.onOpenDirectChat,
    this.viewerAccountLoader,
  });

  /// 主页身份主键 = CID 号（cid_number）。资料/关注/帖子全按此 cid 寻址。
  final String cidNumber;

  /// 本人主页（可编辑资料）还是他人主页。
  final bool isSelf;

  /// 当前浏览者账户加载器（测试可注入）；默认取本机默认热钱包地址。
  /// 用于判定「他人视角看的其实是自己账户」时把动作按钮置灰。
  final Future<String?> Function()? viewerAccountLoader;

  /// 首屏可选注入的资料（缓存或上层已拉到的）。
  final CitizenProfile? initialProfile;

  /// 数据入口，测试可注入替身。
  final CitizenProfileApi? api;
  final CitizenProfileCache? cache;
  final SquareSessionProvider? sessionProvider;

  /// 私聊入口，测试可注入 spy；默认走正式 ChatRuntime。
  final DirectChatOpener? onOpenDirectChat;

  @override
  State<UserProfilePage> createState() => _UserProfilePageState();
}

class _UserProfilePageState extends State<UserProfilePage> {
  /// 顶部头图高度（不含状态栏）。
  static const double _bannerHeight = 128;

  /// 头部展开总高（头图 + 白底资料区），不含状态栏。资料区为昵称、SS58、
  /// CID、签名与计数预留独立行，避免窄屏时与分类标签重叠。
  static const double _expandedHeight = 372;

  late final CitizenProfileApi _api;
  late final CitizenProfileCache _cache;
  late final SquareSessionProvider _sessionProvider;
  late final DirectChatOpener _directChat;
  CitizenProfile? _profile;
  SquareSession? _session;
  Future<SquareSession?>? _sessionFuture;
  bool _sessionResolved = false;
  int _postsRevision = 0;

  /// 「他人视角」下看的是不是自己账户；true → 关注/私信/通知/订阅按钮置灰不可点。
  bool _isOwnAccount = false;

  @override
  void initState() {
    super.initState();
    _api = widget.api ?? CitizenProfileApi();
    _cache = widget.cache ?? const CitizenProfileCache();
    _sessionProvider = widget.sessionProvider ?? SquareSessionProvider.instance;
    _directChat = widget.onOpenDirectChat ?? openDirectChat;
    _profile = widget.initialProfile;
    // 「他人视角看的其实是自己」判定需要目标当前绑定账户（profile.account_id），
    // 故在资料加载后（_load）再算；注入了初始资料时先算一次。
    if (_profile != null) {
      _resolveOwnAccount(_profile!.accountId);
    }
    _load();
  }

  /// 判定「他人视角看的其实是自己账户」：浏览者身份账户 == 目标当前绑定钱包账户
  /// （[targetAccountId] = profile.account_id）。身份主键是 cid，但浏览者手上只有
  /// 自己的账户（IdentityAccountCache），故用「账户是否一致」判定同一身份。
  /// 本人视角（isSelf）按钮本就隐藏，无需判定；判定失败按非本人处理，不阻塞主页。
  Future<void> _resolveOwnAccount(String? targetAccountId) async {
    if (widget.isSelf || _isOwnAccount) return;
    final target = targetAccountId?.trim() ?? '';
    if (target.isEmpty) return;
    // 浏览者身份账户 = CID 绑定账户（单源 IdentityAccountCache），与本人身份展示同口径。
    final loadViewer = widget.viewerAccountLoader ??
        () async => IdentityAccountCache.instance.accountId();
    try {
      final viewer = (await loadViewer())?.trim() ?? '';
      if (!mounted) return;
      if (viewer.isNotEmpty && viewer == target) {
        setState(() => _isOwnAccount = true);
      }
    } on Exception {
      // 判定失败按非本人处理。
    }
  }

  Future<void> _load() async {
    // 先渲染缓存（若无注入资料），再后台刷新回刷 + 写回缓存。
    if (_profile == null) {
      final cached = await _cache.read(widget.cidNumber);
      if (cached != null && mounted) {
        setState(() => _profile = cached);
        unawaited(_resolveOwnAccount(cached.accountId));
      }
    }
    final session = await _ensureSession();
    try {
      // 带 session 拉取 → is_following 反映当前登录者视角。
      final fresh = await _api.fetchProfile(widget.cidNumber, session: session);
      if (!mounted) return;
      setState(() => _profile = fresh);
      unawaited(_resolveOwnAccount(fresh.accountId));
      await _cache.write(fresh);
    } on Exception {
      // 网络/服务异常保留缓存或占位，不覆盖已展示内容。
    }
  }

  /// 默认热钱包静默登录换 Session；同一时刻只允许一个握手。首次结果落地前
  /// [_sessionResolved] 保持 false，让帖子 Tab 等待而不是用 null 抢跑请求 Worker。
  Future<SquareSession?> _ensureSession({bool refresh = false}) {
    final pending = _sessionFuture;
    if (pending != null) return pending;
    if (refresh && mounted) {
      setState(() => _sessionResolved = false);
    }
    late final Future<SquareSession?> future;
    future = _resolveSession(refresh).whenComplete(() {
      if (identical(_sessionFuture, future)) _sessionFuture = null;
    });
    _sessionFuture = future;
    return future;
  }

  /// 把成功、无钱包和异常统一收敛成同一个共享 Future，保证并发调用者不会有的收到
  /// null、有的却收到未捕获异常。
  Future<SquareSession?> _resolveSession(bool refresh) async {
    try {
      final session = await (refresh
          ? _sessionProvider.refreshSession()
          : _sessionProvider.ensureSession());
      if (mounted) {
        setState(() {
          _session = session;
          _sessionResolved = true;
        });
      }
      return session;
    } on Exception {
      if (mounted) {
        setState(() {
          _session = null;
          _sessionResolved = true;
        });
      }
      return null;
    }
  }

  /// 帖子请求收到 401 时清缓存并只重新握手一次；新 Session 仍由本页统一持有。
  Future<SquareSession?> _refreshSessionAfterUnauthorized() =>
      _ensureSession(refresh: true);

  Future<void> _toggleFollow() async {
    final current = _profile;
    if (current == null) return;
    final session = _session ?? await _ensureSession();
    if (session == null) {
      _snack('请先在「我的 → 我的钱包」创建热钱包');
      return;
    }
    final wasFollowing = current.isFollowing;
    final nextFollowers = wasFollowing
        ? (current.followers > 0 ? current.followers - 1 : 0)
        : current.followers + 1;
    // 乐观更新。
    setState(() {
      _profile = current.copyWith(
        isFollowing: !wasFollowing,
        // 关注即默认开通知，取关即无通知；与 Worker（关注写入默认 notify_enabled=1、
        // 取关删记录）保持一致，铃铛态随关注态即时联动。
        isNotifying: !wasFollowing,
        followers: nextFollowers,
      );
    });
    try {
      if (wasFollowing) {
        await _api.unfollowUser(
          session: session,
          followedCidNumber: widget.cidNumber,
        );
      } else {
        await _api.followUser(
          session: session,
          followedCidNumber: widget.cidNumber,
        );
      }
    } on Exception {
      if (!mounted) return;
      setState(() => _profile = current); // 失败回滚。
      _snack('操作失败，请重试');
    }
  }

  /// 开/关该用户的发帖通知（红点+声音）。通知归属挂在关注关系上：未关注先提示去关注；
  /// 关注后铃铛按用户静音/取消静音，静音不影响其内容在关注流展示。
  Future<void> _toggleNotify() async {
    final current = _profile;
    if (current == null) return;
    if (!current.isFollowing) {
      _snack('请先关注 TA 再开启通知');
      return;
    }
    final session = _session ?? await _ensureSession();
    if (session == null) {
      _snack('请先在「我的 → 我的钱包」创建热钱包');
      return;
    }
    final wasNotifying = current.isNotifying;
    // 乐观更新。
    setState(() {
      _profile = current.copyWith(isNotifying: !wasNotifying);
    });
    try {
      await _api.setNotify(
        session: session,
        followedCidNumber: widget.cidNumber,
        enabled: !wasNotifying,
      );
    } on Exception {
      if (!mounted) return;
      setState(() => _profile = current); // 失败回滚。
      _snack('操作失败，请重试');
    }
  }

  Future<void> _openEditProfile() async {
    final updated = await Navigator.of(context).push<CitizenProfile>(
      MaterialPageRoute<CitizenProfile>(
        builder: (_) => CitizenProfileEditPage(
          cidNumber: widget.cidNumber,
          initialProfile: _profile,
          api: _api,
          sessionProvider: _sessionProvider,
        ),
      ),
    );
    if (updated == null || !mounted) return;
    setState(() => _profile = updated);
    await _cache.write(updated);
  }

  /// 注销用户（仅本人）：二次确认 → 主钥签名(生物识别) → 服务端硬删 → 清本地 → 回落空态。
  /// 无冷静期、硬删不可逆；链上数据与本地钱包不受影响。
  Future<void> _openDeleteAccount() async {
    // 注销目标始终是页面永久 CID；profile.account_id 只作为该 CID 当前绑定账户完成
    // 会话与主钥签名授权，Worker 会再用 finalized 链双向绑定复核，不能决定删除范围。
    final selfAccountId = _profile?.accountId.trim() ?? '';
    if (selfAccountId.isEmpty) {
      _snack('资料尚未加载，请稍后再试');
      return;
    }
    final walletManager = WalletManager();
    final walletIndex = await walletManager.getDefaultWalletIndex();
    if (!mounted) return;
    if (walletIndex == null) {
      _snack('未找到可用热钱包，无法注销');
      return;
    }

    final confirmed = await showDialog<bool>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: const Text('注销用户'),
        content: const Text(
          '注销将立即硬删除你在公民广场/私信的全部数据，无冷静期、不可恢复，链上数据不受注销影响。',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(dialogContext).pop(false),
            child: const Text('取消'),
          ),
          TextButton(
            style: TextButton.styleFrom(foregroundColor: AppTheme.danger),
            onPressed: () => Navigator.of(dialogContext).pop(true),
            child: const Text('确认注销'),
          ),
        ],
      ),
    );
    if (confirmed != true || !mounted) return;

    try {
      await SquareAccountDeletionService().deleteAccount(
        cidNumber: widget.cidNumber,
        accountId: selfAccountId,
        walletIndex: walletIndex,
        // 动钱动权 → sr25519 **身份账户**主钥对 0x1D 摘要签名（selfAccountId = 当前绑定
        // 身份账户，按 accountId 精确取硬件金库 child，弹一次生物识别）。walletIndex 仅供
        // 门控与删除本机设备子钥（设备子钥按 walletIndex 存，与身份账户解耦）。
        signAction: (message) async =>
            '0x${bytesToHex(await walletManager.signForAccountId(selfAccountId, message))}',
      );
    } on SquareAccountLocalCleanupException catch (e) {
      // Worker 已经完成不可逆注销；此时不能误报“注销失败”诱导用户重复提交。
      if (mounted) _snack(e.toString());
      if (mounted) {
        Navigator.of(context).popUntil((route) => route.isFirst);
      }
      return;
    } on SquareApiException catch (e) {
      if (mounted) _snack('注销失败：${e.message}');
      return;
    } on SecureSeedException catch (e) {
      // 生物识别取消 / 无锁屏 / 金库错误：不属 WalletAuthException，
      // 此前会逃逸成无声失败（点注销后无反应）。
      if (mounted) _snack(seedSignErrorMessage(e));
      return;
    } on WalletAuthException catch (e) {
      if (mounted) _snack('注销已取消：${e.message}');
      return;
    } on Exception catch (e) {
      // 兜底：注销签名的任何异常都必须有反馈，永不静默。
      if (mounted) _snack('注销失败：$e');
      return;
    }

    if (!mounted) return;
    _snack('账户已注销');
    Navigator.of(context).popUntil((route) => route.isFirst);
  }

  void _openQrCode() {
    // 名片码载荷 = 钱包账户 account_id（加好友/转账用），取该身份当前绑定账户。
    final accountId = _profile?.accountId.trim() ?? '';
    if (accountId.isEmpty) {
      _snack('资料尚未加载，请稍后再试');
      return;
    }
    Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => UserQrPage(
          cidNumber: widget.cidNumber,
          displayName: _displayName,
          accountId: accountId,
        ),
      ),
    );
  }

  void _openChatWithUser() {
    final peerCidNumber = widget.cidNumber.trim();
    if (peerCidNumber.isEmpty) {
      _snack('资料尚未加载，请稍后再试');
      return;
    }
    _directChat(
      context,
      peerCidNumber: peerCidNumber,
      title: _displayName,
    );
  }

  void _openFollows(FollowsType type) {
    final session = _session;
    if (session == null) {
      _snack('需要钱包账户才能浏览关注列表');
      return;
    }
    Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => FollowsListPage(
          cidNumber: widget.cidNumber,
          type: type,
          session: session,
          api: _api,
        ),
      ),
    );
  }

  void _snack(String message) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(message)),
    );
  }

  /// 公开昵称只取后端 `display_name`；缺失时按 CID 稳定生成本地占位昵称。
  String get _displayName {
    return ProfilePresentation.forIdentityKey(widget.cidNumber)
        .resolveDisplayName(
      publicName: _profile?.displayName,
    );
  }

  String get _title => _displayName;

  String? _mediaUrl(String? objectKey) =>
      objectKey == null ? null : _api.mediaUrl(objectKey);

  Map<String, String>? get _mediaHeaders => _session == null
      ? null
      : <String, String>{
          'authorization': 'Bearer ${_session!.sessionToken}',
        };

  Widget _bannerWidget() {
    final fallback = Image.asset(
      ProfilePresentation.forIdentityKey(widget.cidNumber).bannerAsset,
      fit: BoxFit.cover,
    );
    final url = _mediaUrl(_profile?.bannerObjectKey);
    if (url == null) return fallback;
    return Image.network(
      url,
      headers: _mediaHeaders,
      fit: BoxFit.cover,
      errorBuilder: (_, __, ___) => fallback,
    );
  }

  /// 他人主页的「订阅 TA」按钮：SquarePost 以公民 CID 为创作者唯一主键。
  /// 本人主页不显示；CID 不因换绑改变，因此换绑不会重建或丢失订阅态。
  Widget? _creatorSubscribeButton() {
    if (widget.isSelf) return null;
    final creatorCidNumber = widget.cidNumber.trim();
    if (creatorCidNumber.isEmpty) return null;
    return CreatorSubscribeButton(
      key: ValueKey<String>('creator-subscribe:$creatorCidNumber'),
      creatorCidNumber: creatorCidNumber,
      enabled: !_isOwnAccount,
    );
  }

  void _stub(String label) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text('「$label」待接入')),
    );
  }

  Future<void> _openPost(SquarePost post) async {
    final result = await Navigator.of(context).push<SquarePostDetailResult>(
      MaterialPageRoute<SquarePostDetailResult>(
        builder: (_) => SquarePostDetailPage(post: post),
      ),
    );
    if (result != null && mounted) {
      setState(() => _postsRevision += 1);
    }
  }

  Future<void> _openArticle(SquarePost post) async {
    final result = await Navigator.of(context).push<SquarePostDetailResult>(
      MaterialPageRoute<SquarePostDetailResult>(
        builder: (_) => SquareArticleDetailPage(post: post),
      ),
    );
    if (result != null && mounted) {
      setState(() => _postsRevision += 1);
    }
  }

  Widget _tabBody(ProfileTab tab) {
    final session = _session;
    switch (tab) {
      case ProfileTab.posts:
        return ProfilePostsTab(
          key: ValueKey('posts:$_postsRevision'),
          cidNumber: widget.cidNumber,
          api: _api,
          category: SquarePostCategory.normal,
          contentFormat: SquarePostContentFormat.normal,
          emptyLabel: '还没有帖子',
          session: session,
          sessionReady: _sessionResolved,
          onSessionExpired: _refreshSessionAfterUnauthorized,
          isSelf: widget.isSelf,
          onOpenPost: _openPost,
        );
      case ProfileTab.campaign:
        return ProfilePostsTab(
          key: ValueKey('campaign:$_postsRevision'),
          cidNumber: widget.cidNumber,
          api: _api,
          category: SquarePostCategory.campaign,
          emptyLabel: '还没有竞选内容',
          session: session,
          sessionReady: _sessionResolved,
          onSessionExpired: _refreshSessionAfterUnauthorized,
          isSelf: widget.isSelf,
          onOpenPost: _openPost,
        );
      case ProfileTab.photos:
        return ProfilePostsTab(
          key: ValueKey('photos:$_postsRevision'),
          cidNumber: widget.cidNumber,
          api: _api,
          mediaKind: SquareMediaKind.image,
          emptyLabel: '还没有照片',
          session: session,
          sessionReady: _sessionResolved,
          onSessionExpired: _refreshSessionAfterUnauthorized,
          isSelf: widget.isSelf,
          onOpenPost: _openPost,
        );
      case ProfileTab.videos:
        return ProfilePostsTab(
          key: ValueKey('videos:$_postsRevision'),
          cidNumber: widget.cidNumber,
          api: _api,
          mediaKind: SquareMediaKind.video,
          emptyLabel: '还没有视频',
          session: session,
          sessionReady: _sessionResolved,
          onSessionExpired: _refreshSessionAfterUnauthorized,
          isSelf: widget.isSelf,
          onOpenPost: _openPost,
        );
      case ProfileTab.articles:
        return ProfilePostsTab(
          key: ValueKey('articles:$_postsRevision'),
          cidNumber: widget.cidNumber,
          api: _api,
          contentFormat: SquarePostContentFormat.article,
          emptyLabel: '还没有文章',
          session: session,
          sessionReady: _sessionResolved,
          onSessionExpired: _refreshSessionAfterUnauthorized,
          isSelf: widget.isSelf,
          onOpenPost: _openArticle,
        );
    }
  }

  @override
  Widget build(BuildContext context) {
    return DefaultTabController(
      length: ProfileTab.values.length,
      child: Scaffold(
        body: NestedScrollView(
          headerSliverBuilder: (context, innerBoxIsScrolled) => [
            SliverOverlapAbsorber(
              handle: NestedScrollView.sliverOverlapAbsorberHandleFor(context),
              sliver: SliverAppBar(
                pinned: true,
                expandedHeight: _expandedHeight,
                backgroundColor: AppTheme.primaryDark,
                foregroundColor: Colors.white,
                elevation: 0,
                leading: IconButton(
                  icon: const Icon(Icons.chevron_left),
                  // 背景图明暗不定：加半透明深色圆形底衬保证白色返回箭头始终可读。
                  style: IconButton.styleFrom(
                    backgroundColor: Colors.black.withValues(alpha: 0.32),
                    foregroundColor: Colors.white,
                  ),
                  onPressed: () => Navigator.of(context).maybePop(),
                ),
                actions: [
                  ProfileKebabMenu(
                    isSelf: widget.isSelf,
                    onQrCode: _openQrCode,
                    onEditProfile: _openEditProfile,
                    onDeleteAccount: _openDeleteAccount,
                  ),
                ],
                flexibleSpace: FlexibleSpaceBar(
                  background: CollapsibleHeader(
                    expandedHeight: _expandedHeight,
                    bannerHeight: _bannerHeight,
                    collapsedTitle: _title,
                    banner: _bannerWidget(),
                    foreground: ProfileHeaderCard(
                      cidNumber: widget.cidNumber,
                      profile: _profile,
                      avatarUrl: _mediaUrl(_profile?.avatarObjectKey),
                      avatarHeaders: _mediaHeaders,
                      onFollowing: () => _openFollows(FollowsType.following),
                      onFollowers: () => _openFollows(FollowsType.followers),
                      onPosts: () => _stub('帖子'),
                      actions: ProfileActionIcons(
                        isSelf: widget.isSelf,
                        isFollowing: _profile?.isFollowing ?? false,
                        isNotifying: _profile?.isNotifying ?? false,
                        // 他人视角看的是自己账户时置灰（不能关注/私信/通知自己）。
                        enabled: !_isOwnAccount,
                        onNotify: _toggleNotify,
                        onChat: _openChatWithUser,
                        onToggleFollow: _toggleFollow,
                      ),
                      // 他人主页才显示「订阅 TA / 取消」（订阅创作者会员，上链热签）。
                      // 链上订阅入参需要创作者钱包账户 account_id：从已拉取的
                      // profile.account_id（当前绑定账户）取，资料未就绪或无绑定则不显示。
                      creatorSubscribeButton: _creatorSubscribeButton(),
                    ),
                  ),
                ),
                bottom: const ProfileCategoryTabs(),
              ),
            ),
          ],
          body: TabBarView(
            children: [
              for (final tab in ProfileTab.values) _tabBody(tab),
            ],
          ),
        ),
      ),
    );
  }
}
