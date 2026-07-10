import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:local_auth/local_auth.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/profile/user_profile_page.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/my/myid/myid_page.dart';
import 'package:citizenapp/my/myid/myid_service.dart';
import 'package:citizenapp/security/app_lock_service.dart';
import 'package:citizenapp/security/pin_input_page.dart';
import 'package:citizenapp/qr/pages/qr_scan_page.dart';
import 'package:citizenapp/transaction/onchain-transaction/onchain_payment_page.dart';
import 'package:citizenapp/qr/qr_protocols.dart';
import 'package:citizenapp/qr/envelope.dart';
import 'package:citizenapp/qr/bodies/user_contact_body.dart';
import 'package:citizenapp/my/user/user_service.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/ui/identity_badge.dart';
import 'package:citizenapp/im/open_direct_chat.dart';
import 'package:citizenapp/update/app_update.dart';
import 'package:citizenapp/update/update_badge.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:citizenapp/wallet/pages/wallet_page.dart';

class ProfilePage extends StatefulWidget {
  const ProfilePage({
    super.key,
    this.showSettingsUpdateDot = false,
  });

  final bool showSettingsUpdateDot;

  @override
  State<ProfilePage> createState() => _ProfilePageState();
}

class _ProfilePageState extends State<ProfilePage> {
  final UserProfileService _userProfileService = UserProfileService();

  UserProfileState _userProfile = const UserProfileState();
  WalletProfile? _defaultWallet;
  MyIdState _myIdState =
      const MyIdState(identityStatus: MyIdIdentityStatus.notOnchain);

  /// 默认钱包的会员购买态（徽章「勾」）；best-effort，读失败为 null。
  final SquareApiClient _squareApi = SquareApiClient();
  SquareMembershipState? _membership;

  /// _loadState 世代号:含链上查询(秒级),并发调用乱序完成时旧结果
  /// 不得覆盖新身份(stale-wins 会重现「UI 显示旧身份」分叉)。
  int _loadGeneration = 0;

  /// 用户身份地址 = 默认用户钱包（列表中最靠前的热钱包）地址。
  String get _communicationAddress => _defaultWallet?.address ?? '';

  /// 用户昵称 = 默认用户钱包名称。
  String get _nickname =>
      _defaultWallet?.walletName ?? UserProfileService.defaultNickname;

  bool get _isDefaultWalletCertified =>
      _myIdState.isCertified &&
      _defaultWallet?.address.trim() ==
          _myIdState.identityWalletAccount?.trim();

  /// 默认钱包徽章信号：颜色=链上身份档（仅默认钱包即认证身份钱包时有效）、勾=会员匹配。
  String? get _defaultWalletIdentityLevel =>
      _isDefaultWalletCertified ? _myIdState.identityLevel : null;
  String? get _defaultWalletMembershipLevel => _membership?.membershipLevel;
  bool get _defaultWalletMembershipActive => _membership?.active ?? false;

  @override
  void initState() {
    super.initState();
    // 本页常驻 IndexedStack，initState 只跑一次；默认用户钱包在「我的钱包」
    // 里被切换（拖拽置顶）/增删/改名时经 walletsRevision 广播，这里重读身份，
    // 保证昵称、地址、认证勾和「我的主页」入参始终是当前默认用户。
    WalletManager.walletsRevision.addListener(_onWalletsChanged);
    _loadState();
  }

  @override
  void dispose() {
    WalletManager.walletsRevision.removeListener(_onWalletsChanged);
    super.dispose();
  }

  Future<void> _onWalletsChanged() async {
    // 先廉价比对(纯 Isar 读):默认钱包地址与昵称都没变的操作
    // (如重命名冷钱包、导入新钱包未置顶)不触发链查询,避免无谓刷新。
    final wallet = await WalletManager().getDefaultWallet();
    if (!mounted) return;
    if (wallet?.address == _defaultWallet?.address &&
        wallet?.walletName == _defaultWallet?.walletName) {
      return;
    }
    await _loadState();
  }

