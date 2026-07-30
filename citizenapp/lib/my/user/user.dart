import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:citizenapp/log/app_log.dart';
import 'package:flutter/services.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:local_auth/local_auth.dart';
import 'package:citizenapp/8964/profile/models/citizen_profile.dart';
import 'package:citizenapp/8964/profile/models/profile_presentation.dart';
import 'package:citizenapp/8964/profile/services/citizen_profile_api.dart';
import 'package:citizenapp/8964/profile/services/citizen_profile_cache.dart';
import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/profile/user_profile_page.dart';
import 'package:citizenapp/8964/profile/widgets/local_identity_avatar.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/my/myid/identity_account_cache.dart';
import 'package:citizenapp/my/myid/identity_badge_snapshot_store.dart';
import 'package:citizenapp/my/creator/creator_page.dart';
import 'package:citizenapp/my/membership/membership_page.dart';
import 'package:citizenapp/my/myid/myid_page.dart';
import 'package:citizenapp/my/myid/myid_service.dart';
import 'package:citizenapp/rpc/smoldot_client.dart';
import 'package:citizenapp/security/app_lock_service.dart';
import 'package:citizenapp/security/pin_input_page.dart';
import 'package:citizenapp/security/secure_storage.dart';
import 'package:citizenapp/my/user/contact_book_page.dart';
import 'package:citizenapp/my/user/user_service.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/update/app_update.dart';
import 'package:citizenapp/update/update_badge.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:citizenapp/wallet/pages/wallet_page.dart';

class MyTab extends StatefulWidget {
  const MyTab({
    super.key,
    this.showSettingsUpdateDot = false,
    this.walletManager,
    this.myIdService,
    this.badgeSnapshotStore,
    this.smoldotClientManager,
    this.profileApi,
    this.profileCache,
    this.sessionProvider,
    this.squareApi,
  });

  final bool showSettingsUpdateDot;
  final WalletManager? walletManager;
  final MyIdService? myIdService;
  final IdentityBadgeSnapshotStore? badgeSnapshotStore;
  final SmoldotClientManager? smoldotClientManager;
  final CitizenProfileApi? profileApi;
  final CitizenProfileCache? profileCache;
  final SquareSessionProvider? sessionProvider;
  final SquareApiClient? squareApi;

  @override
  State<MyTab> createState() => _ProfilePageState();
}

class _ProfilePageState extends State<MyTab> {
  final UserProfileService _userProfileService = UserProfileService();
  late final WalletManager _walletManager;
  late final IdentityBadgeSnapshotStore _badgeSnapshotStore;
  late final MyIdService _myIdService;
  late final SmoldotClientManager _smoldotClientManager;
  late final CitizenProfileApi _profileApi;
  late final CitizenProfileCache _profileCache;
  late final SquareSessionProvider _sessionProvider;

  UserProfileState _userProfile = const UserProfileState();
  WalletProfile? _defaultWallet;
  String? _defaultWalletIdentityLevel;
  CitizenProfile? _publicProfile;

  /// 每个 MyTab 生命周期同一 CID 最多后台刷新一次；反复进入页面只读缓存。
  String? _profileRefreshCid;

  /// 默认钱包的会员购买态（徽章「勾」）；best-effort，读失败为 null。
  late final SquareApiClient _squareApi;
  SquareMembershipState? _membership;

  /// _loadState 世代号：本地钱包、资料和徽章快照并发重载时，旧结果
  /// 不得覆盖新默认钱包。
  int _loadGeneration = 0;

  /// 同一次 operational 状态下，同一身份账户只做一次真实链身份刷新。
  String? _operationalIdentityAccount;
  bool _localStateLoaded = false;

  /// 身份账户（CID 当前绑定账户）ID，单源 [IdentityAccountCache]；无 CID 时账户0
  /// 只作访客展示账户，链读失败不虚构身份。头像/背景 seed、昵称和主页入参跟随 CID。
  String _identityAccountId = '';

  /// 当前身份永久 CID；徽章、公开资料和业务数据都以它作为归属主键。
  String _identityCidNumber = '';

  /// 用户身份账户 ID（展示口径）= 当前身份账户（CID 绑定账户，非恒账户0）。
  String get _communicationAccountId => _identityAccountId;

