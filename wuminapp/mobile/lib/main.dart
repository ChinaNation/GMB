import 'package:flutter/material.dart';
import 'package:wuminapp_mobile/pages/profile_page.dart';
import 'package:wuminapp_mobile/trade/trade.dart';

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
    TradePage(),
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
                icon: Icon(Icons.how_to_vote_outlined), label: '投票'),
            NavigationDestination(
                icon: Icon(Icons.message_outlined), label: '消息'),
            NavigationDestination(
                icon: Icon(Icons.travel_explore_outlined), label: '发现'),
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
  static const List<String> _tabs = ['活动', '选举', '治理', '机构'];

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
          const Expanded(child: Center(child: Text('投票页面（开发中）'))),
        ],
      ),
    );
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
