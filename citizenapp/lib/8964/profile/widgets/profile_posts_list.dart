import 'package:flutter/material.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/profile/services/citizen_profile_api.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/8964/services/square_local_post_presenter.dart';
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
    required this.cidNumber,
    required this.api,
    required this.emptyLabel,
    required this.session,
    required this.isSelf,
    this.category,
    this.contentFormat,
    this.mediaKind,
    this.onOpenPost,
  });

  /// 作者身份主键 cid_number（按 cid 分页拉该身份的帖子）。
  final String cidNumber;
  final CitizenProfileApi api;
  final String emptyLabel;

  /// 可选浏览会话：只用于远端请求和受保护媒体，不是本人本地副本的读取凭证。
  ///
  /// 本人身份已经由上层以永久 `cid_number` 判定；断网或 Worker 不可用时会话可能为空，
  /// 此时仍必须允许本人读取本机已校验的发布副本。
  final SquareSession? session;
  final bool isSelf;
  final SquarePostCategory? category;
  final SquarePostContentFormat? contentFormat;
  final SquareMediaKind? mediaKind;
  final void Function(SquarePost post)? onOpenPost;

  @override
  State<ProfilePostsTab> createState() => _ProfilePostsTabState();
}

class _ProfilePostsTabState extends State<ProfilePostsTab> {
  static const int _pageSize = 20;
  static const SquareLocalPostPresenter _localPresenter =
      SquareLocalPostPresenter();

  final List<SquarePost> _posts = [];
  final Map<String, Set<SquareMediaKind>> _unavailableMediaKindsByPostId = {};
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
    var localHasContent = false;
    if (widget.isSelf) {
      try {
        final localCopies =
            await widget.api.fetchLocalPublishedPosts(widget.cidNumber);
        final presentations = localCopies
            .map(_localPresenter.present)
            .where(_matchesLocalFilters)
            .toList(growable: false);
        localHasContent = presentations.isNotEmpty;
        if (!mounted) return;
        setState(() {
          _posts
            ..clear()
            ..addAll(presentations.map((item) => item.post));
          _unavailableMediaKindsByPostId
            ..clear()
            ..addEntries(
              presentations
                  .where((item) => item.unavailableMediaKinds.isNotEmpty)
                  .map(
                    (item) => MapEntry(
                      item.post.postId,
                      item.unavailableMediaKinds,
                    ),
                  ),
            );
        });
      } on Exception {
        // 本地副本损坏时 fail-closed，但仍继续读取 Worker，不让单机磁盘故障遮住远端内容。
      }
    }

