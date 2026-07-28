import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:citizenapp/my/util/screenshot_guard.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:citizenapp/wallet/widgets/add_account_sheet.dart';

/// 账户详情（Lv3）：单个 `//index` 账户的名称、SS58 与私钥。
///
/// 无根多账户下每账户私钥（child mini-secret）独立、单向：导出单账户只暴露该账户，
/// 不牵连锚点账户0 或兄弟账户。私钥展示前需生物识别 + 防截屏 + 纯文本不可复制。
/// 右上角「＋」提供「添加账户」（重新录入助记词追加同一钱包的其他 `//index`）。
class AccountDetailPage extends StatefulWidget {
  const AccountDetailPage({
    super.key,
    required this.account,
    required this.walletName,
  });

  final Account account;
  final String walletName;

  @override
  State<AccountDetailPage> createState() => _AccountDetailPageState();
}

class _AccountDetailPageState extends State<AccountDetailPage> {
  final WalletManager _walletManager = WalletManager();

  String? _privateKey;
  bool _privateKeyVisible = false;
  bool _screenshotGuardActive = false;

  /// 本页是否改动过账户集合（增 / 删）；pop 时回传，供上一页刷新列表。
  bool _changed = false;

  @override
  void dispose() {
    if (_screenshotGuardActive) ScreenshotGuard.disable();
    super.dispose();
  }

