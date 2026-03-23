import 'package:flutter/material.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:local_auth/local_auth.dart';
import 'package:wuminapp_mobile/governance/all_proposals_view.dart';
import 'package:wuminapp_mobile/governance/institution_data.dart';
import 'package:wuminapp_mobile/governance/institution_detail_page.dart';
import 'package:wuminapp_mobile/governance/proposal_context.dart';
import 'package:wuminapp_mobile/rpc/smoldot_client.dart';
import 'package:wuminapp_mobile/security/app_lock_service.dart';
import 'package:wuminapp_mobile/security/pin_input_page.dart';
import 'package:wuminapp_mobile/util/screenshot_guard.dart';
import 'package:wuminapp_mobile/trade/onchain/onchain_trade_page.dart';
import 'package:wuminapp_mobile/user/user.dart';
import 'package:wuminapp_mobile/wallet/capabilities/sfid_binding_service.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // 先销毁可能残留的旧实例（hot restart 场景），再重新初始化。
  // 防止 Rust tokio 线程持有已删除的 Dart FFI 回调导致 SIGABRT。
  SmoldotClientManager.instance.dispose();
  try {
    await SmoldotClientManager.instance.initialize();
  } catch (e) {
    debugPrint('[main] smoldot 轻节点初始化失败: $e');
  }

  runApp(const WuminApp());
}

class WuminApp extends StatelessWidget {
  const WuminApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: '公民',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.blue),
        useMaterial3: true,
      ),
      home: const _AppLockGate(),
    );
  }
}

/// 应用锁入口：先检查 PIN 锁 → 再检查设备锁 → 进入主界面。
class _AppLockGate extends StatefulWidget {
  const _AppLockGate();

  @override
  State<_AppLockGate> createState() => _AppLockGateState();
}

class _AppLockGateState extends State<_AppLockGate>
    with WidgetsBindingObserver {
  final LocalAuthentication _localAuth = LocalAuthentication();
  bool _authenticated = false;
  bool _checking = true;
  bool _showDeviceLock = false;

  /// 后台超过此时长后回到前台需重新验证。
  static const Duration _sessionTimeout = Duration(minutes: 5);
  DateTime? _pausedAt;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    _checkLock();
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.paused) {
      _pausedAt = DateTime.now();
    } else if (state == AppLifecycleState.resumed && _authenticated) {
      final paused = _pausedAt;
      if (paused != null &&
          DateTime.now().difference(paused) > _sessionTimeout) {
        // 超时，重新锁定
        setState(() {
          _authenticated = false;
          _checking = true;
          _showDeviceLock = false;
        });
        _checkLock();
      }
      _pausedAt = null;
    }
  }

  Future<void> _checkLock() async {
    // 1. 检查 PIN 锁
    final pinSet = await AppLockService.isPinSet();
    if (pinSet) {
      if (!mounted) return;
      setState(() => _checking = false);
      _showPinVerify();
      return;
    }

    // 2. 检查设备锁（存储在 SecureStorage，防 root 篡改）
    const secure = FlutterSecureStorage();
    final deviceLockStr = await secure.read(key: 'device_lock_enabled');
    final deviceLockEnabled = deviceLockStr == 'true';
    if (deviceLockEnabled) {
      if (!mounted) return;
      setState(() {
        _checking = false;
        _showDeviceLock = true;
      });
      _authenticateDevice();
      return;
    }

    // 3. 无锁，直接进入
    if (!mounted) return;
    setState(() {
      _authenticated = true;
      _checking = false;
    });
  }

  Future<void> _showPinVerify() async {
    if (!mounted) return;
    final result = await Navigator.of(context).push<bool>(
      MaterialPageRoute(
        builder: (_) => const PinInputPage(mode: PinInputMode.verify),
      ),
    );
    if (!mounted) return;
    if (result == true) {
      setState(() => _authenticated = true);
    }
  }

  Future<void> _authenticateDevice() async {
    try {
      final success = await _localAuth.authenticate(
        localizedReason: '请验证身份以进入应用',
        options: const AuthenticationOptions(
          stickyAuth: true,
          biometricOnly: false,
        ),
      );
      if (!mounted) return;
      if (success) {
        setState(() => _authenticated = true);
      }
    } catch (_) {
      // 认证失败，保持锁定状态
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_checking) {
      return const Scaffold(
        body: Center(child: CircularProgressIndicator()),
      );
    }

    if (_authenticated) {
      return const AppShell();
    }

    if (_showDeviceLock) {
      return Scaffold(
        body: Center(
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              const Icon(Icons.lock_outline, size: 64, color: Color(0xFF007A74)),
              const SizedBox(height: 24),
              const Text(
                '应用已锁定',
                style: TextStyle(fontSize: 20, fontWeight: FontWeight.w600),
              ),
              const SizedBox(height: 8),
              const Text(
                '请验证身份以继续',
                style: TextStyle(fontSize: 14, color: Colors.grey),
              ),
              const SizedBox(height: 32),
              ElevatedButton.icon(
                onPressed: _authenticateDevice,
                icon: const Icon(Icons.fingerprint),
                label: const Text('验证身份'),
                style: ElevatedButton.styleFrom(
                  backgroundColor: const Color(0xFF007A74),
                  foregroundColor: Colors.white,
                  padding:
                      const EdgeInsets.symmetric(horizontal: 32, vertical: 12),
                ),
              ),
            ],
          ),
        ),
      );
    }

    // PIN 锁模式下，PinInputPage 已通过 Navigator 展示
    return const Scaffold(
      body: Center(child: CircularProgressIndicator()),
    );
  }
}