    try {
      final page = await widget.api.fetchAuthorPosts(
        widget.cidNumber,
        category: widget.category,
        contentFormat: widget.contentFormat,
        limit: _pageSize,
        session: widget.session,
      );
      if (!mounted) return;
      setState(() {
        _mergeRemotePosts(page.posts);
        _cursor = page.nextCursor;
        _done = page.nextCursor == null;
        _loading = false;
      });
    } on Exception {
      if (!mounted) return;
      setState(() {
        _loading = false;
        _failedFirst = _posts.isEmpty && !localHasContent;
      });
    }
  }

  Future<void> _loadMore() async {
    if (_loading || _done || _cursor == null) return;
    setState(() => _loading = true);
    try {
      final page = await widget.api.fetchAuthorPosts(
        widget.cidNumber,
        category: widget.category,
        contentFormat: widget.contentFormat,
        limit: _pageSize,
        cursor: _cursor,
        session: widget.session,
      );
      if (!mounted) return;
      setState(() {
        _mergeRemotePosts(page.posts);
        _cursor = page.nextCursor;
        _done = page.nextCursor == null;
        _loading = false;
      });
    } on Exception {
      if (!mounted) return;
      setState(() => _loading = false);
    }
  }

  bool _matchesLocalFilters(SquareLocalPostPresentation presentation) {
    final post = presentation.post;
    if (widget.category != null && post.postCategory != widget.category) {
      return false;
    }
    if (widget.contentFormat != null &&
        post.contentFormat != widget.contentFormat) {
      return false;
    }
    final mediaKind = widget.mediaKind;
    return mediaKind == null ||
        presentation.unavailableMediaKinds.contains(mediaKind);
  }

  /// 远端资料与媒体元数据优先；远端未返回的本地正文继续保留。
  ///
  /// 同一 post_id 的本地媒体声明只在远端缺少对应媒体时保留“云端已清理”标记，
  /// 不会构造第二套媒体对象或覆盖 Worker 返回的作者资料。
  void _mergeRemotePosts(List<SquarePost> remotePosts) {
    final byPostId = <String, SquarePost>{
      for (final post in _posts) post.postId: post,
    };
    for (final remote in remotePosts) {
      byPostId[remote.postId] = remote;
      final unavailable = _unavailableMediaKindsByPostId[remote.postId];
      if (unavailable != null) {
        final remaining = unavailable.difference(
          remote.mediaItems.map((media) => media.mediaKind).toSet(),
        );
        if (remaining.isEmpty) {
          _unavailableMediaKindsByPostId.remove(remote.postId);
        } else {
          _unavailableMediaKindsByPostId[remote.postId] =
              Set<SquareMediaKind>.unmodifiable(remaining);
        }
      }
    }
    final merged = byPostId.values.toList()
      ..sort((left, right) {
        final byCreatedAt = right.createdAt.compareTo(left.createdAt);
        if (byCreatedAt != 0) return byCreatedAt;
        return right.postId.compareTo(left.postId);
      });
    _posts
      ..clear()
      ..addAll(merged);
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
            final session = widget.session;
            final avatarHeaders = session == null
                ? null
                : <String, String>{
                    'authorization': 'Bearer ${session.sessionToken}',
                  };
            if (widget.contentFormat == SquarePostContentFormat.article) {
              return Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  SquareArticleCard(
                    post: post,
                    onTap: () => widget.onOpenPost?.call(post),
                    onAuthorTap: () => widget.onOpenPost?.call(post),
                    avatarUrl: avatarUrl,
                    avatarHeaders: avatarHeaders,
                  ),
                  if (_hasUnavailableMedia(post.postId))
                    const _UnavailableMediaNotice(),
                ],
              );
            }
            return Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                SquarePostCard(
                  post: post,
                  onTap: () => widget.onOpenPost?.call(post),
                  onAuthorTap: () => widget.onOpenPost?.call(post),
                  avatarUrl: avatarUrl,
                  avatarHeaders: avatarHeaders,
                ),
                if (_hasUnavailableMedia(post.postId))
                  const _UnavailableMediaNotice(),
              ],
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
    final unavailable = _posts.any(
      (post) =>
          _unavailableMediaKindsByPostId[post.postId]
              ?.contains(widget.mediaKind) ??
          false,
    );
    if (entries.isEmpty && unavailable) {
      return const [
        SliverFillRemaining(
          hasScrollBody: false,
          child: Center(child: _UnavailableMediaNotice()),
        ),
      ];
    }
    if (entries.isEmpty) {
      return [_message(widget.emptyLabel)];
    }
    return [
      if (unavailable)
        const SliverToBoxAdapter(
          child: Padding(
            padding: EdgeInsets.fromLTRB(12, 12, 12, 0),
            child: _UnavailableMediaNotice(),
          ),
        ),
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

  bool _hasUnavailableMedia(String postId) =>
      _unavailableMediaKindsByPostId[postId]?.isNotEmpty ?? false;

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

class _UnavailableMediaNotice extends StatelessWidget {
  const _UnavailableMediaNotice();

  @override
  Widget build(BuildContext context) {
    return const Padding(
      padding: EdgeInsets.symmetric(horizontal: 12, vertical: 10),
      child: Text(
        '媒体已从云端清理，本机仅保留正文',
        textAlign: TextAlign.center,
        style: TextStyle(
          color: AppTheme.textTertiary,
          fontSize: 12,
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
