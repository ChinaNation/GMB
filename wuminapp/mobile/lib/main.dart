import 'package:flutter/material.dart';
import 'package:wuminapp_mobile/pages/profile_page.dart';
import 'package:wuminapp_mobile/pages/qr_scan_page.dart';
import 'package:wuminapp_mobile/services/api_client.dart';

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
  int _currentIndex = 0;
  final List<String> _titles = const ['首页', '投票', '交易', '我的'];

  final List<Widget> _pages = const [
    HomePage(),
    VotingPage(),
    TransactionPage(),
    ProfilePage(),
  ];

  @override
  Widget build(BuildContext context) {
    final isProfileTab = _currentIndex == 3;
    return Scaffold(
      appBar: AppBar(
        title: Text(_titles[_currentIndex]),
        centerTitle: true,
        actions: isProfileTab
            ? [
                IconButton(
                  onPressed: () {
                    Navigator.of(context).push(
                      MaterialPageRoute(builder: (_) => const QrScanPage()),
                    );
                  },
                  tooltip: '扫码',
                  icon: const Icon(Icons.document_scanner_outlined),
                ),
              ]
            : null,
      ),
      body: _pages[_currentIndex],
      bottomNavigationBar: NavigationBar(
        selectedIndex: _currentIndex,
        onDestinationSelected: (index) {
          setState(() {
            _currentIndex = index;
          });
        },
        destinations: const [
          NavigationDestination(icon: Icon(Icons.home_outlined), label: '首页'),
          NavigationDestination(icon: Icon(Icons.how_to_vote_outlined), label: '投票'),
          NavigationDestination(icon: Icon(Icons.receipt_long_outlined), label: '交易'),
          NavigationDestination(icon: Icon(Icons.person_outline), label: '我的'),
        ],
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
  late Future<HealthStatus> _healthFuture;

  @override
  void initState() {
    super.initState();
    _healthFuture = ApiClient().fetchHealth();
  }

  void _reloadHealth() {
    setState(() {
      _healthFuture = ApiClient().fetchHealth();
    });
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(16),
      child: FutureBuilder<HealthStatus>(
        future: _healthFuture,
        builder: (context, snapshot) {
          if (snapshot.connectionState != ConnectionState.done) {
            return const Center(child: CircularProgressIndicator());
          }

          final hasError = snapshot.hasError;
          final statusText = hasError ? 'ERROR' : snapshot.data!.status;
          final statusColor = hasError ? Colors.red : Colors.green;

          return Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Text(
                '首页',
                style: TextStyle(fontSize: 22, fontWeight: FontWeight.w700),
              ),
              const SizedBox(height: 12),
              Card(
                child: Padding(
                  padding: const EdgeInsets.all(16),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        children: [
                          const Text(
                            'Backend status: ',
                            style: TextStyle(fontWeight: FontWeight.w600),
                          ),
                          Text(
                            statusText,
                            style: TextStyle(
                              fontWeight: FontWeight.w700,
                              color: statusColor,
                            ),
                          ),
                          const Spacer(),
                          IconButton(
                            onPressed: _reloadHealth,
                            icon: const Icon(Icons.refresh),
                            tooltip: '刷新',
                          ),
                        ],
                      ),
                      if (hasError) ...[
                        Text('Error: ${snapshot.error}'),
                        const SizedBox(height: 8),
                        const Text(
                          '请先启动 backend：cargo run --manifest-path /Users/rhett/GMB/wuminapp/backend/Cargo.toml',
                        ),
                      ] else ...[
                        Text('Service: ${snapshot.data!.service}'),
                        Text('Version: ${snapshot.data!.version}'),
                      ],
                    ],
                  ),
                ),
              ),
            ],
          );
        },
      ),
    );
  }
}

class VotingPage extends StatelessWidget {
  const VotingPage({super.key});

  @override
  Widget build(BuildContext context) {
    return const Center(child: Text('投票页面（开发中）'));
  }
}

class TransactionPage extends StatelessWidget {
  const TransactionPage({super.key});

  @override
  Widget build(BuildContext context) {
    return const Center(child: Text('交易页面（开发中）'));
  }
}
