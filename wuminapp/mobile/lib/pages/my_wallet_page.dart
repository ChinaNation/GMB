import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:wuminapp_mobile/login/pages/qr_scan_page.dart';
import 'package:wuminapp_mobile/services/wallet_service.dart';

class MyWalletPage extends StatefulWidget {
  const MyWalletPage({
    super.key,
    this.selectForTrade = false,
  });

  final bool selectForTrade;

  @override
  State<MyWalletPage> createState() => _MyWalletPageState();
}

class _MyWalletPageState extends State<MyWalletPage> {
  final WalletService _walletService = WalletService();
  static const double _actionIconSize = 18;
  static const double _actionSlotWidth = 34;
  late Future<List<WalletProfile>> _walletsFuture;
  int? _activeWalletIndex;

  @override
  void initState() {
    super.initState();
    _walletsFuture = _walletService.getWallets();
    _loadActiveWallet();
  }

  void _reload() {
    setState(() {
      _walletsFuture = _walletService.getWallets();
    });
    _loadActiveWallet();
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

  Future<void> _openWalletDetail(WalletProfile wallet) async {
    if (widget.selectForTrade) {
      await _walletService.setActiveWallet(wallet.walletIndex);
      if (!mounted) {
        return;
      }
      Navigator.of(context).pop(true);
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

  Widget _buildWalletCard(WalletProfile wallet) {
    return Card(
      color: _activeWalletIndex == wallet.walletIndex
          ? const Color(0xFFE9F5EF)
          : null,
      child: InkWell(
        borderRadius: BorderRadius.circular(12),
        onTap: () => _openWalletDetail(wallet),
        child: Padding(
          padding: const EdgeInsets.fromLTRB(14, 8, 14, 10),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Expanded(
                    child: Transform.translate(
                      offset: const Offset(0, -2),
                      child: Text(
                        '名称: ${wallet.walletName}',
                      ),
                    ),
                  ),
                  if (!widget.selectForTrade)
                    SizedBox(
                      width: _actionSlotWidth,
                      child: Transform.translate(
                        offset: const Offset(2, -6),
                        child: InkWell(
                          borderRadius: BorderRadius.circular(8),
                          onTap: () {
                            Clipboard.setData(
                              ClipboardData(text: wallet.address),
                            );
                            ScaffoldMessenger.of(context).showSnackBar(
                              SnackBar(
                                content: Text('已复制: ${wallet.address}'),
                              ),
                            );
                          },
                          child: Padding(
                            padding: const EdgeInsets.all(4),
                            child: SvgPicture.asset(
                              'assets/icons/copy.svg',
                              width: _actionIconSize,
                              height: _actionIconSize,
                            ),
                          ),
                        ),
                      ),
                    ),
                  if (!widget.selectForTrade)
                    SizedBox(
                      width: _actionSlotWidth,
                      child: Transform.translate(
                        offset: const Offset(2, -6),
                        child: IconButton(
                          tooltip: '扫码',
                          padding: EdgeInsets.zero,
                          constraints: const BoxConstraints(),
                          visualDensity: VisualDensity.compact,
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
                    ),
                ],
              ),
              const SizedBox(height: 1),
              Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Expanded(
                    child: Text(
                      '地址: ${wallet.address}',
                      maxLines: 2,
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(widget.selectForTrade ? '选择交易钱包' : '我的钱包'),
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
              return Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Text(
                    '还没有钱包，请先创建或导入钱包。',
                    style: TextStyle(fontSize: 16),
                  ),
                  const SizedBox(height: 16),
                  FilledButton(
                    onPressed: _showWalletEntryChooser,
                    child: const Text('导入钱包/创建钱包'),
                  ),
                ],
              );
            }

            return ListView(
              children: [
                for (int i = 0; i < wallets.length; i++) ...[
                  (widget.selectForTrade
                      ? _buildWalletCard(wallets[i])
                      : Dismissible(
                          key: ValueKey(wallets[i].walletIndex),
                          direction: DismissDirection.endToStart,
                          confirmDismiss: (_) => _confirmDelete(wallets[i]),
                          onDismissed: (_) => _deleteWallet(wallets[i]),
                          background: Container(
                            alignment: Alignment.centerRight,
                            padding: const EdgeInsets.symmetric(horizontal: 20),
                            decoration: BoxDecoration(
                              color: Colors.red.shade400,
                              borderRadius: BorderRadius.circular(12),
                            ),
                            child: const Icon(
                              Icons.delete_outline,
                              color: Colors.white,
                            ),
                          ),
                          child: _buildWalletCard(wallets[i]),
                        )),
                  const SizedBox(height: 10),
                ],
                if (!widget.selectForTrade) ...[
                  const SizedBox(height: 12),
                  OutlinedButton(
                    onPressed: _showWalletEntryChooser,
                    child: const Text('导入钱包/创建钱包'),
                  ),
                ],
              ],
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
  final WalletService _walletService = WalletService();
  late final TextEditingController _nameController;
  bool _saving = false;

  @override
  void initState() {
    super.initState();
    _nameController = TextEditingController(text: widget.wallet.walletName);
  }

  @override
  void dispose() {
    _nameController.dispose();
    super.dispose();
  }

  Future<void> _saveName() async {
    final name = _nameController.text.trim();
    if (name.isEmpty) {
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(const SnackBar(content: Text('钱包名称不能为空')));
      return;
    }
    if (name == widget.wallet.walletName) {
      Navigator.of(context).pop(false);
      return;
    }
    setState(() {
      _saving = true;
    });
    try {
      await _walletService.renameWallet(widget.wallet.walletIndex, name);
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
    final accountAddress = '0x${widget.wallet.pubkeyHex}';
    return Scaffold(
      appBar: AppBar(
        title: const Text('钱包详情'),
        centerTitle: true,
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          TextField(
            controller: _nameController,
            decoration: const InputDecoration(
              labelText: '钱包名称',
              hintText: '请输入钱包名称',
              border: OutlineInputBorder(),
            ),
            textInputAction: TextInputAction.done,
          ),
          const SizedBox(height: 12),
          Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Expanded(
                child: SelectableText('钱包地址: ${widget.wallet.address}'),
              ),
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
          const SizedBox(height: 8),
          Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Expanded(
                child: SelectableText('账户地址: $accountAddress'),
              ),
              IconButton(
                tooltip: '复制账户地址',
                onPressed: () {
                  Clipboard.setData(ClipboardData(text: accountAddress));
                  ScaffoldMessenger.of(
                    context,
                  ).showSnackBar(const SnackBar(content: Text('账户地址已复制')));
                },
                icon: SvgPicture.asset(
                  'assets/icons/copy.svg',
                  width: 18,
                  height: 18,
                ),
              ),
            ],
          ),
          const SizedBox(height: 20),
          FilledButton(
            onPressed: _saving ? null : _saveName,
            child: Text(_saving ? '保存中...' : '保存名称'),
          ),
        ],
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
      final created = await WalletService().createWallet();
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
      await WalletService().importWallet(_mnemonicController.text);
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
