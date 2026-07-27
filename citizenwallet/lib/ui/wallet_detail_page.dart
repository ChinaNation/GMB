import 'package:flutter/material.dart';

import '../util/screenshot_guard.dart';
import '../wallet/wallet_manager.dart';
import 'account_detail_page.dart';
import 'app_theme.dart';

/// Lv2 钱包详情：钱包(master)名 + 助记词备份 + 账户列表 + 添加账户。
///
/// 冷钱包按钱包存种子/助记词;助记词是钱包级根备份,在身份卡内展示(隐藏→确认→
/// 生物识别→显示,防截屏、不可复制)。账户按 `//index` 派生,点账户进 Lv3。
class WalletDetailPage extends StatefulWidget {
  const WalletDetailPage({super.key, required this.wallet});

  final Wallet wallet;

  @override
  State<WalletDetailPage> createState() => _WalletDetailPageState();
}

class _WalletDetailPageState extends State<WalletDetailPage> {
  final WalletManager _walletManager = WalletManager();

  List<Account> _accounts = [];
  bool _loading = true;
  bool _addingAccount = false;

  String? _mnemonic;
  bool _mnemonicVisible = false;
  bool _screenshotGuardActive = false;

  @override
  void initState() {
    super.initState();
    _load();
  }

  @override
  void dispose() {
    if (_screenshotGuardActive) {
      ScreenshotGuard.disable(_onSecurityEvent);
    }
    super.dispose();
  }

  Future<void> _load() async {
    final accounts = await _walletManager.getAccounts(widget.wallet.masterId);
    if (!mounted) return;
    setState(() {
      _accounts = accounts;
      _loading = false;
    });
  }

  void _enableScreenshotGuard() {
    if (!_screenshotGuardActive) {
      _screenshotGuardActive = true;
      ScreenshotGuard.enable(_onSecurityEvent);
    }
  }