  Future<void> _revealPrivateKey() async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('查看私钥'),
        content: const Text(
          '私钥泄露将导致该账户资产被盗（仅该账户，不影响本钱包其他账户）。\n\n确认要查看吗？',
        ),
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
    if (confirmed != true || !mounted) return;
    try {
      final key =
          await _walletManager.getAccountPrivateKey(widget.account.accountId);
      if (!mounted) return;
      if (!_screenshotGuardActive) {
        _screenshotGuardActive = true;
        await ScreenshotGuard.enable();
        if (!mounted) return;
      }
      setState(() {
        _privateKey = key;
        _privateKeyVisible = true;
      });
    } on WalletAuthException catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('验证失败：${e.message}')),
      );
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('验证失败：$e')),
      );
    }
  }

  void _hidePrivateKey() {
    setState(() {
      _privateKeyVisible = false;
      _privateKey = null;
    });
  }

  Future<void> _addAccount() async {
    final added = await showAddAccountSheet(
      context,
      masterId: widget.account.masterId,
    );
    if (added == true && mounted) {
      setState(() => _changed = true);
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('已追加账户')),
      );
    }
  }

  /// 删除：账户0 是钱包锚点 → 删除整只热钱包（连带全部账户）；其余账户 → 单删。
  Future<void> _delete() async {
    final isAnchor = widget.account.accountIndex == 0;
    final title = isAnchor ? '删除钱包' : '删除该账户';
    final message = isAnchor
        ? '账户0 是「${widget.walletName}」的锚点，删除将移除该钱包下的全部账户，且无法撤销。'
            '请确认已备份助记词（可用它重新恢复）。'
        : '确定删除「${widget.account.accountName}」？该账户可用钱包助记词重新派生找回。';
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(title),
        content: Text(message),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: const Text('取消'),
          ),
          TextButton(
            onPressed: () => Navigator.of(context).pop(true),
            style: TextButton.styleFrom(foregroundColor: AppTheme.danger),
            child: const Text('删除'),
          ),
        ],
      ),
    );
    if (confirmed != true || !mounted) return;
    try {
      if (isAnchor) {
        await _deleteWholeWallet();
      } else {
        await _walletManager.deleteAccount(widget.account.accountId);
      }
      if (!mounted) return;
      Navigator.of(context).pop(true);
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('删除失败：$e')),
      );
    }
  }

  /// 账户0 无 walletIndex 入参：按 masterId 反查这只热钱包再整只删除。
  Future<void> _deleteWholeWallet() async {
    final wallets = await _walletManager.getWallets();
    final hot = wallets.where(
      (wallet) =>
          wallet.isHotWallet && wallet.accountId == widget.account.masterId,
    );
    if (hot.isEmpty) {
      throw Exception('未找到对应钱包');
    }
    await _walletManager.deleteWallet(hot.first.walletIndex);
  }

  void _copy(String text, String label) {
    Clipboard.setData(ClipboardData(text: text));
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text('$label已复制'), duration: const Duration(seconds: 1)),
    );
  }

  @override
  Widget build(BuildContext context) {
    final account = widget.account;
    return PopScope(
      canPop: false,
      onPopInvokedWithResult: (didPop, _) {
        if (!didPop) Navigator.of(context).pop(_changed);
      },
      child: Scaffold(
        appBar: AppBar(
          title: const Text('账户详情'),
          centerTitle: true,
          actions: [
            IconButton(
              tooltip: '添加账户',
              onPressed: _addAccount,
              icon: const Icon(Icons.add),
            ),
          ],
        ),
        body: ListView(
          padding: const EdgeInsets.all(16),
          children: [
            _buildHeader(),
            const SizedBox(height: 20),
            Container(
              decoration: AppTheme.cardDecoration(radius: AppTheme.radiusLg),
              child: Column(
                children: [
                  _buildSs58Tile(),
                  const Divider(height: 1, indent: 16, endIndent: 16),
                  _buildPrivateKeyTile(),
                ],
              ),
            ),
            const SizedBox(height: 24),
            OutlinedButton.icon(
              onPressed: _delete,
              style: OutlinedButton.styleFrom(
                foregroundColor: AppTheme.danger,
                side: const BorderSide(color: AppTheme.danger),
              ),
              icon: Icon(
                account.accountIndex == 0
                    ? Icons.delete_forever_outlined
                    : Icons.delete_outline,
                size: 18,
              ),
              label: Text(account.accountIndex == 0 ? '删除钱包' : '删除该账户'),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildHeader() {
    final account = widget.account;
    return Container(
      padding: const EdgeInsets.all(20),
      decoration: BoxDecoration(
        gradient: AppTheme.primaryGradient,
        borderRadius: BorderRadius.circular(AppTheme.radiusLg),
      ),
      child: Row(
        children: [
          Container(
            width: 44,
            height: 44,
            alignment: Alignment.center,
            decoration: BoxDecoration(
              color: Colors.white.withAlpha(38),
              borderRadius: BorderRadius.circular(AppTheme.radiusSm),
            ),
            child: Text(
              '#${account.accountIndex}',
              style: const TextStyle(
                fontSize: 14,
                fontWeight: FontWeight.w700,
                color: Colors.white,
              ),
            ),
          ),
          const SizedBox(width: 14),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  account.accountName,
                  style: const TextStyle(
                    fontSize: 18,
                    fontWeight: FontWeight.w700,
                    color: Colors.white,
                  ),
                  overflow: TextOverflow.ellipsis,
                ),
                const SizedBox(height: 4),
                Text(
                  '${widget.walletName} · ${account.derivationPath}',
                  style: TextStyle(
                    fontSize: 12,
                    color: Colors.white.withAlpha(200),
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildSs58Tile() {
    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'SS58 地址',
            style: TextStyle(
              fontSize: 12,
              color: AppTheme.textSecondary,
              fontWeight: FontWeight.w500,
            ),
          ),
          const SizedBox(height: 8),
          Row(
            children: [
              Expanded(
                child: SelectableText(
                  widget.account.ss58Address,
                  style: const TextStyle(
                    fontSize: 13,
                    fontFamily: 'monospace',
                    color: AppTheme.textPrimary,
                    letterSpacing: 0.3,
                  ),
                ),
              ),
              IconButton(
                icon: const Icon(Icons.copy_rounded,
                    size: 16, color: AppTheme.primaryLight),
                tooltip: '复制',
                onPressed: () =>
                    _copy(widget.account.ss58Address, 'SS58 地址'),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildPrivateKeyTile() {
    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            '私钥',
            style: TextStyle(
              fontSize: 12,
              color: AppTheme.textSecondary,
              fontWeight: FontWeight.w500,
            ),
          ),
          const SizedBox(height: 8),
          if (!_privateKeyVisible)
            Material(
              color: Colors.transparent,
              child: InkWell(
                onTap: _revealPrivateKey,
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
                      Text(
                        '点击查看私钥',
                        style: TextStyle(
                            color: AppTheme.textTertiary, fontSize: 13),
                      ),
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
                _privateKey ?? '无数据',
                style: const TextStyle(
                  fontSize: 13,
                  fontFamily: 'monospace',
                  color: AppTheme.textPrimary,
                  height: 1.6,
                ),
              ),
            ),
            const SizedBox(height: 10),
            Row(
              children: [
                const Expanded(
                  child: Text(
                    '请手抄备份，不支持复制；导出即等于该账户控制权',
                    style: TextStyle(
                      color: AppTheme.danger,
                      fontSize: 12,
                      fontWeight: FontWeight.w500,
                    ),
                  ),
                ),
                TextButton.icon(
                  onPressed: _hidePrivateKey,
                  icon: const Icon(Icons.visibility_off_rounded, size: 16),
                  label: const Text('隐藏'),
                ),
              ],
            ),
          ],
        ],
      ),
    );
  }
}
