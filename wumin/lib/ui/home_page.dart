import 'package:flutter/material.dart';

import '../qr/offline_sign_page.dart';
import '../wallet/wallet_manager.dart';
import 'create_wallet_page.dart';
import 'import_wallet_page.dart';
import 'scan_page.dart';
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
  int? _activeIndex;
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _loadWallets();
  }

  Future<void> _loadWallets() async {
    setState(() => _loading = true);
    try {
      final wallets = await _walletManager.getWallets();
      final activeIndex = await _walletManager.getActiveWalletIndex();
      if (!mounted) return;
      setState(() {
        _wallets = wallets;
        _activeIndex = activeIndex;
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
        title: const Text('公民冷钱包'),
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
          : hasWallets
              ? _buildWalletList()
              : _buildEmptyState(),
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

  Widget _buildWalletList() {
    return ListView.builder(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      itemCount: _wallets.length,
      itemBuilder: (context, index) {
        final wallet = _wallets[index];
        final isActive = wallet.walletIndex == _activeIndex;
        return _buildWalletCard(wallet, isActive);
      },
    );
  }

  Widget _buildWalletCard(WalletProfile wallet, bool isActive) {
    return Card(
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
                        if (isActive) ...[
                          const SizedBox(width: 8),
                          Container(
                            padding: const EdgeInsets.symmetric(
                              horizontal: 6,
                              vertical: 2,
                            ),
                            decoration: BoxDecoration(
                              color: Theme.of(context)
                                  .colorScheme
                                  .primary
                                  .withValues(alpha: 0.1),
                              borderRadius: BorderRadius.circular(4),
                            ),
                            child: Text(
                              '当前',
                              style: TextStyle(
                                fontSize: 11,
                                color: Theme.of(context).colorScheme.primary,
                                fontWeight: FontWeight.w600,
                              ),
                            ),
                          ),
                        ],
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
                icon: const Icon(Icons.qr_code_scanner),
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
