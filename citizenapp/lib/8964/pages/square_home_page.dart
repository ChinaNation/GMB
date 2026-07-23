import 'dart:async';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/pages/square_article_detail_page.dart';
import 'package:citizenapp/8964/compose/compose_page.dart';
import 'package:citizenapp/8964/pages/square_post_detail_page.dart';
import 'package:citizenapp/8964/profile/user_profile_page.dart';
import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/8964/services/square_identity_state.dart';
import 'package:citizenapp/8964/widgets/square_feed_tabs.dart';
import 'package:citizenapp/8964/widgets/square_article_card.dart';
import 'package:citizenapp/8964/widgets/square_post_card.dart';
import 'package:citizenapp/rpc/smoldot_client.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

typedef SquareMembershipLoader = Future<SquareMembershipState?> Function();

class SquareHomePage extends StatefulWidget {
  const SquareHomePage({
    super.key,
    this.identityService = const SquareIdentityService(),
    this.feedSource,
    this.initialFeed = SquareFeedKind.recommended,
    this.seedPosts = const <SquarePost>[],
    this.smoldotClientManager,
    this.membershipLoader,
    this.onSquareUnreadChanged,
    this.selectedTab,
    this.tabIndex = 0,
  });

  final SquareIdentityService identityService;
  final SquareFeedSource? feedSource;
  final SquareFeedKind initialFeed;
  final List<SquarePost> seedPosts;
  final SmoldotClientManager? smoldotClientManager;
  final SquareMembershipLoader? membershipLoader;

  /// 广场底部 tab 红点计数回调（上抛给 AppShell 挂 Badge）。
  final ValueChanged<int>? onSquareUnreadChanged;

  /// 底部导航当前活动 tab 广播；值 == [tabIndex] 时视为「进广场」，清广场红点。
  final ValueNotifier<int>? selectedTab;
  final int tabIndex;

  @override
  State<SquareHomePage> createState() => _SquareHomePageState();
}

class _SquareHomePageState extends State<SquareHomePage> {
  late SquareFeedKind _selectedFeed = widget.initialFeed;
  late Future<SquareIdentityState> _identityFuture;
  late final SquareFeedSource _feedSource;
  late Future<List<SquarePost>> _feedFuture;
  final List<SquarePost> _localPosts = [];

  /// 最近一次身份加载结果的钱包快照,供 _onWalletsChanged 廉价比对。
  String? _identityAddress;
  String? _identityWalletName;

  final SquareApiClient _squareApi = SquareApiClient();

  /// 最近一次 feed 加载的 session token，供卡片头像鉴权头复用。
  String? _feedSessionToken;
  late final SmoldotClientManager _smoldotClientManager;

  /// 同一次 operational 状态下，同一默认钱包只触发一次真实链刷新。
  String? _operationalIdentityAccount;

  /// 关注子 tab 红点数（服务端 following_unread）。广场底部 tab 数经回调上抛。
  int _followingUnread = 0;

  /// 发帖通知红点轮询；仅生产真实数据源下开启，测试注入 fake feedSource 时跳过不触网。
  static const Duration _notifyPollInterval = Duration(seconds: 45);
  Timer? _notifyTimer;

  @override
  void initState() {
    super.initState();
    _smoldotClientManager =
        widget.smoldotClientManager ?? SmoldotClientManager.instance;
    _feedSource = widget.feedSource ?? SquareApiClient();
    _identityFuture = _loadIdentity(readLiveChain: false);
    _feedFuture = _loadFeed();
    // 本页常驻 IndexedStack；切换默认用户钱包（= 切换身份）后经
    // walletsRevision 广播重载身份，保证身份图标与作者点击的 isSelf
    // 判定始终基于当前默认用户。
    WalletManager.walletsRevision.addListener(_onWalletsChanged);
    _smoldotClientManager.healthStatusListenable
        .addListener(_onChainHealthChanged);
    _onChainHealthChanged();
    // 发帖通知红点：仅生产真实数据源下开启（fake feedSource 的测试不触网）。
    if (_feedSource is SquareApiClient) {
      widget.selectedTab?.addListener(_onSelectedTabChanged);
      // 落地即广场活动：清广场游标 + 首拉。
      unawaited(_onSquareActivated());
      _notifyTimer = Timer.periodic(
        _notifyPollInterval,
        (_) => unawaited(_refreshNotify()),
      );
    }
  }