class AppShell extends StatefulWidget {
  const AppShell({super.key});

  @override
  State<AppShell> createState() => _AppShellState();
}

class _AppShellState extends State<AppShell> {
  static const Color _navSelectedColor = Color(0xFF007A74);
  static const Color _navUnselectedColor = Color(0xFF111111);
  int _currentIndex = 2;
  int _pendingVoteCount = 0;
  bool _isRooted = false;

  @override
  void initState() {
    super.initState();
    _checkRootStatus();
  }

  Future<void> _checkRootStatus() async {
    final rooted = await ScreenshotGuard.isDeviceRooted();
    if (!mounted) return;
    setState(() => _isRooted = rooted);
  }

  late final List<Widget> _pages = [
    VotingPage(onPendingVoteCountChanged: (count) {
      if (mounted && count != _pendingVoteCount) {
        setState(() => _pendingVoteCount = count);
      }
    }),
    const MessagePage(),
    const OnchainTradePage(),
    const ProfilePage(),
  ];

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Column(
        children: [
          if (_isRooted)
            Container(
              width: double.infinity,
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
              color: Colors.red.shade700,
              child: SafeArea(
                bottom: false,
                child: Row(
                  children: const [
                    Icon(Icons.warning, color: Colors.white, size: 18),
                    SizedBox(width: 8),
                    Expanded(
                      child: Text(
                        '检测到设备已 root/越狱，密钥安全无法保障',
                        style: TextStyle(color: Colors.white, fontSize: 13),
                      ),
                    ),
                  ],
                ),
              ),
            ),
          Expanded(child: _pages[_currentIndex]),
        ],
      ),
      bottomNavigationBar: NavigationBarTheme(
        data: NavigationBarThemeData(
          indicatorColor: const Color(0xFFD7E9E1),
          iconTheme: WidgetStateProperty.resolveWith((states) {
            if (states.contains(WidgetState.selected)) {
              return const IconThemeData(color: _navSelectedColor);
            }
            return const IconThemeData(color: _navUnselectedColor);
          }),
          labelTextStyle: WidgetStateProperty.resolveWith((states) {
            if (states.contains(WidgetState.selected)) {
              return const TextStyle(
                color: _navSelectedColor,
                fontWeight: FontWeight.w700,
                height: 0.9,
              );
            }
            return const TextStyle(color: _navUnselectedColor, height: 0.9);
          }),
        ),
        child: NavigationBar(
          height: 68,
          selectedIndex: _currentIndex,
          onDestinationSelected: (index) {
            setState(() {
              _currentIndex = index;
            });
          },
          destinations: [
            NavigationDestination(
                icon: Badge(
                  isLabelVisible: _pendingVoteCount > 0,
                  label: Text('$_pendingVoteCount',
                      style: const TextStyle(fontSize: 10)),
                  child: const Icon(Icons.how_to_vote_outlined),
                ),
                label: '公民'),
            NavigationDestination(
              icon: SvgPicture.asset(
                'assets/icons/message-square-text.svg',
                width: 22,
                height: 22,
                colorFilter: const ColorFilter.mode(
                  _navUnselectedColor,
                  BlendMode.srcIn,
                ),
              ),
              selectedIcon: SvgPicture.asset(
                'assets/icons/message-square-text.svg',
                width: 22,
                height: 22,
                colorFilter: const ColorFilter.mode(
                  _navSelectedColor,
                  BlendMode.srcIn,
                ),
              ),
              label: '消息',
            ),
            NavigationDestination(
              icon: SvgPicture.asset(
                'assets/icons/scale.svg',
                width: 22,
                height: 22,
                colorFilter: const ColorFilter.mode(
                  _navUnselectedColor,
                  BlendMode.srcIn,
                ),
              ),
              selectedIcon: SvgPicture.asset(
                'assets/icons/scale.svg',
                width: 22,
                height: 22,
                colorFilter: const ColorFilter.mode(
                  _navSelectedColor,
                  BlendMode.srcIn,
                ),
              ),
              label: '交易',
            ),
            const NavigationDestination(
                icon: Icon(Icons.person_outline), label: '我的'),
          ],
        ),
      ),
    );
  }
}

