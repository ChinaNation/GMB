import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:local_auth/local_auth.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/governance/all_proposals_view.dart';
import 'package:wuminapp_mobile/governance/institution_data.dart';
import 'package:wuminapp_mobile/governance/institution_detail_page.dart';
import 'package:wuminapp_mobile/trade/onchain/onchain_trade_page.dart';
import 'package:wuminapp_mobile/user/user.dart';
import 'package:wuminapp_mobile/wallet/capabilities/sfid_binding_service.dart';

void main() {
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

/// 应用锁入口：检查是否需要认证才能进入主界面。
class _AppLockGate extends StatefulWidget {
  const _AppLockGate();

  @override
  State<_AppLockGate> createState() => _AppLockGateState();
}

class _AppLockGateState extends State<_AppLockGate> {
  final LocalAuthentication _localAuth = LocalAuthentication();
  bool _authenticated = false;
  bool _checking = true;

  @override
  void initState() {
    super.initState();
    _checkAppLock();
  }

  Future<void> _checkAppLock() async {
    final prefs = await SharedPreferences.getInstance();
    final lockEnabled = prefs.getBool('app_lock_enabled') ?? false;

    if (!lockEnabled) {
      if (!mounted) return;
      setState(() {
        _authenticated = true;
        _checking = false;
      });
      return;
    }

    // 应用锁已开启，需要认证
    if (!mounted) return;
    setState(() => _checking = false);
    _authenticate();
  }

  Future<void> _authenticate() async {
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

    // 锁定界面
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
              onPressed: _authenticate,
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
      body: _pages[_currentIndex],
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
        return const _InstitutionCategoryView(
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

class _InstitutionCategoryView extends StatelessWidget {
  const _InstitutionCategoryView({
    required this.nationalCouncil,
    required this.provincialCouncils,
    required this.provincialBanks,
  });

  final List<InstitutionInfo> nationalCouncil;
  final List<InstitutionInfo> provincialCouncils;
  final List<InstitutionInfo> provincialBanks;

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
          institutions: nationalCouncil,
        ),
        _InstitutionSection(
          title: '省储会',
          icon: Icons.groups_2_outlined,
          badgeColor: const Color(0xFF0E5A44),
          institutions: provincialCouncils,
        ),
        _InstitutionSection(
          title: '省储行',
          icon: Icons.account_balance_wallet_outlined,
          badgeColor: const Color(0xFF176650),
          institutions: provincialBanks,
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
  });

  final String title;
  final IconData icon;
  final Color badgeColor;
  final List<InstitutionInfo> institutions;

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
                return _InstitutionCard(
                  institution: institutions[index],
                  icon: icon,
                  badgeColor: badgeColor,
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
  });

  final InstitutionInfo institution;
  final IconData icon;
  final Color badgeColor;

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: EdgeInsets.zero,
      elevation: 0,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: BorderSide(color: badgeColor.withValues(alpha: 0.18)),
      ),
      child: InkWell(
        onTap: () {
          Navigator.of(context).push(
            MaterialPageRoute(
              builder: (_) => InstitutionDetailPage(
                institution: institution,
                icon: icon,
                badgeColor: badgeColor,
              ),
            ),
          );
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
                  color: badgeColor.withValues(alpha: 0.12),
                  borderRadius: BorderRadius.circular(6),
                ),
                child: Icon(icon, size: 14, color: badgeColor),
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