  @override
  void dispose() {
    WalletManager.walletsRevision.removeListener(_onWalletsChanged);
    _smoldotClientManager.healthStatusListenable
        .removeListener(_onChainHealthChanged);
    _notifyTimer?.cancel();
    widget.selectedTab?.removeListener(_onSelectedTabChanged);
    super.dispose();
  }

  /// 底部导航切到广场（值 == tabIndex）→ 清广场红点。
  void _onSelectedTabChanged() {
    if (widget.selectedTab?.value == widget.tabIndex) {
      unawaited(_onSquareActivated());
    }
  }

  Future<SquareSession?> _notifySession() async {
    try {
      return await SquareSessionProvider.instance.ensureSession();
    } on Exception {
      return null;
    }
  }

  /// 拉双游标红点：广场数经回调上抛底部 tab，关注数留本地驱动关注子 tab 徽章。
  Future<void> _refreshNotify() async {
    if (_feedSource is! SquareApiClient) return;
    final session = await _notifySession();
    if (session == null) return;
    try {
      final counts = await _squareApi.fetchNotifyUnread(session: session);
      if (!mounted) return;
      widget.onSquareUnreadChanged?.call(counts.squareUnread);
      if (counts.followingUnread != _followingUnread) {
        setState(() => _followingUnread = counts.followingUnread);
      }
    } on Exception {
      // 红点拉取失败静默：不影响广场浏览。
    }
  }

  /// 进广场：清广场游标 → 底部红点归零，随后回拉（关注游标不动，关注红点保留）。
  Future<void> _onSquareActivated() async {
    if (_feedSource is! SquareApiClient) return;
    final session = await _notifySession();
    if (session == null) return;
    try {
      await _squareApi.markNotifyRead(session: session, scope: 'square');
      if (mounted) widget.onSquareUnreadChanged?.call(0);
    } on Exception {
      // 清读失败静默；下次轮询以服务端为准。
    }
    await _refreshNotify();
  }

  /// 进关注子 tab：清关注游标 → 关注红点归零。
  Future<void> _onFollowingActivated() async {
    if (_feedSource is! SquareApiClient) return;
    if (mounted && _followingUnread != 0) {
      setState(() => _followingUnread = 0);
    }
    final session = await _notifySession();
    if (session == null) return;
    try {
      await _squareApi.markNotifyRead(session: session, scope: 'following');
    } on Exception {
      // 清读失败静默；本地已归零，下次轮询以服务端为准。
    }
  }

  Future<SquareIdentityState> _loadIdentity({
    required bool readLiveChain,
  }) async {
    final identity = await widget.identityService.loadCurrent(
      readLiveChain: readLiveChain,
    );
    _identityAddress = identity.accountId;
    _identityWalletName = identity.walletName;
    return identity;
  }

  void _onChainHealthChanged() {
    if (_smoldotClientManager.healthStatus != ChainHealthStatus.operational) {
      _operationalIdentityAccount = null;
      return;
    }
    unawaited(_refreshIdentityAfterChainOperational());
  }

  Future<void> _refreshIdentityAfterChainOperational() async {
    final manager = widget.identityService.walletManager ?? WalletManager();
    final wallet = await manager.getDefaultWallet();
    if (!mounted ||
        _smoldotClientManager.healthStatus != ChainHealthStatus.operational) {
      return;
    }
    final accountId = wallet?.accountId ?? '';
    if (accountId.isEmpty || _operationalIdentityAccount == accountId) {
      return;
    }
    _operationalIdentityAccount = accountId;
    final future = _loadIdentity(readLiveChain: true);
    // setState 回调必须返回 void；赋值表达式会返回 Future，Flutter 会把它判定为
    // 异步 setState 并抛异常，因此改成语句块明确只做同步状态赋值。
    setState(() {
      _identityFuture = future;
    });
    try {
      await future;
    } catch (e) {
      debugPrint('square identity refresh after chain sync failed: $e');
    }
  }

