import 'package:flutter/material.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/pages/square_article_detail_page.dart';
import 'package:citizenapp/8964/pages/square_post_detail_page.dart';
import 'package:citizenapp/8964/profile/follows_list_page.dart';
import 'package:citizenapp/8964/profile/models/citizen_profile.dart';
import 'package:citizenapp/8964/profile/profile_edit_page.dart';
import 'package:citizenapp/8964/profile/services/citizen_profile_api.dart';
import 'package:citizenapp/8964/profile/services/citizen_profile_cache.dart';
import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/profile/user_qr_page.dart';
import 'package:citizenapp/8964/profile/widgets/collapsible_header.dart';
import 'package:citizenapp/8964/profile/widgets/profile_action_icons.dart';
import 'package:citizenapp/8964/profile/widgets/profile_category_tabs.dart';
import 'package:citizenapp/8964/profile/widgets/profile_header_card.dart';
import 'package:citizenapp/8964/profile/widgets/profile_kebab_menu.dart';
import 'package:citizenapp/8964/profile/widgets/profile_posts_list.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/im/im_tab_page.dart';
import 'package:citizenapp/im/open_direct_chat.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 推特式用户主页。
///
/// 折叠虚化头部 + 圆角方形头像/背景（R2）+ 认证勾 + 展示名/地址/签名/计数 +
/// 三图标（本人 通知/聊天/关注 · 他人 关注/消息）+ ⋮（二维码/编辑资料/举报）+
/// 帖子/竞选/照片/视频/文章五 Tab。身份 = 默认热钱包地址；cache-first 加载，
/// 关注复用登录 session 静默签名，公开资料只进 R2、不上链。
class UserProfilePage extends StatefulWidget {
  const UserProfilePage({
    super.key,
    required this.ownerAccount,
    required this.isSelf,
    this.initialProfile,
    this.api,
    this.cache,
    this.sessionProvider,
    this.onOpenDirectChat,
  });

  /// 主页身份 = 默认热钱包地址。
  final String ownerAccount;

  /// 本人主页（可编辑资料）还是他人主页。
  final bool isSelf;

  /// 首屏可选注入的资料（缓存或上层已拉到的）。
  final CitizenProfile? initialProfile;

  /// 数据入口，测试可注入替身。
  final CitizenProfileApi? api;
  final CitizenProfileCache? cache;
  final SquareSessionProvider? sessionProvider;

  /// 私聊入口，测试可注入 spy；默认走真 IM。
  final DirectChatOpener? onOpenDirectChat;

  @override
  State<UserProfilePage> createState() => _UserProfilePageState();
}

class _UserProfilePageState extends State<UserProfilePage> {
  static const double _expandedHeight = 300;

  late final CitizenProfileApi _api;
  late final CitizenProfileCache _cache;
  late final SquareSessionProvider _sessionProvider;
  late final DirectChatOpener _directChat;
  CitizenProfile? _profile;
  SquareSession? _session;

  @override
  void initState() {
    super.initState();
    _api = widget.api ?? CitizenProfileApi();
    _cache = widget.cache ?? const CitizenProfileCache();
    _sessionProvider = widget.sessionProvider ?? SquareSessionProvider.instance;
    _directChat = widget.onOpenDirectChat ?? openDirectChat;
    _profile = widget.initialProfile;
    _load();
  }

  Future<void> _load() async {
    // 先渲染缓存（若无注入资料），再后台刷新回刷 + 写回缓存。
    if (_profile == null) {
      final cached = await _cache.read(widget.ownerAccount);
      if (cached != null && mounted) {
        setState(() => _profile = cached);
      }
    }
    final session = await _ensureSession();
    try {
      // 带 session 拉取 → is_following 反映当前登录者视角。
      final fresh =
          await _api.fetchProfile(widget.ownerAccount, session: session);
      if (!mounted) return;
      setState(() => _profile = fresh);
      await _cache.write(fresh);
    } on Exception {
      // 网络/服务异常保留缓存或占位，不覆盖已展示内容。
    }
  }