  Future<void> _loadState() async {
    final generation = ++_loadGeneration;
    final profile = await _userProfileService.getState();
    final defaultWallet = await WalletManager().getDefaultWallet();
    MyIdState myIdState;
    try {
      myIdState = await MyIdService().getState();
    } catch (e) {
      myIdState = MyIdState(
        identityStatus: MyIdIdentityStatus.queryFailed,
        errorMessage: '$e',
      );
    }
    if (!mounted || generation != _loadGeneration) {
      return;
    }
    setState(() {
      _userProfile = profile;
      _defaultWallet = defaultWallet;
      _myIdState = myIdState;
    });
    // 会员购买态（徽章勾）非阻塞加载：昵称/头像先渲染，勾稍后补上。
    unawaited(_refreshMembership(generation));
  }

  Future<void> _refreshMembership(int generation) async {
    try {
      final session = await SquareSessionProvider.instance.ensureSession();
      final membership =
          session != null ? await _squareApi.fetchMembership(session) : null;
      if (!mounted || generation != _loadGeneration) return;
      setState(() => _membership = membership);
    } on Exception catch (e) {
      debugPrint('profile membership load failed: $e');
    }
  }

  Future<void> _openContacts() async {
    await Navigator.of(context).push<void>(
      MaterialPageRoute(
        builder: (_) => ContactBookPage(
          selfAddress: _communicationAddress,
        ),
      ),
    );
    await _loadState();
  }

