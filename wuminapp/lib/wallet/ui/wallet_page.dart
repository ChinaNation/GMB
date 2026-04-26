import 'package:flutter/material.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';
import 'package:wuminapp_mobile/rpc/smoldot_client.dart';
import 'package:wuminapp_mobile/trade/local_tx_store.dart';
import 'package:wuminapp_mobile/Isar/wallet_isar.dart';
import 'package:wuminapp_mobile/user/user_service.dart' show UserProfileService;
import 'package:wuminapp_mobile/ui/widgets/bip39_input.dart';
import 'package:wuminapp_mobile/ui/widgets/shimmer_loading.dart';
import 'package:wuminapp_mobile/util/amount_format.dart';
import 'package:wuminapp_mobile/util/screenshot_guard.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/ui/widgets/chain_progress_banner.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';
import 'package:wuminapp_mobile/wallet/ui/cards/wallet_action_card.dart';
import 'package:wuminapp_mobile/wallet/ui/cards/wallet_identity_card.dart';
import 'package:wuminapp_mobile/wallet/ui/cards/wallet_onchain_balance_card.dart';
import 'package:wuminapp_mobile/wallet/ui/transaction_history_page.dart';
import 'package:wuminapp_mobile/rpc/chain_tx_monitor.dart';
// 清算行设置占位页(替代原「扫码支付(清算行)」统一入口)。
import 'package:wuminapp_mobile/trade/offchain/clearing_bank_settings_page.dart';

class MyWalletPage extends StatefulWidget {
  const MyWalletPage({
    super.key,
    this.selectForTrade = false,
    this.selectForBind = false,
  });

  final bool selectForTrade;
  final bool selectForBind;

  @override
  State<MyWalletPage> createState() => _MyWalletPageState();
}

/// v6 改版（2026-04-24）：
/// - 卡片样式抄 wumin/lib/ui/home_page.dart 的钱包卡片，单列横向布局；
/// - 无 active 视觉，点击卡片直接进详情(已删「当前」小标签)；
/// - 钱包图标按冷热配色：热=墨绿主色 / 冷=蓝(离线签名设备调性)；
/// - 三点菜单只保留「重命名 / 删除钱包」2 项，去掉 wumin 端的「钱包详情」；
/// - 列表布局从 GridView 2 列改为 ReorderableListView 单列，长按拖拽排序；
/// - 删除入口从 Dismissible 滑动改为三点菜单「删除钱包」+ 二次确认。
class _MyWalletPageState extends State<MyWalletPage> {
  final WalletManager _walletService = WalletManager();
  final ChainRpc _chainRpc = ChainRpc();

  /// 中文注释：拖拽要求同步可控的列表，FutureBuilder 异步流不便参与重排，
  /// 因此把钱包列表常驻在 state 上，加载完成后再 setState 触发渲染。
  List<WalletProfile>? _wallets;
  bool _walletsLoading = true;
  bool _balanceRefreshing = false;

  bool get _isSelectionMode => widget.selectForTrade || widget.selectForBind;

  @override
  void initState() {
    super.initState();
    _loadWallets();
    if (!_isSelectionMode) {
      _refreshBalancesFromChain();
    }
  }

  Future<void> _loadWallets() async {
    final list = await _walletService.getWallets();
    if (!mounted) return;
    setState(() {
      _wallets = list;
      _walletsLoading = false;
    });
  }

  void _reload() {
    _loadWallets();
    _refreshBalancesFromChain();
  }

