import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:wuminapp_mobile/login/pages/qr_scan_page.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

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

class _MyWalletPageState extends State<MyWalletPage> {
  final WalletManager _walletService = WalletManager();
  final ChainRpc _chainRpc = ChainRpc();
  static const double _actionIconSize = 20;
  late Future<List<WalletProfile>> _walletsFuture;
  int? _activeWalletIndex;
  bool _balanceRefreshing = false;

  bool get _isSelectionMode => widget.selectForTrade || widget.selectForBind;

  @override
  void initState() {
    super.initState();
    _walletsFuture = _walletService.getWallets();
    _loadActiveWallet();
    _refreshBalancesFromChain();
  }

  void _reload() {
    setState(() {
      _walletsFuture = _walletService.getWallets();
    });
    _loadActiveWallet();
    _refreshBalancesFromChain();
  }

  Future<void> _refreshBalancesFromChain() async {
    if (_balanceRefreshing) return;
    setState(() { _balanceRefreshing = true; });
    try {
      final wallets = await _walletService.getWallets();
      bool updated = false;
      for (final wallet in wallets) {
        try {
          final balance = await _chainRpc.fetchBalance(wallet.pubkeyHex);
          if (balance != wallet.balance) {
            await _walletService.setWalletBalance(
                wallet.walletIndex, balance);
            updated = true;
          }
        } catch (e) {
          debugPrint(
              'wallet balance refresh failed: ${wallet.address}, err=$e');
        }
      }
      if (!mounted) return;
      if (updated) {
        setState(() {
          _walletsFuture = _walletService.getWallets();
        });
      }
    } finally {
      if (mounted) {
        setState(() { _balanceRefreshing = false; });
      }
    }
  }

  Future<void> _loadActiveWallet() async {
    final active = await _walletService.getActiveWalletIndex();
    if (!mounted) {
      return;
    }
    setState(() {
      _activeWalletIndex = active;
    });
  }