  Future<void> _openMyProfile() async {
    final address = _communicationAddress;
    if (address.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('请先在「我的 → 我的钱包」创建热钱包')),
      );
      return;
    }
    await Navigator.of(context).push<void>(
      MaterialPageRoute(
        builder: (_) => UserProfilePage(
          ownerAccount: address,
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

  Widget _buildProfileCard() {
    return Padding(
      padding: const EdgeInsets.fromLTRB(14, 14, 14, 14),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _SquareAvatar(
            path: _userProfile.avatarPath,
            size: 84,
            seed: _communicationAddress,
            identityLevel: _defaultWalletIdentityLevel,
            membershipLevel: _defaultWalletMembershipLevel,
            membershipActive: _defaultWalletMembershipActive,
          ),
          const SizedBox(width: 12),
          Expanded(
            child: Text(
              _nickname,
              style: const TextStyle(
                color: Colors.white,
                fontSize: 19,
                fontWeight: FontWeight.w600,
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

  Widget _buildEntryCard({
    required Widget leading,
    required String title,
    required VoidCallback onTap,
  }) {
    return Container(
      decoration: AppTheme.cardDecoration(),
      child: Material(
        color: Colors.transparent,
        child: InkWell(
          borderRadius: BorderRadius.circular(AppTheme.radiusMd),
          onTap: onTap,
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
            child: Row(
              children: [
                Container(
                  width: 36,
                  height: 36,
                  decoration: BoxDecoration(
                    color: AppTheme.surfaceMuted,
                    borderRadius: BorderRadius.circular(10),
                  ),
                  child: Center(child: leading),
                ),
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
                const Icon(Icons.chevron_right,
                    size: 20, color: AppTheme.textTertiary),
              ],
            ),
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
            const SizedBox(height: 16),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: _buildEntryCard(
                leading: SvgPicture.asset(
                  'assets/icons/wallet.svg',
                  width: 22,
                  height: 22,
                  colorFilter:
                      const ColorFilter.mode(AppTheme.danger, BlendMode.srcIn),
                ),
                title: '钱包',
                onTap: () {
                  Navigator.of(context).push(
                    MaterialPageRoute(builder: (_) => const MyWalletPage()),
                  );
                },
              ),
            ),
            const SizedBox(height: 12),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: _buildEntryCard(
                leading: const Icon(
                  Icons.workspace_premium_outlined,
                  color: AppTheme.warning,
                  size: 22,
                ),
                title: '会员',
                onTap: _openMembership,
              ),
            ),
            const SizedBox(height: 12),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: _buildEntryCard(
                leading: SvgPicture.asset(
                  'assets/icons/contact-round.svg',
                  width: 22,
                  height: 22,
                  colorFilter:
                      const ColorFilter.mode(AppTheme.primary, BlendMode.srcIn),
                ),
                title: '通讯录',
                onTap: _openContacts,
              ),
            ),
            const SizedBox(height: 12),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: _buildEntryCard(
                leading: const Icon(
                  Icons.badge_outlined,
                  color: AppTheme.primaryDark,
                  size: 22,
                ),
                title: '电子护照',
                onTap: () {
                  Navigator.of(context).push(
                    MaterialPageRoute(builder: (_) => const MyIdPage()),
                  );
                },
              ),
            ),
            const SizedBox(height: 12),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: _buildEntryCard(
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
            const SizedBox(height: 32),
          ],
        ),
      ),
    );
  }
}

class MembershipPage extends StatefulWidget {
  const MembershipPage({
    super.key,
    SquareApiClient? apiClient,
    SquareSessionProvider? sessionProvider,
  })  : _apiClient = apiClient,
        _sessionProvider = sessionProvider;

  final SquareApiClient? _apiClient;
  final SquareSessionProvider? _sessionProvider;

  @override
  State<MembershipPage> createState() => _MembershipPageState();
}

class _MembershipPageState extends State<MembershipPage> {
  late final SquareApiClient _apiClient =
      widget._apiClient ?? SquareApiClient();
  late final SquareSessionProvider _sessionProvider =
      widget._sessionProvider ?? SquareSessionProvider.instance;
  late Future<_MembershipViewData> _future;

  @override
  void initState() {
    super.initState();
    _future = _load();
  }

  Future<_MembershipViewData> _load() async {
    final session = await _sessionProvider.ensureSession();
    if (session == null) {
      return const _MembershipViewData(
        ownerAccount: '',
        state: null,
      );
    }
    final state = await _apiClient.fetchMembership(session);
    return _MembershipViewData(
      ownerAccount: session.ownerAccount,
      state: state,
    );
  }

  Future<void> _refresh() async {
    setState(() {
      _future = _load();
    });
    await _future;
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('会员')),
      body: FutureBuilder<_MembershipViewData>(
        future: _future,
        builder: (context, snapshot) {
          if (snapshot.connectionState != ConnectionState.done) {
            return const Center(child: CircularProgressIndicator());
          }
          if (snapshot.hasError) {
            return _MembershipMessage(
              title: '会员状态加载失败',
              message: '${snapshot.error}',
              onRetry: _refresh,
            );
          }
          final data = snapshot.data;
          if (data == null || data.state == null) {
            return _MembershipMessage(
              title: '暂无默认热钱包',
              message: '创建默认热钱包后即可显示会员状态。',
              onRetry: _refresh,
            );
          }
          final state = data.state!;
          final plans =
              state.plans.isNotEmpty ? state.plans : _fallbackMembershipPlans;
          return RefreshIndicator(
            onRefresh: _refresh,
            child: ListView(
              padding: const EdgeInsets.fromLTRB(16, 16, 16, 28),
              children: [
                _MembershipStatusCard(
                  ownerAccount: data.ownerAccount,
                  state: state,
                ),
                const SizedBox(height: 14),
                for (final plan in plans) ...[
                  _MembershipPlanCard(
                    plan: plan,
                    currentLevel: state.membershipLevel,
                  ),
                  const SizedBox(height: 12),
                ],
              ],
            ),
          );
        },
      ),
    );
  }
}

class _MembershipViewData {
  const _MembershipViewData({
    required this.ownerAccount,
    required this.state,
  });

  final String ownerAccount;
  final SquareMembershipState? state;
}

class _MembershipMessage extends StatelessWidget {
  const _MembershipMessage({
    required this.title,
    required this.message,
    required this.onRetry,
  });

  final String title;
  final String message;
  final Future<void> Function() onRetry;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(
              title,
              style: const TextStyle(
                color: AppTheme.textPrimary,
                fontWeight: FontWeight.w700,
                fontSize: 18,
              ),
            ),
            const SizedBox(height: 8),
            Text(
              message,
              textAlign: TextAlign.center,
              style: const TextStyle(color: AppTheme.textSecondary),
            ),
            const SizedBox(height: 16),
            OutlinedButton(
              onPressed: onRetry,
              child: const Text('刷新'),
            ),
          ],
        ),
      ),
    );
  }
}

class _MembershipStatusCard extends StatelessWidget {
  const _MembershipStatusCard({
    required this.ownerAccount,
    required this.state,
  });

  final String ownerAccount;
  final SquareMembershipState state;