  /// 公开昵称唯一真源是 CID 资料的 display_name；资料尚未缓存时稳定兜底。
  /// 本机 walletName 只用于钱包列表，绝不进入此展示链路。
  String get _nickname => ProfilePresentation.forIdentityKey(
        _publicProfile?.cidNumber ?? _communicationAccountId,
      ).resolveDisplayName(publicName: _publicProfile?.displayName);

  /// 默认钱包徽章信号：颜色只来自 CID 级链上身份快照，勾来自会员匹配。
  String? get _defaultWalletMembershipLevel => _membership?.membershipLevel;
  bool get _defaultWalletMembershipActive => _membership?.active ?? false;

  // 个人页副标题只组合既有身份与会员快照，不新增第二套身份或订阅真源。
  String get _identityLabel => switch (_defaultWalletIdentityLevel) {
        'candidate' => '竞选身份',
        'voting' => '投票身份',
        _ => '匿名访客',
      };

  String? get _membershipLabel {
    if (!_defaultWalletMembershipActive) return null;
    return switch (_defaultWalletMembershipLevel) {
      'freedom' => '自由会员',
      'democracy' => '民主会员',
      'spark' => '薪火会员',
      _ => null,
    };
  }

  String get _profileSubtitle {
    final membership = _membershipLabel;
    return membership == null
        ? _identityLabel
        : '$_identityLabel · $membership';
  }

  @override
  void initState() {
    super.initState();
    _walletManager = widget.walletManager ?? WalletManager();
    _badgeSnapshotStore =
        widget.badgeSnapshotStore ?? IdentityBadgeSnapshotStore();
    _myIdService = widget.myIdService ??
        MyIdService(
          walletManager: _walletManager,
          badgeSnapshotStore: _badgeSnapshotStore,
        );
    _smoldotClientManager =
        widget.smoldotClientManager ?? SmoldotClientManager.instance;
    _profileApi = widget.profileApi ?? CitizenProfileApi();
    _profileCache = widget.profileCache ?? const CitizenProfileCache();
    _sessionProvider = widget.sessionProvider ?? SquareSessionProvider.instance;
    _squareApi = widget.squareApi ?? SquareApiClient();
    // 本页常驻 IndexedStack，initState 只跑一次；身份账户（CID 绑定账户）在
    // 「我的钱包」被切换 / CID 换绑 / 增删改名时经 walletsRevision 广播，这里重读身份，
    // 保证昵称、地址、认证勾和「我的主页」入参始终是当前身份账户。
    WalletManager.walletsRevision.addListener(_onWalletsChanged);
    _smoldotClientManager.healthStatusListenable
        .addListener(_onChainHealthChanged);
    _loadState();
  }

  @override
  void dispose() {
    WalletManager.walletsRevision.removeListener(_onWalletsChanged);
    _smoldotClientManager.healthStatusListenable
        .removeListener(_onChainHealthChanged);
    super.dispose();
  }

  Future<void> _onWalletsChanged() async {
    // 先廉价比对（纯 Isar 读）：默认钱包账户没变的操作不触发链查询。
    // walletName 是本机标签，改名不得刷新公开昵称或身份。
    final wallet = await _walletManager.getDefaultWallet();
    if (!mounted) return;
    if (wallet?.accountId == _defaultWallet?.accountId) {
      return;
    }
    _operationalIdentityAccount = null;
    _localStateLoaded = false;
    await _loadState();
  }

