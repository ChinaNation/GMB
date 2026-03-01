import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:wuminapp_mobile/login/models/login_exception.dart';
import 'package:wuminapp_mobile/login/services/login_sign_confirm_service.dart';
import 'package:wuminapp_mobile/pages/my_wallet_page.dart';
import 'package:wuminapp_mobile/services/wallet_service.dart';
import 'package:wuminapp_mobile/trade/onchain/models/onchain_trade_models.dart';
import 'package:wuminapp_mobile/trade/onchain/services/onchain_trade_service.dart';
import 'package:wuminapp_mobile/trade/pages/trade_qr_scan_page.dart';

class OnchainTradePage extends StatefulWidget {
  const OnchainTradePage({super.key});

  @override
  State<OnchainTradePage> createState() => _OnchainTradePageState();
}

class _OnchainTradePageState extends State<OnchainTradePage> {
  final OnchainTradeService _tradeService = OnchainTradeService();
  final LoginSignConfirmService _signConfirmService = LoginSignConfirmService();
  final TextEditingController _toController = TextEditingController();
  final TextEditingController _amountController = TextEditingController();
  static const List<String> _symbols = ['GMB'];
  String _selectedSymbol = 'GMB';

  WalletProfile? _currentWallet;
  bool _loadingWallet = true;
  bool _submitting = false;
  bool _loadingRecords = true;
  bool _syncing = false;
  DateTime? _lastSyncedAt;
  OnchainTxStatus? _statusFilter;
  List<OnchainTxRecord> _records = <OnchainTxRecord>[];
  Timer? _syncTimer;

  @override
  void initState() {
    super.initState();
    _bootstrap();
    _syncTimer = Timer.periodic(
      const Duration(seconds: 6),
      (_) => _reloadRecords(syncPending: true, silent: true),
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
    bool silent = false,
  }) async {
    if (_syncing) {
      return;
    }
    if (!silent && mounted) {
      setState(() {
        _loadingRecords = true;
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
      _loadingRecords = false;
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

  Future<void> _scanToAddress() async {
    final scanned = await Navigator.of(context).push<String>(
      MaterialPageRoute(builder: (_) => const TradeQrScanPage()),
    );
    if (scanned == null || scanned.isEmpty || !mounted) {
      return;
    }
    setState(() {
      _toController.text = scanned;
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

    final amount = double.tryParse(amountText);
    if (amount == null || amount <= 0) {
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(const SnackBar(content: Text('金额格式不正确')));
      return;
    }

    setState(() {
      _submitting = true;
    });
    try {
      await _signConfirmService.confirmBeforeSign(
        localizedReason: '请验证身份后执行交易签名',
      );

      final record = await _tradeService.submitTransfer(
        OnchainTransferDraft(
          toAddress: toAddress,
          amount: amount,
          symbol: _selectedSymbol,
        ),
      );
      if (!mounted) {
        return;
      }
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text('签名成功，交易已发送，tx=${record.txHash}')));
      _toController.clear();
      _amountController.clear();
      await _reloadRecords(syncPending: true, silent: true);
    } on LoginException catch (e) {
      if (!mounted) {
        return;
      }
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('签名失败：${e.message}')),
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

  List<OnchainTxRecord> _filteredRecords() {
    if (_statusFilter == null) {
      return _records;
    }
    return _records.where((it) => it.status == _statusFilter).toList();
  }

  int _countByStatus(OnchainTxStatus status) {
    return _records.where((it) => it.status == status).length;
  }

  Widget _buildWalletCard() {
    return Card(
      child: Padding(
        padding: const EdgeInsets.fromLTRB(12, 8, 12, 8),
        child: _loadingWallet
            ? const Text('加载当前钱包中...')
            : _currentWallet == null
                ? Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const Text('未检测到钱包，无法执行链上签名与交易广播'),
                      const SizedBox(height: 8),
                      FilledButton(
                        onPressed: _openMyWalletPage,
                        child: const Text('去创建/导入钱包'),
                      ),
                    ],
                  )
                : Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        children: [
                          Expanded(
                            child: Text(
                              '当前钱包：${_currentWallet!.walletName}',
                              style:
                                  const TextStyle(fontWeight: FontWeight.w700),
                            ),
                          ),
                          TextButton(
                            onPressed: _openMyWalletPage,
                            style: TextButton.styleFrom(
                              visualDensity: VisualDensity.compact,
                              tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                              padding:
                                  const EdgeInsets.symmetric(horizontal: 8),
                            ),
                            child: const Text('更换'),
                          ),
                        ],
                      ),
                      const SizedBox(height: 2),
                      Text('地址：${_currentWallet!.address}'),
                    ],
                  ),
      ),
    );
  }