class HomePage extends StatefulWidget {
  const HomePage({super.key});

  @override
  State<HomePage> createState() => _HomePageState();
}

class _HomePageState extends State<HomePage> {
  int _selectedTab = 0;
  static const List<String> _tabs = ['推荐', '视频', '图文'];

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: Column(
        children: [
          const SizedBox(height: 10),
          _PipeTabs(
            tabs: _tabs,
            selectedIndex: _selectedTab,
            onSelected: (index) {
              setState(() {
                _selectedTab = index;
              });
            },
          ),
          const Expanded(child: Center(child: Text('广场页面（开发中）'))),
        ],
      ),
    );
  }
}

class VotingPage extends StatefulWidget {
  const VotingPage({super.key, this.onPendingVoteCountChanged});

  final ValueChanged<int>? onPendingVoteCountChanged;

  @override
  State<VotingPage> createState() => _VotingPageState();
}

class _VotingPageState extends State<VotingPage> {
  int _selectedTab = 1;
  static const List<String> _tabs = ['投票', '治理', '机构'];

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: Column(
        children: [
          const SizedBox(height: 10),
          _PipeTabs(
            tabs: _tabs,
            selectedIndex: _selectedTab,
            onSelected: (index) {
              setState(() {
                _selectedTab = index;
              });
            },
          ),
          Expanded(child: _buildVotingTabContent()),
        ],
      ),
    );
  }

  Widget _buildVotingTabContent() {
    assert(kProvincialCouncils.length == 43);
    assert(kProvincialBanks.length == 43);

    switch (_selectedTab) {
      case 0:
        return const Center(
          child: Text(
            '正在开发中',
            style: TextStyle(fontSize: 16, color: Colors.black54),
          ),
        );
      case 1:
        return AllProposalsView(
          onPendingVoteCountChanged: widget.onPendingVoteCountChanged,
        );
      case 2:
        return _InstitutionCategoryView(
          nationalCouncil: kNationalCouncil,
          provincialCouncils: kProvincialCouncils,
          provincialBanks: kProvincialBanks,
        );
      default:
        return const SizedBox.shrink();
    }
  }
}

class MessagePage extends StatefulWidget {
  const MessagePage({super.key});

  @override
  State<MessagePage> createState() => _MessagePageState();
}

class _MessagePageState extends State<MessagePage> {
  final SfidBindingService _sfidBindingService = SfidBindingService();
  String _selfAccountPubkeyHex = '';

  @override
  void initState() {
    super.initState();
    _loadSelfAccount();
  }

  Future<void> _loadSelfAccount() async {
    final state = await _sfidBindingService.getState();
    if (!mounted) {
      return;
    }
    setState(() {
      _selfAccountPubkeyHex = state.walletPubkeyHex?.trim() ?? '';
    });
  }