  Future<void> _loadState() async {
    final generation = ++_loadGeneration;
    final profile = await _userProfileService.getState();
    final defaultWallet = await _walletManager.getDefaultWallet();
    // CID 是快照归属主键；当前绑定账户只负责链读和签名。
    final identity = await IdentityAccountCache.instance.resolve();
    final identityAccountId =
        identity?.accountId ?? defaultWallet?.accountId ?? '';
    final identityCidNumber = identity?.snapshot?.cidNumber ?? '';
    String? identityLevel;
    try {
      final snapshot = identityCidNumber.isEmpty
          ? null
          : await _badgeSnapshotStore.read(identityCidNumber);
      identityLevel = switch (snapshot?.identityLevel) {
        'voting' || 'candidate' => snapshot!.identityLevel,
        _ => null,
      };
    } catch (e) {
      AppLog.d('profile badge snapshot load failed: $e');
    }
    if (!mounted || generation != _loadGeneration) {
      return;
    }
    final identityChanged = identityAccountId != _identityAccountId ||
        identityCidNumber != _identityCidNumber;
    setState(() {
      _userProfile = profile;
      _defaultWallet = defaultWallet;
      _identityAccountId = identityAccountId;
      _identityCidNumber = identityCidNumber;
      _defaultWalletIdentityLevel = identityLevel;
      if (identityChanged) {
        _publicProfile = null;
        _profileRefreshCid = null;
      }
      _localStateLoaded = true;
    });
    // 公开资料与会员态均非阻塞加载：昵称/头像先用缓存或稳定占位渲染。
    unawaited(_refreshRemoteState(generation));
    _onChainHealthChanged();
  }

  void _onChainHealthChanged() {
    if (!_localStateLoaded) return;
    if (_smoldotClientManager.healthStatus != ChainHealthStatus.operational) {
      _operationalIdentityAccount = null;
      return;
    }
    unawaited(_refreshIdentityAfterChainOperational());
  }

  Future<void> _refreshIdentityAfterChainOperational() async {
    final wallet = await _walletManager.getDefaultWallet();
    if (!mounted ||
        _smoldotClientManager.healthStatus != ChainHealthStatus.operational) {
      return;
    }
    if (wallet == null) return;
    final walletAccountId = wallet.accountId;
    // 账户用于当前签名者比对，CID 用于徽章快照归属。默认钱包本身是否被换掉仍按
    // 钱包账户做再入守卫，避免账户0 与子账户混淆导致守卫误判。
    final identity = await IdentityAccountCache.instance.resolve();
    final identityAccountId = identity?.accountId ?? walletAccountId;
    final identityCidNumber =
        identity?.snapshot?.cidNumber ?? _identityCidNumber;
    if (!mounted || _defaultWallet?.accountId != walletAccountId) return;
    if (identityAccountId.isEmpty ||
        _operationalIdentityAccount == identityAccountId) {
      return;
    }
    _operationalIdentityAccount = identityAccountId;

    final state = await _myIdService.getState();
    if (!mounted || _defaultWallet?.accountId != walletAccountId) return;

    String? refreshedLevel;
    if (state.isCitizen &&
        state.votingAccountId?.trim() == identityAccountId &&
        (state.identityLevel == 'voting' ||
            state.identityLevel == 'candidate')) {
      refreshedLevel = state.identityLevel;
    } else if (state.status == MyIdStatus.queryFailed) {
      // 链上一人一 CID 一账户一身份,故无多身份冲突;仅链读失败时回落徽章快照。
      final snapshot = identityCidNumber.isEmpty
          ? null
          : await _badgeSnapshotStore.read(identityCidNumber);
      refreshedLevel = switch (snapshot?.identityLevel) {
        'voting' || 'candidate' => snapshot!.identityLevel,
        _ => null,
      };
    }
    if (!mounted || _defaultWallet?.accountId != walletAccountId) return;
    setState(() {
      _identityCidNumber = state.cidNumber?.trim().isNotEmpty == true
          ? state.cidNumber!.trim()
          : identityCidNumber;
      _defaultWalletIdentityLevel = refreshedLevel;
    });
  }

  Future<void> _refreshRemoteState(int generation) async {
    final SquareSession? session;
    try {
      session = await _sessionProvider.ensureSession();
    } on Exception catch (e) {
      AppLog.d('profile session load failed: $e');
      return;
    }
    if (session == null) return;

    await _loadPublicProfile(session, generation);

    try {
      final membership = await _squareApi.fetchMembership(session);
      if (!mounted || generation != _loadGeneration) return;
      setState(() => _membership = membership);
    } on Exception catch (e) {
      AppLog.d('profile membership load failed: $e');
    }
  }

