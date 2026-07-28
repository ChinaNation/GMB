import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:citizenapp/isar/app_isar.dart';
import 'package:citizenapp/my/util/screenshot_guard.dart';
import 'package:citizenapp/transaction/offchain-transaction/pages/clearing_bank_settings_page.dart';
import 'package:citizenapp/transaction/shared/local_tx_store.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:citizenapp/wallet/pages/transaction_history_page.dart';
import 'package:citizenapp/wallet/widgets/wallet_action_card.dart';

/// 账户详情（Lv3）：单个 `//index` 账户 = 单钱包多账户下「以前的钱包详情」。
///
/// 承载该账户的全部钱包功能，一律按 `account_id` 键控：
/// - 充值 / 提现 / 零钱包（[WalletActionCard]，链下清算行零钱包按账户独立绑定）；
/// - 清算行（[ClearingBankSettingsPage] 绑定 / 切换）；
/// - 交易记录（[TransactionHistoryPage]，按账户 `account_id` 查询）；
/// - SS58 地址与私钥（child mini-secret 独立、单向；导出单账户不牵连锚点账户0 或
///   兄弟账户，展示前生物识别 + 防截屏 + 纯文本不可复制）；
/// - 删除该账户 / 删除整只钱包（账户0 为锚点）。
///
/// 追加账户不在本页：收在「我的钱包」列表右上角「＋」的「添加下一个账户 / 添加指定账户」。
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

  /// 充值/提现/零钱包动作卡:下拉刷新时通过此 key 触发清算行余额重查。
  final GlobalKey<WalletActionCardState> _actionCardKey =
      GlobalKey<WalletActionCardState>();

  String? _privateKey;
  bool _privateKeyVisible = false;
  bool _screenshotGuardActive = false;

  /// 该账户最近交易记录(最多 5 条),按 `account_id` 查询。
  List<LocalTxEntity> _recentRecords = const [];

  @override
  void initState() {
    super.initState();
    // 初始化加载最近交易记录;链上到账由钱包 Tab 的 ChainTxMonitor 后台写入本地库,
    // 本页通过下拉刷新 / 操作后回刷重新读取(不重复启动监听、不劫持全局回调)。
    _loadRecentRecords();
  }

  @override
  void dispose() {
    if (_screenshotGuardActive) ScreenshotGuard.disable();
    super.dispose();
  }

  Future<void> _loadRecentRecords() async {
    try {
      final records = await LocalTxStore.queryRecentByAccountId(
        widget.account.accountId,
        limit: 5,
      );
      if (!mounted) return;
      setState(() => _recentRecords = records);
    } catch (_) {
      // 加载失败静默忽略,账户详情其余功能不受影响。
    }
  }

  /// 下拉刷新:清算行余额卡 + 最近交易记录。
  Future<void> _onPullRefresh() async {
    await Future.wait<void>([
      Future(() async {
        try {
          await _actionCardKey.currentState?.refresh();
        } catch (_) {
          // 清算行节点可能暂不可达,动作卡内部会展示节点不可达。
        }
      }),
      _loadRecentRecords(),
    ]);
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

  /// 进「设置清算行」页(按账户绑定 / 切换);返回后刷新动作卡的零钱包余额。
  Future<void> _openClearingBank() async {
    await Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => ClearingBankSettingsPage(
          accountId: widget.account.accountId,
          ss58Address: widget.account.ss58Address,
        ),
      ),
    );
    if (!mounted) return;
    await _actionCardKey.currentState?.refresh();
  }

  @override
  Widget build(BuildContext context) {
    final account = widget.account;
    return PopScope(
      canPop: false,
      onPopInvokedWithResult: (didPop, _) {
        // 删除账户走 _delete 内的 pop(true);普通返回不改动账户集合,回传 false。
        if (!didPop) Navigator.of(context).pop(false);
      },
      child: Scaffold(
        appBar: AppBar(
          title: const Text('账户详情'),
          centerTitle: true,
        ),
        body: RefreshIndicator(
          onRefresh: _onPullRefresh,
          child: ListView(
            padding: const EdgeInsets.all(16),
            physics: const AlwaysScrollableScrollPhysics(),
            children: [
              _buildHeader(),
              const SizedBox(height: 16),
              // 充值 / 提现 / 零钱包(按 account_id,链下清算行零钱包按账户独立绑定)。
              Container(
                clipBehavior: Clip.antiAlias,
                decoration: AppTheme.cardDecoration(radius: AppTheme.radiusLg),
                child: WalletActionCard(
                  key: _actionCardKey,
                  accountId: account.accountId,
                  ss58Address: account.ss58Address,
                ),
              ),
              const SizedBox(height: 12),
              _buildClearingBankRow(),
              const SizedBox(height: 12),
              _buildTransactionHistoryCard(),
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

  /// 清算行入口:整卡点击进「设置清算行」(绑定 / 切换)。
  Widget _buildClearingBankRow() {
    return Material(
      color: Colors.transparent,
      child: InkWell(
        borderRadius: BorderRadius.circular(AppTheme.radiusLg),
        onTap: _openClearingBank,
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
          decoration: AppTheme.cardDecoration(radius: AppTheme.radiusLg),
          child: Row(
            children: [
              Container(
                width: 40,
                height: 40,
                alignment: Alignment.center,
                decoration: BoxDecoration(
                  color: AppTheme.primary.withAlpha(20),
                  borderRadius: BorderRadius.circular(AppTheme.radiusSm),
                ),
                child: const Icon(Icons.account_balance_outlined,
                    size: 20, color: AppTheme.primaryDark),
              ),
              const SizedBox(width: 12),
              const Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Text(
                      '清算行',
                      style: TextStyle(
                        fontSize: 15,
                        fontWeight: FontWeight.w600,
                        color: AppTheme.textPrimary,
                      ),
                    ),
                    SizedBox(height: 2),
                    Text(
                      '绑定 / 切换清算行',
                      style: TextStyle(
                        fontSize: 12,
                        color: AppTheme.textTertiary,
                      ),
                    ),
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

  /// 交易记录卡片:标题跳转完整列表 + 最近 5 条。
  Widget _buildTransactionHistoryCard() {
    return Container(
      clipBehavior: Clip.antiAlias,
      decoration: AppTheme.cardDecoration(radius: AppTheme.radiusLg),
      child: Column(children: _buildTransactionHistorySection()),
    );
  }

  List<Widget> _buildTransactionHistorySection() {
    return [
      InkWell(
        onTap: () {
          Navigator.of(context).push(
            MaterialPageRoute(
              builder: (_) => TransactionHistoryPage(
                ss58Address: widget.account.ss58Address,
                accountId: widget.account.accountId,
              ),
            ),
          );
        },
        child: const Padding(
          padding: EdgeInsets.fromLTRB(16, 14, 12, 14),
          child: Row(
            children: [
              Text(
                '交易记录',
                style: TextStyle(
                  fontSize: 16,
                  fontWeight: FontWeight.w700,
                ),
              ),
              Spacer(),
              Icon(Icons.chevron_right, size: 20, color: AppTheme.textTertiary),
            ],
          ),
        ),
      ),
      const Divider(height: 1),
      if (_recentRecords.isEmpty)
        const Padding(
          padding: EdgeInsets.symmetric(vertical: 36),
          child: Center(
            child: Text(
              '暂无交易记录',
              style: TextStyle(color: AppTheme.textTertiary),
            ),
          ),
        )
      else
        ...List.generate(_recentRecords.length, (index) {
          final record = _recentRecords[index];
          return Column(
            children: [
              LocalTxRecordTile(
                record: record,
                showChevron: true,
                onTap: () {
                  Navigator.of(context).push(
                    MaterialPageRoute(
                      builder: (_) => LocalTxRecordDetailPage(record: record),
                    ),
                  );
                },
              ),
              if (index < _recentRecords.length - 1) const Divider(height: 1),
            ],
          );
        }),
    ];
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