  /// 拉取会员购买态，仅供发布门禁（`_openCompose`）判定；不再驱动任何 UI。
  Future<SquareMembershipState?> _refreshMembership() async {
    try {
      final loader = widget.membershipLoader;
      final membership = loader != null
          ? await loader()
          : await () async {
              final session =
                  await SquareSessionProvider.instance.ensureSession();
              return session != null
                  ? _squareApi.fetchMembership(session)
                  : null;
            }();
      return membership;
    } on Exception {
      return null;
    }
  }

  Future<void> _onWalletsChanged() async {
    // 先廉价比对(纯 Isar 读):默认身份地址与昵称都没变时跳过,
    // 避免无关钱包操作触发 CID 链查询。乱序安全由 FutureBuilder
    // 只跟踪最新 _identityFuture 保证。
    final manager = widget.identityService.walletManager ?? WalletManager();
    final wallet = await manager.getDefaultWallet();
    if (!mounted) return;
    if ((wallet?.accountId ?? '') == (_identityAddress ?? '') &&
        wallet?.walletName == _identityWalletName) {
      return;
    }
    _operationalIdentityAccount = null;
    setState(() {
      _identityFuture = _loadIdentity(readLiveChain: false);
    });
    _onChainHealthChanged();
  }

  Future<void> _openCompose() async {
    final membership = await _refreshMembership();
    if (!mounted) return;
    if (membership?.active != true) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('需要有效会员才能发布广场内容')),
      );
      return;
    }
    // 类型（动态/文章/竞选）在统一发布页内经头像旁下拉选择，不再底部分流。
    final post = await Navigator.of(context).push<SquarePost>(
      MaterialPageRoute<SquarePost>(
        builder: (_) =>
            SquareComposePage(identityService: widget.identityService),
      ),
    );
    if (post == null || !mounted) return;
    setState(() => _localPosts.insert(0, post));
    await _refreshFeed();
  }

  Future<void> _openAuthor(String accountId) async {
    if (accountId.isEmpty) return;
    final identity = await _identityFuture;
    if (!mounted) return;
    final isSelf =
        identity.accountId.isNotEmpty && identity.accountId == accountId;
    await Navigator.of(context).push<void>(
      MaterialPageRoute(
        builder: (_) => UserProfilePage(
          accountId: accountId,
          isSelf: isSelf,
        ),
      ),
    );
  }

  Future<void> _openDetail(SquarePost post) async {
    final result = await Navigator.of(context).push<SquarePostDetailResult>(
      MaterialPageRoute<SquarePostDetailResult>(
        builder: (_) => post.contentFormat == SquarePostContentFormat.article
            ? SquareArticleDetailPage(post: post)
            : SquarePostDetailPage(post: post),
      ),
    );
    if (result == null || !mounted) return;
    setState(() {
      _localPosts.removeWhere((item) => item.postId == post.postId);
      final replacement = result.replacement;
      if (replacement != null) {
        _localPosts.removeWhere((item) => item.postId == replacement.postId);
        _localPosts.insert(0, replacement);
      }
    });
    await _refreshFeed();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      // 发布=右下角正圆悬浮 primary FAB（endFloat=底部导航「我的」tab 上方）。
      floatingActionButton: FloatingActionButton(
        shape: const CircleBorder(),
        onPressed: _openCompose,
        tooltip: '发布动态',
        backgroundColor: AppTheme.primary,
        foregroundColor: Colors.white,
        child: const Icon(Icons.edit_rounded),
      ),
      floatingActionButtonLocation: FloatingActionButtonLocation.endFloat,
      body: SafeArea(
        child: Column(
          children: [
            // 头像入口已删（进自己主页只走「我的-背景图」），分类栏上移到顶部省空间。
            Padding(
              padding: const EdgeInsets.fromLTRB(16, 8, 16, 8),
              child: SizedBox(
                width: double.infinity,
                child: SquareFeedTabs(
                  selected: _selectedFeed,
                  followingUnread: _followingUnread,
                  onChanged: (feed) {
                    setState(() {
                      _selectedFeed = feed;
                      _feedFuture = _loadFeed();
                    });
                    // 进关注子 tab → 清关注红点。
                    if (feed == SquareFeedKind.following) {
                      unawaited(_onFollowingActivated());
                    }
                  },
                ),
              ),
            ),
            Expanded(
              child: Stack(
                children: [
                  // 页面中央若隐若现的坦克水印（= 广场 tab 图标）：常驻背景、不拦触摸，
                  // 动态卡片浮于其上；无动态时只见水印，取代原空态图标+文字。
                  Positioned.fill(
                    child: IgnorePointer(
                      child: Center(
                        child: Opacity(
                          opacity: 0.05,
                          child: ImageFiltered(
                            imageFilter:
                                ui.ImageFilter.blur(sigmaX: 2.2, sigmaY: 2.2),
                            child: SvgPicture.asset(
                              'assets/icons/tank.svg',
                              key: const ValueKey<String>(
                                'square-tank-watermark',
                              ),
                              width: 220,
                              height: 220,
                              colorFilter: const ColorFilter.mode(
                                AppTheme.primary,
                                BlendMode.srcIn,
                              ),
                            ),
                          ),
                        ),
                      ),
                    ),
                  ),
                  FutureBuilder<List<SquarePost>>(
                    future: _feedFuture,
                    builder: (context, snapshot) {
                      final posts = _composeFeed(
                        snapshot.data ?? const <SquarePost>[],
                      );
                      if (snapshot.connectionState != ConnectionState.done &&
                          posts.isEmpty) {
                        return const Center(child: CircularProgressIndicator());
                      }
                      // Session 已不再以链上账户或余额作门禁；广场加载失败统一按当前
                      // 接口语义处理，不保留已删除门禁的专用错误分支。
                      final errorMessage =
                          snapshot.hasError ? '广场内容加载失败' : null;
                      return RefreshIndicator(
                        onRefresh: _refreshFeed,
                        child: _FeedBody(
                          posts: posts,
                          errorMessage: errorMessage,
                          onOpenPost: (post) => _openDetail(post),
                          onOpenAuthor: _openAuthor,
                          mediaUrlOf: _squareApi.mediaUrl,
                          avatarHeaders: _feedSessionToken == null
                              ? null
                              : {'authorization': 'Bearer $_feedSessionToken'},
                        ),
                      );
                    },
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Future<List<SquarePost>> _loadFeed() async {
    SquareSession? session;
    if (_feedSource is SquareApiClient) {
      session = await SquareSessionProvider.instance.ensureSession();
      if (session == null) {
        throw const SquareApiException('需要钱包账户才能浏览广场');
      }
    }
    final posts = await _feedSource.fetchFeed(
      feedKind: _selectedFeed,
      session: session,
    );
    // 存 session token 供 feed 卡片头像 Image.network 带鉴权头（读任意作者头像同域可读）。
    _feedSessionToken = session?.sessionToken;
    return posts;
  }

  Future<void> _refreshFeed() async {
    final next = _loadFeed();
    setState(() => _feedFuture = next);
    await next;
  }

  /// 按当前 feed 组装最终列表。[serverPosts] 是 `_loadFeed` 已按所选 feed 从
  /// Worker 拉回的结果。关注流由服务端 `square_posts JOIN square_follows` 过滤，
  /// 直接渲染服务端结果——本地草稿与种子帖不属于关注流，只在其余分类混入。
  List<SquarePost> _composeFeed(List<SquarePost> serverPosts) {
    final merged = [..._localPosts, ...serverPosts, ...widget.seedPosts];
    switch (_selectedFeed) {
      case SquareFeedKind.recommended:
        return merged;
      case SquareFeedKind.following:
        return serverPosts;
      case SquareFeedKind.campaign:
        return merged
            .where((post) => post.postCategory == SquarePostCategory.campaign)
            .toList(growable: false);
      case SquareFeedKind.article:
        return merged
            .where(
                (post) => post.contentFormat == SquarePostContentFormat.article)
            .toList(growable: false);
      case SquareFeedKind.photos:
        // 照片=普通图文帖且含图无视频（文章归文章档、视频归视频档，不重复出现）。
        return merged
            .where((post) =>
                post.contentFormat == SquarePostContentFormat.normal &&
                _hasMedia(post, SquareMediaKind.image) &&
                !_hasMedia(post, SquareMediaKind.video))
            .toList(growable: false);
      case SquareFeedKind.videos:
        return merged
            .where((post) =>
                post.contentFormat == SquarePostContentFormat.normal &&
                _hasMedia(post, SquareMediaKind.video))
            .toList(growable: false);
    }
  }

  static bool _hasMedia(SquarePost post, SquareMediaKind kind) =>
      post.mediaItems.any((media) => media.mediaKind == kind);
}

class _FeedBody extends StatelessWidget {
  const _FeedBody({
    required this.posts,
    required this.errorMessage,
    required this.onOpenPost,
    required this.onOpenAuthor,
    required this.mediaUrlOf,
    required this.avatarHeaders,
  });

  final List<SquarePost> posts;
  final String? errorMessage;
  final ValueChanged<SquarePost> onOpenPost;
  final ValueChanged<String> onOpenAuthor;

  /// 把 object_key 解析成可读媒体地址（作者头像等）。
  final String Function(String objectKey) mediaUrlOf;

  /// 头像 `Image.network` 鉴权头（钱包 session Bearer）；未登录为空。
  final Map<String, String>? avatarHeaders;

  String? _avatarUrl(SquareAuthor author) {
    final key = author.avatarObjectKey;
    return key == null ? null : mediaUrlOf(key);
  }

  @override
  Widget build(BuildContext context) {
    if (posts.isEmpty) {
      // 空态不再展示图标+文字，仅保留可下拉刷新的空滚动区，让底层坦克水印透出；
      // 有错误时顶部仍显示错误横幅。
      return ListView(
        physics: const AlwaysScrollableScrollPhysics(),
        padding: const EdgeInsets.fromLTRB(16, 4, 16, 20),
        children: [
          if (errorMessage != null) _errorBanner(errorMessage!),
        ],
      );
    }

    return ListView.separated(
      // 底部留白给右下角发布 FAB，避免盖住末条动态的互动区。
      padding: const EdgeInsets.fromLTRB(16, 4, 16, 88),
      itemBuilder: (context, index) {
        if (index == 0 && errorMessage != null) {
          return _errorBanner(errorMessage!);
        }
        final postIndex = errorMessage == null ? index : index - 1;
        final post = posts[postIndex];
        final avatarUrl = _avatarUrl(post.author);
        // 文章走标题/正文在上、强制横屏首图在下的文章卡；其余走图文卡。
        if (post.contentFormat == SquarePostContentFormat.article) {
          return SquareArticleCard(
            post: post,
            onTap: () => onOpenPost(post),
            onAuthorTap: () => onOpenAuthor(post.author.accountId),
            avatarUrl: avatarUrl,
            avatarHeaders: avatarHeaders,
          );
        }
        return SquarePostCard(
          post: post,
          onTap: () => onOpenPost(post),
          onAuthorTap: () => onOpenAuthor(post.author.accountId),
          avatarUrl: avatarUrl,
          avatarHeaders: avatarHeaders,
        );
      },
      separatorBuilder: (_, __) => const SizedBox(height: 10),
      itemCount: posts.length + (errorMessage == null ? 0 : 1),
    );
  }

  Widget _errorBanner(String message) {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: AppTheme.bannerDecoration(AppTheme.warning),
      child: Text(
        message,
        style: const TextStyle(
          color: AppTheme.textPrimary,
          fontSize: 13,
          height: 1.35,
        ),
      ),
    );
  }
}
