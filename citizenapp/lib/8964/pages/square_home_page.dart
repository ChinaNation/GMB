import 'dart:async';

import 'package:flutter/material.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/pages/square_article_detail_page.dart';
import 'package:citizenapp/8964/pages/square_article_compose_page.dart';
import 'package:citizenapp/8964/pages/square_compose_page.dart';
import 'package:citizenapp/8964/pages/square_post_detail_page.dart';
import 'package:citizenapp/8964/profile/user_profile_page.dart';
import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/8964/services/square_identity_state.dart';
import 'package:citizenapp/8964/storage/square_draft_store.dart';
import 'package:citizenapp/8964/widgets/square_empty_state.dart';
import 'package:citizenapp/8964/widgets/square_feed_tabs.dart';
import 'package:citizenapp/8964/widgets/square_post_card.dart';
import 'package:citizenapp/rpc/smoldot_client.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/ui/identity_badge.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

enum _ComposeKind { post, article }

class SquareHomePage extends StatefulWidget {
  const SquareHomePage({
    super.key,
    this.identityService = const SquareIdentityService(),
    this.feedSource,
    this.draftStore,
    this.initialFeed = SquareFeedKind.recommended,
    this.seedPosts = const <SquarePost>[],
    this.smoldotClientManager,
  });

  final SquareIdentityService identityService;
  final SquareFeedSource? feedSource;
  // 页面测试可注入空草稿仓库；正式运行由发布页使用默认本机草稿存储。
  final SquareDraftRepository? draftStore;
  final SquareFeedKind initialFeed;
  final List<SquarePost> seedPosts;
  final SmoldotClientManager? smoldotClientManager;

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
    setState(() => _identityFuture = future);
    try {
      await future;
    } catch (e) {
      debugPrint('square identity refresh after chain sync failed: $e');
    }
  }

  Future<void> _refreshMembership() async {
    try {
      final session = await SquareSessionProvider.instance.ensureSession();
      final membership =
          session != null ? await _squareApi.fetchMembership(session) : null;
      if (mounted) setState(() => _membership = membership);
    } on Exception {
      // 会员拉取失败不影响顶栏身份显示（无勾）。
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
    final choice = await showModalBottomSheet<_ComposeKind>(
      context: context,
      builder: (sheetContext) => SafeArea(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            ListTile(
              leading: const Icon(Icons.dynamic_feed_outlined),
              title: const Text('发动态'),
              subtitle: const Text('短文 + 图片/视频'),
              onTap: () => Navigator.of(sheetContext).pop(_ComposeKind.post),
            ),
            ListTile(
              leading: const Icon(Icons.article_outlined),
              title: const Text('发文章'),
              subtitle: const Text('长文：标题 + 首图 + 正文'),
              onTap: () => Navigator.of(sheetContext).pop(_ComposeKind.article),
            ),
          ],
        ),
      ),
    );
    if (choice == null || !mounted) return;

    final post = await Navigator.of(context).push<SquarePost>(
      MaterialPageRoute<SquarePost>(
        builder: (_) => switch (choice) {
          _ComposeKind.post => SquareComposePage(
              identityService: widget.identityService,
              draftStore: widget.draftStore,
            ),
          _ComposeKind.article => SquareArticleComposePage(
              identityService: widget.identityService,
            ),
        },
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
                    const Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            '广场',
                            style: TextStyle(
                              color: AppTheme.textPrimary,
                              fontSize: 24,
                              fontWeight: FontWeight.w800,
                            ),
                          ),
                          SizedBox(height: 2),
                          Text(
                            '推荐',
                            style: TextStyle(
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
                return RefreshIndicator(
                  onRefresh: _refreshFeed,
                  child: _FeedBody(
                    feedKind: _selectedFeed,
                    posts: posts,
                    errorMessage: snapshot.hasError ? '广场内容加载失败' : null,
                    onOpenPost: (post) => _openDetail(post),
                    onOpenAuthor: _openAuthor,
                  ),
                );
              },
            ),
          ),
        ],
      ),
    );
  }

  Future<List<SquarePost>> _loadFeed() {
    return _feedSource.fetchFeed(feedKind: _selectedFeed);
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
  });

  final SquareFeedKind feedKind;
  final List<SquarePost> posts;
  final String? errorMessage;
  final ValueChanged<SquarePost> onOpenPost;
  final ValueChanged<String> onOpenAuthor;

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
        return SquarePostCard(
          post: post,
          onTap: () => onOpenPost(post),
          onAuthorTap: () => onOpenAuthor(post.author.ownerAccount),
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