  Future<void> _openContactsPage() async {
    await Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) =>
            ContactBookPage(selfAccountPubkeyHex: _selfAccountPubkeyHex),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: Column(
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(4, 10, 4, 0),
            child: Row(
              children: [
                IconButton(
                  onPressed: _openContactsPage,
                  icon: SvgPicture.asset(
                    'assets/icons/contact-round.svg',
                    width: 22,
                    height: 22,
                  ),
                ),
                const Expanded(
                  child: Center(
                    child: Text(
                      '消息',
                      style:
                          TextStyle(fontSize: 20, fontWeight: FontWeight.w700),
                    ),
                  ),
                ),
                const SizedBox(width: 48),
              ],
            ),
          ),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Container(
              height: 40,
              padding: const EdgeInsets.symmetric(horizontal: 12),
              decoration: BoxDecoration(
                color: const Color(0xFFF4F4F4),
                borderRadius: BorderRadius.circular(10),
              ),
              child: const Row(
                children: [
                  Icon(Icons.search, color: Colors.grey, size: 20),
                  SizedBox(width: 8),
                  Text('搜索', style: TextStyle(color: Colors.grey)),
                ],
              ),
            ),
          ),
          const Expanded(child: Center(child: Text('消息页面（开发中）'))),
        ],
      ),
    );
  }
}

class _InstitutionCategoryView extends StatefulWidget {
  const _InstitutionCategoryView({
    required this.nationalCouncil,
    required this.provincialCouncils,
    required this.provincialBanks,
  });

  final List<InstitutionInfo> nationalCouncil;
  final List<InstitutionInfo> provincialCouncils;
  final List<InstitutionInfo> provincialBanks;

  @override
  State<_InstitutionCategoryView> createState() =>
      _InstitutionCategoryViewState();
}

class _InstitutionCategoryViewState extends State<_InstitutionCategoryView> {
  /// 对列表按"管理员机构优先"排序。
  List<InstitutionInfo> _sorted(List<InstitutionInfo> list) {
    final sorted = List<InstitutionInfo>.from(list);
    sorted.sort((a, b) {
      final aAdmin = ProposalContextResolver.isAdminInstitution(a.shenfenId);
      final bAdmin = ProposalContextResolver.isAdminInstitution(b.shenfenId);
      if (aAdmin && !bAdmin) return -1;
      if (!aAdmin && bAdmin) return 1;
      return 0;
    });
    return sorted;
  }

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.fromLTRB(12, 10, 12, 20),
      children: [
        const Text(
          '机构分类',
          style: TextStyle(
            fontSize: 20,
            fontWeight: FontWeight.w700,
            color: Color(0xFF0B3D2E),
          ),
        ),
        const SizedBox(height: 4),
        const SizedBox(height: 12),
        _InstitutionSection(
          title: '国储会',
          icon: Icons.account_balance,
          badgeColor: const Color(0xFF0B3D2E),
          institutions: widget.nationalCouncil,
          onReturnFromDetail: () => setState(() {}),
        ),
        _InstitutionSection(
          title: '省储会',
          icon: Icons.groups_2_outlined,
          badgeColor: const Color(0xFF0E5A44),
          institutions: _sorted(widget.provincialCouncils),
          onReturnFromDetail: () => setState(() {}),
        ),
        _InstitutionSection(
          title: '省储行',
          icon: Icons.account_balance_wallet_outlined,
          badgeColor: const Color(0xFF176650),
          institutions: _sorted(widget.provincialBanks),
          onReturnFromDetail: () => setState(() {}),
        ),
      ],
    );
  }
}

class _InstitutionSection extends StatelessWidget {
  const _InstitutionSection({
    required this.title,
    required this.icon,
    required this.badgeColor,
    required this.institutions,
    this.onReturnFromDetail,
  });

