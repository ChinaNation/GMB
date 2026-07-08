import 'package:flutter/material.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:citizenapp/wallet/pages/create_wallet_flow.dart';
import 'package:citizenapp/wallet/pages/create_wallet_onboarding_page.dart';

/// 应用级账户门禁：公民 App 的唯一账户是钱包账户，必须至少有 1 个热钱包。
///
/// 三态：检查中（与应用锁检查同款极简 loading）→ 无热钱包（含仅有冷钱包）
/// → 强制创建页；有热钱包 → 放行 [child]。只在冷启动判定一次，使用中删光
/// 钱包不做即时踢回。
class WalletGate extends StatefulWidget {
  const WalletGate({super.key, required this.child, this.defaultWalletLoader});

  final Widget child;

  /// 默认钱包加载器，测试注入用；默认 [WalletManager.getDefaultWallet]
  /// （列表最靠前的热钱包，无热钱包返回 null）。
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
    _check();
  }

  Future<void> _check() async {
    try {
      final loader =
          widget.defaultWalletLoader ?? WalletManager().getDefaultWallet;
      final wallet = await loader();
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
