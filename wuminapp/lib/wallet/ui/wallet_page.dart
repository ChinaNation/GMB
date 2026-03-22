import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:qr/qr.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:saver_gallery/saver_gallery.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';
import 'package:wuminapp_mobile/trade/onchain/onchain_trade_models.dart';
import 'package:wuminapp_mobile/qr/transfer/transfer_qr_models.dart';
import 'package:wuminapp_mobile/trade/onchain/onchain_trade_repository.dart';
import 'package:wuminapp_mobile/user/user_service.dart' show UserProfileService;
import 'package:wuminapp_mobile/ui/widgets/bip39_input.dart';
import 'package:wuminapp_mobile/util/screenshot_guard.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';
import 'package:wuminapp_mobile/wallet/ui/transaction_history_page.dart';

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
    setState(() {
      _balanceRefreshing = true;
    });
    try {
      final wallets = await _walletService.getWallets();
      bool updated = false;
      for (final wallet in wallets) {
        try {
          final balance = await _chainRpc.fetchBalance(wallet.pubkeyHex);
          if (balance != wallet.balance) {
            await _walletService.setWalletBalance(wallet.walletIndex, balance);
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
        setState(() {
          _balanceRefreshing = false;
        });
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

  Widget _buildWalletCard(WalletProfile wallet, {required bool isLast}) {
    final cardColor = isLast
        ? const Color(0xFFFFF4E3)
        : (_activeWalletIndex == wallet.walletIndex
            ? const Color(0xFFE9F5EF)
            : null);

    // 根据金额长度自动选择字号
    final balanceStr = wallet.balance.toStringAsFixed(2);
    final balanceFontSize = balanceStr.length > 10
        ? 16.0
        : balanceStr.length > 7
            ? 20.0
            : 24.0;

    return Card(
      color: cardColor,
      child: InkWell(
        borderRadius: BorderRadius.circular(12),
        onTap: () => _openWalletDetail(wallet),
        child: Padding(
          padding: const EdgeInsets.fromLTRB(10, 6, 10, 4),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // 顶部：冷热图标 + 钱包名称
              Row(
                children: [
                  Container(
                    width: 20,
                    height: 20,
                    decoration: BoxDecoration(
                      color: wallet.isHotWallet
                          ? const Color(0xFFFFE0B2)
                          : const Color(0xFFB3E5FC),
                      borderRadius: BorderRadius.circular(5),
                    ),
                    child: Center(
                      child: Text(
                        wallet.isHotWallet ? '热' : '冷',
                        style: TextStyle(
                          fontSize: 9,
                          fontWeight: FontWeight.w700,
                          color: wallet.isHotWallet
                              ? const Color(0xFFE65100)
                              : const Color(0xFF01579B),
                        ),
                      ),
                    ),
                  ),
                  const SizedBox(width: 6),
                  Expanded(
                    child: Text(
                      wallet.walletName,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: const TextStyle(
                        fontSize: 14,
                        fontWeight: FontWeight.w600,
                      ),
                    ),
                  ),
                ],
              ),
              // 中间：余额（自适应字号）
              Expanded(
                child: Center(
                  child: FittedBox(
                    fit: BoxFit.scaleDown,
                    child: Text(
                      balanceStr,
                      style: TextStyle(
                        fontSize: balanceFontSize,
                        fontWeight: FontWeight.w700,
                        color: const Color(0xFF0B3D2E),
                      ),
                    ),
                  ),
                ),
              ),
              // 底部：GMB 右下
              const Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  SizedBox(width: 16),
                  Text(
                    'GMB',
                    style: TextStyle(
                      fontSize: 12,
                      fontWeight: FontWeight.w500,
                      color: Colors.black38,
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
              color: const Color(0xFFFFF8E1),
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
                padding: const EdgeInsets.symmetric(horizontal: 8),
                children: [
                  GridView.builder(
                    shrinkWrap: true,
                    physics: const NeverScrollableScrollPhysics(),
                    gridDelegate:
                        const SliverGridDelegateWithFixedCrossAxisCount(
                      crossAxisCount: 2,
                      mainAxisSpacing: 8,
                      crossAxisSpacing: 8,
                      childAspectRatio: 1.8,
                    ),
                    itemCount: wallets.length,
                    itemBuilder: (context, i) {
                      final card = _buildWalletCard(
                        wallets[i],
                        isLast: i == wallets.length - 1,
                      );
                      if (_isSelectionMode) return card;
                      return Dismissible(
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
                        child: card,
                      );
                    },
                  ),
                  const SizedBox(height: 12),
                ],
              ),
            );
          },
        ),
      ),
    );
  }
}

/// 自绘二维码，中央 [hollowSize] 像素区域不绘制任何模块（真正留白）。
class _HollowQrPainter extends CustomPainter {
  _HollowQrPainter({required this.data, required this.hollowSize});

  final String data;
  final double hollowSize;

  @override
  void paint(Canvas canvas, Size size) {
    final qrCode = QrCode.fromData(
      data: data,
      errorCorrectLevel: QrErrorCorrectLevel.H,
    );
    final qrImage = QrImage(qrCode);
    final moduleCount = qrImage.moduleCount;
    final moduleSize = size.width / moduleCount;
    final paint = Paint()..color = const Color(0xFF000000);

    // 中央留白区域（以像素为单位转换为模块范围）
    final hollowModules = (hollowSize / moduleSize).ceil();
    final hollowStart = (moduleCount - hollowModules) ~/ 2;
    final hollowEnd = hollowStart + hollowModules;

    for (var row = 0; row < moduleCount; row++) {
      for (var col = 0; col < moduleCount; col++) {
        if (qrImage.isDark(row, col)) {
          // 跳过中央区域
          if (row >= hollowStart &&
              row < hollowEnd &&
              col >= hollowStart &&
              col < hollowEnd) {
            continue;
          }
          canvas.drawRect(
            Rect.fromLTWH(
              col * moduleSize,
              row * moduleSize,
              moduleSize,
              moduleSize,
            ),
            paint,
          );
        }
      }
    }
  }

  @override
  bool shouldRepaint(_HollowQrPainter oldDelegate) {
    return oldDelegate.data != data || oldDelegate.hollowSize != hollowSize;
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
  final OnchainTradeRepository _txRepo = LocalOnchainTradeRepository();
  final GlobalKey _qrKey = GlobalKey();
  late String _walletName;
  bool _isEditingName = false;
  bool _hasChanged = false;
  bool _isSavingQr = false;
  late final TextEditingController _nameEditController;
  List<OnchainTxRecord> _recentRecords = const [];
  bool _screenshotGuardActive = false;

  @override
  void dispose() {
    _nameEditController.dispose();
    if (_screenshotGuardActive) ScreenshotGuard.disable();
    super.dispose();
  }

  @override
  void initState() {
    super.initState();
    _walletName = widget.wallet.walletName;
    _nameEditController = TextEditingController(text: _walletName);
    _loadRecentRecords();
  }

  Future<void> _loadRecentRecords() async {
    final all = await _txRepo.listRecent();
    final addr = widget.wallet.address.toLowerCase();
    final filtered = all
        .where((r) =>
            r.fromAddress.toLowerCase() == addr ||
            r.toAddress.toLowerCase() == addr)
        .take(5)
        .toList(growable: false);
    if (!mounted) return;
    setState(() {
      _recentRecords = filtered;
    });
  }

  Future<void> _onMenuAction(String action) async {
    switch (action) {
      case 'seed':
        await _revealSecret('私钥', () async {
          final seed = await _walletService.getSeedHex(widget.wallet.walletIndex);
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
            style: TextButton.styleFrom(foregroundColor: Colors.red),
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
                decoration: BoxDecoration(
                  color: Colors.red.shade50,
                  borderRadius: BorderRadius.circular(8),
                  border: Border.all(color: Colors.red.shade200),
                ),
                child: Text(
                  value ?? '无数据',
                  style: const TextStyle(fontSize: 14, fontFamily: 'monospace'),
                ),
              ),
              const SizedBox(height: 8),
              const Text(
                '请手抄备份，不支持复制',
                style: TextStyle(color: Colors.red, fontSize: 12),
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
      // 双向同步：如果该钱包是通信账户，同步更新用户资料中的昵称
      final profileService = UserProfileService();
      final profileState = await profileService.getState();
      if (profileState.communicationWalletIndex == widget.wallet.walletIndex) {
        await profileService.updateCommunicationWalletName(trimmed);
      }
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
    setState(() {
      _isSavingQr = true;
    });
    try {
      final boundary =
          _qrKey.currentContext?.findRenderObject() as RenderRepaintBoundary?;
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
        setState(() {
          _isSavingQr = false;
        });
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
          actions: [
            if (widget.wallet.isHotWallet)
              PopupMenuButton<String>(
                icon: const Icon(Icons.more_vert),
                onSelected: _onMenuAction,
                itemBuilder: (_) => const [
                  PopupMenuItem(value: 'seed', child: Text('查看私钥')),
                  PopupMenuItem(value: 'mnemonic', child: Text('查看助记词')),
                ],
              ),
          ],
        ),
        body: ListView(
          padding: const EdgeInsets.all(16),
          children: [
            // 余额卡片（含钱包名称）
            Container(
              width: double.infinity,
              padding: const EdgeInsets.symmetric(vertical: 14, horizontal: 14),
              decoration: BoxDecoration(
                color: const Color(0xFFE9F5EF),
                borderRadius: BorderRadius.circular(12),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  // 钱包名称（左上角，点击可编辑）
                  _isEditingName
                      ? SizedBox(
                          width: 180,
                          child: TextField(
                            controller: _nameEditController,
                            autofocus: true,
                            style: const TextStyle(
                              fontSize: 15,
                              fontWeight: FontWeight.w600,
                              color: Color(0xFF0B3D2E),
                            ),
                            decoration: const InputDecoration(
                              border: UnderlineInputBorder(),
                              isDense: true,
                              contentPadding: EdgeInsets.symmetric(vertical: 4),
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
                              fontSize: 15,
                              fontWeight: FontWeight.w600,
                              color: Color(0xFF0B3D2E),
                            ),
                          ),
                        ),
                  const SizedBox(height: 8),
                  Center(
                    child: Row(
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
                  ),
                ],
              ),
            ),
            const SizedBox(height: 20),
            // 钱包二维码（中央真正留白 + 下载按钮）
            Center(
              child: Stack(
                alignment: Alignment.center,
                children: [
                  RepaintBoundary(
                    key: _qrKey,
                    child: Container(
                      color: Colors.white,
                      padding: const EdgeInsets.all(8),
                      child: CustomPaint(
                        size: const Size(240, 240),
                        painter: _HollowQrPainter(
                          data: TransferQrPayload(
                            to: widget.wallet.address,
                            name: _walletName,
                          ).toRawJson(),
                          hollowSize: 48,
                        ),
                      ),
                    ),
                  ),
                  Container(
                    width: 36,
                    height: 36,
                    decoration: BoxDecoration(
                      color: Colors.white,
                      borderRadius: BorderRadius.circular(4),
                      border: Border.all(
                        color: Colors.grey.shade300,
                        width: 1,
                      ),
                    ),
                    child: IconButton(
                      tooltip: '保存二维码到相册',
                      constraints: const BoxConstraints(),
                      padding: EdgeInsets.zero,
                      onPressed: _isSavingQr ? null : _saveQrToGallery,
                      icon: _isSavingQr
                          ? const SizedBox(
                              width: 16,
                              height: 16,
                              child: CircularProgressIndicator(strokeWidth: 2),
                            )
                          : SvgPicture.asset(
                              'assets/icons/download.svg',
                              width: 18,
                              height: 18,
                              colorFilter: const ColorFilter.mode(
                                Colors.black54,
                                BlendMode.srcIn,
                              ),
                            ),
                    ),
                  ),
                ],
              ),
            ),
            const SizedBox(height: 12),
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
            const SizedBox(height: 24),
            // 交易记录标题行
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
              child: Padding(
                padding: const EdgeInsets.symmetric(vertical: 8),
                child: Row(
                  children: [
                    const Text(
                      '交易记录',
                      style: TextStyle(
                        fontSize: 16,
                        fontWeight: FontWeight.w700,
                      ),
                    ),
                    const Spacer(),
                    Icon(Icons.chevron_right,
                        size: 20, color: Colors.grey.shade400),
                  ],
                ),
              ),
            ),
            const Divider(height: 1),
            // 最近交易记录
            if (_recentRecords.isEmpty)
              const Padding(
                padding: EdgeInsets.symmetric(vertical: 32),
                child: Center(
                  child: Text(
                    '暂无交易记录',
                    style: TextStyle(color: Colors.grey),
                  ),
                ),
              )
            else
              ...List.generate(_recentRecords.length, (index) {
                final record = _recentRecords[index];
                return Column(
                  children: [
                    TxRecordTile(
                      record: record,
                      selfAddress: widget.wallet.address,
                      onTap: () {
                        Navigator.of(context).push(
                          MaterialPageRoute(
                            builder: (_) => TxRecordDetailPage(
                              record: record,
                              selfAddress: widget.wallet.address,
                            ),
                          ),
                        );
                      },
                    ),
                    if (index < _recentRecords.length - 1)
                      const Divider(height: 1),
                  ],
                );
              }),
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
              style: TextStyle(color: Colors.grey.shade600, fontSize: 12),
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
      // 导入成功后清空剪贴板，防止助记词残留
      await Clipboard.setData(const ClipboardData(text: ''));
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
              style: const TextStyle(color: Colors.red),
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
              style: TextStyle(color: Colors.black54, fontSize: 13),
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