  @override
  Widget build(BuildContext context) {
    final levelName = _membershipLevelName(state.membershipLevel);
    final statusColor = state.active ? AppTheme.success : AppTheme.warning;
    return Container(
      decoration: AppTheme.cardDecoration(),
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Expanded(
                child: Text(
                  levelName,
                  style: const TextStyle(
                    color: AppTheme.textPrimary,
                    fontSize: 20,
                    fontWeight: FontWeight.w800,
                  ),
                ),
              ),
              Container(
                padding:
                    const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
                decoration: BoxDecoration(
                  color: statusColor.withValues(alpha: 0.12),
                  borderRadius: BorderRadius.circular(999),
                ),
                child: Text(
                  state.active ? '已生效' : '未生效',
                  style: TextStyle(
                    color: statusColor,
                    fontWeight: FontWeight.w700,
                    fontSize: 12,
                  ),
                ),
              ),
            ],
          ),
          const SizedBox(height: 10),
          _MembershipMetaLine(label: '钱包账户', value: ownerAccount),
          _MembershipMetaLine(
            label: '链上身份',
            value: _identityLevelName(state.identityLevel),
          ),
          _MembershipMetaLine(
            label: '订阅状态',
            value: state.subscriptionStatus ?? 'none',
          ),
          if (state.expiresAt > 0)
            _MembershipMetaLine(
              label: '有效期至',
              value: _formatDateTime(state.expiresAt),
            ),
          if ((state.inactiveMessage ?? '').isNotEmpty) ...[
            const SizedBox(height: 10),
            Text(
              state.inactiveMessage!,
              style: const TextStyle(
                color: AppTheme.warning,
                fontSize: 13,
                height: 1.45,
              ),
            ),
          ],
        ],
      ),
    );
  }
}

class _MembershipMetaLine extends StatelessWidget {
  const _MembershipMetaLine({
    required this.label,
    required this.value,
  });

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(top: 7),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 68,
            child: Text(
              label,
              style: const TextStyle(
                color: AppTheme.textTertiary,
                fontSize: 12,
              ),
            ),
          ),
          Expanded(
            child: Text(
              value,
              style: const TextStyle(
                color: AppTheme.textSecondary,
                fontSize: 13,
                height: 1.35,
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class _MembershipPlanCard extends StatelessWidget {
  const _MembershipPlanCard({
    required this.plan,
    required this.currentLevel,
  });

  final SquareMembershipPlan plan;
  final String? currentLevel;

  @override
  Widget build(BuildContext context) {
    final selected = plan.membershipLevel == currentLevel;
    return Container(
      decoration: AppTheme.cardDecoration(selected: selected),
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Expanded(
                child: Text(
                  plan.displayName,
                  style: const TextStyle(
                    color: AppTheme.textPrimary,
                    fontWeight: FontWeight.w800,
                    fontSize: 17,
                  ),
                ),
              ),
              Text(
                plan.priceLabel,
                style: const TextStyle(
                  color: AppTheme.warning,
                  fontWeight: FontWeight.w800,
                  fontSize: 15,
                ),
              ),
            ],
          ),
          const SizedBox(height: 8),
          Text(
            plan.identityLabel,
            style: const TextStyle(
              color: AppTheme.textTertiary,
              fontSize: 12,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 12),
          Text(
            plan.dynamicLabel,
            style: const TextStyle(
              color: AppTheme.textSecondary,
              fontSize: 13,
              height: 1.45,
            ),
          ),
          const SizedBox(height: 6),
          Text(
            plan.articleLabel,
            style: const TextStyle(
              color: AppTheme.textSecondary,
              fontSize: 13,
              height: 1.45,
            ),
          ),
        ],
      ),
    );
  }
}