  final String title;
  final IconData icon;
  final Color badgeColor;
  final List<InstitutionInfo> institutions;
  final VoidCallback? onReturnFromDetail;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Icon(icon, size: 18, color: badgeColor),
            const SizedBox(width: 6),
            Text(
              '$title（${institutions.length}）',
              style: const TextStyle(fontSize: 16, fontWeight: FontWeight.w700),
            ),
          ],
        ),
        const SizedBox(height: 8),
        LayoutBuilder(
          builder: (context, constraints) {
            if (constraints.maxWidth <= 0) {
              return const SizedBox.shrink();
            }
            // 机构列表固定一行两列，避免不同 Android 机型出现列数漂移。
            const crossAxisCount = 2;
            final childAspectRatio = constraints.maxWidth < 360 ? 2.6 : 2.9;
            return GridView.builder(
              shrinkWrap: true,
              physics: const NeverScrollableScrollPhysics(),
              itemCount: institutions.length,
              gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
                crossAxisCount: crossAxisCount,
                mainAxisSpacing: 8,
                crossAxisSpacing: 8,
                childAspectRatio: childAspectRatio,
              ),
              itemBuilder: (context, index) {
                final inst = institutions[index];
                final isAdmin = ProposalContextResolver.isAdminInstitution(
                  inst.shenfenId,
                );
                return _InstitutionCard(
                  institution: inst,
                  icon: icon,
                  badgeColor: badgeColor,
                  isAdmin: isAdmin,
                  onReturnFromDetail: onReturnFromDetail,
                );
              },
            );
          },
        ),
        const SizedBox(height: 12),
      ],
    );
  }
}

class _InstitutionCard extends StatelessWidget {
  const _InstitutionCard({
    required this.institution,
    required this.icon,
    required this.badgeColor,
    this.isAdmin = false,
    this.onReturnFromDetail,
  });

  final InstitutionInfo institution;
  final IconData icon;
  final Color badgeColor;
  final bool isAdmin;
  final VoidCallback? onReturnFromDetail;

  static const Color _adminGreen = Color(0xFF2E7D32);

  @override
  Widget build(BuildContext context) {
    final effectiveColor = isAdmin ? _adminGreen : badgeColor;
    return Card(
      margin: EdgeInsets.zero,
      elevation: 0,
      color: isAdmin ? const Color(0xFFE8F5E9) : null,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: BorderSide(
          color: isAdmin
              ? _adminGreen.withValues(alpha: 0.4)
              : badgeColor.withValues(alpha: 0.18),
        ),
      ),
      child: InkWell(
        onTap: () async {
          await Navigator.of(context).push(
            MaterialPageRoute(
              builder: (_) => InstitutionDetailPage(
                institution: institution,
                icon: icon,
                badgeColor: effectiveColor,
              ),
            ),
          );
          onReturnFromDetail?.call();
        },
        borderRadius: BorderRadius.circular(12),
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
          child: Row(
            children: [
              Container(
                width: 24,
                height: 24,
                decoration: BoxDecoration(
                  color: effectiveColor.withValues(alpha: 0.12),
                  borderRadius: BorderRadius.circular(6),
                ),
                child: Icon(icon, size: 14, color: effectiveColor),
              ),
              const SizedBox(width: 8),
              Expanded(
                child: Text(
                  institution.name,
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(fontSize: 13),
                ),
              ),
              Icon(
                Icons.chevron_right,
                size: 16,
                color: Colors.grey[400],
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _PipeTabs extends StatelessWidget {
  const _PipeTabs({
    required this.tabs,
    required this.selectedIndex,
    required this.onSelected,
  });

  final List<String> tabs;
  final int selectedIndex;
  final ValueChanged<int> onSelected;

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        for (int i = 0; i < tabs.length; i++) ...[
          GestureDetector(
            onTap: () => onSelected(i),
            child: Text(
              tabs[i],
              style: TextStyle(
                fontSize: 20,
                fontWeight:
                    i == selectedIndex ? FontWeight.w700 : FontWeight.w400,
                color: i == selectedIndex
                    ? const Color(0xFF0B3D2E)
                    : Colors.black54,
              ),
            ),
          ),
          if (i != tabs.length - 1)
            const Padding(
              padding: EdgeInsets.symmetric(horizontal: 10),
              child: Text(
                '|',
                style: TextStyle(color: Colors.black45),
              ),
            ),
        ],
      ],
    );
  }
}