  Future<bool?> _confirmDelete(WalletProfile wallet) {
    return showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('删除钱包'),
        content: Text('确认删除钱包${wallet.walletIndex}？'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: const Text('取消'),
          ),
          FilledButton(
            onPressed: () => Navigator.of(context).pop(true),
            child: const Text('删除'),
          ),
        ],
      ),
    );
  }

  Future<void> _deleteWallet(WalletProfile wallet) async {
    await _walletService.deleteWallet(wallet.walletIndex);
    if (!mounted) {
      return;
    }
    _reload();
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(SnackBar(content: Text('已删除钱包${wallet.walletIndex}')));
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

  Future<void> _showColdWalletPlaceholder(String actionTitle) async {
    await showDialog<void>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(actionTitle),
        content: const Text('当前版本先展示冷钱包入口，私钥不落地的冷钱包流程将在后续接入。'),
        actions: [
          FilledButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('知道了'),
          ),
        ],
      ),
    );
  }

  Future<bool?> _confirmBindWallet(WalletProfile wallet) {
    return showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('绑定身份'),
        content: Text(
          '确认使用该钱包绑定身份？\n\n地址：${wallet.address}\n\n公钥：0x${wallet.pubkeyHex}',
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
                leading: const Icon(Icons.file_download_outlined),
                title: const Text('导入钱包'),
                onTap: () {
                  Navigator.of(context).pop();
                  _openImportPage();
                },
              ),
              ListTile(
                leading: const Icon(Icons.add_circle_outline),
                title: const Text('创建钱包'),
                onTap: () {
                  Navigator.of(context).pop();
                  _openCreatePage();
                },
              ),
            ],
          ),
        );
      },
    );
  }

  Widget _buildWalletCard(WalletProfile wallet, {required bool isLast}) {
    final iconData = WalletIconRegistry.iconFor(wallet.walletIcon);
    final cardColor = isLast
        ? const Color(0xFFFFF4E3)
        : (_activeWalletIndex == wallet.walletIndex
            ? const Color(0xFFE9F5EF)
            : null);
    return Card(
      color: cardColor,
      child: InkWell(
        borderRadius: BorderRadius.circular(12),
        onTap: () => _openWalletDetail(wallet),
        child: Padding(
          padding: const EdgeInsets.fromLTRB(14, 12, 14, 12),
          child: Stack(
            children: [
              Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      Container(
                        width: 36,
                        height: 36,
                        decoration: BoxDecoration(
                          color: const Color(0xFFE3EFE8),
                          borderRadius: BorderRadius.circular(10),
                        ),
                        child: Icon(
                          iconData,
                          color: const Color(0xFF0B3D2E),
                          size: 20,
                        ),
                      ),
                      const SizedBox(width: 10),
                      Expanded(
                        child: Text(
                          wallet.walletName,
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: const TextStyle(
                            fontSize: 16,
                            fontWeight: FontWeight.w600,
                          ),
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 8),
                  Padding(
                    padding: const EdgeInsets.only(left: 8, top: 4),
                    child: Row(
                      children: [
                        const Icon(
                          Icons.monetization_on_outlined,
                          size: 22,
                          color: Color(0xFF0B3D2E),
                        ),
                        const SizedBox(width: 8),
                        Text(
                          _formatBalance(wallet.balance),
                          style: const TextStyle(
                            fontSize: 18,
                            fontWeight: FontWeight.w600,
                          ),
                        ),
                        if (_balanceRefreshing) ...[
                          const SizedBox(width: 8),
                          const SizedBox(
                            width: 14,
                            height: 14,
                            child: CircularProgressIndicator(
                              strokeWidth: 2,
                            ),
                          ),
                        ],
                      ],
                    ),
                  ),
                  if (!_isSelectionMode) const SizedBox(height: 24),
                ],
              ),
              if (!_isSelectionMode)
                Positioned(
                  right: 0,
                  bottom: 0,
                  child: IconButton(
                    tooltip: '扫码',
                    onPressed: () {
                      Navigator.of(context).push(
                        MaterialPageRoute(
                          builder: (_) => QrScanPage(
                            walletIndex: wallet.walletIndex,
                            walletAddress: wallet.address,
                          ),
                        ),
                      );
                    },
                    icon: SvgPicture.asset(
                      'assets/icons/scan-line.svg',
                      width: _actionIconSize,
                      height: _actionIconSize,
                    ),
                  ),
                ),
            ],
          ),
        ),
      ),
    );
  }

  String _formatBalance(double balance) {
    return '${balance.toStringAsFixed(2)} GMB';
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
                    color: Colors.black87,
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
              color: const Color(0xFFFFE4E1),
              title: '创建热钱包',
              description: '创建私钥存在本机的热钱包',
              onTap: _openCreatePage,
            ),
            _buildWalletEntryOption(
              color: const Color(0xFFE0FFFF),
              title: '导入热钱包',
              description: '导入钱包并将私钥存在本机',
              onTap: _openImportPage,
            ),
            _buildWalletEntryOption(
              color: const Color(0xFFFFF4CC),
              title: '创建冷钱包',
              description: '创建钱包后，自行保管私钥，本机不保存私钥',
              onTap: () => _showColdWalletPlaceholder('创建冷钱包'),
            ),
            _buildWalletEntryOption(
              color: const Color(0xFFE6E6FA),
              title: '导入冷钱包',
              description: '导入钱包，本机不保存私钥',
              onTap: () => _showColdWalletPlaceholder('导入冷钱包'),
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
      ),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: FutureBuilder<List<WalletProfile>>(
          future: _walletsFuture,
          builder: (context, walletsSnapshot) {
            if (walletsSnapshot.connectionState != ConnectionState.done) {
              return const Center(child: CircularProgressIndicator());
            }

            final wallets = walletsSnapshot.data ?? const <WalletProfile>[];
            if (wallets.isEmpty) {
              return _buildEmptyWalletChoices();
            }

            return RefreshIndicator(
              onRefresh: _refreshBalancesFromChain,
              child: ListView(
                children: [
                  for (int i = 0; i < wallets.length; i++) ...[
                    (_isSelectionMode
                        ? _buildWalletCard(
                            wallets[i],
                            isLast: i == wallets.length - 1,
                          )
                        : Dismissible(
                            key: ValueKey(wallets[i].walletIndex),
                            direction: DismissDirection.endToStart,
                            confirmDismiss: (_) => _confirmDelete(wallets[i]),
                            onDismissed: (_) => _deleteWallet(wallets[i]),
                            background: Container(
                              alignment: Alignment.centerRight,
                              padding:
                                  const EdgeInsets.symmetric(horizontal: 20),
                              decoration: BoxDecoration(
                                color: Colors.red.shade400,
                                borderRadius: BorderRadius.circular(12),
                              ),
                              child: const Icon(
                                Icons.delete_outline,
                                color: Colors.white,
                              ),
                            ),
                            child: _buildWalletCard(
                              wallets[i],
                              isLast: i == wallets.length - 1,
                            ),
                          )),
                    const SizedBox(height: 10),
                  ],
                  if (!_isSelectionMode) ...[
                    const SizedBox(height: 12),
                    OutlinedButton(
                      onPressed: _showWalletEntryChooser,
                      child: const Text('导入钱包/创建钱包'),
                    ),
                  ],
                ],
              ),
            );
          },
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
  late final TextEditingController _nameController;
  late String _selectedWalletIcon;
  bool _iconPanelExpanded = false;
  bool _saving = false;

  @override
  void initState() {
    super.initState();
    _nameController = TextEditingController(text: widget.wallet.walletName);
    _selectedWalletIcon = widget.wallet.walletIcon;
  }

  @override
  void dispose() {
    _nameController.dispose();
    super.dispose();
  }

  Future<void> _saveWalletDisplay() async {
    final name = _nameController.text.trim();
    if (name.isEmpty) {
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(const SnackBar(content: Text('钱包名称不能为空')));
      return;
    }
    final hasChanged = name != widget.wallet.walletName ||
        _selectedWalletIcon != widget.wallet.walletIcon;
    if (!hasChanged) {
      Navigator.of(context).pop(false);
      return;
    }
    setState(() {
      _saving = true;
    });
    try {
      await _walletService.updateWalletDisplay(
        widget.wallet.walletIndex,
        walletName: name,
        walletIcon: _selectedWalletIcon,
      );
      if (!mounted) {
        return;
      }
      Navigator.of(context).pop(true);
    } catch (e) {
      if (!mounted) {
        return;
      }
      ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('$e')));
    } finally {
      if (mounted) {
        setState(() {
          _saving = false;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('钱包详情'),
        centerTitle: true,
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Row(
            children: [
              const Expanded(
                child: Text(
                  '钱包图标',
                  style: TextStyle(fontSize: 14, fontWeight: FontWeight.w600),
                ),
              ),
              InkWell(
                borderRadius: BorderRadius.circular(6),
                onTap: () {
                  setState(() {
                    _iconPanelExpanded = !_iconPanelExpanded;
                  });
                },
                child: Padding(
                  padding: const EdgeInsets.all(4),
                  child: Icon(
                    _iconPanelExpanded
                        ? Icons.keyboard_arrow_down
                        : Icons.keyboard_arrow_right,
                    size: 20,
                  ),
                ),
              ),
            ],
          ),
          const SizedBox(height: 8),
          Wrap(
            spacing: 10,
            runSpacing: 10,
            children: [
              for (final option in (_iconPanelExpanded
                  ? WalletIconRegistry.options
                  : WalletIconRegistry.options.take(4)))
                _WalletIconChoiceChip(
                  option: option,
                  selected: option.key == _selectedWalletIcon,
                  onTap: () {
                    setState(() {
                      _selectedWalletIcon = option.key;
                    });
                  },
                ),
            ],
          ),
          const SizedBox(height: 16),
          Container(
            width: double.infinity,
            padding: const EdgeInsets.symmetric(vertical: 16, horizontal: 14),
            decoration: BoxDecoration(
              color: const Color(0xFFE9F5EF),
              borderRadius: BorderRadius.circular(12),
            ),
            child: Column(
              children: [
                const Text(
                  '余额',
                  style: TextStyle(fontSize: 13, color: Colors.black54),
                ),
                const SizedBox(height: 6),
                Text(
                  '${widget.wallet.balance.toStringAsFixed(2)} GMB',
                  style: const TextStyle(
                    fontSize: 26,
                    fontWeight: FontWeight.w700,
                    color: Color(0xFF0B3D2E),
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 16),
          const Text(
            '钱包地址：',
            style: TextStyle(fontSize: 14, fontWeight: FontWeight.w700),
          ),
          const SizedBox(height: 4),
          Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Expanded(child: SelectableText(widget.wallet.address)),
              IconButton(
                tooltip: '复制钱包地址',
                onPressed: () {
                  Clipboard.setData(ClipboardData(text: widget.wallet.address));
                  ScaffoldMessenger.of(
                    context,
                  ).showSnackBar(const SnackBar(content: Text('钱包地址已复制')));
                },
                icon: SvgPicture.asset(
                  'assets/icons/copy.svg',
                  width: 18,
                  height: 18,
                ),
              ),
            ],
          ),
          const SizedBox(height: 10),
          const SizedBox(height: 12),
          TextField(
            controller: _nameController,
            decoration: const InputDecoration(
              labelText: '钱包名称',
              hintText: '请输入钱包名称',
              border: OutlineInputBorder(),
            ),
            textInputAction: TextInputAction.done,
          ),
          const SizedBox(height: 20),
          Align(
            alignment: Alignment.center,
            child: SizedBox(
              width: 190,
              child: FilledButton(
                onPressed: _saving ? null : _saveWalletDisplay,
                child: Text(
                  _saving ? '保存中...' : '保存钱包信息',
                  style: const TextStyle(
                    fontWeight: FontWeight.w800,
                    fontSize: 16,
                  ),
                ),
              ),
            ),
          ),
        ],
      ),
    );
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

class _WalletIconChoiceChip extends StatelessWidget {
  const _WalletIconChoiceChip({
    required this.option,
    required this.selected,
    required this.onTap,
  });

  final WalletIconOption option;
  final bool selected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return InkWell(
      borderRadius: BorderRadius.circular(12),
      onTap: onTap,
      child: Container(
        width: 46,
        height: 46,
        decoration: BoxDecoration(
          color: selected ? const Color(0xFFD7E9E1) : const Color(0xFFF4F7F5),
          border: Border.all(
            color: selected ? const Color(0xFF0B3D2E) : const Color(0xFFD3DAD6),
          ),
          borderRadius: BorderRadius.circular(12),
        ),
        child: Icon(
          option.icon,
          size: 20,
          color: const Color(0xFF0B3D2E),
        ),
      ),
    );
  }
}

class CreateWalletPage extends StatefulWidget {
  const CreateWalletPage({super.key});

  @override
  State<CreateWalletPage> createState() => _CreateWalletPageState();
}

class _CreateWalletPageState extends State<CreateWalletPage> {
  bool _isSaving = false;

  Future<void> _create() async {
    setState(() {
      _isSaving = true;
    });
    try {
      final created = await WalletManager().createWallet();
      if (!mounted) {
        return;
      }
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
                const Text('这是恢复钱包的唯一凭证，请离线抄写并妥善保管。'),
                const SizedBox(height: 12),
                SelectableText(
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
      appBar: AppBar(title: const Text('导入钱包')),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text('请输入助记词（至少 12 个单词）：'),
            const SizedBox(height: 8),
            const Text('仅使用默认派生路径，不暴露自定义路径。'),
            const SizedBox(height: 12),
            TextField(
              controller: _mnemonicController,
              minLines: 3,
              maxLines: 5,
              decoration: const InputDecoration(
                hintText: 'word1 word2 ...',
                border: OutlineInputBorder(),
              ),
            ),
            const SizedBox(height: 12),
            if (_error != null)
              Text(
                _error!,
                style: const TextStyle(color: Colors.red),
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
