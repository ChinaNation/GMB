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
import 'package:citizenapp/8964/profile/widgets/profile_action_icons.dart';
import 'package:citizenapp/8964/profile/widgets/profile_category_tabs.dart';
import 'package:citizenapp/8964/profile/widgets/profile_header_card.dart';
import 'package:citizenapp/8964/profile/widgets/profile_kebab_menu.dart';
import 'package:citizenapp/8964/profile/widgets/profile_posts_list.dart';
import 'package:citizenapp/8964/services/square_account_deletion_service.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/chat/open_direct_chat.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/wallet/core/device_subkey.dart' show bytesToHex;
import 'package:citizenapp/wallet/core/secure_seed_store.dart';
import 'package:citizenapp/wallet/core/seed_sign_error.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 推特式用户主页。
///
/// 折叠虚化头部 + 圆角方形头像/背景（R2）+ 认证勾 + 展示名/地址/签名/计数 +
/// 三图标（本人 通知/聊天/关注 · 他人 关注/消息）+ ⋮（二维码/编辑资料）+
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

  /// 私聊入口，测试可注入 spy；默认走正式 ChatRuntime。
  final DirectChatOpener? onOpenDirectChat;

  @override
  State<UserProfilePage> createState() => _UserProfilePageState();
}

class _UserProfilePageState extends State<UserProfilePage> {
  /// 顶部头图高度（不含状态栏）。
  static const double _bannerHeight = 128;

  /// 头部展开总高（头图 + 白底资料区），不含状态栏。
  static const double _expandedHeight = 348;

  late final CitizenProfileApi _api;
  late final CitizenProfileCache _cache;
  late final SquareSessionProvider _sessionProvider;
  late final DirectChatOpener _directChat;
  CitizenProfile? _profile;
  SquareSession? _session;
  int _postsRevision = 0;

  /// 本机钱包名称 = 昵称，作为展示名兜底（本人主页）。他人主页无本机钱包，
  /// 留空 → 由后端 display_name 兜底。
  String _walletName = '';

  @override
  void initState() {
    super.initState();
    _api = widget.api ?? CitizenProfileApi();
    _cache = widget.cache ?? const CitizenProfileCache();
    _sessionProvider = widget.sessionProvider ?? SquareSessionProvider.instance;
    _directChat = widget.onOpenDirectChat ?? openDirectChat;
    _profile = widget.initialProfile;
    _loadWalletName();
    _load();
  }

  /// 加载本机钱包名称作为昵称兜底（本人主页 = 默认身份钱包）。
  Future<void> _loadWalletName() async {
    if (!widget.isSelf) return;
    try {
      final wallet = await WalletManager().getDefaultWallet();
      final name = wallet?.walletName.trim() ?? '';
      if (name.isNotEmpty && mounted) {
        setState(() => _walletName = name);
      }
    } on Exception {
      // 钱包名兜底失败不影响主页展示，静默忽略。
    }
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

  /// 默认热钱包静默登录换 session；无热钱包或异常返回 null（按不可用降级处理）。
  Future<SquareSession?> _ensureSession() async {
    try {
      final session = await _sessionProvider.ensureSession();
      if (mounted) setState(() => _session = session);
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

  /// 注销用户（仅本人）：二次确认 → 主钥签名(生物识别) → 服务端硬删 → 清本地 → 回落空态。
  /// 无冷静期、硬删不可逆；链上数据与本地钱包不受影响。
  Future<void> _openDeleteAccount() async {
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
        ownerAccount: widget.ownerAccount,
        walletIndex: walletIndex,
        // 动钱动权 → sr25519 主钥对 0x1D 摘要签名（读硬件金库，弹一次生物识别）。
        signAction: (message) async =>
            '0x${bytesToHex(await walletManager.signWithWallet(walletIndex, message))}',
      );
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
    Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => UserQrPage(
          contactName: _displayName,
          address: widget.ownerAccount,
        ),
      ),
    );
  }

  void _openChatWithUser() {
    _directChat(
      context,
      peerAddress: widget.ownerAccount,
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
          ownerAccount: widget.ownerAccount,
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

  /// 本人钱包名是昵称真源，后端 display_name 是公开镜像；均缺失时使用
  /// 按账户稳定选择的本地昵称，账户本身永远不会出现在昵称位置。
  String get _displayName {
    return ProfilePresentation.forAccount(widget.ownerAccount)
        .resolveDisplayName(
      walletName: widget.isSelf ? _walletName : null,
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
      ProfilePresentation.forAccount(widget.ownerAccount).bannerAsset,
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
    if (session == null) {
      return const Center(child: CircularProgressIndicator());
    }
    switch (tab) {
      case ProfileTab.posts:
        return ProfilePostsTab(
          key: ValueKey('posts:$_postsRevision'),
          ownerAccount: widget.ownerAccount,
          api: _api,
          category: SquarePostCategory.normal,
          contentFormat: SquarePostContentFormat.normal,
          emptyLabel: '还没有帖子',
          session: session,
          onOpenPost: _openPost,
        );
      case ProfileTab.campaign:
        return ProfilePostsTab(
          key: ValueKey('campaign:$_postsRevision'),
          ownerAccount: widget.ownerAccount,
          api: _api,
          category: SquarePostCategory.campaign,
          emptyLabel: '还没有竞选内容',
          session: session,
          onOpenPost: _openPost,
        );
      case ProfileTab.photos:
        return ProfilePostsTab(
          key: ValueKey('photos:$_postsRevision'),
          ownerAccount: widget.ownerAccount,
          api: _api,
          mediaKind: SquareMediaKind.image,
          emptyLabel: '还没有照片',
          session: session,
          onOpenPost: _openPost,
        );
      case ProfileTab.videos:
        return ProfilePostsTab(
          key: ValueKey('videos:$_postsRevision'),
          ownerAccount: widget.ownerAccount,
          api: _api,
          mediaKind: SquareMediaKind.video,
          emptyLabel: '还没有视频',
          session: session,
          onOpenPost: _openPost,
        );
      case ProfileTab.articles:
        return ProfilePostsTab(
          key: ValueKey('articles:$_postsRevision'),
          ownerAccount: widget.ownerAccount,
          api: _api,
          contentFormat: SquarePostContentFormat.article,
          emptyLabel: '还没有文章',
          session: session,
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
                      ownerAccount: widget.ownerAccount,
                      profile: _profile,
                      fallbackName: _walletName,
                      avatarUrl: _mediaUrl(_profile?.avatarObjectKey),
                      avatarHeaders: _mediaHeaders,
                      onFollowing: () => _openFollows(FollowsType.following),
                      onFollowers: () => _openFollows(FollowsType.followers),
                      onPosts: () => _stub('帖子'),
                      actions: ProfileActionIcons(
                        isSelf: widget.isSelf,
                        isFollowing: _profile?.isFollowing ?? false,
                        onSubscribe: () => _stub('订阅动态'),
                        onChat: _openChatWithUser,
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