  Future<void> _refreshBalancesFromChain() async {
    if (_balanceRefreshing) return;
    setState(() {
      _balanceRefreshing = true;
    });
    Object? refreshError;
    try {
      // 诊断：打印轻节点状态，帮助定位链路问题
      await SmoldotClientManager.instance.printDiagnostics();

      final wallets = await _walletService.getWallets();
      bool updated = false;
      bool hasError = false;

      if (wallets.isEmpty) {
        // 无钱包，跳过
      } else {
        try {
          // 批量查询所有钱包余额（一次网络请求）
          final pubkeys = wallets.map((w) => w.pubkeyHex).toList();
          final balances = await _chainRpc.fetchBalances(pubkeys);
          for (final wallet in wallets) {
            final balance = balances[wallet.pubkeyHex] ?? 0.0;
            if (balance != wallet.balance) {
              await _walletService.setWalletBalance(
                  wallet.walletIndex, balance);
              updated = true;
            }
          }
        } catch (e) {
          debugPrint('wallet batch balance refresh failed: $e');
          hasError = true;
          refreshError = e;
        }
      }
      if (!mounted) return;
      if (updated) {
        await _loadWallets();
        if (!mounted) return;
      }
      if (hasError) {
        final msg =
            SmoldotClientManager.instance.buildUserFacingError(refreshError);
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(msg)),
        );
      }
    } finally {
      if (mounted) {
        setState(() {
          _balanceRefreshing = false;
        });
      }
    }
  }

  /// 删除钱包：二次确认 + 调用 WalletManager.deleteWallet + 重新加载列表。
  Future<void> _deleteWallet(WalletProfile wallet) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('删除钱包'),
        content: Text('确认删除「${wallet.walletName}」？此操作无法撤销。'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: const Text('取消'),
          ),
          FilledButton(
            style: FilledButton.styleFrom(backgroundColor: AppTheme.danger),
            onPressed: () => Navigator.of(context).pop(true),
            child: const Text('删除'),
          ),
        ],
      ),
    );
    if (confirmed != true || !mounted) return;
    try {
      await _walletService.deleteWallet(wallet.walletIndex);
      if (!mounted) return;
      _reload();
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('已删除「${wallet.walletName}」')),
      );
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context)
          .showSnackBar(SnackBar(content: Text('删除失败：$e')));
    }
  }

  /// 重命名钱包：弹 AlertDialog + TextField，确认后落盘并重新加载列表。
  Future<void> _renameWallet(WalletProfile wallet) async {
    final controller = TextEditingController(text: wallet.walletName);
    final newName = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('重命名钱包'),
        content: TextField(
          controller: controller,
          autofocus: true,
          maxLength: 30,
          decoration: const InputDecoration(
            hintText: '输入新的钱包名称',
            border: OutlineInputBorder(),
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('取消'),
          ),
          FilledButton(
            onPressed: () => Navigator.of(context).pop(controller.text.trim()),
            child: const Text('保存'),
          ),
        ],
      ),
    );
    if (newName == null || newName.isEmpty || newName == wallet.walletName) {
      return;
    }
    try {
      await _walletService.renameWallet(wallet.walletIndex, newName);
      // 双向同步：如果该钱包是通信账户，同步更新用户资料中的昵称。
      final profileService = UserProfileService();
      final profileState = await profileService.getState();
      if (profileState.communicationWalletIndex == wallet.walletIndex) {
        await profileService.updateCommunicationWalletName(newName);
      }
      if (!mounted) return;
      await _loadWallets();
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context)
          .showSnackBar(SnackBar(content: Text('重命名失败：$e')));
    }
  }

  /// 拖拽排序回调：先 setState 反馈到 UI，再异步落盘 sortOrder。
  /// 落盘失败只提示，不回滚 UI，避免视觉跳动。
  Future<void> _onReorder(int oldIdx, int newIdx) async {
    final wallets = _wallets;
    if (wallets == null) return;
    if (newIdx > oldIdx) newIdx -= 1;
    setState(() {
      final w = wallets.removeAt(oldIdx);
      wallets.insert(newIdx, w);
    });
    try {
      await _walletService.reorderWallets(
        wallets.map((w) => w.walletIndex).toList(),
      );
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('保存顺序失败:$e')),
      );
    }
  }

  Future<void> _openCreatePage() async {
    final created = await Navigator.of(context).push<bool>(
      MaterialPageRoute(builder: (_) => const CreateWalletPage()),
    );
    if (created == true) {
      _reload();
    }
  }

  Future<void> _openImportPage() async {
    final imported = await Navigator.of(context).push<bool>(
      MaterialPageRoute(builder: (_) => const ImportWalletPage()),
    );
    if (imported == true) {
      _reload();
    }
  }

  Future<void> _openImportColdWalletPage() async {
    final imported = await Navigator.of(context).push<bool>(
      MaterialPageRoute(builder: (_) => const ImportColdWalletPage()),
    );
    if (imported == true) {
      _reload();
    }
  }

  Future<bool?> _confirmBindWallet(WalletProfile wallet) {
    return showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('设置账户'),
        content: Text(
          '确定使用「${wallet.walletName}」作为通信账户吗？',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: const Text('取消'),
          ),
          FilledButton(
            onPressed: () => Navigator.of(context).pop(true),
            child: const Text('确认'),
          ),
        ],
      ),
    );
  }

  Future<void> _openWalletDetail(WalletProfile wallet) async {
    if (widget.selectForTrade) {
      await _walletService.setActiveWallet(wallet.walletIndex);
      if (!mounted) {
        return;
      }
      Navigator.of(context).pop(true);
      return;
    }
    if (widget.selectForBind) {
      final confirmed = await _confirmBindWallet(wallet);
      if (!mounted || confirmed != true) {
        return;
      }
      if (!mounted) {
        return;
      }
      Navigator.of(context).pop(wallet);
      return;
    }
    final changed = await Navigator.of(context).push<bool>(
      MaterialPageRoute(builder: (_) => WalletDetailPage(wallet: wallet)),
    );
    if (changed == true) {
      _reload();
    }
  }

  Future<void> _showWalletEntryChooser() async {
    await showModalBottomSheet<void>(
      context: context,
      builder: (context) {
        return SafeArea(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              ListTile(
                leading: const Icon(Icons.add_circle_outline),
                title: const Text('创建热钱包'),
                subtitle: const Text('私钥存在本机'),
                onTap: () {
                  Navigator.of(context).pop();
                  _openCreatePage();
                },
              ),
              ListTile(
                leading: const Icon(Icons.file_download_outlined),
                title: const Text('导入热钱包'),
                subtitle: const Text('通过助记词导入'),
                onTap: () {
                  Navigator.of(context).pop();
                  _openImportPage();
                },
              ),
              ListTile(
                leading: const Icon(Icons.shield_outlined),
                title: const Text('导入冷钱包'),
                subtitle: const Text('仅导入公钥，私钥保留在签名设备'),
                onTap: () {
                  Navigator.of(context).pop();
                  _openImportColdWalletPage();
                },
              ),
            ],
          ),
        );
      },
    );
  }

  Widget _buildWalletEntryOption({
    required Color color,
    required String title,
    required String description,
    required VoidCallback onTap,
  }) {
    return Material(
      color: Colors.transparent,
      child: InkWell(
        borderRadius: BorderRadius.circular(18),
        onTap: onTap,
        child: Ink(
          decoration: BoxDecoration(
            color: color,
            borderRadius: BorderRadius.circular(18),
          ),
          child: Padding(
            padding: const EdgeInsets.fromLTRB(14, 14, 14, 14),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Center(
                  child: Text(
                    title,
                    textAlign: TextAlign.center,
                    style: const TextStyle(
                      fontSize: 15,
                      fontWeight: FontWeight.w700,
                    ),
                  ),
                ),
                const Spacer(),
                Text(
                  description,
                  style: const TextStyle(
                    fontSize: 12,
                    height: 1.45,
                    color: AppTheme.textSecondary,
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildEmptyWalletChoices() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text(
          '还没有钱包，请选择一种方式开始。',
          style: TextStyle(
            fontSize: 16,
            fontWeight: FontWeight.w600,
          ),
        ),
        const SizedBox(height: 16),
        GridView.count(
          shrinkWrap: true,
          physics: const NeverScrollableScrollPhysics(),
          crossAxisCount: 2,
          mainAxisSpacing: 12,
          crossAxisSpacing: 12,
          childAspectRatio: 1.08,
          children: [
            _buildWalletEntryOption(
              color: AppTheme.danger.withAlpha(15),
              title: '创建热钱包',
              description: '创建私钥存在本机的热钱包',
              onTap: _openCreatePage,
            ),
            _buildWalletEntryOption(
              color: AppTheme.info.withAlpha(15),
              title: '导入热钱包',
              description: '导入钱包并将私钥存在本机',
              onTap: _openImportPage,
            ),
            _buildWalletEntryOption(
              color: AppTheme.warning.withAlpha(15),
              title: '导入冷钱包',
              description: '仅导入公钥，签名在外部设备',
              onTap: _openImportColdWalletPage,
            ),
          ],
        ),
      ],
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(
          widget.selectForTrade
              ? '选择交易钱包'
              : (widget.selectForBind ? '选择绑定钱包' : '我的钱包'),
        ),
        centerTitle: true,
        actions: [
          if (!_isSelectionMode)
            IconButton(
              tooltip: '创建/导入钱包',
              onPressed: _showWalletEntryChooser,
              icon: const Icon(Icons.add, size: 26),
            ),
        ],
      ),
      body: Builder(
        builder: (context) {
          if (_walletsLoading) {
            return Padding(
              padding: const EdgeInsets.all(16),
              child: ListSkeleton(
                itemCount: 3,
                itemBuilder: (_, __) => const WalletCardSkeleton(),
              ),
            );
          }
          final wallets = _wallets ?? const <WalletProfile>[];
          if (wallets.isEmpty) {
            return Padding(
              padding: const EdgeInsets.all(16),
              child: _buildEmptyWalletChoices(),
            );
          }
          return Column(
            children: [
              if (!_isSelectionMode)
                Padding(
                  padding: const EdgeInsets.fromLTRB(16, 8, 16, 0),
                  child: ChainProgressBanner(busy: _balanceRefreshing),
                ),
              Expanded(
                child: RefreshIndicator(
                  onRefresh: _refreshBalancesFromChain,
                  child: ReorderableListView.builder(
                    padding: const EdgeInsets.fromLTRB(16, 8, 16, 16),
                    itemCount: wallets.length,
                    onReorder: _onReorder,
                    buildDefaultDragHandles: !_isSelectionMode,
                    itemBuilder: (ctx, idx) {
                      final wallet = wallets[idx];
                      return Padding(
                        key: ValueKey('wallet_${wallet.walletIndex}'),
                        padding: const EdgeInsets.only(bottom: 8),
                        child: WalletListTile(
                          wallet: wallet,
                          showActions: !_isSelectionMode,
                          onTap: () => _openWalletDetail(wallet),
                          onRename: () => _renameWallet(wallet),
                          onDelete: () => _deleteWallet(wallet),
                        ),
                      );
                    },
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

/// 单行钱包卡片（横向布局）：
/// 左侧 46×46 钱包图标(冷热分色) → 中间钱包名 + 千分位余额 →
/// 右侧三点菜单（重命名 / 删除）。
///
/// 整卡 InkWell 点击进入钱包详情；菜单按钮内嵌 PopupMenuButton，
/// Flutter 默认会拦截 tap 事件，不会冒泡触发整卡 onTap。
///
/// 中文注释:仅供 wallet_page 自己使用,通过 `@visibleForTesting`
/// 暴露给 widget 测试。其他模块禁止直接引用。
@visibleForTesting
class WalletListTile extends StatelessWidget {
  const WalletListTile({
    super.key,
    required this.wallet,
    required this.showActions,
    required this.onTap,
    required this.onRename,
    required this.onDelete,
  });

  final WalletProfile wallet;

  /// 选择模式下隐藏右侧菜单（避免误操作）。
  final bool showActions;
  final VoidCallback onTap;
  final VoidCallback onRename;
  final VoidCallback onDelete;

  @override
  Widget build(BuildContext context) {
    // 中文注释:钱包图标按冷热区分配色 —— 热=墨绿主色(链上主用),冷=蓝(离线签名设备调性)。
    final isHot = wallet.isHotWallet;
    final iconBg = isHot ? AppTheme.primary.withAlpha(20) : AppTheme.info.withAlpha(20);
    final iconColor = isHot ? AppTheme.primaryDark : AppTheme.info;
    return Material(
      color: Colors.transparent,
      child: InkWell(
        borderRadius: BorderRadius.circular(AppTheme.radiusMd),
        onTap: onTap,
        child: Container(
          padding: const EdgeInsets.all(16),
          decoration: AppTheme.cardDecoration(radius: AppTheme.radiusMd),
          child: Row(
            children: [
              // 左：46×46 钱包图标（按冷热分色）
              Container(
                width: 46,
                height: 46,
                decoration: BoxDecoration(
                  color: iconBg,
                  borderRadius: BorderRadius.circular(AppTheme.radiusSm),
                ),
                child: Icon(
                  Icons.account_balance_wallet_rounded,
                  color: iconColor,
                  size: 24,
                ),
              ),
              const SizedBox(width: 12),
              // 中：钱包名 + 千分位余额
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Text(
                      wallet.walletName,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: const TextStyle(
                        fontSize: 18,
                        fontWeight: FontWeight.w600,
                        color: AppTheme.primaryDark,
                      ),
                    ),
                    const SizedBox(height: 4),
                    Text(
                      AmountFormat.formatThousands(wallet.balance),
                      style: const TextStyle(
                        fontSize: 13,
                        color: AppTheme.textSecondary,
                      ),
                    ),
                  ],
                ),
              ),
              // 右：三点菜单（仅非选择模式）
              if (showActions) ...[
                const SizedBox(width: 4),
                PopupMenuButton<String>(
                  icon: const Icon(
                    Icons.more_vert,
                    color: AppTheme.textTertiary,
                    size: 20,
                  ),
                  onSelected: (v) {
                    switch (v) {
                      case 'rename':
                        onRename();
                      case 'delete':
                        onDelete();
                    }
                  },
                  itemBuilder: (_) => const [
                    PopupMenuItem(value: 'rename', child: Text('重命名')),
                    PopupMenuItem(
                      value: 'delete',
                      child: Text(
                        '删除钱包',
                        style: TextStyle(color: AppTheme.danger),
                      ),
                    ),
                  ],
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }
}

class WalletDetailPage extends StatefulWidget {
  const WalletDetailPage({super.key, required this.wallet});

  final WalletProfile wallet;

  @override
  State<WalletDetailPage> createState() => _WalletDetailPageState();
}

class _WalletDetailPageState extends State<WalletDetailPage> {
  final WalletManager _walletService = WalletManager();

  /// 本页是否有修改落盘过(用于 pop 时回传给上一页刷新列表)。
  bool _hasChanged = false;
  List<LocalTxEntity> _recentRecords = const [];
  bool _screenshotGuardActive = false;

  /// 中文注释:外层下拉刷新通过此 key 触发链上余额卡的 refresh()。
  final GlobalKey<WalletOnchainBalanceCardState> _balanceCardKey =
      GlobalKey<WalletOnchainBalanceCardState>();

  /// 整页下拉刷新:
  /// - 链上余额卡:通过 GlobalKey 调 refresh()
  /// - 交易记录:复用 _loadRecentRecords()
  /// - 清算行余额(动作卡"余额"列):本轮 0.00 元 写死占位,
  ///   待清算行功能落地后在此追加刷新调用。TODO(清算行)
  Future<void> _onPullRefresh() async {
    await Future.wait<void>([
      Future(() async {
        try {
          await _balanceCardKey.currentState?.refresh();
        } catch (_) {
          // 中文注释:链上余额刷新失败已在卡片内置错误态处理,这里不打断其他刷新
        }
      }),
      _loadRecentRecords(),
    ]);
  }

  @override
  void dispose() {
    ChainTxMonitor.instance.onBalanceChanged = null;
    if (_screenshotGuardActive) ScreenshotGuard.disable();
    super.dispose();
  }

  @override
  void initState() {
    super.initState();
    _loadRecentRecords();
    // 中文注释：启动链上交易监控（余额变化触发模式）。
    ChainTxMonitor.instance.watchWallet(
      widget.wallet.address,
      widget.wallet.pubkeyHex,
    );
    // 中文注释：注册余额变动回调，刷新交易记录和余额显示。
    ChainTxMonitor.instance.onBalanceChanged = (address, newBalance) {
      if (mounted && address == widget.wallet.address) {
        _loadRecentRecords();
        // 更新本地存储的余额
        _walletService.setWalletBalance(widget.wallet.walletIndex, newBalance);
      }
    };
    ChainTxMonitor.instance.start();
  }

  Future<void> _loadRecentRecords() async {
    try {
      final records = await LocalTxStore.queryRecent(
        widget.wallet.address,
        limit: 5,
      );
      if (!mounted) return;
      setState(() {
        _recentRecords = records;
      });
    } catch (_) {
      // 加载失败静默忽略，钱包详情页仍可正常使用
    }
  }

  String _localTxTypeLabel(String txType, String direction) {
    switch (txType) {
      case 'transfer':
        return direction == 'out' ? '转账支出' : '转账收入';
      case 'offchain_pay':
        return direction == 'out' ? '扫码支付' : '扫码收款';
      case 'proposal_transfer':
        return direction == 'out' ? '提案转出' : '提案转入';
      case 'fee_withdraw':
        return '手续费';
      case 'fee_deposit':
        return '手续费分成';
      case 'block_reward':
        return '出块奖励';
      case 'bank_interest':
        return '银行利息';
      case 'gov_issuance':
        return '治理增发';
      case 'lightnode_reward':
        return '认证奖励';
      case 'duoqian_create':
        return '多签出资';
      case 'duoqian_close':
        return direction == 'out' ? '多签关闭' : '多签收款';
      case 'fund_destroy':
        return '资金销毁';
      default:
        return txType;
    }
  }

  String _shortAddress(String? address) {
    if (address == null || address.length <= 12) return address ?? '-';
    return '${address.substring(0, 6)}...${address.substring(address.length - 6)}';
  }

  String _pad(int n) => n.toString().padLeft(2, '0');

  Future<void> _onMenuAction(String action) async {
    switch (action) {
      case 'clearing_bank':
        // 中文注释:跳转「设置清算行」占位页。真实搜索/绑定流程等后续任务卡。
        await Navigator.of(context).push(
          MaterialPageRoute(
            builder: (_) =>
                ClearingBankSettingsPage(wallet: widget.wallet),
          ),
        );
      case 'seed':
        await _revealSecret('私钥', () async {
          final seed =
              await _walletService.getSeedHex(widget.wallet.walletIndex);
          return seed != null ? '0x$seed' : null;
        });
      case 'mnemonic':
        await _revealSecret('助记词', () async {
          return _walletService.getMnemonic(widget.wallet.walletIndex);
        });
    }
  }

  Future<void> _revealSecret(
      String label, Future<String?> Function() fetcher) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (_) => AlertDialog(
        title: Text('查看$label'),
        content: Text('$label是核心机密信息，泄露将导致资产被盗。\n\n确认要查看吗？'),
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
      final value = await fetcher();
      if (!mounted) return;
      if (!_screenshotGuardActive) {
        _screenshotGuardActive = true;
        await ScreenshotGuard.enable();
        if (!mounted) return;
      }
      await showDialog<void>(
        context: context,
        builder: (_) => AlertDialog(
          title: Text(label),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Container(
                padding: const EdgeInsets.all(12),
                decoration: AppTheme.bannerDecoration(AppTheme.danger),
                child: Text(
                  value ?? '无数据',
                  style: const TextStyle(fontSize: 14, fontFamily: 'monospace'),
                ),
              ),
              const SizedBox(height: 8),
              const Text(
                '请手抄备份，不支持复制',
                style: TextStyle(color: AppTheme.danger, fontSize: 12),
              ),
            ],
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('关闭'),
            ),
          ],
        ),
      );
    } on WalletAuthException catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('验证失败：${e.message}')),
      );
    }
  }

  /// 钱包名持久化(纯落盘 + 通信账户昵称双向同步)。
  ///
  /// 中文注释:
  /// - 编辑态和回滚逻辑已搬到 [WalletIdentityCard],这里仅负责落盘 + 同步。
  /// - 调用方(WalletIdentityCard)传进来的 newName 已 trim,但 updateWalletDisplay
  ///   内部再 trim 一次也无副作用,保持签名稳定。
  /// - 若该钱包绑定的是当前通信账户,需要同步更新 UserProfile 里的昵称。
  /// - 出错时重新抛出,让 WalletIdentityCard 走回滚分支。
  Future<void> _saveWalletName(String newName) async {
    try {
      await _walletService.updateWalletDisplay(
        widget.wallet.walletIndex,
        walletName: newName,
        walletIcon: widget.wallet.walletIcon,
      );
      // 双向同步：如果该钱包是通信账户，同步更新用户资料中的昵称
      final profileService = UserProfileService();
      final profileState = await profileService.getState();
      if (profileState.communicationWalletIndex == widget.wallet.walletIndex) {
        await profileService.updateCommunicationWalletName(newName);
      }
      if (!mounted) return;
      setState(() {
        _hasChanged = true;
      });
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context)
            .showSnackBar(SnackBar(content: Text('$e')));
      }
      rethrow;
    }
  }

  @override
  Widget build(BuildContext context) {
    return PopScope(
      canPop: false,
      onPopInvokedWithResult: (didPop, _) {
        if (!didPop) {
          Navigator.of(context).pop(_hasChanged);
        }
      },
      child: Scaffold(
        appBar: AppBar(
          title: const Text('钱包详情'),
          centerTitle: true,
          actions: [
            PopupMenuButton<String>(
              icon: const Icon(Icons.more_vert),
              onSelected: _onMenuAction,
              itemBuilder: (_) => [
                const PopupMenuItem(
                  value: 'clearing_bank',
                  child: Text('清算行'),
                ),
                if (widget.wallet.isHotWallet) ...[
                  const PopupMenuItem(value: 'seed', child: Text('查看私钥')),
                  const PopupMenuItem(value: 'mnemonic', child: Text('查看助记词')),
                ],
              ],
            ),
          ],
        ),
        body: RefreshIndicator(
          onRefresh: _onPullRefresh,
          child: ListView(
            padding: const EdgeInsets.all(16),
            physics: const AlwaysScrollableScrollPhysics(),
            children: [
              // 第 1 张卡:钱包身份卡(钱包名 + 短地址 + QR 入口)。
              WalletIdentityCard(
                wallet: widget.wallet,
                onNameChanged: _saveWalletName,
              ),
              const SizedBox(height: 16),
              // 第 2 张卡:充值 / 提现 / 余额(3 列,余额为静态展示)。
              WalletActionCard(wallet: widget.wallet),
              const SizedBox(height: 16),
              // 第 3 张卡:链上 total 余额(free + reserved)。
              WalletOnchainBalanceCard(
                key: _balanceCardKey,
                wallet: widget.wallet,
              ),
              const SizedBox(height: 24),
              // 交易记录区块(保留原实现)。
              ..._buildTransactionHistorySection(),
            ],
          ),
        ),
      ),
    );
  }

  /// 交易记录区块:标题跳转 + 最近 5 条列表。从原 build 方法拆出,便于阅读。
  List<Widget> _buildTransactionHistorySection() {
    return [
      InkWell(
        onTap: () {
          Navigator.of(context).push(
            MaterialPageRoute(
              builder: (_) => TransactionHistoryPage(
                walletAddress: widget.wallet.address,
              ),
            ),
          );
        },
        child: const Padding(
          padding: EdgeInsets.symmetric(vertical: 8),
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
              Icon(Icons.chevron_right,
                  size: 20, color: AppTheme.textTertiary),
            ],
          ),
        ),
      ),
      const Divider(height: 1),
      if (_recentRecords.isEmpty)
        const Padding(
          padding: EdgeInsets.symmetric(vertical: 32),
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
          final isOut = record.direction == 'out';
          final label = _localTxTypeLabel(record.txType, record.direction);
          final counterparty = isOut
              ? _shortAddress(record.toAddress)
              : _shortAddress(record.fromAddress);
          final dt = DateTime.fromMillisecondsSinceEpoch(record.createdAtMillis)
              .toLocal();
          final timeStr =
              '${dt.year}-${_pad(dt.month)}-${_pad(dt.day)} ${_pad(dt.hour)}:${_pad(dt.minute)}';
          final amountColor = isOut ? AppTheme.danger : AppTheme.primaryDark;
          return Column(
            children: [
              ListTile(
                onTap: () {
                  Navigator.of(context).push(
                    MaterialPageRoute(
                      builder: (_) => TransactionHistoryPage(
                        walletAddress: widget.wallet.address,
                      ),
                    ),
                  );
                },
                leading: CircleAvatar(
                  radius: 18,
                  backgroundColor: isOut
                      ? AppTheme.danger.withAlpha(20)
                      : AppTheme.success.withAlpha(20),
                  child: Icon(
                    isOut ? Icons.arrow_upward : Icons.arrow_downward,
                    size: 18,
                    color: amountColor,
                  ),
                ),
                title: Text(
                  label,
                  style: const TextStyle(
                      fontSize: 15, fontWeight: FontWeight.w600),
                ),
                subtitle: Text(
                  '$counterparty\n$timeStr',
                  style: const TextStyle(fontSize: 12, height: 1.5),
                ),
                isThreeLine: true,
                trailing: Text(
                  '${isOut ? "-" : "+"}${AmountFormat.format(record.amountYuan, symbol: '')}',
                  style: TextStyle(
                    fontSize: 15,
                    fontWeight: FontWeight.w700,
                    color: amountColor,
                  ),
                ),
              ),
              if (index < _recentRecords.length - 1)
                const Divider(height: 1),
            ],
          );
        }),
    ];
  }
}

class WalletIconOption {
  const WalletIconOption({
    required this.key,
    required this.label,
    required this.icon,
  });

  final String key;
  final String label;
  final IconData icon;
}

class WalletIconRegistry {
  static const List<WalletIconOption> options = [
    WalletIconOption(
        key: 'wallet',
        label: '钱包',
        icon: Icons.account_balance_wallet_outlined),
    WalletIconOption(key: 'shield', label: '盾牌', icon: Icons.shield_outlined),
    WalletIconOption(key: 'star', label: '星标', icon: Icons.star_border),
    WalletIconOption(key: 'leaf', label: '树叶', icon: Icons.eco_outlined),
    WalletIconOption(key: 'key', label: '钥匙', icon: Icons.vpn_key_outlined),
    WalletIconOption(
        key: 'safe', label: '保险箱', icon: Icons.inventory_2_outlined),
  ];

  static IconData iconFor(String key) {
    for (final option in options) {
      if (option.key == key) {
        return option.icon;
      }
    }
    return Icons.account_balance_wallet_outlined;
  }
}

class CreateWalletPage extends StatefulWidget {
  const CreateWalletPage({super.key});

  @override
  State<CreateWalletPage> createState() => _CreateWalletPageState();
}

class _CreateWalletPageState extends State<CreateWalletPage> {
  bool _isSaving = false;
  int _wordCount = 12;

  Future<void> _create() async {
    setState(() {
      _isSaving = true;
    });
    try {
      final created = await WalletManager().createWallet(wordCount: _wordCount);
      if (!mounted) {
        return;
      }
      await ScreenshotGuard.enable();
      if (!mounted) return;
      await showDialog<void>(
        context: context,
        barrierDismissible: false,
        builder: (context) {
          return AlertDialog(
            title: const Text('请备份助记词'),
            content: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text(
                  '助记词已加密存储在本机，后续可在钱包详情中查看。\n'
                  '请务必手抄备份并妥善保管，这是恢复钱包的唯一凭证。\n'
                  '不支持复制，不支持截屏。',
                ),
                const SizedBox(height: 12),
                Text(
                  created.mnemonic,
                  style: const TextStyle(fontWeight: FontWeight.w600),
                ),
              ],
            ),
            actions: [
              TextButton(
                onPressed: () => Navigator.of(context).pop(),
                child: const Text('我已备份'),
              ),
            ],
          );
        },
      );
      await ScreenshotGuard.disable();
      if (!mounted) {
        return;
      }
      Navigator.of(context).pop(true);
    } finally {
      if (mounted) {
        setState(() {
          _isSaving = false;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('创建钱包')),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text('将创建一个 sr25519 钱包，并生成 SS58(2027) 地址。'),
            const SizedBox(height: 8),
            const Text('仅使用默认派生路径，不暴露自定义路径。'),
            const SizedBox(height: 16),
            SegmentedButton<int>(
              segments: const [
                ButtonSegment(value: 12, label: Text('12 个单词')),
                ButtonSegment(value: 24, label: Text('24 个单词')),
              ],
              selected: {_wordCount},
              onSelectionChanged: (v) => setState(() => _wordCount = v.first),
            ),
            const SizedBox(height: 8),
            Text(
              _wordCount == 24 ? '256 位熵，安全性更高' : '128 位熵，标准安全强度',
              style:
                  const TextStyle(color: AppTheme.textSecondary, fontSize: 12),
            ),
            const SizedBox(height: 16),
            FilledButton(
              onPressed: _isSaving ? null : _create,
              child: Text(_isSaving ? '创建中...' : '确认创建'),
            ),
          ],
        ),
      ),
    );
  }
}

class ImportWalletPage extends StatefulWidget {
  const ImportWalletPage({super.key});

  @override
  State<ImportWalletPage> createState() => _ImportWalletPageState();
}

class _ImportWalletPageState extends State<ImportWalletPage> {
  final TextEditingController _mnemonicController = TextEditingController();
  bool _isImporting = false;
  String? _error;

  Future<void> _import() async {
    setState(() {
      _error = null;
      _isImporting = true;
    });
    try {
      await WalletManager().importWallet(_mnemonicController.text);
      _mnemonicController.clear();
      if (!mounted) {
        return;
      }
      Navigator.of(context).pop(true);
    } catch (e) {
      setState(() {
        _error = '$e';
      });
    } finally {
      if (mounted) {
        setState(() {
          _isImporting = false;
        });
      }
    }
  }

  @override
  void dispose() {
    _mnemonicController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('导入热钱包')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          const Text('逐个输入单词，从候选列表中选择匹配项'),
          const SizedBox(height: 8),
          const Text('仅使用默认派生路径，不暴露自定义路径。'),
          const SizedBox(height: 12),
          Bip39InputField(controller: _mnemonicController, wordCount: 0),
          const SizedBox(height: 12),
          if (_error != null)
            Text(
              _error!,
              style: const TextStyle(color: AppTheme.danger),
            ),
          FilledButton(
            onPressed: _isImporting ? null : _import,
            child: Text(_isImporting ? '导入中...' : '确认导入'),
          ),
        ],
      ),
    );
  }
}

