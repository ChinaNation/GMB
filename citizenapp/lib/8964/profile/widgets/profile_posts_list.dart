import 'package:flutter/material.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/profile/services/citizen_profile_api.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/8964/widgets/square_article_card.dart';
import 'package:citizenapp/8964/widgets/square_post_card.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 单个分类 Tab 的内容：按作者分页拉帖，游标触底加载。
///
/// [mediaKind] 为空 → 帖子卡列表（[category] 过滤 normal/campaign）。
/// [mediaKind] 非空 → 从帖子媒体派生的照片/视频九宫格（不建表，派生视图）。
class ProfilePostsTab extends StatefulWidget {
  const ProfilePostsTab({
    super.key,
    required this.accountId,
    required this.api,
    required this.emptyLabel,
    required this.session,
    this.category,
    this.contentFormat,
    this.mediaKind,
    this.onOpenPost,
  });

  final String accountId;
  final CitizenProfileApi api;
  final String emptyLabel;
  final SquareSession session;
  final SquarePostCategory? category;
  final SquarePostContentFormat? contentFormat;
  final SquareMediaKind? mediaKind;
  final void Function(SquarePost post)? onOpenPost;

  @override
  State<ProfilePostsTab> createState() => _ProfilePostsTabState();
}

class _ProfilePostsTabState extends State<ProfilePostsTab> {
  static const int _pageSize = 20;

  final List<SquarePost> _posts = [];
  int? _cursor;
  bool _loading = false;
  bool _done = false;
  bool _failedFirst = false;

  @override
  void initState() {
    super.initState();
    _loadFirst();
  }

  Future<void> _loadFirst() async {
    setState(() {
      _loading = true;
      _failedFirst = false;
    });
    try {
      final page = await widget.api.fetchAuthorPosts(
        widget.accountId,
        category: widget.category,
        contentFormat: widget.contentFormat,
        limit: _pageSize,
        session: widget.session,
      );
      if (!mounted) return;
      setState(() {
        _posts
          ..clear()
          ..addAll(page.posts);
        _cursor = page.nextCursor;
        _done = page.nextCursor == null;
        _loading = false;
      });
    } on Exception {
      if (!mounted) return;
      setState(() {
        _loading = false;
        _failedFirst = _posts.isEmpty;
      });
    }
  }

  Future<void> _loadMore() async {
    if (_loading || _done || _cursor == null) return;
    setState(() => _loading = true);
    try {
      final page = await widget.api.fetchAuthorPosts(
        widget.accountId,
        category: widget.category,
        contentFormat: widget.contentFormat,
        limit: _pageSize,
        cursor: _cursor,
        session: widget.session,
      );
      if (!mounted) return;
      setState(() {
        _posts.addAll(page.posts);
        _cursor = page.nextCursor;
        _done = page.nextCursor == null;
        _loading = false;
      });
    } on Exception {
      if (!mounted) return;
      setState(() => _loading = false);
    }
  }

  bool _onScroll(ScrollNotification notification) {
    if (notification.metrics.pixels >=
        notification.metrics.maxScrollExtent - 400) {
      _loadMore();
    }
    return false;
  }

  @override
  Widget build(BuildContext context) {
    return NotificationListener<ScrollNotification>(
      onNotification: _onScroll,
      child: CustomScrollView(
        key: PageStorageKey<String>(
          '${widget.category?.name ?? 'all'}:${widget.mediaKind?.name ?? 'posts'}',
        ),
        slivers: [
          SliverOverlapInjector(
            handle: NestedScrollView.sliverOverlapAbsorberHandleFor(context),
          ),
          ..._contentSlivers(),
        ],
      ),
    );
  }

