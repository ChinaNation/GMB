import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:isar/isar.dart';

import '../isar/wallet_isar.dart';
import '../util/screenshot_guard.dart';
import '../wallet/wallet_manager.dart';
import 'create_wallet_page.dart';
import 'group_management_page.dart';
import 'import_wallet_page.dart';
import 'scan_page.dart';
import 'settings_page.dart';
import 'wallet_detail_page.dart';

/// 钱包列表首页。
class HomePage extends StatefulWidget {
  const HomePage({super.key});

  @override
  State<HomePage> createState() => _HomePageState();
}

class _HomePageState extends State<HomePage> {
  final WalletManager _walletManager = WalletManager();
  List<WalletProfile> _wallets = [];
  List<WalletGroupEntity> _groups = [];
  String _selectedGroup = '全部';
  int? _activeIndex;
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
      final activeIndex = await _walletManager.getActiveWalletIndex();
      final isar = await WalletIsar.instance.db();
      final groups = await isar.walletGroupEntitys
          .where()
          .sortBySortOrder()
          .findAll();
      if (!mounted) return;
      setState(() {
        _wallets = wallets;
        _activeIndex = activeIndex;
        _groups = groups;
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

  List<WalletProfile> get _filteredWallets {
    if (_selectedGroup == '全部') return _wallets;
    return _wallets.where((w) => w.inGroup(_selectedGroup)).toList();
  }

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

  void _showAddWalletMenu() {
    showModalBottomSheet(
      context: context,
      builder: (context) => SafeArea(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            ListTile(
              leading: const Icon(Icons.add),
              title: const Text('创建钱包'),
              onTap: () {
                Navigator.pop(context);
                _openCreateWallet();
              },
            ),
            ListTile(
              leading: const Icon(Icons.download),
              title: const Text('导入钱包'),
              onTap: () {
                Navigator.pop(context);
                _openImportWallet();
              },
            ),
          ],
        ),
      ),
    );
  }

