import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:local_auth/local_auth.dart';
import 'package:citizenapp/8964/profile/models/profile_presentation.dart';
import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/profile/user_profile_page.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/my/myid/identity_badge_snapshot_store.dart';
import 'package:citizenapp/my/membership/membership_page.dart';
import 'package:citizenapp/my/myid/myid_page.dart';
import 'package:citizenapp/my/myid/myid_service.dart';
import 'package:citizenapp/rpc/smoldot_client.dart';
import 'package:citizenapp/security/app_lock_service.dart';
import 'package:citizenapp/security/pin_input_page.dart';
import 'package:citizenapp/my/user/contact_book_page.dart';
import 'package:citizenapp/my/user/user_service.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/ui/identity_badge.dart';
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
  });

  final bool showSettingsUpdateDot;
  final WalletManager? walletManager;
  final MyIdService? myIdService;
  final IdentityBadgeSnapshotStore? badgeSnapshotStore;
  final SmoldotClientManager? smoldotClientManager;

  @override
  State<MyTab> createState() => _ProfilePageState();
}

class _ProfilePageState extends State<MyTab> {
  final UserProfileService _userProfileService = UserProfileService();
  late final WalletManager _walletManager;
  late final IdentityBadgeSnapshotStore _badgeSnapshotStore;
  late final MyIdService _myIdService;
  late final SmoldotClientManager _smoldotClientManager;

  UserProfileState _userProfile = const UserProfileState();
  WalletProfile? _defaultWallet;
  String? _defaultWalletIdentityLevel;

  /// 默认钱包的会员购买态（徽章「勾」）；best-effort，读失败为 null。
  final SquareApiClient _squareApi = SquareApiClient();
  SquareMembershipState? _membership;

  /// _loadState 世代号：本地钱包、资料和徽章快照并发重载时，旧结果
  /// 不得覆盖新默认钱包。
  int _loadGeneration = 0;

  /// 同一次 operational 状态下，同一默认钱包只做一次真实链身份刷新。
  String? _operationalIdentityAccount;
  bool _localStateLoaded = false;

  /// 用户身份地址 = 默认用户钱包（列表中最靠前的热钱包）地址。
  String get _communicationAddress => _defaultWallet?.address ?? '';

  /// 用户昵称 = 默认钱包名称；钱包名称异常缺失时使用与统一主页一致的本地昵称，
  /// 绝不把钱包账户放进昵称位置。
  String get _nickname => ProfilePresentation.forAccount(
        _communicationAddress,
      ).resolveDisplayName(walletName: _defaultWallet?.walletName);

  /// 默认钱包徽章信号：颜色只来自账户级链上身份快照，勾来自会员匹配。
  String? get _defaultWalletMembershipLevel => _membership?.membershipLevel;
  bool get _defaultWalletMembershipActive => _membership?.active ?? false;

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
    // 本页常驻 IndexedStack，initState 只跑一次；默认用户钱包在「我的钱包」
    // 里被切换（拖拽置顶）/增删/改名时经 walletsRevision 广播，这里重读身份，
    // 保证昵称、地址、认证勾和「我的主页」入参始终是当前默认用户。
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
    // 先廉价比对(纯 Isar 读):默认钱包地址与昵称都没变的操作
    // (如重命名冷钱包、导入新钱包未置顶)不触发链查询,避免无谓刷新。
    final wallet = await _walletManager.getDefaultWallet();
    if (!mounted) return;
    if (wallet?.address == _defaultWallet?.address &&
        wallet?.walletName == _defaultWallet?.walletName) {
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
    String? identityLevel;
    try {
      final ownerAccount = defaultWallet?.address.trim() ?? '';
      final snapshot = ownerAccount.isEmpty
          ? null
          : await _badgeSnapshotStore.read(ownerAccount);
      identityLevel = switch (snapshot?.identityLevel) {
        'voting' || 'candidate' => snapshot!.identityLevel,
        _ => null,
      };
    } catch (e) {
      debugPrint('profile badge snapshot load failed: $e');
    }
    if (!mounted || generation != _loadGeneration) {
      return;
    }
    setState(() {
      _userProfile = profile;
      _defaultWallet = defaultWallet;
      _defaultWalletIdentityLevel = identityLevel;
      _localStateLoaded = true;
    });
    // 会员购买态（徽章勾）非阻塞加载：昵称/头像先渲染，勾稍后补上。
    unawaited(_refreshMembership(generation));
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
    final ownerAccount = wallet?.address.trim() ?? '';
    if (ownerAccount.isEmpty || _operationalIdentityAccount == ownerAccount) {
      return;
    }
    _operationalIdentityAccount = ownerAccount;

    final state = await _myIdService.getState();
    if (!mounted || _defaultWallet?.address.trim() != ownerAccount) return;

    String? refreshedLevel;
    if (state.isCitizen &&
        state.votingAccount?.trim() == ownerAccount &&
        (state.identityLevel == 'voting' ||
            state.identityLevel == 'candidate')) {
      refreshedLevel = state.identityLevel;
    } else if (state.status == MyIdStatus.queryFailed) {
      // 纯默认用户模型下不再有多身份冲突;仅链读失败时回落徽章快照。
      final snapshot = await _badgeSnapshotStore.read(ownerAccount);
      refreshedLevel = switch (snapshot?.identityLevel) {
        'voting' || 'candidate' => snapshot!.identityLevel,
        _ => null,
      };
    }
    if (!mounted || _defaultWallet?.address.trim() != ownerAccount) return;
    setState(() => _defaultWalletIdentityLevel = refreshedLevel);
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
        builder: (_) => const ContactBookPage(),
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
                      seed: _communicationAddress,
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
                    MaterialPageRoute(builder: (_) => const WalletTab()),
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

    final fallback = ProfilePresentation.forAccount(seed).bannerAsset;
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
                  : Image.asset(
                      ProfilePresentation.forAccount(seed).avatarAsset,
                      width: size,
                      height: size,
                      fit: BoxFit.cover,
                    ),
            ),
          ),
          if (badgeStyle != null)
            Positioned(
              right: -4,
              bottom: -4,
              child: IdentityBadge(
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
