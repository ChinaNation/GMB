import 'dart:async';

import 'package:flutter/material.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:citizenapp/wallet/pages/create_wallet_flow.dart';
import 'package:citizenapp/wallet/pages/create_wallet_onboarding_page.dart';

/// 应用级账户门禁：公民 App 的唯一账户是钱包账户，必须至少有 1 个**有效热钱包**。
///
/// 三态：检查中（与应用锁检查同款极简 loading）→ 无有效热钱包 → 强制初始化页
/// （可创建新钱包或用助记词恢复）；有有效热钱包 → 放行 [child]。
///
/// 「有效」由 [WalletManager.isUsableHotWallet] 单源判定：热钱包 + accountId 规范
/// + ss58 与 accountId 一致 + 严档种子条目存在。**冷钱包与半残钱包一律不作为依据**
/// ——只判 null 会让「行还在、身份字段为空」的半残钱包畅通过闸（fail-open）。
///
/// 冷启动判定一次；此后监听 [WalletManager.walletsRevision]，运行期删光钱包
/// 即时踢回初始化页。
class WalletGate extends StatefulWidget {
  const WalletGate({super.key, required this.child, this.defaultWalletLoader});

  final Widget child;

  /// 有效热钱包加载器，测试注入用；默认 [WalletManager.getValidDefaultWallet]
  /// （列表最靠前的**有效**热钱包，没有则返回 null）。
  final Future<WalletProfile?> Function()? defaultWalletLoader;

  @override
  State<WalletGate> createState() => _WalletGateState();
}

enum _GateStatus { checking, needsWallet, ready }

class _WalletGateState extends State<WalletGate> {
  _GateStatus _status = _GateStatus.checking;
  String? _error;

  @override
  void initState() {
    super.initState();
    WalletManager.walletsRevision.addListener(_onWalletsChanged);
    _check();
  }

  @override
  void dispose() {
    WalletManager.walletsRevision.removeListener(_onWalletsChanged);
    super.dispose();
  }

  Future<WalletProfile?> _loadValidWallet() {
    final loader =
        widget.defaultWalletLoader ?? WalletManager().getValidDefaultWallet;
    return loader();
  }

  Future<void> _check() async {
    try {
      final wallet = await _loadValidWallet();
      if (!mounted) return;
      setState(() {
        _status = wallet == null ? _GateStatus.needsWallet : _GateStatus.ready;
      });
    } catch (e) {
      // 本地库读取失败既不能误判成「无钱包」（会把老用户锁进创建页），
      // 也不能直接放行（无身份进广场），停在错误态由用户重试。
      if (!mounted) return;
      setState(() => _error = walletLocalStoreErrorMessage(e));
    }
  }

  /// 运行期钱包增删（我的 → 钱包列表）后重判。
  /// 只在已放行状态下才需要重判——其余状态本就没进 App。
  void _onWalletsChanged() {
    if (!mounted || _status != _GateStatus.ready) return;
    unawaited(_kickOutIfNoValidWallet());
  }

  Future<void> _kickOutIfNoValidWallet() async {
    WalletProfile? wallet;
    try {
      wallet = await _loadValidWallet();
    } catch (e) {
      if (!mounted) return;
      setState(() => _error = walletLocalStoreErrorMessage(e));
      return;
    }
    if (!mounted || wallet != null) return;
    // 踢回前必须清空 AppShell 内已 push 的页面栈：删钱包这个动作本身就发生在
    // 深层页面（我的 → 钱包列表），不清栈的话初始化页会被旧页面盖住，
    // 用户看上去仍留在 App 里。
    Navigator.of(context).popUntil((route) => route.isFirst);
    if (!mounted) return;
    setState(() => _status = _GateStatus.needsWallet);
  }

  void _retry() {
    setState(() {
      _error = null;
      _status = _GateStatus.checking;
    });
    _check();
  }

  @override
  Widget build(BuildContext context) {
    if (_error != null) {
      return Scaffold(
        backgroundColor: AppTheme.scaffoldBg,
        body: Center(
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              const Icon(
                Icons.error_outline,
                size: 40,
                color: AppTheme.textTertiary,
              ),
              const SizedBox(height: 16),
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 32),
                child: Text(
                  _error!,
                  textAlign: TextAlign.center,
                  style: const TextStyle(
                    fontSize: 14,
                    color: AppTheme.textSecondary,
                  ),
                ),
              ),
              const SizedBox(height: 24),
              FilledButton(
                onPressed: _retry,
                child: const Text('重试'),
              ),
            ],
          ),
        ),
      );
    }

    switch (_status) {
      case _GateStatus.checking:
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
      case _GateStatus.needsWallet:
        return CreateWalletOnboardingPage(
          onCreated: () {
            if (!mounted) return;
            setState(() => _status = _GateStatus.ready);
          },
        );
      case _GateStatus.ready:
        return widget.child;
    }
  }
}