  Widget _buildSubmitCard() {
    return Card(
      child: Padding(
        padding: const EdgeInsets.fromLTRB(12, 4, 12, 12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Expanded(
                  child: Text(
                    '发起链上转账',
                    style: TextStyle(fontWeight: FontWeight.w700, fontSize: 16),
                  ),
                ),
                IconButton(
                  tooltip: '扫码填入收款地址',
                  onPressed: _scanToAddress,
                  icon: SvgPicture.asset(
                    'assets/icons/scan-line.svg',
                    width: 18,
                    height: 18,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 4),
            TextField(
              controller: _toController,
              decoration: const InputDecoration(
                border: OutlineInputBorder(),
                labelText: '收款地址',
              ),
            ),
            const SizedBox(height: 12),
            Row(
              children: [
                Expanded(
                  child: TextField(
                    controller: _amountController,
                    keyboardType: TextInputType.number,
                    decoration: const InputDecoration(
                      border: OutlineInputBorder(),
                      labelText: '金额',
                    ),
                  ),
                ),
                const SizedBox(width: 10),
                SizedBox(
                  width: 120,
                  child: DropdownButtonFormField<String>(
                    initialValue: _selectedSymbol,
                    decoration: const InputDecoration(
                      border: OutlineInputBorder(),
                      labelText: '币种',
                    ),
                    items: _symbols
                        .map(
                          (symbol) => DropdownMenuItem<String>(
                            value: symbol,
                            child: Text(symbol),
                          ),
                        )
                        .toList(growable: false),
                    onChanged: (value) {
                      if (value == null) {
                        return;
                      }
                      setState(() {
                        _selectedSymbol = value;
                      });
                    },
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            SizedBox(
              width: double.infinity,
              child: FilledButton(
                onPressed:
                    (_submitting || _loadingWallet || _currentWallet == null)
                        ? null
                        : _submit,
                child: Text(_submitting ? '签名中' : '签名交易'),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildSummaryCard() {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              '交易状态',
              style: TextStyle(fontWeight: FontWeight.w700, fontSize: 16),
            ),
            const SizedBox(height: 10),
            Wrap(
              spacing: 10,
              runSpacing: 10,
              children: [
                _buildStatChip(
                    '总数', _records.length.toString(), Colors.blueGrey),
                _buildStatChip(
                  '待确认',
                  _countByStatus(OnchainTxStatus.pending).toString(),
                  Colors.orange,
                ),
                _buildStatChip(
                  '已确认',
                  _countByStatus(OnchainTxStatus.confirmed).toString(),
                  Colors.green,
                ),
                _buildStatChip(
                  '失败',
                  _countByStatus(OnchainTxStatus.failed).toString(),
                  Colors.red,
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildStatChip(String label, String value, MaterialColor color) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
      decoration: BoxDecoration(
        color: color.shade50,
        borderRadius: BorderRadius.circular(10),
      ),
      child: Text(
        '$label: $value',
        style: TextStyle(
          color: color.shade800,
          fontWeight: FontWeight.w600,
        ),
      ),
    );
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
            label: Text(label),
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
    if (_loadingRecords) {
      return const Padding(
        padding: EdgeInsets.symmetric(vertical: 24),
        child: Center(child: CircularProgressIndicator()),
      );
    }

    final records = _filteredRecords();
    if (records.isEmpty) {
      return const Card(
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
                          color:
                              _statusColor(item.status).withValues(alpha: 0.12),
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
      ],
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('链上交易'),
        centerTitle: true,
        actions: [
          IconButton(
            tooltip: '刷新',
            onPressed: () => _reloadRecords(syncPending: true),
            icon: const Icon(Icons.refresh),
          ),
        ],
      ),
      body: RefreshIndicator(
        onRefresh: () => _reloadRecords(syncPending: true),
        child: ListView(
          physics: const AlwaysScrollableScrollPhysics(),
          padding: const EdgeInsets.all(16),
          children: [
            _buildWalletCard(),
            const SizedBox(height: 12),
            _buildSubmitCard(),
            const SizedBox(height: 12),
            _buildSummaryCard(),
            const SizedBox(height: 12),
            const Text(
              '交易记录',
              style: TextStyle(fontWeight: FontWeight.w700, fontSize: 16),
            ),
            const SizedBox(height: 8),
            _buildFilterRow(),
            const SizedBox(height: 8),
            _buildRecordsSection(),
            if (_lastSyncedAt != null) ...[
              const SizedBox(height: 10),
              Text(
                '最近同步：${_formatTime(_lastSyncedAt!)}',
                style: const TextStyle(color: Colors.black54, fontSize: 12),
              ),
            ],
          ],
        ),
      ),
    );
  }
}
