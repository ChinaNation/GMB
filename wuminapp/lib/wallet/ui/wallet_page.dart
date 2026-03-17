import 'dart:ui' as ui;
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:saver_gallery/saver_gallery.dart';
import 'package:wuminapp_mobile/qr/pages/qr_scan_page.dart';
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

  Future<void> _openCreateColdWalletPage() async {
    final created = await Navigator.of(context).push<bool>(
      MaterialPageRoute(builder: (_) => const CreateColdWalletPage()),
    );
    if (created == true) {
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
                leading: const Icon(Icons.ac_unit),
                title: const Text('创建冷钱包'),
                subtitle: const Text('本机不保存私钥'),
                onTap: () {
                  Navigator.of(context).pop();
                  _openCreateColdWalletPage();
                },
              ),
              ListTile(
                leading: const Icon(Icons.qr_code_scanner),
                title: const Text('导入冷钱包'),
                subtitle: const Text('通过地址导入'),
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

  Widget _buildWalletCard(WalletProfile wallet, {required bool isLast}) {
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
          padding: const EdgeInsets.fromLTRB(14, 4, 6, 12),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // 第一行：热/冷标识 + 钱包名称 + 扫码图标
              Row(
                children: [
                  Container(
                    width: 28,
                    height: 28,
                    decoration: BoxDecoration(
                      color: wallet.isHotWallet
                          ? const Color(0xFFFFE0B2)
                          : const Color(0xFFB3E5FC),
                      borderRadius: BorderRadius.circular(7),
                    ),
                    child: Center(
                      child: Text(
                        wallet.isHotWallet ? '热' : '冷',
                        style: TextStyle(
                          fontSize: 12,
                          fontWeight: FontWeight.w700,
                          color: wallet.isHotWallet
                              ? const Color(0xFFE65100)
                              : const Color(0xFF01579B),
                        ),
                      ),
                    ),
                  ),
                  const SizedBox(width: 8),
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
                  if (!_isSelectionMode)
                    Padding(
                      padding: const EdgeInsets.only(right: 0),
                      child: IconButton(
                        tooltip: '扫码',
                        constraints: const BoxConstraints(),
                        padding: const EdgeInsets.all(4),
                        onPressed: () {
                          Navigator.of(context).push(
                            MaterialPageRoute(
                              builder: (_) => QrScanPage(
                                mode: QrScanMode.login,
                                walletIndex: wallet.walletIndex,
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
              const SizedBox(height: 12),
              // 第二行：余额居中，GMB 缩小减淡
              Center(
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.baseline,
                  textBaseline: TextBaseline.alphabetic,
                  children: [
                    Text(
                      wallet.balance.toStringAsFixed(2),
                      style: const TextStyle(
                        fontSize: 30,
                        fontWeight: FontWeight.w700,
                        color: Color(0xFF0B3D2E),
                      ),
                    ),
                    Text(
                      '元',
                      style: const TextStyle(
                        fontSize: 20,
                        fontWeight: FontWeight.w700,
                        color: Color(0xFF0B3D2E),
                      ),
                    ),
                    const SizedBox(width: 6),
                    const Text(
                      'GMB',
                      style: TextStyle(
                        fontSize: 14,
                        fontWeight: FontWeight.w500,
                        color: Colors.black38,
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
              const SizedBox(height: 4),
            ],
          ),
        ),
      ),
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
              onTap: _openCreateColdWalletPage,
            ),
            _buildWalletEntryOption(
              color: const Color(0xFFE6E6FA),
              title: '导入冷钱包',
              description: '导入钱包，本机不保存私钥',
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
  final GlobalKey _qrKey = GlobalKey();
  late String _walletName;
  bool _isEditingName = false;
  bool _hasChanged = false;
  bool _isSavingQr = false;
  late final TextEditingController _nameEditController;

  @override
  void initState() {
    super.initState();
    _walletName = widget.wallet.walletName;
    _nameEditController = TextEditingController(text: _walletName);
  }

  @override
  void dispose() {
    _nameEditController.dispose();
    super.dispose();
  }

  Future<void> _saveWalletName(String name) async {
    final trimmed = name.trim();
    if (trimmed.isEmpty || trimmed == _walletName) {
      setState(() {
        _isEditingName = false;
        _nameEditController.text = _walletName;
      });
      return;
    }
    try {
      await _walletService.updateWalletDisplay(
        widget.wallet.walletIndex,
        walletName: trimmed,
        walletIcon: widget.wallet.walletIcon,
      );
      if (!mounted) return;
      setState(() {
        _walletName = trimmed;
        _isEditingName = false;
        _hasChanged = true;
      });
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('$e')));
    }
  }

  Future<void> _saveQrToGallery() async {
    if (_isSavingQr) return;
    setState(() { _isSavingQr = true; });
    try {
      final boundary = _qrKey.currentContext?.findRenderObject()
          as RenderRepaintBoundary?;
      if (boundary == null) return;
      final image = await boundary.toImage(pixelRatio: 3.0);
      final byteData = await image.toByteData(
        format: ui.ImageByteFormat.png,
      );
      if (byteData == null) return;
      final pngBytes = byteData.buffer.asUint8List();
      final fileName = 'wallet_qr_${DateTime.now().millisecondsSinceEpoch}.png';
      final result = await SaverGallery.saveImage(
        Uint8List.fromList(pngBytes),
        fileName: fileName,
        skipIfExists: false,
      );
      if (!mounted) return;
      final success = result.isSuccess;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(success ? '二维码已保存到相册' : '保存失败，请检查相册权限'),
        ),
      );
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('保存失败：$e')),
      );
    } finally {
      if (mounted) {
        setState(() { _isSavingQr = false; });
      }
    }
  }

  /// 将地址拆成两行显示，第一行长一些，第二行短一些。
  String _formatAddressTwoLines(String address) {
    if (address.length <= 20) return address;
    final firstLineLen = (address.length * 2) ~/ 3;
    return '${address.substring(0, firstLineLen)}\n${address.substring(firstLineLen)}';
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
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          // 钱包名称（点击可编辑）
          Center(
            child: _isEditingName
                ? SizedBox(
                    width: 200,
                    child: TextField(
                      controller: _nameEditController,
                      autofocus: true,
                      textAlign: TextAlign.center,
                      style: const TextStyle(
                        fontSize: 20,
                        fontWeight: FontWeight.w700,
                      ),
                      decoration: const InputDecoration(
                        border: UnderlineInputBorder(),
                        isDense: true,
                        contentPadding:
                            EdgeInsets.symmetric(vertical: 6),
                      ),
                      textInputAction: TextInputAction.done,
                      onSubmitted: _saveWalletName,
                      onTapOutside: (_) {
                        _saveWalletName(_nameEditController.text);
                      },
                    ),
                  )
                : GestureDetector(
                    onTap: () {
                      setState(() {
                        _isEditingName = true;
                        _nameEditController.text = _walletName;
                      });
                    },
                    child: Text(
                      _walletName,
                      style: const TextStyle(
                        fontSize: 20,
                        fontWeight: FontWeight.w700,
                      ),
                    ),
                  ),
          ),
          const SizedBox(height: 16),
          // 余额
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
                Row(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.baseline,
                  textBaseline: TextBaseline.alphabetic,
                  children: [
                    Text(
                      widget.wallet.balance.toStringAsFixed(2),
                      style: const TextStyle(
                        fontSize: 32,
                        fontWeight: FontWeight.w700,
                        color: Color(0xFF0B3D2E),
                      ),
                    ),
                    const Text(
                      '元',
                      style: TextStyle(
                        fontSize: 22,
                        fontWeight: FontWeight.w700,
                        color: Color(0xFF0B3D2E),
                      ),
                    ),
                    const SizedBox(width: 6),
                    const Text(
                      'GMB',
                      style: TextStyle(
                        fontSize: 15,
                        fontWeight: FontWeight.w500,
                        color: Colors.black38,
                      ),
                    ),
                  ],
                ),
              ],
            ),
          ),
          const SizedBox(height: 20),
          // 钱包二维码
          Center(
            child: RepaintBoundary(
              key: _qrKey,
              child: Container(
                color: Colors.white,
                padding: const EdgeInsets.all(8),
                child: QrImageView(
                  data: 'gmb://account/${widget.wallet.address}',
                  version: QrVersions.auto,
                  size: 240,
                ),
              ),
            ),
          ),
          const SizedBox(height: 8),
          // 下载图标
          Center(
            child: IconButton(
              tooltip: '保存二维码到相册',
              constraints: const BoxConstraints(),
              padding: const EdgeInsets.all(4),
              onPressed: _isSavingQr ? null : _saveQrToGallery,
              icon: _isSavingQr
                  ? const SizedBox(
                      width: 20,
                      height: 20,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                  : SvgPicture.asset(
                      'assets/icons/download.svg',
                      width: 22,
                      height: 22,
                      colorFilter: const ColorFilter.mode(
                        Colors.black54,
                        BlendMode.srcIn,
                      ),
                    ),
            ),
          ),
          const SizedBox(height: 8),
          // 钱包地址居中 + 复制图标在右侧
          Stack(
            alignment: Alignment.center,
            children: [
              // 地址居中（与二维码对齐）
              GestureDetector(
                onTap: () {
                  Clipboard.setData(
                      ClipboardData(text: widget.wallet.address));
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(content: Text('钱包地址已复制')),
                  );
                },
                child: Text(
                  _formatAddressTwoLines(widget.wallet.address),
                  textAlign: TextAlign.center,
                  style: const TextStyle(
                    fontSize: 13,
                    color: Colors.black54,
                  ),
                ),
              ),
              // 复制图标定位到右侧
              Positioned(
                right: 16,
                child: IconButton(
                  tooltip: '复制钱包地址',
                  constraints: const BoxConstraints(),
                  padding: EdgeInsets.zero,
                  iconSize: 18,
                  onPressed: () {
                    Clipboard.setData(
                        ClipboardData(text: widget.wallet.address));
                    ScaffoldMessenger.of(context).showSnackBar(
                      const SnackBar(content: Text('钱包地址已复制')),
                    );
                  },
                  icon: SvgPicture.asset(
                    'assets/icons/copy.svg',
                    width: 16,
                    height: 16,
                  ),
                ),
              ),
            ],
          ),
        ],
      ),
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
                const Text(
                  '助记词仅此一次展示，本机不保存助记词。\n请离线抄写并妥善保管，这是恢复钱包的唯一凭证。',
                ),
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

class CreateColdWalletPage extends StatefulWidget {
  const CreateColdWalletPage({super.key});

  @override
  State<CreateColdWalletPage> createState() => _CreateColdWalletPageState();
}

class _CreateColdWalletPageState extends State<CreateColdWalletPage> {
  bool _isSaving = false;

  Future<void> _create() async {
    setState(() {
      _isSaving = true;
    });
    try {
      final created = await WalletManager().createColdWallet();
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
                const Text(
                  '⚠️ 冷钱包：本机不保存任何密钥材料。\n'
                  '助记词是恢复钱包的唯一凭证，请务必离线抄写并妥善保管。',
                  style: TextStyle(color: Colors.red, fontWeight: FontWeight.w600),
                ),
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
      appBar: AppBar(title: const Text('创建冷钱包')),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text('将创建一个冷钱包（仅存公钥，不存私钥）。'),
            const SizedBox(height: 8),
            const Text(
              '交易和登录需要通过扫码签名完成。',
              style: TextStyle(color: Colors.black54),
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
            const Text('请输入 SS58 格式的钱包地址：'),
            const SizedBox(height: 8),
            const Text(
              '冷钱包仅存储公钥，交易和登录需通过扫码签名。',
              style: TextStyle(color: Colors.black54),
            ),
            const SizedBox(height: 12),
            TextField(
              controller: _addressController,
              decoration: const InputDecoration(
                hintText: '例如：5GrwvaEF5zXb26Fz9rcQpDWS...',
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