  /// 默认热钱包静默登录换 session；无热钱包或异常返回 null（按公开只读处理）。
  Future<SquareSession?> _ensureSession() async {
    try {
      final session = await _sessionProvider.ensureSession();
      if (mounted) _session = session;
      return session;
    } on Exception {
      return null;
    }
  }

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
        followers: nextFollowers,
      );
    });
    try {
      if (wasFollowing) {
        await _api.unfollowUser(
          session: session,
          followedAccount: widget.ownerAccount,
        );
      } else {
        await _api.followUser(
          session: session,
          followedAccount: widget.ownerAccount,
        );
      }
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
          ownerAccount: widget.ownerAccount,
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

  void _openQrCode() {
    Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => UserQrPage(
          contactName: _profile?.resolvedDisplayName('') ??
              _shortenAccount(widget.ownerAccount),
          address: widget.ownerAccount,
        ),
      ),
    );
  }

  void _openChatWithUser() {
    final title = _profile?.resolvedDisplayName('') ??
        _shortenAccount(widget.ownerAccount);
    _directChat(context, peerAddress: widget.ownerAccount, title: title);
  }

  void _openImList() {
    Navigator.of(context).push(
      MaterialPageRoute<void>(builder: (_) => ImTabPage()),
    );
  }

  void _openNotifications() {
    Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => const _NotificationsPlaceholderPage(),
      ),
    );
  }

  void _openFollows(FollowsType type) {
    Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => FollowsListPage(
          ownerAccount: widget.ownerAccount,
          type: type,
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

  String get _title =>
      _profile?.resolvedDisplayName('') ?? _shortenAccount(widget.ownerAccount);

  String? _mediaUrl(String? objectKey) =>
      objectKey == null ? null : _api.mediaUrl(objectKey);

  Widget? _bannerWidget() {
    final url = _mediaUrl(_profile?.bannerObjectKey);
    if (url == null) return null;
    return Image.network(
      url,
      fit: BoxFit.cover,
      errorBuilder: (_, __, ___) => const SizedBox.shrink(),
    );
  }

  void _stub(String label) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text('「$label」待接入')),
    );
  }

  void _openPost(SquarePost post) {
    Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => SquarePostDetailPage(post: post),
      ),
    );
  }

  void _openArticle(SquarePost post) {
    Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => SquareArticleDetailPage(post: post),
      ),
    );
  }

  Widget _tabBody(ProfileTab tab) {
    switch (tab) {
      case ProfileTab.posts:
        return ProfilePostsTab(
          ownerAccount: widget.ownerAccount,
          api: _api,
          category: SquarePostCategory.normal,
          contentFormat: SquarePostContentFormat.normal,
          emptyLabel: '还没有帖子',
          onOpenPost: _openPost,
        );
      case ProfileTab.campaign:
        return ProfilePostsTab(
          ownerAccount: widget.ownerAccount,
          api: _api,
          category: SquarePostCategory.campaign,
          emptyLabel: '还没有竞选内容',
          onOpenPost: _openPost,
        );
      case ProfileTab.photos:
        return ProfilePostsTab(
          ownerAccount: widget.ownerAccount,
          api: _api,
          mediaKind: SquareMediaKind.image,
          emptyLabel: '还没有照片',
          onOpenPost: _openPost,
        );
      case ProfileTab.videos:
        return ProfilePostsTab(
          ownerAccount: widget.ownerAccount,
          api: _api,
          mediaKind: SquareMediaKind.video,
          emptyLabel: '还没有视频',
          onOpenPost: _openPost,
        );
      case ProfileTab.articles:
        return ProfilePostsTab(
          ownerAccount: widget.ownerAccount,
          api: _api,
          contentFormat: SquarePostContentFormat.article,
          emptyLabel: '还没有文章',
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
                  icon: const Icon(Icons.arrow_back),
                  onPressed: () => Navigator.of(context).maybePop(),
                ),
                actions: [
                  ProfileKebabMenu(
                    isSelf: widget.isSelf,
                    onQrCode: _openQrCode,
                    onEditProfile: _openEditProfile,
                    onReport: () => _stub('举报'),
                  ),
                ],
                flexibleSpace: FlexibleSpaceBar(
                  background: CollapsibleHeader(
                    expandedHeight: _expandedHeight,
                    collapsedTitle: _title,
                    bottomInset: ProfileCategoryTabs.height,
                    banner: _bannerWidget(),
                    foreground: ProfileHeaderCard(
                      ownerAccount: widget.ownerAccount,
                      profile: _profile,
                      avatarUrl: _mediaUrl(_profile?.avatarObjectKey),
                      onFollowing: () => _openFollows(FollowsType.following),
                      onFollowers: () => _openFollows(FollowsType.followers),
                      onPosts: () => _stub('帖子'),
                      actions: ProfileActionIcons(
                        isSelf: widget.isSelf,
                        isFollowing: _profile?.isFollowing ?? false,
                        onNotifications: _openNotifications,
                        onChat: widget.isSelf ? _openImList : _openChatWithUser,
                        onFollowingList: () =>
                            _openFollows(FollowsType.following),
                        onToggleFollow: _toggleFollow,
                      ),
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

/// 通知系统尚未建，先给一个占位页。
class _NotificationsPlaceholderPage extends StatelessWidget {
  const _NotificationsPlaceholderPage();

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('通知'), centerTitle: true),
      body: const Center(
        child: Text(
          '通知功能即将上线',
          style: TextStyle(color: AppTheme.textTertiary),
        ),
      ),
    );
  }
}

String _shortenAccount(String account) {
  if (account.length <= 12) return account;
  return '${account.substring(0, 6)}...'
      '${account.substring(account.length - 6)}';
}
