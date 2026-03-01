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
  final List<String> _titles = const ['首页', '投票', '交易', '我的'];

  final List<Widget> _pages = const [
    HomePage(),
    VotingPage(),
    TradePage(),
    ProfilePage(),
  ];

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(_titles[_currentIndex]),
        centerTitle: true,
      ),
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
              );
            }
            return const TextStyle(color: Colors.black54);
          }),
        ),
        child: NavigationBar(
          selectedIndex: _currentIndex,
          onDestinationSelected: (index) {
            setState(() {
              _currentIndex = index;
            });
          },
          destinations: const [
            NavigationDestination(icon: Icon(Icons.home_outlined), label: '首页'),
            NavigationDestination(
                icon: Icon(Icons.how_to_vote_outlined), label: '投票'),
            NavigationDestination(
                icon: Icon(Icons.receipt_long_outlined), label: '交易'),
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
  @override
  Widget build(BuildContext context) {
    return const Center(child: Text('首页页面（开发中）'));
  }
}

class VotingPage extends StatelessWidget {
  const VotingPage({super.key});

  @override
  Widget build(BuildContext context) {
    return const Center(child: Text('投票页面（开发中）'));
  }
}
