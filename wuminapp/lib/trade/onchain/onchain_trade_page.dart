import 'dart:async';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';
import 'package:wuminapp_mobile/rpc/onchain.dart';
import 'package:wuminapp_mobile/trade/onchain/onchain_trade_models.dart';
import 'package:wuminapp_mobile/trade/onchain/onchain_trade_service.dart';
import 'package:wuminapp_mobile/qr/pages/qr_scan_page.dart';
import 'package:wuminapp_mobile/qr/pages/qr_sign_session_page.dart';
import 'package:wuminapp_mobile/signer/qr_signer.dart';
import 'package:wuminapp_mobile/user/user.dart' show ContactBookPage;
import 'package:wuminapp_mobile/user/user_service.dart' show UserContact;
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';
import 'package:wuminapp_mobile/wallet/ui/wallet_page.dart';

class OnchainTradePage extends StatefulWidget {
  const OnchainTradePage({super.key, this.initialToAddress});

  /// 预填收款地址（从通讯录等入口跳转时使用）。
  final String? initialToAddress;

  @override
  State<OnchainTradePage> createState() => _OnchainTradePageState();
}

class _OnchainTradePageState extends State<OnchainTradePage> {
  static const Color _brandPrimaryColor = Color(0xFF007A74);
  static const Color _inputFieldColor = Color(0xFFF7F7F7);
  static const Color _cardBgColor = Color(0xFFF5F5F5);
  static const Color _inputTextColor = Colors.black87;
  static const Color _inputBorderColor = Color(0xFFD0D0D0);

  /// 链的 SS58 地址前缀。
  static const int _ss58Prefix = 2027;

  /// 链上存在性保证金（Existential Deposit）= 111 分 = 1.11 元。
  /// 来源：primitives::core_const::ACCOUNT_EXISTENTIAL_DEPOSIT = 111
  static const double _edYuan = 1.11;
  final OnchainTradeService _tradeService = OnchainTradeService();
  final TextEditingController _toController = TextEditingController();
  final TextEditingController _amountController = TextEditingController();
  static const List<String> _symbols = ['GMB'];
  String _selectedSymbol = 'GMB';

  WalletProfile? _currentWallet;
  bool _loadingWallet = true;
  bool _submitting = false;
  bool _syncing = false;
  DateTime? _lastSyncedAt;
  List<OnchainTxRecord> _records = <OnchainTxRecord>[];
  Timer? _syncTimer;

  @override
  void initState() {
    super.initState();
    if (widget.initialToAddress != null) {
      _toController.text = widget.initialToAddress!;
    }
    _bootstrap();
    _syncTimer = Timer.periodic(
      const Duration(seconds: 6),
      (_) => _reloadRecords(syncPending: true),
    );
  }

  @override
  void dispose() {
    _syncTimer?.cancel();
    _toController.dispose();
    _amountController.dispose();
    super.dispose();
  }

  Future<void> _bootstrap() async {
    await _reloadWallet();
    await _reloadRecords(syncPending: true);
  }

  Future<void> _reloadRecords({
    bool syncPending = false,
  }) async {
    if (_syncing) {
      return;
    }
    _syncing = true;
    List<OnchainTxRecord> records = const <OnchainTxRecord>[];
    try {
      records = syncPending
          ? await _tradeService.refreshPendingRecords()
          : await _tradeService.listRecentRecords();
    } finally {
      _syncing = false;
    }
    if (!mounted) {
      return;
    }
    setState(() {
      _records = records;
      _lastSyncedAt = DateTime.now();
    });
  }

  Future<void> _reloadWallet() async {
    final wallet = await _tradeService.getCurrentWallet();
    if (!mounted) {
      return;
    }
    setState(() {
      _currentWallet = wallet;
      _loadingWallet = false;
    });
  }

  Future<void> _openMyWalletPage() async {
    final changed = await Navigator.of(context).push<bool>(
      MaterialPageRoute(
        builder: (_) => const MyWalletPage(selectForTrade: true),
      ),
    );
    if (changed == true) {
      await _reloadWallet();
    }
  }

