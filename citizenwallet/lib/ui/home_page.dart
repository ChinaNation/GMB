import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';

import '../util/screenshot_guard.dart';
import '../wallet/wallet_manager.dart';
import 'app_theme.dart';
import 'create_wallet_page.dart';
import 'import_wallet_page.dart';
import 'scan_page.dart';
import 'settings_page.dart';
import 'wallet_detail_page.dart';

/// Lv1 钱包列表首页：只显示钱包名。点钱包进 Lv2 详情；扫码入口在每张钱包卡片(只扫本钱包)。
class HomePage extends StatefulWidget {
  const HomePage({super.key});

  @override
  State<HomePage> createState() => _HomePageState();
}

class _HomePageState extends State<HomePage> {
  final WalletManager _walletManager = WalletManager();
  List<Wallet> _wallets = [];
  bool _loading = true;
  bool _isRooted = false;

  @override
  void initState() {
    super.initState();
    _loadAll(showLoading: true);
    _checkRootStatus();
  }

  Future<void> _checkRootStatus() async {
    final rooted = await ScreenshotGuard.isDeviceRooted();
    if (!mounted) return;
    setState(() => _isRooted = rooted);
  }

  Future<void> _loadAll({bool showLoading = false}) async {
    if (showLoading) setState(() => _loading = true);
    try {
      final wallets = await _walletManager.getWallets();
      if (!mounted) return;
      setState(() {
        _wallets = wallets;
        _loading = false;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() => _loading = false);
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('加载钱包失败：$e')),
      );
    }
  }

  Future<void> _loadWallets() => _loadAll();

  Future<void> _openCreateWallet() async {
    final created = await Navigator.of(context).push<bool>(
      MaterialPageRoute(builder: (_) => const CreateWalletPage()),
    );
    if (created == true) {
      await _loadWallets();
    }
  }

  Future<void> _openImportWallet() async {
    final imported = await Navigator.of(context).push<bool>(
      MaterialPageRoute(builder: (_) => const ImportWalletPage()),
    );
    if (imported == true) {
      await _loadWallets();
    }
  }

  /// 本钱包扫码签名：只匹配该钱包名下账户,跨钱包签名请求拒绝。
  Future<void> _openWalletScan(Wallet wallet) async {
    await Navigator.of(context).push(
      MaterialPageRoute(builder: (_) => ScanPage(wallet: wallet)),
    );
    await _loadWallets();
  }

  void _showAddWalletMenu() {
    showModalBottomSheet(
      context: context,
      builder: (context) => SafeArea(
        child: Padding(
          padding: const EdgeInsets.fromLTRB(16, 8, 16, 16),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Container(
                width: 36,
                height: 4,
                margin: const EdgeInsets.only(bottom: 16),
                decoration: BoxDecoration(
                  color: AppTheme.textTertiary,
                  borderRadius: BorderRadius.circular(2),
                ),
              ),
              _buildBottomSheetItem(
                icon: Icons.add_circle_outline,
                label: '创建钱包',
                subtitle: '生成新的助记词和账户0',
                onTap: () {
                  Navigator.pop(context);
                  _openCreateWallet();
                },
              ),
              const SizedBox(height: 8),
              _buildBottomSheetItem(
                icon: Icons.download_rounded,
                label: '导入钱包',
                subtitle: '通过助记词恢复已有钱包',
                onTap: () {
                  Navigator.pop(context);
                  _openImportWallet();
                },
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildBottomSheetItem({
    required IconData icon,
    required String label,
    required String subtitle,
    required VoidCallback onTap,
  }) {
    return Material(
      color: Colors.transparent,
      child: InkWell(
        borderRadius: BorderRadius.circular(AppTheme.radiusMd),
        onTap: onTap,
        child: Container(
          padding: const EdgeInsets.all(16),
          decoration: AppTheme.cardDecoration(),
          child: Row(
            children: [
              Container(
                width: 44,
                height: 44,
                decoration: BoxDecoration(
                  color: AppTheme.primary.withAlpha(25),
                  borderRadius: BorderRadius.circular(AppTheme.radiusSm),
                ),
                child: Icon(icon, color: AppTheme.primaryLight, size: 22),
              ),
              const SizedBox(width: 14),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(label,
                        style: const TextStyle(
                            fontSize: 15,
                            fontWeight: FontWeight.w600,
                            color: AppTheme.textPrimary)),
                    const SizedBox(height: 2),
                    Text(subtitle,
                        style: const TextStyle(
                            fontSize: 12, color: AppTheme.textSecondary)),
                  ],
                ),
              ),
              const Icon(Icons.chevron_right,
                  color: AppTheme.textTertiary, size: 20),
            ],
          ),
        ),
      ),
    );
  }

  Future<void> _openWalletDetail(Wallet wallet) async {
    await Navigator.of(context).push(
      MaterialPageRoute(builder: (_) => WalletDetailPage(wallet: wallet)),
    );
    await _loadWallets();
  }

  Future<void> _confirmDelete(Wallet wallet) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('删除钱包'),
        content: Text('确定删除「${wallet.walletName}」？\n'
            '删除后该钱包全部账户与私钥将被清除，请确保已备份助记词。'),
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
    if (confirmed != true) return;
    try {
      await _walletManager.deleteWallet(wallet.masterId);
      await _loadWallets();
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('删除失败：$e')),
      );
    }
  }

  Future<void> _renameWallet(Wallet wallet) async {
    final controller = TextEditingController(text: wallet.walletName);
    final newName = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('重命名钱包'),
        content: TextField(
          controller: controller,
          autofocus: true,
          maxLength: WalletManager.maxWalletNameLength,
          decoration: const InputDecoration(hintText: '请输入新名称', counterText: ''),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('取消'),
          ),
          TextButton(
            onPressed: () => Navigator.of(context).pop(controller.text),
            child: const Text('确定'),
          ),
        ],
      ),
    );
    if (newName == null || newName.trim().isEmpty) return;
    try {
      await _walletManager.renameWallet(wallet.masterId, newName);
      await _loadWallets();
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('重命名失败：$e')),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    final hasWallets = _wallets.isNotEmpty;
    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: const Icon(Icons.settings_outlined, size: 22),
          tooltip: '设置',
          onPressed: () => Navigator.of(context).push(
            MaterialPageRoute(builder: (_) => const SettingsPage()),
          ),
        ),
        title: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Container(
              width: 28,
              height: 28,
              decoration: BoxDecoration(
                gradient: AppTheme.primaryGradient,
                borderRadius: BorderRadius.circular(7),
              ),
              child: const Icon(Icons.shield_outlined,
                  color: Colors.white, size: 16),
            ),
            const SizedBox(width: 8),
            const Text('公民钱包'),
          ],
        ),
        centerTitle: true,
        actions: [
          if (hasWallets)
            IconButton(
              icon: Container(
                width: 32,
                height: 32,
                decoration: BoxDecoration(
                  color: AppTheme.primary.withAlpha(25),
                  borderRadius: BorderRadius.circular(8),
                ),
                child: const Icon(Icons.add,
                    size: 20, color: AppTheme.primaryLight),
              ),
              tooltip: '添加钱包',
              onPressed: _showAddWalletMenu,
            ),
        ],
      ),
      body: _loading
          ? const Center(
              child: CircularProgressIndicator(
                  color: AppTheme.primary, strokeWidth: 2.5))
          : Column(
              children: [
                if (_isRooted)
                  Container(
                    width: double.infinity,
                    margin: const EdgeInsets.fromLTRB(16, 4, 16, 0),
                    padding: const EdgeInsets.symmetric(
                        horizontal: 12, vertical: 10),
                    decoration: AppTheme.bannerDecoration(AppTheme.danger),
                    child: const Row(
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
                                fontWeight: FontWeight.w500),
                          ),
                        ),
                      ],
                    ),
                  ),
                Expanded(
                  child: hasWallets ? _buildWalletList() : _buildEmptyState(),
                ),
              ],
            ),
    );
  }

  Widget _buildEmptyState() {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Container(
              width: 88,
              height: 88,
              decoration: BoxDecoration(
                color: AppTheme.surfaceCard,
                borderRadius: BorderRadius.circular(24),
                border: Border.all(color: AppTheme.border),
              ),
              child: const Icon(Icons.account_balance_wallet_outlined,
                  size: 40, color: AppTheme.textTertiary),
            ),
            const SizedBox(height: 24),
            const Text('还没有钱包',
                style: TextStyle(
                    fontSize: 20,
                    fontWeight: FontWeight.w700,
                    color: AppTheme.textPrimary)),
            const SizedBox(height: 8),
            const Text('创建或导入一个钱包来开始使用',
                style: TextStyle(color: AppTheme.textSecondary, fontSize: 14)),
            const SizedBox(height: 36),
            SizedBox(
              width: 220,
              child: FilledButton.icon(
                onPressed: _openCreateWallet,
                icon: const Icon(Icons.add, size: 20),
                label: const Text('创建钱包'),
              ),
            ),
            const SizedBox(height: 12),
            SizedBox(
              width: 220,
              child: OutlinedButton.icon(
                onPressed: _openImportWallet,
                icon: const Icon(Icons.download_rounded, size: 20),
                label: const Text('导入钱包'),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildWalletList() {
    final wallets = _wallets;
    return ReorderableListView.builder(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      itemCount: wallets.length,
      onReorder: (oldIndex, newIndex) =>
          _onReorderWallet(wallets, oldIndex, newIndex),
      proxyDecorator: (child, index, animation) => AnimatedBuilder(
        animation: animation,
        builder: (context, child) => Material(
          elevation: 0,
          color: Colors.transparent,
          borderRadius: BorderRadius.circular(AppTheme.radiusMd),
          child: child,
        ),
        child: child,
      ),
      itemBuilder: (context, index) {
        final wallet = wallets[index];
        return _buildWalletCard(
          wallet,
          key: ValueKey(wallet.walletIndex),
        );
      },
    );
  }

  Future<void> _onReorderWallet(
    List<Wallet> displayedWallets,
    int oldIndex,
    int newIndex,
  ) async {
    if (oldIndex < newIndex) {
      newIndex -= 1;
    }
    if (oldIndex == newIndex) return;
    if (oldIndex < 0 ||
        oldIndex >= displayedWallets.length ||
        newIndex < 0 ||
        newIndex >= displayedWallets.length) {
      return;
    }

    final reorderedDisplayed = List<Wallet>.of(displayedWallets);
    final moved = reorderedDisplayed.removeAt(oldIndex);
    reorderedDisplayed.insert(newIndex, moved);

    final displayedIndexes =
        displayedWallets.map((w) => w.walletIndex).toSet();
    final displayedSlotCount = _wallets
        .where((w) => displayedIndexes.contains(w.walletIndex))
        .length;
    if (displayedSlotCount != reorderedDisplayed.length) return;

    // 乐观更新前先快照,写库失败可回滚,避免"UI 已排好、库没保存、冷启动跳回"的静默失败。
    final previous = _wallets;
    var cursor = 0;
    final next = _wallets.map((w) {
      if (!displayedIndexes.contains(w.walletIndex)) return w;
      return reorderedDisplayed[cursor++];
    }).toList();
    setState(() => _wallets = next);

    try {
      await _walletManager
          .reorderWallets(next.map((w) => w.masterId).toList());
    } catch (e) {
      if (!mounted) return;
      setState(() => _wallets = previous);
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('保存排序失败：$e')),
      );
    }
  }

  Widget _buildWalletCard(Wallet wallet, {Key? key}) {
    return Padding(
      key: key,
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Material(
        color: Colors.transparent,
        child: InkWell(
          borderRadius: BorderRadius.circular(AppTheme.radiusMd),
          onTap: () => _openWalletDetail(wallet),
          child: Container(
            padding: const EdgeInsets.all(16),
            decoration: AppTheme.cardDecoration(),
            child: Row(
              children: [
                Container(
                  width: 46,
                  height: 46,
                  decoration: BoxDecoration(
                    color: AppTheme.surfaceElevated,
                    borderRadius: BorderRadius.circular(AppTheme.radiusSm),
                  ),
                  child: const Icon(Icons.account_balance_wallet_rounded,
                      color: AppTheme.primaryLight, size: 22),
                ),
                const SizedBox(width: 14),
                Expanded(
                  child: Text(
                    wallet.walletName,
                    style: const TextStyle(
                        fontSize: 15,
                        fontWeight: FontWeight.w600,
                        color: AppTheme.textPrimary),
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
                // 本钱包扫码签名(只扫本钱包账户)。
                IconButton(
                  icon: SvgPicture.asset(
                    'assets/icons/scan-line.svg',
                    width: 20,
                    height: 20,
                    colorFilter: const ColorFilter.mode(
                        AppTheme.primaryLight, BlendMode.srcIn),
                  ),
                  tooltip: '扫码签名',
                  onPressed: () => _openWalletScan(wallet),
                ),
                PopupMenuButton<String>(
                  icon: const Icon(Icons.more_vert,
                      color: AppTheme.textTertiary, size: 20),
                  onSelected: (value) {
                    switch (value) {
                      case 'detail':
                        _openWalletDetail(wallet);
                      case 'rename':
                        _renameWallet(wallet);
                      case 'delete':
                        _confirmDelete(wallet);
                    }
                  },
                  itemBuilder: (context) => [
                    const PopupMenuItem(
                      value: 'rename',
                      child: Row(children: [
                        Icon(Icons.edit_outlined,
                            size: 18, color: AppTheme.textSecondary),
                        SizedBox(width: 10),
                        Text('重命名'),
                      ]),
                    ),
                    const PopupMenuItem(
                      value: 'detail',
                      child: Row(children: [
                        Icon(Icons.info_outline,
                            size: 18, color: AppTheme.textSecondary),
                        SizedBox(width: 10),
                        Text('钱包详情'),
                      ]),
                    ),
                    const PopupMenuItem(
                      value: 'delete',
                      child: Row(children: [
                        Icon(Icons.delete_outline,
                            size: 18, color: AppTheme.danger),
                        SizedBox(width: 10),
                        Text('删除钱包', style: TextStyle(color: AppTheme.danger)),
                      ]),
                    ),
                  ],
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