  void _onSecurityEvent(String event) {
    if (!mounted) return;
    if (event == 'screenshot_taken' || event == 'screen_recording_started') {
      setState(() {
        _mnemonicVisible = false;
        _mnemonic = null;
      });
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(event == 'screenshot_taken'
              ? '检测到截屏，助记词已隐藏。请勿截屏保存助记词。'
              : '检测到屏幕录制，助记词已隐藏'),
          duration: const Duration(seconds: 3),
        ),
      );
    }
  }

  Future<void> _revealMnemonic() async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('查看助记词'),
        content: const Text('助记词可恢复本钱包全部账户，泄露将导致资产被盗。\n\n确认要查看吗？'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: const Text('取消'),
          ),
          TextButton(
            onPressed: () => Navigator.of(context).pop(true),
            style: TextButton.styleFrom(foregroundColor: AppTheme.danger),
            child: const Text('查看'),
          ),
        ],
      ),
    );
    if (confirmed != true) return;
    try {
      final mnemonic =
          await _walletManager.getMasterMnemonic(widget.wallet.masterId);
      if (!mounted) return;
      _enableScreenshotGuard();
      setState(() {
        _mnemonic = mnemonic;
        _mnemonicVisible = true;
      });
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('验证失败：$e')),
      );
    }
  }

  Future<void> _addAccount() async {
    if (_addingAccount) return;
    setState(() => _addingAccount = true);
    try {
      await _walletManager.addAccount(widget.wallet.masterId);
      await _load();
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('添加账户失败：$e')),
      );
    } finally {
      if (mounted) setState(() => _addingAccount = false);
    }
  }

  Future<void> _openAccount(Account account) async {
    await Navigator.of(context).push<bool>(
      MaterialPageRoute(
        builder: (_) => AccountDetailPage(
          account: account,
          walletName: widget.wallet.walletName,
        ),
      ),
    );
    await _load();
  }

  String _shortAddress(String address) {
    if (address.length <= 16) return address;
    return '${address.substring(0, 8)}...${address.substring(address.length - 6)}';
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('钱包详情'), centerTitle: true),
      body: _loading
          ? const Center(child: CircularProgressIndicator())
          : ListView(
              padding: const EdgeInsets.all(16),
              children: [
                _buildIdentityCard(),
                const SizedBox(height: 20),
                _buildAccountsSection(),
              ],
            ),
    );
  }

  Widget _buildIdentityCard() {
    return Container(
      decoration: AppTheme.cardDecoration(radius: AppTheme.radiusLg),
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // 上排：钱包图标 + 名称。
          Row(
            children: [
              const Icon(Icons.account_balance_wallet_rounded,
                  size: 22, color: AppTheme.primaryLight),
              const SizedBox(width: 10),
              Expanded(
                child: Text(
                  widget.wallet.walletName,
                  style: const TextStyle(
                      fontSize: 18,
                      fontWeight: FontWeight.w700,
                      color: AppTheme.textPrimary),
                  overflow: TextOverflow.ellipsis,
                ),
              ),
            ],
          ),
          const SizedBox(height: 16),
          // 下方：助记词区（隐藏→确认→生物识别→显示，样式同账户私钥区）。
          _buildMnemonicArea(),
        ],
      ),
    );
  }

  Widget _buildMnemonicArea() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text('助记词（钱包备份，一句恢复全部账户）',
            style: TextStyle(
                fontSize: 12,
                color: AppTheme.textSecondary,
                fontWeight: FontWeight.w500)),
        const SizedBox(height: 8),
        if (!_mnemonicVisible)
          Material(
            color: Colors.transparent,
            child: InkWell(
              onTap: _revealMnemonic,
              borderRadius: BorderRadius.circular(AppTheme.radiusSm),
              child: Container(
                width: double.infinity,
                padding:
                    const EdgeInsets.symmetric(vertical: 16, horizontal: 12),
                decoration: BoxDecoration(
                  color: AppTheme.surfaceElevated,
                  borderRadius: BorderRadius.circular(AppTheme.radiusSm),
                  border: Border.all(color: AppTheme.border),
                ),
                child: const Row(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    Icon(Icons.visibility_off_rounded,
                        color: AppTheme.textTertiary, size: 18),
                    SizedBox(width: 8),
                    Text('点击查看助记词',
                        style: TextStyle(
                            color: AppTheme.textTertiary, fontSize: 13)),
                  ],
                ),
              ),
            ),
          )
        else ...[
          Container(
            width: double.infinity,
            padding: const EdgeInsets.all(14),
            decoration: BoxDecoration(
              color: AppTheme.danger.withAlpha(15),
              borderRadius: BorderRadius.circular(AppTheme.radiusSm),
              border: Border.all(color: AppTheme.danger.withAlpha(40)),
            ),
            // 纯 Text（非 SelectableText）→ 不可复制。
            child: Text(
              _mnemonic ?? '无数据',
              style: const TextStyle(
                  fontSize: 14,
                  fontFamily: 'monospace',
                  color: AppTheme.textPrimary,
                  height: 1.6),
            ),
          ),
          const SizedBox(height: 10),
          Row(
            children: [
              const Expanded(
                child: Text('请手抄备份，不支持复制；泄露即等于整钱包控制权',
                    style: TextStyle(
                        color: AppTheme.danger,
                        fontSize: 12,
                        fontWeight: FontWeight.w500)),
              ),
              TextButton.icon(
                onPressed: () => setState(() {
                  _mnemonicVisible = false;
                  _mnemonic = null;
                }),
                icon: const Icon(Icons.visibility_off_rounded, size: 16),
                label: const Text('隐藏'),
              ),
            ],
          ),
        ],
      ],
    );
  }

  Widget _buildAccountsSection() {
    return Container(
      decoration: AppTheme.cardDecoration(radius: AppTheme.radiusLg),
      child: Column(
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 5, 8, 6),
            child: Row(
              children: [
                const Text('账户',
                    style: TextStyle(
                        fontSize: 14,
                        fontWeight: FontWeight.w600,
                        color: AppTheme.textPrimary)),
                const Spacer(),
                TextButton.icon(
                  onPressed: _addingAccount ? null : _addAccount,
                  icon: _addingAccount
                      ? const SizedBox(
                          width: 14,
                          height: 14,
                          child: CircularProgressIndicator(strokeWidth: 2))
                      : const Icon(Icons.add, size: 18),
                  label: const Text('添加账户'),
                ),
              ],
            ),
          ),
          ..._accounts.map(_buildAccountRow),
          const Padding(
            padding: EdgeInsets.fromLTRB(16, 4, 16, 14),
            child: Text(
              '提示:重装或换设备后,重导助记词只自动恢复账户0,其余账户需在此按顺序重新添加(离线设备无法链上探活)。',
              style: TextStyle(fontSize: 11, color: AppTheme.textTertiary),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildAccountRow(Account account) {
    return Material(
      color: Colors.transparent,
      child: InkWell(
        onTap: () => _openAccount(account),
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
          child: Row(
            children: [
              Container(
                width: 38,
                height: 38,
                decoration: BoxDecoration(
                  color: AppTheme.surfaceElevated,
                  borderRadius: BorderRadius.circular(AppTheme.radiusSm),
                ),
                alignment: Alignment.center,
                child: Text('#${account.accountIndex}',
                    style: const TextStyle(
                        fontSize: 13,
                        fontWeight: FontWeight.w600,
                        color: AppTheme.primaryLight)),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(account.accountName,
                        style: const TextStyle(
                            fontSize: 14,
                            fontWeight: FontWeight.w600,
                            color: AppTheme.textPrimary)),
                    const SizedBox(height: 2),
                    Text(_shortAddress(account.ss58Address),
                        style: const TextStyle(
                            fontSize: 12,
                            color: AppTheme.textSecondary,
                            fontFamily: 'monospace')),
                  ],
                ),
              ),
              const Icon(Icons.chevron_right,
                  size: 20, color: AppTheme.textTertiary),
            ],
          ),
        ),
      ),
    );
  }
}