const List<SquareMembershipPlan> _fallbackMembershipPlans = [
  SquareMembershipPlan(
    membershipLevel: 'visitor',
    displayName: '访客会员',
    priceUsdMonthly: '2.99',
    requiredIdentityLevel: 'visitor',
    dynamicTextMaxChars: 300,
    dynamicImageQuality: 'sd',
    dynamicMaxImages: 9,
    dynamicVideoQuality: 'sd',
    dynamicMaxVideos: 1,
    dynamicMaxVideoSeconds: 60,
    articleTitleMinChars: 10,
    articleTitleMaxChars: 50,
    articleBodyMaxChars: 20000,
    articleCoverQuality: 'hd',
    articleImageQuality: 'sd',
    articleMaxImages: 50,
  ),
  SquareMembershipPlan(
    membershipLevel: 'voting',
    displayName: '投票公民会员',
    priceUsdMonthly: '9.99',
    requiredIdentityLevel: 'voting',
    dynamicTextMaxChars: 300,
    dynamicImageQuality: 'hd',
    dynamicMaxImages: 9,
    dynamicVideoQuality: 'hd',
    dynamicMaxVideos: 1,
    dynamicMaxVideoSeconds: 1800,
    articleTitleMinChars: 10,
    articleTitleMaxChars: 50,
    articleBodyMaxChars: 30000,
    articleCoverQuality: 'hd',
    articleImageQuality: 'hd',
    articleMaxImages: 100,
  ),
  SquareMembershipPlan(
    membershipLevel: 'candidate',
    displayName: '竞选公民会员',
    priceUsdMonthly: '99.99',
    requiredIdentityLevel: 'candidate',
    dynamicTextMaxChars: 300,
    dynamicImageQuality: 'hd',
    dynamicMaxImages: 9,
    dynamicVideoQuality: 'hd',
    dynamicMaxVideos: 1,
    dynamicMaxVideoSeconds: 10800,
    articleTitleMinChars: 10,
    articleTitleMaxChars: 50,
    articleBodyMaxChars: 30000,
    articleCoverQuality: 'hd',
    articleImageQuality: 'hd',
    articleMaxImages: 100,
  ),
];

String _membershipLevelName(String? value) => switch (value) {
      'candidate' => '竞选公民会员',
      'voting' => '投票公民会员',
      'visitor' => '访客会员',
      _ => '暂无会员',
    };

String _identityLevelName(String? value) => switch (value) {
      'candidate' => '竞选公民',
      'voting' => '投票公民',
      _ => '访客',
    };

String _formatDateTime(int millis) {
  final date = DateTime.fromMillisecondsSinceEpoch(millis).toLocal();
  final month = date.month.toString().padLeft(2, '0');
  final day = date.day.toString().padLeft(2, '0');
  final hour = date.hour.toString().padLeft(2, '0');
  final minute = date.minute.toString().padLeft(2, '0');
  return '${date.year}-$month-$day $hour:$minute';
}

class ContactBookPage extends StatefulWidget {
  const ContactBookPage({
    super.key,
    required this.selfAddress,
    this.selectForTrade = false,
  });

  final String selfAddress;

  /// 为 true 时，点击联系人直接返回该联系人（而非弹窗修改昵称）。
  final bool selectForTrade;

  @override
  State<ContactBookPage> createState() => _ContactBookPageState();
}

class _ContactBookPageState extends State<ContactBookPage> {
  final UserContactService _userContactService = UserContactService();
  late Future<List<UserContact>> _contactsFuture;

  @override
  void initState() {
    super.initState();
    _contactsFuture = _userContactService.getContacts();
  }

  void _reload() {
    setState(() {
      _contactsFuture = _userContactService.getContacts();
    });
  }