  Future<void> _openScan(WalletProfile wallet) async {
    await Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => ScanPage(wallet: wallet),
      ),
    );
  }

  Future<void> _openWalletDetail(WalletProfile wallet) async {
    await Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => WalletDetailPage(wallet: wallet),
      ),
    );
    await _loadWallets();
  }

  Future<void> _setActive(int walletIndex) async {
    try {
      await _walletManager.setActiveWallet(walletIndex);
      await _loadWallets();
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('切换钱包失败：$e')),
      );
    }
  }

  Future<void> _confirmDelete(WalletProfile wallet) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('删除钱包'),
        content: Text('确定删除「${wallet.walletName}」？\n删除后私钥将被清除，请确保已备份助记词。'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: const Text('取消'),
          ),
          TextButton(
            onPressed: () => Navigator.of(context).pop(true),
            style: TextButton.styleFrom(foregroundColor: Colors.red),
            child: const Text('删除'),
          ),
        ],
      ),
    );
    if (confirmed != true) return;
    try {
      await _walletManager.deleteWallet(wallet.walletIndex);
      await _loadWallets();
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('删除失败：$e')),
      );
    }
  }

  Future<void> _renameWallet(WalletProfile wallet) async {
    final controller = TextEditingController(text: wallet.walletName);
    final newName = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('重命名钱包'),
        content: TextField(
          controller: controller,
          autofocus: true,
          decoration: const InputDecoration(hintText: '请输入新名称'),
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
      await _walletManager.renameWallet(wallet.walletIndex, newName);
      await _loadWallets();
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('重命名失败：$e')),
      );
    }
  }

  String _truncateAddress(String address) {
    if (address.length <= 16) return address;
    return '${address.substring(0, 8)}...${address.substring(address.length - 6)}';
  }

  @override
  Widget build(BuildContext context) {
    final hasWallets = _wallets.isNotEmpty;
    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: const Icon(Icons.settings_outlined),
          tooltip: '设置',
          onPressed: () => Navigator.of(context).push(
            MaterialPageRoute(builder: (_) => const SettingsPage()),
          ),
        ),
        title: const Text('公民钱包'),
        centerTitle: true,
        actions: [
          if (hasWallets)
            IconButton(
              icon: const Icon(Icons.add),
              tooltip: '添加钱包',
              onPressed: _showAddWalletMenu,
            ),
        ],
      ),
      body: _loading
          ? const Center(child: CircularProgressIndicator())
          : Column(
              children: [
                if (_isRooted)
                  Container(
                    width: double.infinity,
                    padding: const EdgeInsets.symmetric(
                        horizontal: 12, vertical: 8),
                    color: Colors.red.shade700,
                    child: const Row(
                      children: [
                        Icon(Icons.warning, color: Colors.white, size: 18),
                        SizedBox(width: 8),
                        Expanded(
                          child: Text(
                            '检测到设备已 root/越狱，密钥安全无法保障',
                            style: TextStyle(
                                color: Colors.white, fontSize: 13),
                          ),
                        ),
                      ],
                    ),
                  ),
                Expanded(
                  child: hasWallets
                      ? _buildWalletList()
                      : _buildEmptyState(),
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
            Icon(
              Icons.account_balance_wallet_outlined,
              size: 64,
              color: Colors.grey.shade400,
            ),
            const SizedBox(height: 16),
            const Text(
              '还没有钱包',
              style: TextStyle(fontSize: 18, fontWeight: FontWeight.w600),
            ),
            const SizedBox(height: 8),
            const Text(
              '创建或导入一个钱包来开始使用',
              style: TextStyle(color: Colors.black54),
            ),
            const SizedBox(height: 32),
            FilledButton.icon(
              onPressed: _openCreateWallet,
              icon: const Icon(Icons.add),
              label: const Text('创建钱包'),
            ),
            const SizedBox(height: 12),
            OutlinedButton.icon(
              onPressed: _openImportWallet,
              icon: const Icon(Icons.download),
              label: const Text('导入钱包'),
            ),
          ],
        ),
      ),
    );
  }

  Future<void> _openGroupManagement() async {
    await Navigator.of(context).push(
      MaterialPageRoute(builder: (_) => const GroupManagementPage()),
    );
    await _loadAll();
  }

  Widget _buildGroupRow() {
    final otherGroups = _groups.where((g) => g.name != '全部').toList();
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 8, 8, 0),
      child: Row(
        children: [
          // "全部"固定在左侧，不跟随滚动
          Padding(
            padding: const EdgeInsets.only(right: 8),
            child: ChoiceChip(
              label: const Text('全部'),
              selected: _selectedGroup == '全部',
              onSelected: (_) {
                setState(() => _selectedGroup = '全部');
              },
            ),
          ),
          // 其余分组可左右滑动
          Expanded(
            child: SingleChildScrollView(
              scrollDirection: Axis.horizontal,
              child: Row(
                children: otherGroups.map((g) {
                  final selected = g.name == _selectedGroup;
                  return Padding(
                    padding: const EdgeInsets.only(right: 8),
                    child: ChoiceChip(
                      label: Text(g.name),
                      selected: selected,
                      onSelected: (_) {
                        setState(() => _selectedGroup = g.name);
                      },
                    ),
                  );
                }).toList(),
              ),
            ),
          ),
          IconButton(
            icon: const Icon(Icons.chevron_right),
            tooltip: '分组管理',
            onPressed: _openGroupManagement,
          ),
        ],
      ),
    );
  }

  Widget _buildWalletList() {
    final wallets = _filteredWallets;
    return Column(
      children: [
        _buildGroupRow(),
        Expanded(
          child: wallets.isEmpty
              ? Center(
                  child: Text(
                    '该分组下没有钱包',
                    style: TextStyle(color: Colors.grey.shade500),
                  ),
                )
              : ReorderableListView.builder(
                  padding:
                      const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
                  itemCount: wallets.length,
                  onReorder: (oldIndex, newIndex) =>
                      _onReorder(wallets, oldIndex, newIndex),
                  proxyDecorator: (child, index, animation) {
                    return AnimatedBuilder(
                      animation: animation,
                      builder: (context, child) => Material(
                        elevation: 4,
                        borderRadius: BorderRadius.circular(12),
                        child: child,
                      ),
                      child: child,
                    );
                  },
                  itemBuilder: (context, index) {
                    final wallet = wallets[index];
                    final isActive = wallet.walletIndex == _activeIndex;
                    return _buildWalletCard(
                      wallet,
                      isActive,
                      key: ValueKey(wallet.walletIndex),
                    );
                  },
                ),
        ),
      ],
    );
  }

  Future<void> _onReorder(
    List<WalletProfile> displayedWallets,
    int oldIndex,
    int newIndex,
  ) async {
    if (newIndex > oldIndex) newIndex--;
    if (oldIndex == newIndex) return;

    // 找到在全量列表中的真实索引
    final movedWalletIndex = displayedWallets[oldIndex].walletIndex;
    final targetWalletIndex = displayedWallets[newIndex].walletIndex;

    final fromGlobal =
        _wallets.indexWhere((w) => w.walletIndex == movedWalletIndex);
    var toGlobal =
        _wallets.indexWhere((w) => w.walletIndex == targetWalletIndex);

    if (fromGlobal < 0 || toGlobal < 0) return;

    final item = _wallets.removeAt(fromGlobal);
    // removeAt 后索引可能偏移
    if (fromGlobal < toGlobal) toGlobal--;
    _wallets.insert(toGlobal, item);

    setState(() {});

    // 持久化新顺序
    final indexes = _wallets.map((w) => w.walletIndex).toList();
    await _walletManager.reorderWallets(indexes);
  }

  Widget _buildWalletCard(WalletProfile wallet, bool isActive, {Key? key}) {
    return Card(
      key: key,
      margin: const EdgeInsets.symmetric(vertical: 6),
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: isActive
            ? BorderSide(color: Theme.of(context).colorScheme.primary, width: 2)
            : BorderSide.none,
      ),
      child: InkWell(
        borderRadius: BorderRadius.circular(12),
        onTap: () => _setActive(wallet.walletIndex),
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Row(
            children: [
              CircleAvatar(
                backgroundColor: isActive
                    ? Theme.of(context).colorScheme.primary
                    : Colors.grey.shade300,
                child: Icon(
                  Icons.account_balance_wallet,
                  color: isActive ? Colors.white : Colors.grey.shade600,
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        Flexible(
                          child: Text(
                            wallet.walletName,
                            style: const TextStyle(
                              fontSize: 16,
                              fontWeight: FontWeight.w600,
                            ),
                            overflow: TextOverflow.ellipsis,
                          ),
                        ),
                      ],
                    ),
                    const SizedBox(height: 4),
                    Text(
                      _truncateAddress(wallet.address),
                      style: TextStyle(
                        fontSize: 13,
                        color: Colors.grey.shade600,
                        fontFamily: 'monospace',
                      ),
                    ),
                  ],
                ),
              ),
              IconButton(
                icon: SvgPicture.asset(
                  'assets/icons/scan-line.svg',
                  width: 22,
                  height: 22,
                ),
                tooltip: '扫码签名',
                onPressed: () => _openScan(wallet),
              ),
              PopupMenuButton<String>(
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
                    child: Text('重命名'),
                  ),
                  const PopupMenuItem(
                    value: 'detail',
                    child: Text('钱包详情'),
                  ),
                  const PopupMenuItem(
                    value: 'delete',
                    child: Text('删除钱包',
                        style: TextStyle(color: Colors.red)),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}