  List<Widget> _contentSlivers() {
    if (_loading && _posts.isEmpty) {
      return const [
        SliverFillRemaining(
          hasScrollBody: false,
          child: Center(child: CircularProgressIndicator()),
        ),
      ];
    }
    if (_failedFirst) {
      return [_message('加载失败，下拉重试')];
    }
    if (widget.mediaKind != null) {
      return _mediaSlivers();
    }
    if (_posts.isEmpty) {
      return [_message(widget.emptyLabel)];
    }
    return [
      SliverPadding(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 12),
        sliver: SliverList.separated(
          itemCount: _posts.length,
          separatorBuilder: (_, __) => const SizedBox(height: 10),
          itemBuilder: (context, index) {
            final post = _posts[index];
            final avatarKey = post.author.avatarObjectKey;
            final avatarUrl =
                avatarKey == null ? null : widget.api.mediaUrl(avatarKey);
            final avatarHeaders = {
              'authorization': 'Bearer ${widget.session.sessionToken}',
            };
            if (widget.contentFormat == SquarePostContentFormat.article) {
              return SquareArticleCard(
                post: post,
                onTap: () => widget.onOpenPost?.call(post),
                onAuthorTap: () => widget.onOpenPost?.call(post),
                avatarUrl: avatarUrl,
                avatarHeaders: avatarHeaders,
              );
            }
            return SquarePostCard(
              post: post,
              onTap: () => widget.onOpenPost?.call(post),
              onAuthorTap: () => widget.onOpenPost?.call(post),
              avatarUrl: avatarUrl,
              avatarHeaders: avatarHeaders,
            );
          },
        ),
      ),
      _footer(),
    ];
  }

  List<Widget> _mediaSlivers() {
    final entries = <({SquarePost post, SquareMediaItem media})>[];
    for (final post in _posts) {
      for (final media in post.mediaItems) {
        if (media.mediaKind == widget.mediaKind) {
          entries.add((post: post, media: media));
        }
      }
    }
    if (entries.isEmpty) {
      return [_message(widget.emptyLabel)];
    }
    return [
      SliverPadding(
        padding: const EdgeInsets.fromLTRB(12, 12, 12, 12),
        sliver: SliverGrid(
          gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
            crossAxisCount: 3,
            crossAxisSpacing: 6,
            mainAxisSpacing: 6,
          ),
          delegate: SliverChildBuilderDelegate(
            (context, index) {
              final entry = entries[index];
              return _MediaTile(
                media: entry.media,
                onTap: () => widget.onOpenPost?.call(entry.post),
              );
            },
            childCount: entries.length,
          ),
        ),
      ),
      _footer(),
    ];
  }

  Widget _footer() {
    if (!_loading || _posts.isEmpty) {
      return const SliverToBoxAdapter(child: SizedBox.shrink());
    }
    return const SliverToBoxAdapter(
      child: Padding(
        padding: EdgeInsets.symmetric(vertical: 16),
        child: Center(
          child: SizedBox(
            width: 20,
            height: 20,
            child: CircularProgressIndicator(strokeWidth: 2),
          ),
        ),
      ),
    );
  }

  Widget _message(String text) {
    return SliverFillRemaining(
      hasScrollBody: false,
      child: Center(
        child: Text(
          text,
          style: const TextStyle(color: AppTheme.textTertiary),
        ),
      ),
    );
  }
}

class _MediaTile extends StatelessWidget {
  const _MediaTile({required this.media, this.onTap});

  final SquareMediaItem media;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final isVideo = media.mediaKind == SquareMediaKind.video;
    final imageUrl = isVideo ? (media.coverUrl ?? '') : media.url;
    return GestureDetector(
      onTap: onTap,
      child: ClipRRect(
        borderRadius: BorderRadius.circular(AppTheme.radiusSm),
        child: DecoratedBox(
          decoration: const BoxDecoration(color: AppTheme.surfaceElevated),
          child: Stack(
            fit: StackFit.expand,
            children: [
              if (imageUrl.isNotEmpty)
                Image.network(
                  imageUrl,
                  fit: BoxFit.cover,
                  errorBuilder: (_, __, ___) => _fallbackIcon(isVideo),
                )
              else
                _fallbackIcon(isVideo),
              if (isVideo)
                const Center(
                  child: Icon(Icons.play_circle_fill_rounded,
                      size: 34, color: Colors.white70),
                ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _fallbackIcon(bool isVideo) {
    return Center(
      child: Icon(
        isVideo ? Icons.play_circle_fill_rounded : Icons.image_rounded,
        size: 34,
        color: AppTheme.textTertiary,
      ),
    );
  }
}