  Future<void> _openContactsPage() async {
    final contact = await Navigator.of(context).push<UserContact>(
      MaterialPageRoute(
        builder: (_) => ContactBookPage(
          selfAccountPubkeyHex: _currentWallet?.pubkeyHex ?? '',
          selectForTrade: true,
        ),
      ),
    );
    if (!mounted || contact == null) return;
    setState(() {
      _toController.text = contact.accountPubkeyHex;
    });
  }

  Future<void> _openTradeRecordsPage() async {
    await Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => TradeRecordsPage(
          initialRecords: _records,
          initialLastSyncedAt: _lastSyncedAt,
        ),
      ),
    );
  }

  Future<void> _scanToAddress() async {
    final result = await Navigator.of(context).push<QrScanTransferResult>(
      MaterialPageRoute(
          builder: (_) => const QrScanPage(mode: QrScanMode.transfer)),
    );
    if (result == null || !mounted) {
      return;
    }
    setState(() {
      _toController.text = result.toAddress;
      if (result.amount != null && result.amount!.isNotEmpty) {
        _amountController.text = result.amount!;
      }
      if (result.symbol != null && result.symbol!.isNotEmpty) {
        _selectedSymbol = result.symbol!;
      }
    });
  }

  Future<void> _submit() async {
    if (_loadingWallet) {
      return;
    }
    if (_currentWallet == null) {
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(const SnackBar(content: Text('请先创建或导入钱包')));
      await _openMyWalletPage();
      return;
    }

    final toAddress = _toController.text.trim();
    final amountText = _amountController.text.trim();
    if (toAddress.isEmpty || amountText.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('请先填写完整的收款地址和金额')),
      );
      return;
    }

    // SS58 地址校验（prefix 2027）
    try {
      final decoded = Keyring().decodeAddress(toAddress);
      // 验证 prefix：重新编码后比对
      final reEncoded = Keyring().encodeAddress(decoded, _ss58Prefix);
      if (reEncoded != toAddress) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('收款地址不是本链地址（SS58 前缀不匹配）')),
        );
        return;
      }
    } catch (_) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('收款地址格式错误，请输入有效的 SS58 地址')),
      );
      return;
    }

    final amount = double.tryParse(amountText);
    if (amount == null || amount <= 0) {
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(const SnackBar(content: Text('金额格式不正确')));
      return;
    }

    // 预估手续费，展示确认对话框
    final estimatedFee = OnchainRpc.estimateTransferFeeYuan(amount);

    // 余额校验：转账金额 + 手续费 ≤ 可用余额（余额 - ED）
    final availableBalance = _currentWallet!.balance - _edYuan;
    if (amount + estimatedFee > availableBalance) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(
            '余额不足，可用余额：${availableBalance.toStringAsFixed(2)} 元'
            '（已扣除 ED ${_edYuan.toStringAsFixed(2)} 元）',
          ),
        ),
      );
      return;
    }
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (_) => AlertDialog(
        title: const Text('确认交易'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('转账金额：$amount $_selectedSymbol'),
            const SizedBox(height: 4),
            Text('预估手续费：$estimatedFee $_selectedSymbol'),
            const Divider(height: 16),
            Text(
              '合计：${(amount + estimatedFee).toStringAsFixed(2)} $_selectedSymbol',
              style: const TextStyle(fontWeight: FontWeight.w700),
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(_, false),
            child: const Text('取消'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(_, true),
            child: const Text('确认'),
          ),
        ],
      ),
    );
    if (confirmed != true) return;

    setState(() {
      _submitting = true;
    });
    try {
      final wallet = _currentWallet!;
      final Future<Uint8List> Function(Uint8List payload) signCallback;

      if (wallet.isHotWallet) {
        // 热钱包：通过 WalletManager 签名，seed 不出类。
        final walletManager = WalletManager();
        signCallback = (payload) =>
            walletManager.signWithWallet(wallet.walletIndex, payload);
      } else {
        // 冷钱包：扫码签名。
        signCallback = (Uint8List payload) async {
          final qrSigner = QrSigner();
          final requestId = QrSigner.generateRequestId(prefix: 'tx-');
          final toAddr = _toController.text.trim();
          final amountText = _amountController.text.trim();
          // 统一格式化为两位小数，与 PayloadDecoder._fenToYuan 对齐
          final amountFormatted =
              (double.tryParse(amountText) ?? 0).toStringAsFixed(2);
          final rv = await ChainRpc().fetchRuntimeVersion();
          final request = qrSigner.buildRequest(
            requestId: requestId,
            account: wallet.address,
            pubkey: '0x${wallet.pubkeyHex}',
            payloadHex: '0x${_toHex(payload)}',
            specVersion: rv.specVersion,
            display: {
              'action': 'transfer',
              'summary': '转账 $amountFormatted $_selectedSymbol 给 $toAddr',
              'fields': {
                'to': toAddr,
                'amount_yuan': amountFormatted,
                'symbol': _selectedSymbol,
              },
            },
          );
          final requestJson = qrSigner.encodeRequest(request);

          final response = await Navigator.push<QrSignResponse>(
            context,
            MaterialPageRoute(
              builder: (_) => QrSignSessionPage(
                request: request,
                requestJson: requestJson,
                expectedPubkey: '0x${wallet.pubkeyHex}',
              ),
            ),
          );

          if (response == null) {
            throw Exception('签名已取消');
          }

          return Uint8List.fromList(_hexToBytes(response.signature));
        };
      }

      final record = await _tradeService.submitTransfer(
        OnchainTransferDraft(
          toAddress: toAddress,
          amount: amount,
          symbol: _selectedSymbol,
        ),
        sign: signCallback,
      );
      if (!mounted) {
        return;
      }
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text('签名成功，交易已发送，tx=${record.txHash}')));
      _toController.clear();
      _amountController.clear();
      await _reloadRecords(syncPending: true);
    } on WalletAuthException catch (e) {
      if (!mounted) {
        return;
      }
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(e.message)),
      );
    } on OnchainTradeException catch (e) {
      if (!mounted) {
        return;
      }
      final message = e.code == OnchainTradeErrorCode.broadcastFailed
          ? '交易发送失败：${e.message}'
          : '签名失败：${e.message}';
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(message)),
      );
    } catch (_) {
      if (!mounted) {
        return;
      }
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(const SnackBar(content: Text('签名失败，请稍后重试')));
    } finally {
      if (mounted) {
        setState(() {
          _submitting = false;
        });
      }
    }
  }

  Color _statusColor(OnchainTxStatus status) {
    switch (status) {
      case OnchainTxStatus.pending:
        return Colors.orange.shade700;
      case OnchainTxStatus.confirmed:
        return Colors.green.shade700;
      case OnchainTxStatus.failed:
        return Colors.red.shade700;
    }
  }

  int _countByStatus(OnchainTxStatus status) {
    return _records.where((it) => it.status == status).length;
  }

  Widget _buildSubmitCard() {
    return Card(
      color: _cardBgColor,
      child: Padding(
        padding: const EdgeInsets.fromLTRB(12, 16, 12, 16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            if (_currentWallet != null)
              Padding(
                padding: const EdgeInsets.only(left: 4, bottom: 12),
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.center,
                  children: [
                    Text(
                      '可用余额：${_currentWallet!.balance.toStringAsFixed(2)} 元',
                      style: TextStyle(
                        fontSize: 13,
                        color: Colors.grey.shade600,
                      ),
                    ),
                    const Spacer(),
                    Container(
                      width: 32,
                      height: 18,
                      decoration: BoxDecoration(
                        color: _brandPrimaryColor,
                        borderRadius: BorderRadius.circular(100),
                      ),
                      child: const Center(
                        child: Text(
                          '链上',
                          style: TextStyle(
                            fontSize: 8,
                            color: Colors.white,
                            fontWeight: FontWeight.w600,
                            height: 1.0,
                          ),
                        ),
                      ),
                    ),
                  ],
                ),
              ),
            TextField(
              controller: _toController,
              decoration: InputDecoration(
                enabledBorder: const OutlineInputBorder(
                  borderSide: BorderSide(color: _inputBorderColor),
                ),
                focusedBorder: const OutlineInputBorder(
                  borderSide: BorderSide(color: _brandPrimaryColor),
                ),
                labelText: '收款地址',
                filled: true,
                fillColor: _inputFieldColor,
                suffixIcon: IconButton(
                  tooltip: '扫码填入收款地址',
                  onPressed: _scanToAddress,
                  icon: SvgPicture.asset(
                    'assets/icons/scan-line.svg',
                    width: 18,
                    height: 18,
                  ),
                ),
              ),
            ),
            const SizedBox(height: 12),
            Row(
              children: [
                Expanded(
                  child: TextField(
                    controller: _amountController,
                    keyboardType: TextInputType.number,
                    style: const TextStyle(color: _inputTextColor),
                    decoration: const InputDecoration(
                      enabledBorder: OutlineInputBorder(
                        borderSide: BorderSide(color: _inputBorderColor),
                      ),
                      focusedBorder: OutlineInputBorder(
                        borderSide: BorderSide(color: _brandPrimaryColor),
                      ),
                      labelText: '金额',
                      filled: true,
                      fillColor: _inputFieldColor,
                    ),
                  ),
                ),
                const SizedBox(width: 10),
                SizedBox(
                  width: 120,
                  child: InputDecorator(
                    decoration: const InputDecoration(
                      enabledBorder: OutlineInputBorder(
                        borderSide: BorderSide(color: _inputBorderColor),
                      ),
                      labelText: '币种',
                      filled: true,
                      fillColor: _inputFieldColor,
                      contentPadding: EdgeInsets.symmetric(vertical: 16),
                    ),
                    child: Text(
                      _selectedSymbol,
                      textAlign: TextAlign.center,
                      style: const TextStyle(
                        color: Colors.black45,
                        fontSize: 16,
                      ),
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            SizedBox(
              width: double.infinity,
              child: FilledButton(
                style: FilledButton.styleFrom(
                  backgroundColor: _brandPrimaryColor,
                ),
                onPressed:
                    (_submitting || _loadingWallet || _currentWallet == null)
                        ? null
                        : _submit,
                child: Text(_submitting ? '签名中' : '签名交易'),
              ),
            ),
            const SizedBox(height: 12),
            _buildTradeStatusRow(),
          ],
        ),
      ),
    );
  }

  Widget _buildTradeStatusRow() {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        Expanded(
          child: Wrap(
            spacing: 16,
            runSpacing: 8,
            children: [
              _buildStatusText(
                '待确认',
                _countByStatus(OnchainTxStatus.pending),
                _statusColor(OnchainTxStatus.pending),
              ),
              _buildStatusText(
                '已确认',
                _countByStatus(OnchainTxStatus.confirmed),
                _statusColor(OnchainTxStatus.confirmed),
              ),
              _buildStatusText(
                '失败',
                _countByStatus(OnchainTxStatus.failed),
                _statusColor(OnchainTxStatus.failed),
              ),
            ],
          ),
        ),
        InkWell(
          onTap: _openTradeRecordsPage,
          borderRadius: BorderRadius.circular(8),
          child: const Padding(
            padding: EdgeInsets.all(6),
            child: Icon(Icons.chevron_right, size: 22),
          ),
        ),
      ],
    );
  }

  Widget _buildStatusText(String label, int count, Color color) {
    return Text(
      '$label $count',
      style: TextStyle(
        color: color,
        fontWeight: FontWeight.w700,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: Column(
          children: [
            Padding(
              padding: const EdgeInsets.fromLTRB(4, 10, 4, 0),
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.center,
                children: [
                  IconButton(
                    tooltip: '我的通讯录',
                    onPressed: _openContactsPage,
                    icon: SvgPicture.asset(
                      'assets/icons/contact-round.svg',
                      width: 22,
                      height: 22,
                    ),
                  ),
                  const Expanded(
                    child: Center(
                      child: Text(
                        '交易',
                        style: TextStyle(
                            fontSize: 20, fontWeight: FontWeight.w700),
                      ),
                    ),
                  ),
                  IconButton(
                    tooltip: '选择交易钱包',
                    onPressed: _openMyWalletPage,
                    icon: SvgPicture.asset(
                      'assets/icons/wallet.svg',
                      width: 22,
                      height: 22,
                    ),
                  ),
                ],
              ),
            ),
            Expanded(
              child: RefreshIndicator(
                onRefresh: () => _reloadRecords(syncPending: true),
                child: ListView(
                  physics: const AlwaysScrollableScrollPhysics(),
                  padding: const EdgeInsets.all(16),
                  children: [
                    // 多签交易入口
                    Card(
                      color: _cardBgColor,
                      child: InkWell(
                        onTap: () {
                          ScaffoldMessenger.of(context).showSnackBar(
                            const SnackBar(content: Text('多签交易功能开发中')),
                          );
                        },
                        borderRadius: BorderRadius.circular(12),
                        child: Padding(
                          padding: const EdgeInsets.fromLTRB(16, 14, 8, 14),
                          child: Row(
                            children: [
                              const Text(
                                '多签交易',
                                style: TextStyle(
                                  fontSize: 16,
                                  fontWeight: FontWeight.w600,
                                ),
                              ),
                              const Spacer(),
                              const Icon(Icons.chevron_right, size: 22),
                            ],
                          ),
                        ),
                      ),
                    ),
                    const SizedBox(height: 12),
                    if (_currentWallet == null && !_loadingWallet)
                      Padding(
                        padding: const EdgeInsets.only(bottom: 12),
                        child: Card(
                          color: _cardBgColor,
                          child: Padding(
                            padding: const EdgeInsets.fromLTRB(12, 12, 12, 12),
                            child: Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                const Text('未检测到钱包，无法执行链上签名与交易广播'),
                                const SizedBox(height: 8),
                                FilledButton(
                                  onPressed: _openMyWalletPage,
                                  child: const Text('去创建/导入钱包'),
                                ),
                              ],
                            ),
                          ),
                        ),
                      ),
                    _buildSubmitCard(),
                    const SizedBox(height: 24),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class TradeRecordsPage extends StatefulWidget {
  const TradeRecordsPage({
    super.key,
    required this.initialRecords,
    this.initialLastSyncedAt,
  });

  final List<OnchainTxRecord> initialRecords;
  final DateTime? initialLastSyncedAt;

  @override
  State<TradeRecordsPage> createState() => _TradeRecordsPageState();
}

class _TradeRecordsPageState extends State<TradeRecordsPage> {
  static const Color _brandPrimaryColor = Color(0xFF007A74);
  static const Color _cardBgColor = Color(0xFFF5F5F5);
  final OnchainTradeService _tradeService = OnchainTradeService();

  bool _loading = true;
  bool _syncing = false;
  DateTime? _lastSyncedAt;
  OnchainTxStatus? _statusFilter;
  List<OnchainTxRecord> _records = <OnchainTxRecord>[];

  @override
  void initState() {
    super.initState();
    _records = widget.initialRecords;
    _lastSyncedAt = widget.initialLastSyncedAt;
    _loading = widget.initialRecords.isEmpty;
    _reloadRecords(syncPending: true, silent: widget.initialRecords.isNotEmpty);
  }

  Future<void> _reloadRecords({
    bool syncPending = false,
    bool silent = false,
  }) async {
    if (_syncing) {
      return;
    }
    if (!silent && mounted) {
      setState(() {
        _loading = true;
      });
    }
    _syncing = true;
    List<OnchainTxRecord> records = const <OnchainTxRecord>[];
    try {
      records = syncPending
          ? await _tradeService.refreshPendingRecords()
          : await _tradeService.listRecentRecords();
    } finally {
      _syncing = false;
    }
    if (!mounted) {
      return;
    }
    setState(() {
      _records = records;
      _lastSyncedAt = DateTime.now();
      _loading = false;
    });
  }

  List<OnchainTxRecord> _filteredRecords() {
    if (_statusFilter == null) {
      return _records;
    }
    return _records.where((it) => it.status == _statusFilter).toList();
  }

  String _statusLabel(OnchainTxStatus status) {
    switch (status) {
      case OnchainTxStatus.pending:
        return '待确认';
      case OnchainTxStatus.confirmed:
        return '已确认';
      case OnchainTxStatus.failed:
        return '失败';
    }
  }

  Color _statusColor(OnchainTxStatus status) {
    switch (status) {
      case OnchainTxStatus.pending:
        return Colors.orange.shade700;
      case OnchainTxStatus.confirmed:
        return Colors.green.shade700;
      case OnchainTxStatus.failed:
        return Colors.red.shade700;
    }
  }

  String _shortAddress(String address) {
    if (address.length <= 14) {
      return address;
    }
    return '${address.substring(0, 8)}...${address.substring(address.length - 6)}';
  }

  String _formatTime(DateTime dt) {
    final y = dt.year.toString().padLeft(4, '0');
    final m = dt.month.toString().padLeft(2, '0');
    final d = dt.day.toString().padLeft(2, '0');
    final h = dt.hour.toString().padLeft(2, '0');
    final min = dt.minute.toString().padLeft(2, '0');
    final s = dt.second.toString().padLeft(2, '0');
    return '$y-$m-$d $h:$min:$s';
  }

  Widget _buildFilterRow() {
    final options = <(String, OnchainTxStatus?)>[
      ('全部', null),
      ('待确认', OnchainTxStatus.pending),
      ('已确认', OnchainTxStatus.confirmed),
      ('失败', OnchainTxStatus.failed),
    ];
    return Wrap(
      spacing: 8,
      children: [
        for (final (label, value) in options)
          ChoiceChip(
            selected: _statusFilter == value,
            selectedColor: _brandPrimaryColor,
            label: Text(
              label,
              style: TextStyle(
                color: _statusFilter == value ? Colors.white : Colors.black87,
                fontWeight:
                    _statusFilter == value ? FontWeight.w700 : FontWeight.w500,
              ),
            ),
            onSelected: (_) {
              setState(() {
                _statusFilter = value;
              });
            },
          ),
      ],
    );
  }

  Widget _buildRecordsSection() {
    if (_loading) {
      return const Padding(
        padding: EdgeInsets.symmetric(vertical: 24),
        child: Center(child: CircularProgressIndicator()),
      );
    }

    final records = _filteredRecords();
    if (records.isEmpty) {
      return const Card(
        color: _cardBgColor,
        child: Padding(
          padding: EdgeInsets.all(12),
          child: Text('暂无交易记录'),
        ),
      );
    }

    return Column(
      children: [
        for (final item in records)
          Card(
            color: _cardBgColor,
            child: InkWell(
              borderRadius: BorderRadius.circular(12),
              onTap: () {
                Navigator.of(context).push(
                  MaterialPageRoute(
                    builder: (_) => TradeRecordDetailPage(record: item),
                  ),
                );
              },
              child: Padding(
                padding: const EdgeInsets.all(12),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        Expanded(
                          child: Text(
                            '${item.amount} ${item.symbol}',
                            style: const TextStyle(
                              fontWeight: FontWeight.w700,
                              fontSize: 16,
                            ),
                          ),
                        ),
                        Container(
                          padding: const EdgeInsets.symmetric(
                            horizontal: 8,
                            vertical: 4,
                          ),
                          decoration: BoxDecoration(
                            color: _statusColor(item.status)
                                .withValues(alpha: 0.12),
                            borderRadius: BorderRadius.circular(999),
                          ),
                          child: Text(
                            _statusLabel(item.status),
                            style: TextStyle(
                              color: _statusColor(item.status),
                              fontWeight: FontWeight.w700,
                            ),
                          ),
                        ),
                      ],
                    ),
                    if (item.estimatedFee != null)
                      Padding(
                        padding: const EdgeInsets.only(top: 4),
                        child: Text(
                          '手续费：${item.estimatedFee} ${item.symbol}',
                          style: const TextStyle(
                            color: Colors.black54,
                            fontSize: 13,
                          ),
                        ),
                      ),
                    const SizedBox(height: 8),
                    Text('from: ${_shortAddress(item.fromAddress)}'),
                    Text('to: ${_shortAddress(item.toAddress)}'),
                    Text('tx: ${item.txHash}'),
                    Text('time: ${_formatTime(item.createdAt)}'),
                    if (item.failureReason != null) ...[
                      const SizedBox(height: 4),
                      Text(
                        'error: ${item.failureReason}',
                        style: const TextStyle(color: Colors.red),
                      ),
                    ],
                  ],
                ),
              ),
            ),
          ),
      ],
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('交易记录'),
        centerTitle: true,
      ),
      body: RefreshIndicator(
        onRefresh: () => _reloadRecords(syncPending: true),
        child: ListView(
          physics: const AlwaysScrollableScrollPhysics(),
          padding: const EdgeInsets.all(16),
          children: [
            _buildFilterRow(),
            if (_lastSyncedAt != null) ...[
              const SizedBox(height: 10),
              Text(
                '最近同步：${_formatTime(_lastSyncedAt!)}',
                style: const TextStyle(color: Colors.black54, fontSize: 12),
              ),
            ],
            const SizedBox(height: 8),
            _buildRecordsSection(),
          ],
        ),
      ),
    );
  }
}

class TradeRecordDetailPage extends StatelessWidget {
  const TradeRecordDetailPage({
    super.key,
    required this.record,
  });

  final OnchainTxRecord record;

  String _statusLabel(OnchainTxStatus status) {
    switch (status) {
      case OnchainTxStatus.pending:
        return '待确认';
      case OnchainTxStatus.confirmed:
        return '已确认';
      case OnchainTxStatus.failed:
        return '失败';
    }
  }

  Color _statusColor(OnchainTxStatus status) {
    switch (status) {
      case OnchainTxStatus.pending:
        return Colors.orange.shade700;
      case OnchainTxStatus.confirmed:
        return Colors.green.shade700;
      case OnchainTxStatus.failed:
        return Colors.red.shade700;
    }
  }

  String _formatTime(DateTime dt) {
    final y = dt.year.toString().padLeft(4, '0');
    final m = dt.month.toString().padLeft(2, '0');
    final d = dt.day.toString().padLeft(2, '0');
    final h = dt.hour.toString().padLeft(2, '0');
    final min = dt.minute.toString().padLeft(2, '0');
    final s = dt.second.toString().padLeft(2, '0');
    return '$y-$m-$d $h:$min:$s';
  }

  Widget _buildDetailRow(String label, String value, {Color? color}) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            label,
            style: const TextStyle(
              color: Colors.black54,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 4),
          SelectableText(
            value,
            style: TextStyle(
              color: color ?? Colors.black87,
              height: 1.5,
            ),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('交易记录详情'),
        centerTitle: true,
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  _buildDetailRow('金额', '${record.amount} ${record.symbol}'),
                  if (record.estimatedFee != null)
                    _buildDetailRow(
                      '手续费',
                      '${record.estimatedFee} ${record.symbol}',
                    ),
                  _buildDetailRow(
                    '状态',
                    _statusLabel(record.status),
                    color: _statusColor(record.status),
                  ),
                  _buildDetailRow('付款地址', record.fromAddress),
                  _buildDetailRow('收款地址', record.toAddress),
                  _buildDetailRow('交易哈希', record.txHash),
                  _buildDetailRow('创建时间', _formatTime(record.createdAt)),
                  if (record.failureReason != null)
                    _buildDetailRow(
                      '失败原因',
                      record.failureReason!,
                      color: Colors.red,
                    ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}

String _toHex(List<int> bytes) {
  const chars = '0123456789abcdef';
  final buf = StringBuffer();
  for (final b in bytes) {
    buf
      ..write(chars[(b >> 4) & 0x0f])
      ..write(chars[b & 0x0f]);
  }
  return buf.toString();
}

List<int> _hexToBytes(String input) {
  final text = input.startsWith('0x') ? input.substring(2) : input;
  if (text.isEmpty || text.length.isOdd) return const <int>[];
  final out = <int>[];
  for (var i = 0; i < text.length; i += 2) {
    out.add(int.parse(text.substring(i, i + 2), radix: 16));
  }
  return out;
}
