import 'dart:async';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:citizenapp/log/app_log.dart';
import 'package:flutter_svg/flutter_svg.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/pages/square_article_detail_page.dart';
import 'package:citizenapp/8964/compose/compose_page.dart';
import 'package:citizenapp/8964/pages/square_post_detail_page.dart';
import 'package:citizenapp/8964/profile/user_profile_page.dart';
import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/8964/services/square_identity_state.dart';
import 'package:citizenapp/8964/services/square_post_sync_service.dart';
import 'package:citizenapp/8964/widgets/square_feed_tabs.dart';
import 'package:citizenapp/8964/widgets/square_article_card.dart';
import 'package:citizenapp/8964/widgets/square_post_card.dart';
import 'package:citizenapp/my/myid/identity_account_cache.dart';
import 'package:citizenapp/my/myid/register_identity_flow.dart';
import 'package:citizenapp/rpc/smoldot_client.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/ui/widgets/identity_register_guide.dart';
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
    this.sessionProvider,
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
  final SquareSessionProvider? sessionProvider;
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
  int _feedLoadGeneration = 0;
  final List<SquarePost> _localPosts = [];

  /// 最近一次身份加载结果的身份账户与永久 CID，供身份 revision 广播后成对比对。
  String? _identityAddress;
  String? _identityCidNumber;

  final SquareApiClient _squareApi = SquareApiClient();
  final SquarePostSyncService _postSyncService = SquarePostSyncService();

  /// 最近一次 feed 加载的 session token，供卡片头像鉴权头复用。
  String? _feedSessionToken;
  late final SmoldotClientManager _smoldotClientManager;
  late final SquareSessionProvider _sessionProvider;

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
    _sessionProvider = widget.sessionProvider ?? SquareSessionProvider.instance;
    _feedSource = widget.feedSource ?? SquareApiClient();
    _identityFuture = _loadIdentity(readLiveChain: false);
    // 浏览态首帧直接挂载页面并并行加载 feed；身份与会员只在发布等写操作前严格校验，
    // 避免链读取或会话握手把整个广场长期挡在转圈页后面。
    _feedFuture = _beginFeedLoad();
    // 本页常驻 IndexedStack；切换身份账户（CID 换绑 / 切钱包）后经
    // walletsRevision 广播重载身份，保证身份图标与作者点击的 isSelf
    // 判定始终基于当前身份账户。
    WalletManager.walletsRevision.addListener(_onWalletsChanged);
    _smoldotClientManager.healthStatusListenable
        .addListener(_onChainHealthChanged);
    _onChainHealthChanged();
    // 发帖通知红点：仅生产真实数据源下开启（fake feedSource 的测试不触网）。
    if (_feedSource is SquareApiClient) {
      widget.selectedTab?.addListener(_onSelectedTabChanged);
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
      // 未注册 CID(身份缓存命中):与 feed 同理短路,红点轮询不发注定
      // 403 cid_not_bound 的登录挑战。
      final identity =
          await IdentityAccountCache.instance.resolve(allowChainRead: false);
      if (identity != null && !identity.isRegistered) return null;
      return await _sessionProvider.ensureSession();
    } on Object {
      // 后台通知由 unawaited 启动，任何会话失败都只能降级为不刷新红点，
      // 不得逸出为未捕获异步异常并影响广场浏览。
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
    } on Object {
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
    } on Object {
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
    } on Object {
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
    _identityCidNumber = identity.cidNumber;
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
    if (!mounted ||
        _smoldotClientManager.healthStatus != ChainHealthStatus.operational) {
      return;
    }
    // 身份账户廉价读只接受已命中的缓存，不启动 smoldot；未命中返回空，
    // 绝不虚构账户0为已注册身份。
    final accountId =
        await IdentityAccountCache.instance.accountId(allowChainRead: false) ??
            '';
    if (!mounted ||
        _smoldotClientManager.healthStatus != ChainHealthStatus.operational) {
      return;
    }
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
      AppLog.d('square identity refresh after chain sync failed: $e');
    }
  }

  /// 拉取会员购买态，仅供发布门禁（`_openCompose`）判定；不再驱动任何 UI。
  /// 生产路径要求 Worker 在即将拒绝时点查 finalized 链，异常必须上抛给入口区分提示。
  Future<SquareMembershipState?> _refreshMembership() async {
    final loader = widget.membershipLoader;
    if (loader != null) return loader();
    final session = await _sessionProvider.ensureSession();
    if (session == null) {
      throw const SquareApiException(
        '广场登录状态未就绪，请重试',
        statusCode: 401,
        errorCode: 'missing_session',
      );
    }
    return _squareApi.fetchMembership(session, verifyOnDeny: true);
  }

  Future<void> _onWalletsChanged() async {
    // CID 占号可在 account_id 不变时把 cid_number 从空推进为有效值；收到显式身份
    // revision 后必须读取完整身份，不能沿用只比较账户的旧优化。
    final identity = await IdentityAccountCache.instance.resolve();
    final identityAccountId = identity?.accountId ?? '';
    final identityCidNumber = identity?.snapshot?.cidNumber ?? '';
    if (!mounted) return;
    if (identityAccountId == (_identityAddress ?? '') &&
        identityCidNumber == (_identityCidNumber ?? '')) {
      return;
    }
    _operationalIdentityAccount = null;
    setState(() {
      _identityFuture = _loadIdentity(readLiveChain: true);
      _feedFuture = _beginFeedLoad();
    });
    _onChainHealthChanged();
  }

  Future<void> _openCompose() async {
    // 浏览不设身份门；发布属于写操作，必须在动作发生时读取真实链上身份并严格拒绝。
    final identity = await widget.identityService.loadCurrent(
      readLiveChain: true,
    );
    if (!mounted) return;
    if (!identity.hasWallet || !identity.isHotWallet) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('请先在「我的 → 我的钱包」创建热钱包')),
      );
      return;
    }
    if (!identity.isCertified) {
      // 未注册:就地弹统一注册面板;占号成功后回刷身份与 feed,发布由用户重新发起。
      final registered = await startCidRegistrationFlow(context);
      if (registered) _onRegisteredFromGuide();
      return;
    }
    final SquareMembershipState? membership;
    try {
      membership = await _refreshMembership();
    } on SquareApiException catch (error) {
      if (!mounted) return;
      final message = error.statusCode == 401
          ? '广场登录状态已失效，请重试'
          : error.errorCode == 'membership_required' ||
                  error.errorCode == 'membership_inactive'
              ? error.message
              : '暂时无法验证会员状态，请稍后重试';
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(message)),
      );
      return;
    } on Object {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('暂时无法验证会员状态，请稍后重试')),
      );
      return;
    }
    if (!mounted) return;
    if (membership == null) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('暂时无法验证会员状态，请稍后重试')),
      );
      return;
    }
    if (!membership.active) {
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

  Future<void> _openAuthor(String cidNumber) async {
    if (cidNumber.isEmpty) return;
    final identity = await _identityFuture;
    if (!mounted) return;
    // 身份主键 = cid_number：作者主键与本人身份都按 cid 比对判定 isSelf。
    final selfCid = identity.cidNumber?.trim() ?? '';
    final isSelf = selfCid.isNotEmpty && selfCid == cidNumber;
    await Navigator.of(context).push<void>(
      MaterialPageRoute(
        builder: (_) => UserProfilePage(
          cidNumber: cidNumber,
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
                      _feedFuture = _beginFeedLoad();
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
                      final error = snapshot.error;
                      // 未注册 CID:fail-closed 拦截是对的,但呈现必须是注册引导,
                      // 不是假故障文案。本地缓存短路与 Worker 403 cid_not_bound
                      // (真源)都落到这同一个分支。
                      if (error is SquareApiException &&
                          error.errorCode == 'cid_not_bound') {
                        return IdentityRegisterGuide(
                          description: '注册后即可浏览广场、发布动态。',
                          onRegistered: _onRegisteredFromGuide,
                        );
                      }
                      final posts = _composeFeed(
                        snapshot.data ?? const <SquarePost>[],
                      );
                      // Session 已不再以链上账户或余额作门禁；其余加载失败统一按当前
                      // 接口语义处理。
                      final errorMessage =
                          snapshot.hasError ? '广场内容加载失败' : null;
                      return Stack(
                        children: [
                          RefreshIndicator(
                            onRefresh: _refreshFeed,
                            child: _FeedBody(
                              posts: posts,
                              errorMessage: errorMessage,
                              onOpenPost: (post) => _openDetail(post),
                              onOpenAuthor: _openAuthor,
                              mediaUrlOf: _squareApi.mediaUrl,
                              avatarHeaders: _feedSessionToken == null
                                  ? null
                                  : {
                                      'authorization':
                                          'Bearer $_feedSessionToken'
                                    },
                            ),
                          ),
                          if (snapshot.connectionState != ConnectionState.done)
                            const Positioned(
                              top: 0,
                              left: 0,
                              right: 0,
                              child: LinearProgressIndicator(
                                key: ValueKey('square-feed-progress'),
                                minHeight: 2,
                              ),
                            ),
                        ],
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

  Future<List<SquarePost>> _loadFeed(
    SquareFeedKind feedKind,
    int generation,
  ) async {
    SquareSession? session;
    if (_feedSource is SquareApiClient) {
      // 未注册 CID(身份缓存命中的链读结论)直接短路:Worker 登录挑战对未绑定账户
      // 必回 403 cid_not_bound,这笔注定失败的请求不再发。缓存未命中(冷启动)不在
      // 本地武断,照常发起会话,由 Worker 真源判定后走同一 cid_not_bound 呈现分支。
      final identity =
          await IdentityAccountCache.instance.resolve(allowChainRead: false);
      if (identity != null && !identity.isRegistered) {
        throw const SquareApiException(
          '该钱包账户未绑定 CID,无法登录',
          statusCode: 403,
          errorCode: 'cid_not_bound',
        );
      }
      session = await _sessionProvider.ensureSession();
      if (session == null) {
        throw const SquareApiException('需要钱包账户才能浏览广场');
      }
      // 会话和链上当前绑定均已通过后再后台回灌本人副本；不阻塞公共 feed 首屏。
      // 同步失败只保留本地既有内容，下次启动/刷新继续从未推进的检查点重试。
      unawaited(
        _postSyncService.sync(session).catchError((Object error) {
          AppLog.d('[SquareHomePage] local post sync failed: $error');
        }),
      );
    }
    List<SquarePost> posts;
    try {
      posts = await _feedSource.fetchFeed(
        feedKind: feedKind,
        session: session,
      );
    } on SquareApiException catch (error) {
      // 只在 Worker 明确拒绝旧 Session 时重新握手一次；第二次失败原样交给前台，禁止
      // 无限重试。测试/离线数据源不参与生产会话刷新。
      if (error.statusCode != 401 ||
          _feedSource is! SquareApiClient ||
          generation != _feedLoadGeneration) {
        rethrow;
      }
      final refreshed = await _sessionProvider.refreshSession();
      if (refreshed == null) rethrow;
      session = refreshed;
      posts = await _feedSource.fetchFeed(
        feedKind: feedKind,
        session: refreshed,
      );
    }
    // 存 session token 供 feed 卡片头像 Image.network 带鉴权头（读任意作者头像同域可读）。
    // 迟到的旧分类请求不得覆盖当前分类的媒体鉴权头。
    if (generation == _feedLoadGeneration) {
      _feedSessionToken = session?.sessionToken;
    }
    return posts;
  }

  Future<List<SquarePost>> _beginFeedLoad() {
    final generation = ++_feedLoadGeneration;
    final feedKind = _selectedFeed;
    final future = _loadFeed(feedKind, generation);
    // initState 和分类切换会先创建 Future、随后才由下一帧 FutureBuilder 挂监听。
    // Worker 快速失败时必须立刻观察错误，消除这段未处理时间窗；原 Future
    // 不做转换，页面仍能通过 snapshot.hasError 展示真实前台失败态。
    unawaited(
      future.then<void>(
        (_) {},
        onError: (Object error, StackTrace stackTrace) {
          AppLog.d('[SquareHomePage] feed load failed: $error');
        },
      ),
    );
    return future;
  }

  Future<void> _refreshFeed() async {
    final next = _beginFeedLoad();
    setState(() => _feedFuture = next);
    await next;
  }

  /// 引导内占号成功后的就地回刷。服务层已经先失效身份缓存再广播全局 revision；
  /// 当前回调只保证本页在注册流程返回的同一帧重载身份与 feed。
  void _onRegisteredFromGuide() {
    if (!mounted) return;
    setState(() {
      _identityFuture = _loadIdentity(readLiveChain: false);
      _feedFuture = _beginFeedLoad();
    });
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
            onAuthorTap: () => onOpenAuthor(post.author.cidNumber ?? ''),
            avatarUrl: avatarUrl,
            avatarHeaders: avatarHeaders,
          );
        }
        return SquarePostCard(
          post: post,
          onTap: () => onOpenPost(post),
          onAuthorTap: () => onOpenAuthor(post.author.cidNumber ?? ''),
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
