import 'dart:async';

import 'package:flutter/material.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/pages/square_article_detail_page.dart';
import 'package:citizenapp/8964/compose/compose_page.dart';
import 'package:citizenapp/8964/pages/square_post_detail_page.dart';
import 'package:citizenapp/8964/profile/user_profile_page.dart';
import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/8964/services/square_identity_state.dart';
import 'package:citizenapp/8964/widgets/square_empty_state.dart';
import 'package:citizenapp/8964/widgets/square_feed_tabs.dart';
import 'package:citizenapp/8964/widgets/square_article_card.dart';
import 'package:citizenapp/8964/widgets/square_post_card.dart';
import 'package:citizenapp/rpc/smoldot_client.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/ui/identity_badge.dart';
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
  });

  final SquareIdentityService identityService;
  final SquareFeedSource? feedSource;
  final SquareFeedKind initialFeed;
  final List<SquarePost> seedPosts;
  final SmoldotClientManager? smoldotClientManager;
  final SquareMembershipLoader? membershipLoader;

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

  /// 顶栏徽章的会员信号（勾），随身份一起加载；best-effort。
  final SquareApiClient _squareApi = SquareApiClient();
  SquareMembershipState? _membership;
  int? _browseLeft;

  /// 最近一次 feed 加载的 session token，供卡片头像鉴权头复用。
  String? _feedSessionToken;
  late final SmoldotClientManager _smoldotClientManager;

  /// 同一次 operational 状态下，同一默认钱包只触发一次真实链刷新。
  String? _operationalIdentityAccount;

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
  }

  @override
  void dispose() {
    WalletManager.walletsRevision.removeListener(_onWalletsChanged);
    _smoldotClientManager.healthStatusListenable
        .removeListener(_onChainHealthChanged);
    super.dispose();
  }

  Future<SquareIdentityState> _loadIdentity({
    required bool readLiveChain,
  }) async {
    final identity = await widget.identityService.loadCurrent(
      readLiveChain: readLiveChain,
    );
    _identityAddress = identity.ownerAccount;
    _identityWalletName = identity.walletName;
    // 会员购买态（徽章勾）非阻塞加载：身份图标先渲染，勾稍后补上。
    unawaited(_refreshMembership());
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
    final walletAccount = wallet?.address.trim() ?? '';
    if (walletAccount.isEmpty || _operationalIdentityAccount == walletAccount) {
      return;
    }
    _operationalIdentityAccount = walletAccount;
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
      if (mounted) setState(() => _membership = membership);
      return membership;
    } on Exception {
      // 会员拉取失败不影响顶栏身份显示（无勾）。
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
    if ((wallet?.address ?? '') == (_identityAddress ?? '') &&
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

  Future<void> _openAuthor(String ownerAccount) async {
    if (ownerAccount.isEmpty) return;
    final identity = await _identityFuture;
    if (!mounted) return;
    final isSelf = identity.ownerAccount.isNotEmpty &&
        identity.ownerAccount == ownerAccount;
    await Navigator.of(context).push<void>(
      MaterialPageRoute(
        builder: (_) => UserProfilePage(
          ownerAccount: ownerAccount,
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
    return SafeArea(
      child: Column(
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 14, 16, 10),
            child: Column(
              children: [
                Row(
                  children: [
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          const Text(
                            '广场',
                            style: TextStyle(
                              color: AppTheme.textPrimary,
                              fontSize: 24,
                              fontWeight: FontWeight.w800,
                            ),
                          ),
                          const SizedBox(height: 2),
                          Text(
                            _browseLeft == null ? '推荐' : '今日剩余 $_browseLeft 条',
                            style: const TextStyle(
                              color: AppTheme.textSecondary,
                              fontSize: 13,
                            ),
                          ),
                        ],
                      ),
                    ),
                    FutureBuilder<SquareIdentityState>(
                      future: _identityFuture,
                      builder: (context, snapshot) {
                        final identity = snapshot.data;
                        final badge = identityBadgeStyle(
                          identityLevel: identity?.identityLevel,
                          membershipLevel: _membership?.membershipLevel,
                          membershipActive: _membership?.active ?? false,
                        );
                        return Tooltip(
                          message: identity?.accountLabel ?? '当前钱包',
                          child: IconButton.outlined(
                            onPressed: () {},
                            icon: badge != null
                                ? IdentityBadge(style: badge, size: 22)
                                : const Icon(Icons.account_circle_outlined),
                          ),
                        );
                      },
                    ),
                    const SizedBox(width: 8),
                    Tooltip(
                      message: '发布动态',
                      child: IconButton.filled(
                        onPressed: _openCompose,
                        icon: const Icon(Icons.edit_rounded),
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 14),
                SizedBox(
                  width: double.infinity,
                  child: SquareFeedTabs(
                    selected: _selectedFeed,
                    onChanged: (feed) {
                      setState(() {
                        _selectedFeed = feed;
                        _feedFuture = _loadFeed();
                      });
                    },
                  ),
                ),
              ],
            ),
          ),
          Expanded(
            child: FutureBuilder<List<SquarePost>>(
              future: _feedFuture,
              builder: (context, snapshot) {
                final posts = _filterPosts([
                  ..._localPosts,
                  ...(snapshot.data ?? const <SquarePost>[]),
                  ...widget.seedPosts,
                ]);
                if (snapshot.connectionState != ConnectionState.done &&
                    posts.isEmpty) {
                  return const Center(child: CircularProgressIndicator());
                }
                // Session 已不再以链上账户或余额作门禁；广场加载失败统一按当前
                // 接口语义处理，不保留已删除门禁的专用错误分支。
                final errorMessage = snapshot.hasError ? '广场内容加载失败' : null;
                return RefreshIndicator(
                  onRefresh: _refreshFeed,
                  child: _FeedBody(
                    feedKind: _selectedFeed,
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
          ),
        ],
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
    final source = _feedSource;
    if (mounted && source is SquareApiClient) {
      setState(() => _browseLeft = source.lastBrowseState?.browseLeft);
    }
    return posts;
  }

  Future<void> _refreshFeed() async {
    final next = _loadFeed();
    setState(() => _feedFuture = next);
    await next;
  }

  List<SquarePost> _filterPosts(List<SquarePost> posts) {
    switch (_selectedFeed) {
      case SquareFeedKind.recommended:
        return posts;
      case SquareFeedKind.following:
        return const <SquarePost>[];
      case SquareFeedKind.campaign:
        return posts
            .where((post) => post.postCategory == SquarePostCategory.campaign)
            .toList(growable: false);
    }
  }
}

class _FeedBody extends StatelessWidget {
  const _FeedBody({
    required this.feedKind,
    required this.posts,
    required this.errorMessage,
    required this.onOpenPost,
    required this.onOpenAuthor,
    required this.mediaUrlOf,
    required this.avatarHeaders,
  });

  final SquareFeedKind feedKind;
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
      return SquareEmptyState(
        icon: _emptyIcon,
        title: _emptyTitle,
        message: _emptyMessage,
      );
    }

    return ListView.separated(
      padding: const EdgeInsets.fromLTRB(16, 4, 16, 20),
      itemBuilder: (context, index) {
        if (index == 0 && errorMessage != null) {
          return Container(
            padding: const EdgeInsets.all(12),
            decoration: AppTheme.bannerDecoration(AppTheme.warning),
            child: Text(
              errorMessage!,
              style: const TextStyle(
                color: AppTheme.textPrimary,
                fontSize: 13,
                height: 1.35,
              ),
            ),
          );
        }
        final postIndex = errorMessage == null ? index : index - 1;
        final post = posts[postIndex];
        final avatarUrl = _avatarUrl(post.author);
        // 文章走标题/正文在上、强制横屏首图在下的文章卡；其余走图文卡。
        if (post.contentFormat == SquarePostContentFormat.article) {
          return SquareArticleCard(
            post: post,
            onTap: () => onOpenPost(post),
            onAuthorTap: () => onOpenAuthor(post.author.ownerAccount),
            avatarUrl: avatarUrl,
            avatarHeaders: avatarHeaders,
          );
        }
        return SquarePostCard(
          post: post,
          onTap: () => onOpenPost(post),
          onAuthorTap: () => onOpenAuthor(post.author.ownerAccount),
          avatarUrl: avatarUrl,
          avatarHeaders: avatarHeaders,
        );
      },
      separatorBuilder: (_, __) => const SizedBox(height: 10),
      itemCount: posts.length + (errorMessage == null ? 0 : 1),
    );
  }

  IconData get _emptyIcon {
    switch (feedKind) {
      case SquareFeedKind.recommended:
        return Icons.explore_outlined;
      case SquareFeedKind.following:
        return Icons.people_alt_outlined;
      case SquareFeedKind.campaign:
        return Icons.campaign_outlined;
    }
  }

  String get _emptyTitle {
    switch (feedKind) {
      case SquareFeedKind.recommended:
        return '暂无推荐动态';
      case SquareFeedKind.following:
        return '暂无关注动态';
      case SquareFeedKind.campaign:
        return '暂无竞选动态';
    }
  }

  String get _emptyMessage {
    switch (feedKind) {
      case SquareFeedKind.recommended:
        return '新的图文和视频动态会出现在这里。';
      case SquareFeedKind.following:
        return '关注的钱包账户发布内容后会显示在这里。';
      case SquareFeedKind.campaign:
        return '认证公民发布的竞选内容会显示在这里。';
    }
  }
}