  /// 缓存立即回刷；同一页面生命周期、同一 CID 只后台请求一次。
  Future<void> _loadPublicProfile(
    SquareSession session,
    int generation,
  ) async {
    final cidNumber = session.cidNumber.trim();
    if (cidNumber.isEmpty) return;
    try {
      final cached = await _profileCache.read(cidNumber);
      if (cached != null && mounted && generation == _loadGeneration) {
        setState(() => _publicProfile = cached);
      }
    } on Exception catch (e) {
      AppLog.d('public profile cache load failed: $e');
    }

    if (_profileRefreshCid == cidNumber) return;
    _profileRefreshCid = cidNumber;
    try {
      final fresh = await _profileApi.fetchProfile(
        cidNumber,
        session: session,
      );
      await _profileCache.write(fresh);
      if (!mounted || generation != _loadGeneration) return;
      setState(() => _publicProfile = fresh);
    } on Exception catch (e) {
      AppLog.d('public profile refresh failed: $e');
    }
  }

  Future<void> _openContacts() async {
    await Navigator.of(context).push<void>(
      MaterialPageRoute(
        builder: (_) => const ContactBookPage(),
      ),
    );
    await _loadState();
  }

  Future<void> _openMyProfile() async {
    final address = _communicationAccountId;
    if (address.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('请先在「我的 → 我的钱包」创建热钱包')),
      );
      return;
    }
    // 资料页身份主键 = CID 号（cid_number）：按 cid 寻址（换绑不变）。自己的 cid
    // 从身份服务取；未占号（纯访客，无 cid）无法拥有广场资料，提示先注册身份。
    final cidNumber = (await _myIdService.getState()).cidNumber?.trim() ?? '';
    if (!mounted) return;
    if (cidNumber.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('请先完成身份注册再查看个人资料')),
      );
      return;
    }
    await Navigator.of(context).push<void>(
      MaterialPageRoute(
        builder: (_) => UserProfilePage(
          cidNumber: cidNumber,
          isSelf: true,
        ),
      ),
    );
    if (!mounted) return;
    await _loadState();
  }

  Future<void> _openMembership() async {
    await Navigator.of(context).push<void>(
      MaterialPageRoute(builder: (_) => const MembershipPage()),
    );
    await _loadState();
  }

  void _openCreator() {
    // 创作者档位/收入与 MyTab 头部展示无关，跟随「身份/设置」惯例不回读 _loadState。
    Navigator.of(context).push<void>(
      MaterialPageRoute(builder: (_) => const CreatorPage()),
    );
  }

  Widget _buildProfileCard() {
    return Padding(
      padding: const EdgeInsets.fromLTRB(14, 14, 14, 14),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          LocalIdentityAvatar(
            path: _userProfile.avatarPath,
            size: 84,
            seed: _communicationAccountId,
            identityLevel: _defaultWalletIdentityLevel,
            membershipLevel: _defaultWalletMembershipLevel,
            membershipActive: _defaultWalletMembershipActive,
          ),
          const SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  _nickname,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(
                    color: Colors.white,
                    fontSize: 20,
                    fontWeight: FontWeight.w700,
                    shadows: [
                      Shadow(
                        color: Color(0x80000000),
                        blurRadius: 10,
                        offset: Offset(0, 2),
                      ),
                    ],
                  ),
                ),
                const SizedBox(height: 6),
                Text(
                  _profileSubtitle,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(
                    color: Colors.white,
                    fontSize: 14,
                    fontWeight: FontWeight.w500,
                    shadows: [
                      Shadow(
                        color: Color(0x99000000),
                        blurRadius: 8,
                        offset: Offset(0, 1),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
          SizedBox(
            height: 84,
            child: Center(
              child: InkWell(
                onTap: _openMyProfile,
                borderRadius: BorderRadius.circular(8),
                child: const Padding(
                  padding: EdgeInsets.all(4),
                  child: Icon(
                    Icons.chevron_right,
                    size: 24,
                    color: Colors.white,
                    shadows: [
                      Shadow(
                        color: Color(0x80000000),
                        blurRadius: 10,
                        offset: Offset(0, 2),
                      ),
                    ],
                  ),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildPrimaryEntry({
    required Widget leading,
    required String title,
    required String subtitle,
    required VoidCallback onTap,
  }) {
    return Container(
      decoration: AppTheme.cardDecoration(),
      child: Material(
        color: Colors.transparent,
        child: InkWell(
          borderRadius: BorderRadius.circular(AppTheme.radiusLg),
          onTap: onTap,
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 14),
            child: Row(
              children: [
                Container(
                  width: 42,
                  height: 42,
                  decoration: BoxDecoration(
                    color: AppTheme.primary.withAlpha(14),
                    borderRadius: BorderRadius.circular(AppTheme.radiusMd),
                  ),
                  child: Center(child: leading),
                ),
                const SizedBox(width: 10),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Text(
                        title,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: const TextStyle(
                          fontWeight: FontWeight.w700,
                          fontSize: 15,
                          color: AppTheme.textPrimary,
                        ),
                      ),
                      const SizedBox(height: 3),
                      Text(
                        subtitle,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: const TextStyle(
                          fontSize: 12,
                          color: AppTheme.textSecondary,
                        ),
                      ),
                    ],
                  ),
                ),
                const Icon(Icons.chevron_right,
                    size: 18, color: AppTheme.textTertiary),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildServiceEntry({
    required Widget leading,
    required String title,
    required VoidCallback onTap,
    Widget? trailing,
  }) {
    return Material(
      color: Colors.transparent,
      child: InkWell(
        onTap: onTap,
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 14),
          child: Row(
            children: [
              SizedBox(width: 36, height: 36, child: Center(child: leading)),
              const SizedBox(width: 12),
              Expanded(
                child: Text(
                  title,
                  style: const TextStyle(
                    fontWeight: FontWeight.w600,
                    fontSize: 15,
                    color: AppTheme.textPrimary,
                  ),
                ),
              ),
              trailing ??
                  const Icon(
                    Icons.chevron_right,
                    size: 20,
                    color: AppTheme.textTertiary,
                  ),
            ],
          ),
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final topPadding = MediaQuery.of(context).padding.top;
    final headerHeight = topPadding + 260.0;
    return AnnotatedRegion<SystemUiOverlayStyle>(
      value: SystemUiOverlayStyle.light.copyWith(
        statusBarColor: Colors.transparent,
      ),
      child: Scaffold(
        body: ListView(
          padding: EdgeInsets.zero,
          children: [
            SizedBox(
              height: headerHeight,
              child: Stack(
                fit: StackFit.expand,
                children: [
                  GestureDetector(
                    onTap: _openMyProfile,
                    child: _HeaderBackground(
                      path: _userProfile.backgroundPath,
                      height: headerHeight,
                      seed: _communicationAccountId,
                    ),
                  ),
                  Positioned(
                    top: topPadding + 10,
                    left: 0,
                    right: 0,
                    child: const Center(
                      child: Text(
                        '我的',
                        style: TextStyle(
                          color: Colors.white,
                          fontSize: 20,
                          fontWeight: FontWeight.w700,
                          shadows: [
                            Shadow(
                              color: Color(0x66000000),
                              blurRadius: 12,
                              offset: Offset(0, 2),
                            ),
                          ],
                        ),
                      ),
                    ),
                  ),
                  Positioned(
                    left: 16,
                    right: 16,
                    bottom: 22,
                    child: _buildProfileCard(),
                  ),
                ],
              ),
            ),
            const SizedBox(height: 12),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: Row(
                children: [
                  Expanded(
                    child: _buildPrimaryEntry(
                      leading: SvgPicture.asset(
                        'assets/icons/wallet.svg',
                        width: 24,
                        height: 24,
                        colorFilter: const ColorFilter.mode(
                          AppTheme.primary,
                          BlendMode.srcIn,
                        ),
                      ),
                      title: '钱包',
                      subtitle: '管理账户',
                      onTap: () {
                        Navigator.of(context).push(
                          MaterialPageRoute(builder: (_) => const WalletTab()),
                        );
                      },
                    ),
                  ),
                  const SizedBox(width: 10),
                  Expanded(
                    child: _buildPrimaryEntry(
                      leading: const Icon(
                        Icons.badge_outlined,
                        color: AppTheme.primary,
                        size: 24,
                      ),
                      title: '身份',
                      subtitle: '注册与查看',
                      onTap: () {
                        Navigator.of(context).push(
                          MaterialPageRoute(builder: (_) => const MyIdPage()),
                        );
                      },
                    ),
                  ),
                ],
              ),
            ),
            const Padding(
              padding: EdgeInsets.fromLTRB(20, 22, 20, 10),
              child: Text(
                '个人服务',
                style: TextStyle(
                  color: AppTheme.textPrimary,
                  fontSize: 17,
                  fontWeight: FontWeight.w700,
                ),
              ),
            ),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: Container(
                decoration: AppTheme.cardDecoration(radius: AppTheme.radiusLg),
                child: Column(
                  children: [
                    _buildServiceEntry(
                      leading: const Icon(
                        Icons.edit_outlined,
                        color: AppTheme.primary,
                        size: 22,
                      ),
                      title: '创作者',
                      onTap: _openCreator,
                    ),
                    const Divider(height: 1, indent: 62, endIndent: 14),
                    _buildServiceEntry(
                      leading: SvgPicture.asset(
                        'assets/icons/contact-round.svg',
                        width: 22,
                        height: 22,
                        colorFilter: const ColorFilter.mode(
                          AppTheme.primary,
                          BlendMode.srcIn,
                        ),
                      ),
                      title: '通讯录',
                      onTap: _openContacts,
                    ),
                    const Divider(height: 1, indent: 62, endIndent: 14),
                    _buildServiceEntry(
                      leading: const Icon(
                        Icons.workspace_premium_outlined,
                        color: AppTheme.primary,
                        size: 22,
                      ),
                      title: '会员｜订阅',
                      onTap: _openMembership,
                    ),
                  ],
                ),
              ),
            ),
            const SizedBox(height: 16),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: Container(
                decoration: AppTheme.cardDecoration(radius: AppTheme.radiusLg),
                child: _buildServiceEntry(
                  leading: UpdateDotBadge(
                    show: widget.showSettingsUpdateDot,
                    dotKey: const Key('settings-entry-update-dot'),
                    child: const Icon(
                      Icons.settings_outlined,
                      color: AppTheme.textSecondary,
                      size: 22,
                    ),
                  ),
                  title: '设置',
                  onTap: () {
                    Navigator.of(context).push(
                      MaterialPageRoute(builder: (_) => const _SettingsPage()),
                    );
                  },
                ),
              ),
            ),
            const SizedBox(height: 32),
          ],
        ),
      ),
    );
  }
}

class _HeaderBackground extends StatelessWidget {
  const _HeaderBackground({
    required this.path,
    required this.height,
    required this.seed,
  });

  final String? path;
  final double height;
  final String seed;

  @override
  Widget build(BuildContext context) {
    final hasImage = path != null && path!.trim().isNotEmpty;
    final file = hasImage ? File(path!) : null;
    final validImage = file != null && file.existsSync();

    final fallback = ProfilePresentation.forIdentityKey(seed).bannerAsset;
    final ImageProvider<Object> backgroundImage;
    if (validImage) {
      backgroundImage = FileImage(file);
    } else {
      backgroundImage = AssetImage(fallback);
    }
    return Container(
      width: double.infinity,
      height: height,
      decoration: BoxDecoration(
        image: DecorationImage(
          image: backgroundImage,
          fit: BoxFit.cover,
        ),
      ),
      child: DecoratedBox(
        decoration: BoxDecoration(
          gradient: LinearGradient(
            colors: [
              Colors.black.withValues(alpha: 0.08),
              Colors.black.withValues(alpha: 0.18),
            ],
            begin: Alignment.topCenter,
            end: Alignment.bottomCenter,
          ),
        ),
      ),
    );
  }
}

class _SettingsPage extends StatefulWidget {
  const _SettingsPage();

  @override
  State<_SettingsPage> createState() => _SettingsPageState();
}

class _SettingsPageState extends State<_SettingsPage> {
  static const String _deviceLockKey = 'device_lock_enabled';
  final LocalAuthentication _localAuth = LocalAuthentication();
  final AppUpdateController _updateController = AppUpdateController.instance;
  bool _deviceLockEnabled = false;
  bool _pinLockEnabled = false;
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _updateController.addListener(_handleUpdateStateChanged);
    _loadSettings();
    _updateController.check();
  }

  @override
  void dispose() {
    _updateController.removeListener(_handleUpdateStateChanged);
    super.dispose();
  }

  void _handleUpdateStateChanged() {
    if (!mounted) return;
    setState(() {});
  }

  Future<void> _loadSettings() async {
    final deviceLockStr = await appSecureStorage.read(key: _deviceLockKey);
    final pinSet = await AppLockService.isPinSet();
    if (!mounted) return;
    setState(() {
      _deviceLockEnabled = deviceLockStr == 'true';
      _pinLockEnabled = pinSet;
      _loading = false;
    });
  }

  Future<void> _toggleDeviceLock(bool value) async {
    if (value) {
      final canCheck = await _localAuth.canCheckBiometrics;
      final isDeviceSupported = await _localAuth.isDeviceSupported();
      if (!canCheck && !isDeviceSupported) {
        if (!mounted) return;
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('您的设备不支持生物识别或设备密码，无法开启设备锁')),
        );
        return;
      }

      try {
        final authenticated = await _localAuth.authenticate(
          localizedReason: '验证身份以开启设备锁',
          options: const AuthenticationOptions(
            stickyAuth: true,
            biometricOnly: false,
          ),
        );
        if (!authenticated) return;
      } catch (e) {
        if (!mounted) return;
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('身份验证失败：$e')),
        );
        return;
      }
    }

    await appSecureStorage.write(
      key: _deviceLockKey,
      value: value.toString(),
    );
    if (!mounted) return;
    setState(() => _deviceLockEnabled = value);
  }

  Future<void> _togglePinLock(bool value) async {
    if (value) {
      // 开启：进入设置 PIN 页面
      final result = await Navigator.of(context).push<bool>(
        MaterialPageRoute(
          builder: (_) => const PinInputPage(mode: PinInputMode.setup),
        ),
      );
      if (result == true && mounted) {
        setState(() => _pinLockEnabled = true);
      }
    } else {
      // 关闭：进入验证 PIN 页面（验证通过后删除）
      final result = await Navigator.of(context).push<bool>(
        MaterialPageRoute(
          builder: (_) => const PinInputPage(mode: PinInputMode.remove),
        ),
      );
      if (result == true && mounted) {
        setState(() => _pinLockEnabled = false);
      }
    }
  }

  Future<void> _installUpdate() async {
    final started = await _updateController.downloadAndInstall();
    if (!mounted) return;

    final error = _updateController.state.errorMessage;
    if (!started && error != null) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(error)),
      );
      return;
    }

    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('已打开系统安装器，请按系统提示完成更新')),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('设置'),
        centerTitle: true,
      ),
      body: _loading
          ? const Center(
              child: CircularProgressIndicator(color: AppTheme.primary))
          : ListView(
              padding: const EdgeInsets.all(16),
              children: [
                // 安全区标题
                const Padding(
                  padding: EdgeInsets.only(left: 4, bottom: 10),
                  child: Row(
                    children: [
                      Icon(Icons.security_rounded,
                          size: 16, color: AppTheme.primary),
                      SizedBox(width: 8),
                      Text(
                        '安全',
                        style: TextStyle(
                          fontSize: 13,
                          fontWeight: FontWeight.w600,
                          color: AppTheme.primary,
                          letterSpacing: 0.5,
                        ),
                      ),
                    ],
                  ),
                ),
                Container(
                  decoration:
                      AppTheme.cardDecoration(radius: AppTheme.radiusLg),
                  child: Column(
                    children: [
                      _buildSettingTile(
                        icon: Icons.fingerprint_rounded,
                        title: '设备锁',
                        subtitle:
                            _pinLockEnabled ? '请先关闭应用锁' : '启动应用时需要生物识别或设备密码',
                        value: _deviceLockEnabled,
                        onChanged: _pinLockEnabled ? null : _toggleDeviceLock,
                      ),
                      const Divider(height: 1, indent: 56, endIndent: 16),
                      _buildSettingTile(
                        icon: Icons.pin_outlined,
                        title: '应用锁',
                        subtitle: _deviceLockEnabled
                            ? '请先关闭设备锁'
                            : '启动应用时需要输入 6 位数字密码',
                        value: _pinLockEnabled,
                        onChanged: _deviceLockEnabled ? null : _togglePinLock,
                      ),
                    ],
                  ),
                ),
                const SizedBox(height: 28),
                // 关于区标题
                const Padding(
                  padding: EdgeInsets.only(left: 4, bottom: 10),
                  child: Row(
                    children: [
                      Icon(Icons.info_outline_rounded,
                          size: 16, color: AppTheme.primary),
                      SizedBox(width: 8),
                      Text(
                        '关于',
                        style: TextStyle(
                          fontSize: 13,
                          fontWeight: FontWeight.w600,
                          color: AppTheme.primary,
                          letterSpacing: 0.5,
                        ),
                      ),
                    ],
                  ),
                ),
                Container(
                  padding: const EdgeInsets.all(16),
                  decoration:
                      AppTheme.cardDecoration(radius: AppTheme.radiusLg),
                  child: Column(
                    children: [
                      Row(
                        children: [
                          Container(
                            width: 36,
                            height: 36,
                            decoration: BoxDecoration(
                              gradient: AppTheme.primaryGradient,
                              borderRadius: BorderRadius.circular(8),
                            ),
                            child: const Icon(Icons.how_to_vote_rounded,
                                color: Colors.white, size: 18),
                          ),
                          const SizedBox(width: 12),
                          const Text('公民',
                              style: TextStyle(
                                  color: AppTheme.textPrimary,
                                  fontWeight: FontWeight.w600,
                                  fontSize: 16)),
                          const Spacer(),
                          _buildUpdateButton(),
                          const SizedBox(width: 8),
                          Text(_updateController.state.versionLabel,
                              style: const TextStyle(
                                  color: AppTheme.textTertiary, fontSize: 13)),
                        ],
                      ),
                      const SizedBox(height: 10),
                      const Row(
                        children: [
                          SizedBox(width: 48),
                          Text(
                            '公民治理，链上投票',
                            style: TextStyle(
                              color: AppTheme.textTertiary,
                              fontSize: 12,
                            ),
                          ),
                        ],
                      ),
                    ],
                  ),
                ),
              ],
            ),
    );
  }

  Widget _buildSettingTile({
    required IconData icon,
    required String title,
    required String subtitle,
    required bool value,
    required ValueChanged<bool>? onChanged,
  }) {
    final disabled = onChanged == null;
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
      child: Row(
        children: [
          Container(
            width: 36,
            height: 36,
            decoration: BoxDecoration(
              color: disabled
                  ? AppTheme.surfaceElevated
                  : AppTheme.primary.withAlpha(20),
              borderRadius: BorderRadius.circular(8),
            ),
            child: Icon(icon,
                size: 20,
                color: disabled ? AppTheme.textTertiary : AppTheme.primary),
          ),
          const SizedBox(width: 14),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  title,
                  style: TextStyle(
                    fontSize: 15,
                    fontWeight: FontWeight.w500,
                    color:
                        disabled ? AppTheme.textTertiary : AppTheme.textPrimary,
                  ),
                ),
                const SizedBox(height: 2),
                Text(
                  subtitle,
                  style: const TextStyle(
                    fontSize: 12,
                    color: AppTheme.textTertiary,
                  ),
                ),
              ],
            ),
          ),
          Switch(
            value: value,
            onChanged: onChanged,
          ),
        ],
      ),
    );
  }

  Widget _buildUpdateButton() {
    final state = _updateController.state;
    if (!state.hasUpdate) {
      return const SizedBox.shrink();
    }

    final downloading = state.status == AppUpdateStatus.downloading;
    final installing = state.status == AppUpdateStatus.installing;
    final disabled = downloading || installing;
    final progress = (state.progress * 100).clamp(0, 99).round();
    final label = downloading
        ? '$progress%'
        : installing
            ? '安装'
            : '更新';

    return SizedBox(
      height: 30,
      child: FilledButton.icon(
        onPressed: disabled ? null : _installUpdate,
        icon: downloading
            ? const SizedBox(
                width: 12,
                height: 12,
                child: CircularProgressIndicator(strokeWidth: 2),
              )
            : const Icon(Icons.system_update_alt_rounded, size: 14),
        label: Text(label),
        style: FilledButton.styleFrom(
          padding: const EdgeInsets.symmetric(horizontal: 10),
          textStyle: const TextStyle(fontSize: 12, fontWeight: FontWeight.w600),
          visualDensity: VisualDensity.compact,
        ),
      ),
    );
  }
}