/// 导入冷钱包页面：仅输入 SS58 地址或公钥，不导入私钥。
class ImportColdWalletPage extends StatefulWidget {
  const ImportColdWalletPage({super.key});

  @override
  State<ImportColdWalletPage> createState() => _ImportColdWalletPageState();
}

class _ImportColdWalletPageState extends State<ImportColdWalletPage> {
  final TextEditingController _addressController = TextEditingController();
  bool _isImporting = false;
  String? _error;

  Future<void> _import() async {
    setState(() {
      _error = null;
      _isImporting = true;
    });
    try {
      await WalletManager().importColdWallet(
        address: _addressController.text,
      );
      if (!mounted) return;
      Navigator.of(context).pop(true);
    } catch (e) {
      setState(() {
        _error = '$e';
      });
    } finally {
      if (mounted) {
        setState(() {
          _isImporting = false;
        });
      }
    }
  }

  @override
  void dispose() {
    _addressController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('导入冷钱包')),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text('请输入冷钱包的 SS58 地址或公钥：'),
            const SizedBox(height: 8),
            const Text(
              '冷钱包仅保存公钥，私钥保留在 Wumin 签名设备上。\n管理员提案和投票将通过扫码签名完成。',
              style: TextStyle(color: AppTheme.textSecondary, fontSize: 13),
            ),
            const SizedBox(height: 12),
            TextField(
              controller: _addressController,
              minLines: 2,
              maxLines: 3,
              decoration: const InputDecoration(
                hintText: 'SS58 地址或 0x 开头的公钥',
                border: OutlineInputBorder(),
              ),
            ),
            const SizedBox(height: 12),
            if (_error != null)
              Text(
                _error!,
                style: const TextStyle(color: AppTheme.danger),
              ),
            FilledButton(
              onPressed: _isImporting ? null : _import,
              child: Text(_isImporting ? '导入中...' : '确认导入'),
            ),
          ],
        ),
      ),
    );
  }
}
