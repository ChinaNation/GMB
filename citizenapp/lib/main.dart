import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:local_auth/local_auth.dart';
import 'package:citizenapp/8964/square_tab_page.dart';
import 'package:citizenapp/citizen/citizen_tab_page.dart';
import 'package:citizenapp/im/im_runtime.dart';
import 'package:citizenapp/im/im_tab_page.dart';
import 'package:citizenapp/rpc/smoldot_client.dart';
import 'package:citizenapp/security/app_lock_service.dart';
import 'package:citizenapp/security/pin_input_page.dart';
import 'package:citizenapp/transaction/transaction_tab_page.dart';
import 'package:citizenapp/my/util/screenshot_guard.dart';
import 'package:citizenapp/my/user/user.dart';
import 'package:citizenapp/security/app_permission_gate.dart';
import 'package:citizenapp/update/app_update.dart';
import 'package:citizenapp/update/update_badge.dart';
import 'package:citizenapp/wallet/wallet_gate.dart';

import 'ui/app_theme.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();

  // 诊断 — 把所有 framework / widget 静默吞掉的异常都打到 logcat。
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

  // 先销毁可能残留的旧实例（hot restart 场景）。
  // 防止 Rust tokio 线程持有已删除的 Dart FFI 回调导致 SIGABRT。
  SmoldotClientManager.instance.dispose();

  runApp(const CitizenApp());
  WidgetsBinding.instance.addPostFrameCallback((_) {
    unawaited(_initializeSmoldotInBackground());
  });
}

Future<void> _initializeSmoldotInBackground() async {
  try {
    await SmoldotClientManager.instance.initialize();
  } catch (e) {
    debugPrint('[main] smoldot 轻节点初始化失败: $e');
  }
}

class CitizenApp extends StatelessWidget {
  const CitizenApp({super.key});

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
      // 账户门禁排在最后一环：无热钱包先进强制创建页，再放行主界面。
      return const AppPermissionGate(child: WalletGate(child: AppShell()));
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
  final AppUpdateController _updateController = AppUpdateController.instance;
  int _currentIndex = 0;
  int _pendingVoteCount = 0;
  bool _isRooted = false;

  @override
  void initState() {
    super.initState();
    _updateController.addListener(_handleUpdateStateChanged);
    _checkRootStatus();
    // 启动后异步检查正式 Release 更新，只更新设置页状态，不阻塞主界面进入。
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

  Future<void> _checkRootStatus() async {
    final rooted = await ScreenshotGuard.isDeviceRooted();
    if (!mounted) return;
    setState(() => _isRooted = rooted);
  }

  late final Widget _citizenPage =
      CitizenTabPage(onPendingVoteCountChanged: (count) {
    if (mounted && count != _pendingVoteCount) {
      setState(() => _pendingVoteCount = count);
    }
  });

  List<Widget> get _pages => [
        const SquareTabPage(),
        _citizenPage,
        ImTabPage(runtime: ImRuntime()),
        const TransactionTabPage(),
        ProfilePage(showSettingsUpdateDot: _updateController.state.hasUpdate),
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
            const NavigationDestination(
              icon: Icon(Icons.explore_outlined),
              selectedIcon: Icon(Icons.explore),
              label: '广场',
            ),
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
            const NavigationDestination(
              icon: Icon(Icons.chat_bubble_outline_rounded),
              selectedIcon: Icon(Icons.chat_bubble_rounded),
              label: '信息',
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
            NavigationDestination(
                icon: UpdateDotBadge(
                  show: _updateController.state.hasUpdate,
                  dotKey: const Key('my-tab-update-dot'),
                  child: const Icon(Icons.person_outline),
                ),
                selectedIcon: UpdateDotBadge(
                  show: _updateController.state.hasUpdate,
                  dotKey: const Key('my-tab-selected-update-dot'),
                  child: const Icon(Icons.person),
                ),
                label: '我的'),
          ],
        ),
      ),
    );
  }
}
