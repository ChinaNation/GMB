import 'package:flutter/material.dart';
import 'package:wuminapp_mobile/trade/onchain/onchain_trade_page.dart';
import 'package:wuminapp_mobile/user/user.dart';

void main() {
  runApp(const WuminApp());
}

class WuminApp extends StatelessWidget {
  const WuminApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'WuminApp',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.blue),
        useMaterial3: true,
      ),
      home: const AppShell(),
    );
  }
}

class AppShell extends StatefulWidget {
  const AppShell({super.key});

  @override
  State<AppShell> createState() => _AppShellState();
}

class _AppShellState extends State<AppShell> {
  static const Color _inkGreen = Color(0xFF0B3D2E);
  int _currentIndex = 0;

  final List<Widget> _pages = const [
    HomePage(),
    VotingPage(),
    MessagePage(),
    OnchainTradePage(),
    ProfilePage(),
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
              return const IconThemeData(color: _inkGreen);
            }
            return const IconThemeData(color: Colors.black54);
          }),
          labelTextStyle: WidgetStateProperty.resolveWith((states) {
            if (states.contains(WidgetState.selected)) {
              return const TextStyle(
                color: _inkGreen,
                fontWeight: FontWeight.w700,
                height: 0.9,
              );
            }
            return const TextStyle(color: Colors.black54, height: 0.9);
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
          destinations: const [
            NavigationDestination(icon: Icon(Icons.home_outlined), label: '广场'),
            NavigationDestination(
                icon: Icon(Icons.how_to_vote_outlined), label: '治理'),
            NavigationDestination(
                icon: Icon(Icons.message_outlined), label: '消息'),
            NavigationDestination(
                icon: Icon(Icons.travel_explore_outlined), label: '金融'),
            NavigationDestination(
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
  const VotingPage({super.key});

  @override
  State<VotingPage> createState() => _VotingPageState();
}

class _VotingPageState extends State<VotingPage> {
  int _selectedTab = 0;
  static const List<String> _tabs = ['活动', '选举', '机构'];
  static const List<String> _nationalCouncil = ['国家储备委员会'];
  static const List<String> _provincialCouncils = [
    '中枢省储备委员会',
    '岭南省储备委员会',
    '广东省储备委员会',
    '广西省储备委员会',
    '福建省储备委员会',
    '海南省储备委员会',
    '云南省储备委员会',
    '贵州省储备委员会',
    '湖南省储备委员会',
    '江西省储备委员会',
    '浙江省储备委员会',
    '江苏省储备委员会',
    '山东省储备委员会',
    '山西省储备委员会',
    '河南省储备委员会',
    '河北省储备委员会',
    '湖北省储备委员会',
    '陕西省储备委员会',
    '重庆省储备委员会',
    '四川省储备委员会',
    '甘肃省储备委员会',
    '北平省储备委员会',
    '海滨省储备委员会',
    '松江省储备委员会',
    '龙江省储备委员会',
    '吉林省储备委员会',
    '辽宁省储备委员会',
    '宁夏省储备委员会',
    '青海省储备委员会',
    '安徽省储备委员会',
    '台湾省储备委员会',
    '西藏省储备委员会',
    '新疆省储备委员会',
    '西康省储备委员会',
    '阿里省储备委员会',
    '葱岭省储备委员会',
    '天山省储备委员会',
    '河西省储备委员会',
    '昆仑省储备委员会',
    '河套省储备委员会',
    '热河省储备委员会',
    '兴安省储备委员会',
    '合江省储备委员会',
  ];
  static const List<String> _provincialBanks = [
    '中枢省公民储备银行',
    '岭南省公民储备银行',
    '广东省公民储备银行',
    '广西省公民储备银行',
    '福建省公民储备银行',
    '海南省公民储备银行',
    '云南省公民储备银行',
    '贵州省公民储备银行',
    '湖南省公民储备银行',
    '江西省公民储备银行',
    '浙江省公民储备银行',
    '江苏省公民储备银行',
    '山东省公民储备银行',
    '山西省公民储备银行',
    '河南省公民储备银行',
    '河北省公民储备银行',
    '湖北省公民储备银行',
    '陕西省公民储备银行',
    '重庆省公民储备银行',
    '四川省公民储备银行',
    '甘肃省公民储备银行',
    '北平省公民储备银行',
    '滨海省公民储备银行',
    '松江省公民储备银行',
    '龙江省公民储备银行',
    '吉林省公民储备银行',
    '辽宁省公民储备银行',
    '宁夏省公民储备银行',
    '青海省公民储备银行',
    '安徽省公民储备银行',
    '台湾省公民储备银行',
    '西藏省公民储备银行',
    '新疆省公民储备银行',
    '西康省公民储备银行',
    '阿里省公民储备银行',
    '葱岭省公民储备银行',
    '天山省公民储备银行',
    '河西省公民储备银行',
    '昆仑省公民储备银行',
    '河套省公民储备银行',
    '热河省公民储备银行',
    '兴安省公民储备银行',
    '合江省公民储备银行',
  ];

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
    assert(_provincialCouncils.length == 43);
    assert(_provincialBanks.length == 43);

    switch (_selectedTab) {
      case 0:
        return const Center(child: Text('活动页面（开发中）'));
      case 1:
        return const Center(child: Text('选举页面（开发中）'));
      case 2:
        return const _InstitutionCategoryView(
          nationalCouncil: _nationalCouncil,
          provincialCouncils: _provincialCouncils,
          provincialBanks: _provincialBanks,
        );
      default:
        return const SizedBox.shrink();
    }
  }
}

class MessagePage extends StatelessWidget {
  const MessagePage({super.key});

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: Padding(
        padding: const EdgeInsets.fromLTRB(16, 10, 16, 0),
        child: Column(
          children: [
            Row(
              children: [
                IconButton(
                  onPressed: () {},
                  icon: const Icon(Icons.person_outline),
                ),
                Expanded(
                  child: Center(
                    child: Transform.translate(
                      offset: const Offset(0, -3),
                      child: const Text(
                        '消息',
                        style: TextStyle(
                            fontSize: 18, fontWeight: FontWeight.w700),
                      ),
                    ),
                  ),
                ),
                IconButton(
                  onPressed: () {},
                  icon: const Icon(Icons.add),
                ),
              ],
            ),
            Container(
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
            const Expanded(child: Center(child: Text('消息页面（开发中）'))),
          ],
        ),
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

  final List<String> nationalCouncil;
  final List<String> provincialCouncils;
  final List<String> provincialBanks;

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
  final List<String> institutions;

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
        GridView.builder(
          shrinkWrap: true,
          physics: const NeverScrollableScrollPhysics(),
          itemCount: institutions.length,
          gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
            maxCrossAxisExtent: 360,
            mainAxisSpacing: 8,
            crossAxisSpacing: 8,
            childAspectRatio: 2.9,
          ),
          itemBuilder: (context, index) {
            return _InstitutionCard(
              title: institutions[index],
              icon: icon,
              badgeColor: badgeColor,
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
    required this.title,
    required this.icon,
    required this.badgeColor,
  });

  final String title;
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
                title,
                maxLines: 2,
                overflow: TextOverflow.ellipsis,
                style: const TextStyle(fontSize: 13),
              ),
            ),
          ],
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
                fontSize: 17,
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
