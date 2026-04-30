import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:local_auth/local_auth.dart';
import 'package:wuminapp_mobile/citizen/citizen_tab_page.dart';
import 'package:wuminapp_mobile/rpc/smoldot_client.dart';
import 'package:wuminapp_mobile/security/app_lock_service.dart';
import 'package:wuminapp_mobile/security/pin_input_page.dart';
import 'package:wuminapp_mobile/ui/transaction_tab_page.dart';
import 'package:wuminapp_mobile/util/screenshot_guard.dart';
import 'package:wuminapp_mobile/trade/pending_tx_reconciler.dart';
import 'package:wuminapp_mobile/user/user.dart';
import 'package:wuminapp_mobile/wallet/capabilities/sfid_binding_service.dart';

import 'ui/app_theme.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // 中文注释：诊断 — 把所有 framework / widget 静默吞掉的异常都打到 logcat。
  // 默认 ErrorWidget 在某些场景下表现为空白方块（白屏），这里换成显眼的红框 + 文字。
  FlutterError.onError = (details) {
    FlutterError.dumpErrorToConsole(details);
    debugPrint(
        '[FlutterError-Diag] library=${details.library} ctx=${details.context} '
        'exception=${details.exception}');
  };
  ErrorWidget.builder = (FlutterErrorDetails details) {
    debugPrint(
        '[ErrorWidget-Diag] exception=${details.exception}\nstack=${details.stack}');
    return Material(
      color: const Color(0xFFFFEEEE),
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: SingleChildScrollView(
          child: Text(
            'WIDGET ERROR:\n${details.exception}\n\n${details.stack}',
            style: const TextStyle(color: Color(0xFFB00020), fontSize: 12),
          ),
        ),
      ),
    );
  };

  // 状态栏样式
  SystemChrome.setSystemUIOverlayStyle(const SystemUiOverlayStyle(
    statusBarColor: Colors.transparent,
    statusBarIconBrightness: Brightness.dark,
    systemNavigationBarColor: AppTheme.surfaceWhite,
    systemNavigationBarIconBrightness: Brightness.dark,
  ));

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
      theme: AppTheme.lightTheme,
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

  /// 周期性 pending 交易对账定时器。
  Timer? _reconcileTimer;

  /// 冷启动首次对账延迟 timer（必须在 dispose 时取消，否则
  /// flutter test pumpAndSettle 后 widget 已 dispose 但 timer 仍 pending，
  /// 触发 "A Timer is still pending" 断言失败 → CI 红 → APK 不产出）。
  Timer? _initialReconcileTimer;

  /// 周期性对账间隔。
  static const Duration _reconcileInterval = Duration(seconds: 60);

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    _checkLock();
    // 冷启动延迟 3 秒触发首次对账，等 smoldot 同步上来。
    _initialReconcileTimer = Timer(
      const Duration(seconds: 3),
      _triggerReconcile,
    );
    _reconcileTimer = Timer.periodic(
      _reconcileInterval,
      (_) => _triggerReconcile(),
    );
  }

  @override
  void dispose() {
    _initialReconcileTimer?.cancel();
    _reconcileTimer?.cancel();
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  void _triggerReconcile() {
    // Reconciler 内部有并发保护，重复触发安全。
    PendingTxReconciler.instance.reconcileAll().catchError((e) {
      debugPrint('[main] 对账触发失败: $e');
      return 0;
    });
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
      // 回到前台时重跑一次对账，处理后台错过的链上确认。
      _triggerReconcile();
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
        body: Center(
          child: SizedBox(
            width: 24,
            height: 24,
            child: CircularProgressIndicator(
              strokeWidth: 2.5,
              color: AppTheme.primary,
            ),
          ),
        ),
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
              Container(
                width: 80,
                height: 80,
                decoration: BoxDecoration(
                  gradient: AppTheme.primaryGradient,
                  borderRadius: BorderRadius.circular(20),
                  boxShadow: [
                    BoxShadow(
                      color: AppTheme.primary.withAlpha(50),
                      blurRadius: 24,
                      offset: const Offset(0, 8),
                    ),
                  ],
                ),
                child: const Icon(
                  Icons.lock_outline,
                  color: Colors.white,
                  size: 36,
                ),
              ),
              const SizedBox(height: 32),
              const Text(
                '应用已锁定',
                style: TextStyle(
                  fontSize: 22,
                  fontWeight: FontWeight.w700,
                  color: AppTheme.textPrimary,
                  letterSpacing: 1,
                ),
              ),
              const SizedBox(height: 8),
              const Text(
                '请验证身份以继续',
                style: TextStyle(
                  fontSize: 14,
                  color: AppTheme.textSecondary,
                ),
              ),
              const SizedBox(height: 40),
              SizedBox(
                width: 200,
                child: FilledButton.icon(
                  onPressed: _authenticateDevice,
                  icon: const Icon(Icons.fingerprint, size: 22),
                  label: const Text('验证身份'),
                ),
              ),
            ],
          ),
        ),
      );
    }

    // PIN 锁模式下，PinInputPage 已通过 Navigator 展示
    return Scaffold(
      body: Center(
        child: Container(
          width: 64,
          height: 64,
          decoration: BoxDecoration(
            gradient: AppTheme.primaryGradient,
            borderRadius: BorderRadius.circular(16),
          ),
          child: const Icon(
            Icons.how_to_vote_outlined,
            color: Colors.white,
            size: 30,
          ),
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
    CitizenTabPage(onPendingVoteCountChanged: (count) {
      if (mounted && count != _pendingVoteCount) {
        setState(() => _pendingVoteCount = count);
      }
    }),
    const MessagePage(),
    const TransactionTabPage(),
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
              margin: const EdgeInsets.fromLTRB(16, 0, 16, 0),
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
              decoration: AppTheme.bannerDecoration(AppTheme.danger),
              child: const SafeArea(
                bottom: false,
                child: Row(
                  children: [
                    Icon(Icons.warning_rounded,
                        color: AppTheme.danger, size: 18),
                    SizedBox(width: 8),
                    Expanded(
                      child: Text(
                        '检测到设备已 root/越狱，密钥安全无法保障',
                        style: TextStyle(
                          color: AppTheme.danger,
                          fontSize: 13,
                          fontWeight: FontWeight.w500,
                        ),
                      ),
                    ),
                  ],
                ),
              ),
            ),
          Expanded(
            child: IndexedStack(
              index: _currentIndex,
              children: _pages,
            ),
          ),
        ],
      ),
      bottomNavigationBar: Container(
        decoration: const BoxDecoration(
          color: AppTheme.surfaceWhite,
          border: Border(
            top: BorderSide(color: AppTheme.border, width: 0.5),
          ),
        ),
        child: NavigationBar(
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
                selectedIcon: Badge(
                  isLabelVisible: _pendingVoteCount > 0,
                  label: Text('$_pendingVoteCount',
                      style: const TextStyle(fontSize: 10)),
                  child: const Icon(Icons.how_to_vote),
                ),
                label: '公民'),
            NavigationDestination(
              icon: SvgPicture.asset(
                'assets/icons/message-square-text.svg',
                width: 22,
                height: 22,
                colorFilter: const ColorFilter.mode(
                  AppTheme.textTertiary,
                  BlendMode.srcIn,
                ),
              ),
              selectedIcon: SvgPicture.asset(
                'assets/icons/message-square-text.svg',
                width: 22,
                height: 22,
                colorFilter: const ColorFilter.mode(
                  AppTheme.primary,
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
                  AppTheme.textTertiary,
                  BlendMode.srcIn,
                ),
              ),
              selectedIcon: SvgPicture.asset(
                'assets/icons/scale.svg',
                width: 22,
                height: 22,
                colorFilter: const ColorFilter.mode(
                  AppTheme.primary,
                  BlendMode.srcIn,
                ),
              ),
              label: '交易',
            ),
            const NavigationDestination(
                icon: Icon(Icons.person_outline),
                selectedIcon: Icon(Icons.person),
                label: '我的'),
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
          _StyledTabs(
            tabs: _tabs,
            selectedIndex: _selectedTab,
            onSelected: (index) {
              setState(() {
                _selectedTab = index;
              });
            },
          ),
          const Expanded(
            child: Center(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(Icons.explore_outlined,
                      size: 48, color: AppTheme.textTertiary),
                  SizedBox(height: 12),
                  Text(
                    '广场页面（开发中）',
                    style: TextStyle(
                      fontSize: 16,
                      color: AppTheme.textSecondary,
                    ),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
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
                    colorFilter: const ColorFilter.mode(
                      AppTheme.textPrimary,
                      BlendMode.srcIn,
                    ),
                  ),
                ),
                const Expanded(
                  child: Center(
                    child: Text(
                      '消息',
                      style: TextStyle(
                        fontSize: 20,
                        fontWeight: FontWeight.w700,
                        color: AppTheme.textPrimary,
                      ),
                    ),
                  ),
                ),
                const SizedBox(width: 48),
              ],
            ),
          ),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
            child: Container(
              height: 44,
              padding: const EdgeInsets.symmetric(horizontal: 14),
              decoration: BoxDecoration(
                color: AppTheme.surfaceMuted,
                borderRadius: BorderRadius.circular(AppTheme.radiusMd),
                border: Border.all(color: AppTheme.border),
              ),
              child: const Row(
                children: [
                  Icon(Icons.search_rounded,
                      color: AppTheme.textTertiary, size: 20),
                  SizedBox(width: 10),
                  Text('搜索',
                      style: TextStyle(
                        color: AppTheme.textTertiary,
                        fontSize: 15,
                      )),
                ],
              ),
            ),
          ),
          const Expanded(
            child: Center(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(Icons.chat_bubble_outline_rounded,
                      size: 48, color: AppTheme.textTertiary),
                  SizedBox(height: 12),
                  Text(
                    '消息页面（开发中）',
                    style: TextStyle(
                      fontSize: 16,
                      color: AppTheme.textSecondary,
                    ),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}

/// 精致的 tab 切换组件（替代原 _PipeTabs）。
class _StyledTabs extends StatelessWidget {
  const _StyledTabs({
    required this.tabs,
    required this.selectedIndex,
    required this.onSelected,
  });

  final List<String> tabs;
  final int selectedIndex;
  final ValueChanged<int> onSelected;

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: const EdgeInsets.symmetric(horizontal: 48, vertical: 4),
      padding: const EdgeInsets.all(4),
      decoration: BoxDecoration(
        color: AppTheme.surfaceMuted,
        borderRadius: BorderRadius.circular(AppTheme.radiusMd),
        border: Border.all(color: AppTheme.border),
      ),
      child: Row(
        children: [
          for (int i = 0; i < tabs.length; i++)
            Expanded(
              child: GestureDetector(
                onTap: () => onSelected(i),
                child: AnimatedContainer(
                  duration: const Duration(milliseconds: 200),
                  curve: Curves.easeInOut,
                  padding: const EdgeInsets.symmetric(vertical: 8),
                  decoration: BoxDecoration(
                    color: i == selectedIndex
                        ? AppTheme.surfaceWhite
                        : Colors.transparent,
                    borderRadius: BorderRadius.circular(AppTheme.radiusSm),
                    boxShadow: i == selectedIndex
                        ? [
                            BoxShadow(
                              color: AppTheme.primary.withAlpha(15),
                              blurRadius: 4,
                              offset: const Offset(0, 1),
                            ),
                          ]
                        : null,
                  ),
                  child: Text(
                    tabs[i],
                    textAlign: TextAlign.center,
                    style: TextStyle(
                      fontSize: 15,
                      fontWeight: i == selectedIndex
                          ? FontWeight.w700
                          : FontWeight.w500,
                      color: i == selectedIndex
                          ? AppTheme.primary
                          : AppTheme.textSecondary,
                    ),
                  ),
                ),
              ),
            ),
        ],
      ),
    );
  }
}