  Future<void> _scanContactQr() async {
    await Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => QrScanPage(
          mode: QrScanMode.contact,
          selfAddress: widget.selfAddress,
        ),
      ),
    );
    if (!mounted) return;
    _reload();
  }

  Future<void> _openContactDetail(UserContact contact) async {
    await Navigator.of(context).push<void>(
      MaterialPageRoute(
        builder: (_) => _ContactDetailPage(contact: contact),
      ),
    );
    if (!mounted) return;
    _reload();
  }

  Widget _buildEmptyState() {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Container(
              width: 88,
              height: 88,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                color: AppTheme.primary.withAlpha(15),
              ),
              child: const Icon(
                Icons.perm_contact_calendar_outlined,
                size: 44,
                color: AppTheme.primary,
              ),
            ),
            const SizedBox(height: 18),
            const Text(
              '通讯录还是空的',
              style: TextStyle(fontSize: 20, fontWeight: FontWeight.w700),
            ),
            const SizedBox(height: 10),
            const Text(
              '扫描其他用户的二维码后，会把对方的昵称和地址加入通讯录。',
              textAlign: TextAlign.center,
              style: TextStyle(color: AppTheme.textSecondary, height: 1.5),
            ),
          ],
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('我的通讯录'),
        centerTitle: true,
        actions: [
          IconButton(
            onPressed: _scanContactQr,
            icon: SvgPicture.asset(
              'assets/icons/scan-line.svg',
              width: 20,
              height: 20,
            ),
          ),
        ],
      ),
      body: FutureBuilder<List<UserContact>>(
        future: _contactsFuture,
        builder: (context, snapshot) {
          if (snapshot.connectionState != ConnectionState.done) {
            return const Center(child: CircularProgressIndicator());
          }
          final contacts = snapshot.data ?? const <UserContact>[];
          if (contacts.isEmpty) {
            return _buildEmptyState();
          }

          // 按昵称字母排序
          final sorted = List<UserContact>.from(contacts)
            ..sort((a, b) => a.displayNickname
                .toLowerCase()
                .compareTo(b.displayNickname.toLowerCase()));

          return ListView.separated(
            padding: const EdgeInsets.fromLTRB(16, 8, 16, 24),
            itemCount: sorted.length,
            separatorBuilder: (_, __) => const Divider(height: 1),
            itemBuilder: (context, index) {
              final contact = sorted[index];
              return ListTile(
                leading: CircleAvatar(
                  backgroundColor: AppTheme.primary.withAlpha(20),
                  child: Text(
                    contact.displayNickname.characters.first,
                    style: const TextStyle(
                      color: AppTheme.primary,
                      fontWeight: FontWeight.w700,
                    ),
                  ),
                ),
                title: Text(
                  contact.displayNickname,
                  style: const TextStyle(fontWeight: FontWeight.w700),
                ),
                subtitle: Text(
                  contact.address,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(
                    fontSize: 12,
                    color: AppTheme.textTertiary,
                  ),
                ),
                trailing: const Icon(Icons.chevron_right, size: 20),
                onTap: widget.selectForTrade
                    ? () => Navigator.of(context).pop(contact)
                    : () => _openContactDetail(contact),
              );
            },
          );
        },
      ),
    );
  }
}

class _ContactDetailPage extends StatelessWidget {
  const _ContactDetailPage({required this.contact});

  final UserContact contact;

  Future<void> _openMessage(BuildContext context) async {
    await openDirectChat(
      context,
      peerAddress: contact.address,
      title: contact.displayNickname,
    );
  }

