import 'package:flutter/material.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/pages/square_compose_page.dart';
import 'package:citizenapp/8964/pages/square_post_detail_page.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/8964/services/square_identity_state.dart';
import 'package:citizenapp/8964/storage/square_draft_store.dart';
import 'package:citizenapp/8964/widgets/square_empty_state.dart';
import 'package:citizenapp/8964/widgets/square_feed_tabs.dart';
import 'package:citizenapp/8964/widgets/square_post_card.dart';
import 'package:citizenapp/ui/app_theme.dart';

class SquareHomePage extends StatefulWidget {
  const SquareHomePage({
    super.key,
    this.identityService = const SquareIdentityService(),
    this.feedSource,
    this.draftStore,
    this.initialFeed = SquareFeedKind.recommended,
    this.seedPosts = const <SquarePost>[],
  });

  final SquareIdentityService identityService;
  final SquareFeedSource? feedSource;
  // 页面测试可注入空草稿仓库；正式运行由发布页使用默认本机草稿存储。
  final SquareDraftRepository? draftStore;
  final SquareFeedKind initialFeed;
  final List<SquarePost> seedPosts;

  @override
  State<SquareHomePage> createState() => _SquareHomePageState();
}

class _SquareHomePageState extends State<SquareHomePage> {
  late SquareFeedKind _selectedFeed = widget.initialFeed;
  late Future<SquareIdentityState> _identityFuture;
  late final SquareFeedSource _feedSource;
  late Future<List<SquarePost>> _feedFuture;
  final List<SquarePost> _localPosts = [];

  @override
  void initState() {
    super.initState();
    _feedSource = widget.feedSource ?? SquareApiClient();
    _identityFuture = widget.identityService.loadCurrent();
    _feedFuture = _loadFeed();
  }

  Future<void> _openCompose() async {
    final post = await Navigator.of(context).push<SquarePost>(
      MaterialPageRoute<SquarePost>(
        builder: (_) => SquareComposePage(
          identityService: widget.identityService,
          draftStore: widget.draftStore,
        ),
      ),
    );
    if (post == null || !mounted) return;
    setState(() => _localPosts.insert(0, post));
    await _refreshFeed();
  }

  void _openDetail(SquarePost post) {
    Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => SquarePostDetailPage(post: post),
      ),
    );
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
                        return Tooltip(
                          message: identity?.accountLabel ?? '当前钱包',
                          child: IconButton.outlined(
                            onPressed: () {},
                            icon: Icon(
                              identity?.isCertified == true
                                  ? Icons.verified_user_rounded
                                  : Icons.account_circle_outlined,
                            ),
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
                    onOpenPost: _openDetail,
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
  });

  final SquareFeedKind feedKind;
  final List<SquarePost> posts;
  final String? errorMessage;
  final ValueChanged<SquarePost> onOpenPost;

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