  @override
  Widget build(BuildContext context) {
    final qrData = QrEnvelope<UserContactBody>(
      kind: QrKind.userContact,
      id: null,
      issuedAt: null,
      expiresAt: null,
      body: UserContactBody(
        address: contact.address,
        contactName: contact.displayNickname,
      ),
    ).toRawJson();

    return Scaffold(
      appBar: AppBar(
        title: const Text('联系人详情'),
        centerTitle: true,
      ),
      body: ListView(
        padding: const EdgeInsets.all(20),
        children: [
          // 头像 + 昵称
          Center(
            child: Column(
              children: [
                CircleAvatar(
                  radius: 36,
                  backgroundColor: AppTheme.primary.withAlpha(20),
                  child: Text(
                    contact.displayNickname.characters.first,
                    style: const TextStyle(
                      fontSize: 28,
                      color: AppTheme.primary,
                      fontWeight: FontWeight.w700,
                    ),
                  ),
                ),
                const SizedBox(height: 12),
                Text(
                  contact.displayNickname,
                  style: const TextStyle(
                    fontSize: 20,
                    fontWeight: FontWeight.w700,
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 24),
          // 二维码
          Center(
            child: Container(
              padding: const EdgeInsets.all(16),
              decoration: AppTheme.cardDecoration(radius: AppTheme.radiusLg),
              child: QrImageView(
                data: qrData,
                version: QrVersions.auto,
                size: 240,
                backgroundColor: Colors.white,
              ),
            ),
          ),
          const SizedBox(height: 16),
          // 地址 + 复制图标
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 12),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Flexible(
                  child: Text(
                    contact.address,
                    textAlign: TextAlign.center,
                    style: const TextStyle(
                      fontSize: 13,
                      color: AppTheme.textTertiary,
                      height: 1.5,
                    ),
                  ),
                ),
                const SizedBox(width: 4),
                IconButton(
                  constraints: const BoxConstraints(),
                  padding: EdgeInsets.zero,
                  iconSize: 18,
                  onPressed: () {
                    Clipboard.setData(ClipboardData(text: contact.address));
                    ScaffoldMessenger.of(context).showSnackBar(
                      const SnackBar(content: Text('地址已复制')),
                    );
                  },
                  icon: SvgPicture.asset(
                    'assets/icons/copy.svg',
                    width: 16,
                    height: 16,
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 28),
          // 消息 + 转账按钮
          Row(
            children: [
              Expanded(
                child: OutlinedButton.icon(
                  onPressed: () => _openMessage(context),
                  icon: const Icon(Icons.chat_bubble_outline, size: 18),
                  label: const Text('消息'),
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: FilledButton.icon(
                  style: FilledButton.styleFrom(
                    backgroundColor: AppTheme.primary,
                  ),
                  onPressed: () {
                    Navigator.of(context).push(
                      MaterialPageRoute(
                        builder: (_) => OnchainPaymentPage(
                          initialToAddress: contact.address,
                        ),
                      ),
                    );
                  },
                  icon: const Icon(Icons.send, size: 18),
                  label: const Text('转账'),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

class _HeaderBackground extends StatelessWidget {
  const _HeaderBackground({
    required this.path,
    required this.height,
  });

  final String? path;
  final double height;

  @override
  Widget build(BuildContext context) {
    final hasImage = path != null && path!.trim().isNotEmpty;
    final file = hasImage ? File(path!) : null;
    final validImage = file != null && file.existsSync();

    return Container(
      width: double.infinity,
      height: height,
      decoration: BoxDecoration(
        gradient: validImage
            ? null
            : const LinearGradient(
                colors: [
                  AppTheme.primaryDark,
                  AppTheme.primary,
                  AppTheme.primaryLight,
                ],
                begin: Alignment.topLeft,
                end: Alignment.bottomRight,
              ),
        image: validImage
            ? DecorationImage(
                image: FileImage(file),
                fit: BoxFit.cover,
              )
            : null,
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

class _SquareAvatar extends StatelessWidget {
  const _SquareAvatar({
    required this.path,
    required this.size,
    required this.seed,
    this.identityLevel,
    this.membershipLevel,
    this.membershipActive = false,
  });

  final String? path;
  final double size;

  /// 未设头像时按账号稳定选默认头像的种子（默认钱包地址，与用户主页同源）。
  final String seed;

  /// 徽章信号：颜色=链上身份档、勾=会员匹配身份档。
  final String? identityLevel;
  final String? membershipLevel;
  final bool membershipActive;

  @override
  Widget build(BuildContext context) {
    final hasImage = path != null && path!.trim().isNotEmpty;
    final file = hasImage ? File(path!) : null;
    final validImage = file != null && file.existsSync();
    final badgeStyle = identityBadgeStyle(
      identityLevel: identityLevel,
      membershipLevel: membershipLevel,
      membershipActive: membershipActive,
    );

    return SizedBox(
      width: size,
      height: size,
      child: Stack(
        clipBehavior: Clip.none,
        children: [
          Container(
            width: size,
            height: size,
            decoration: BoxDecoration(
              color: AppTheme.primary.withAlpha(20),
              borderRadius: BorderRadius.circular(10),
            ),
            child: ClipRRect(
              borderRadius: BorderRadius.circular(10),
              child: validImage
                  ? Image.file(file, fit: BoxFit.cover)
                  : _DefaultAvatar(seed: seed, size: size),
            ),
          ),
          if (badgeStyle != null)
            Positioned(
              right: -4,
              bottom: -4,
              child: CitizenBadge(
                style: badgeStyle,
                tooltip: identityBadgeLabel(
                  identityLevel: identityLevel,
                  checked: badgeStyle.checked,
                ),
              ),
            ),
        ],
      ),
    );
  }
}

/// 未设头像时按账号稳定选一张默认头像（与用户主页 ProfileHeaderCard 同源：
/// assets/avatars/default_1..6.svg，账号 codeUnits 求和取模，同账号永远同一张）。
class _DefaultAvatar extends StatelessWidget {
  const _DefaultAvatar({required this.seed, required this.size});

  static const int _count = 6;

  final String seed;
  final double size;

  int get _index {
    final sum = seed.codeUnits.fold<int>(0, (acc, unit) => acc + unit);
    return sum % _count + 1;
  }

  @override
  Widget build(BuildContext context) {
    return SvgPicture.asset(
      'assets/avatars/default_$_index.svg',
      width: size,
      height: size,
      fit: BoxFit.cover,
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
  static const FlutterSecureStorage _secure = FlutterSecureStorage();
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
    final deviceLockStr = await _secure.read(key: _deviceLockKey);
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

    await _secure.write(
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
