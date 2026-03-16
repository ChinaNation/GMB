import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:wuminapp_mobile/trade/onchain/onchain_trade_models.dart';
import 'package:wuminapp_mobile/trade/onchain/onchain_trade_service.dart';
import 'package:wuminapp_mobile/trade/onchain/trade_qr_scan_page.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';
import 'package:wuminapp_mobile/wallet/ui/wallet_page.dart';

class OnchainTradePage extends StatefulWidget {
  const OnchainTradePage({super.key});

  @override
  State<OnchainTradePage> createState() => _OnchainTradePageState();
}

class _OnchainTradePageState extends State<OnchainTradePage> {
  static const Color _brandPrimaryColor = Color(0xFF007A74);
  static const Color _inputFieldColor = Color(0xFFF7F7F7);
  static const Color _cardBgColor = Color(0xFFF5F5F5);
  static const Color _inputTextColor = Colors.black87;
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
    await Navigator.of(context).push(
      MaterialPageRoute(builder: (_) => const _ContactsPlaceholderPage()),
    );
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

  Widget _buildWalletCard() {
    return Card(
      color: _cardBgColor,
      child: Padding(
        padding: const EdgeInsets.fromLTRB(12, 4, 12, 8),
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
                      Padding(
                        padding: const EdgeInsets.symmetric(vertical: 2),
                        child: Row(
                          children: [
                            Expanded(
                              child: Text(
                                '付款钱包：${_currentWallet!.walletName}',
                                style: const TextStyle(
                                    fontWeight: FontWeight.w700),
                              ),
                            ),
                            InkWell(
                              onTap: _openMyWalletPage,
                              borderRadius: BorderRadius.circular(6),
                              child: Padding(
                                padding: const EdgeInsets.all(6),
                                child: SvgPicture.asset(
                                  'assets/icons/arrow-right-left.svg',
                                  width: 20,
                                  height: 20,
                                  fit: BoxFit.contain,
                                ),
                              ),
                            ),
                          ],
                        ),
                      ),
                      Text(_currentWallet!.address),
                    ],
                  ),
      ),
    );
  }

  Widget _buildSubmitCard() {
    return Card(
      color: _cardBgColor,
      child: Padding(
        padding: const EdgeInsets.fromLTRB(12, 16, 12, 16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            TextField(
              controller: _toController,
              decoration: InputDecoration(
                border: const OutlineInputBorder(),
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
                      border: OutlineInputBorder(),
                      labelText: '金额',
                      filled: true,
                      fillColor: _inputFieldColor,
                    ),
                  ),
                ),
                const SizedBox(width: 10),
                SizedBox(
                  width: 120,
                  child: DropdownButtonFormField<String>(
                    initialValue: _selectedSymbol,
                    style: const TextStyle(color: _inputTextColor),
                    iconEnabledColor: _inputTextColor,
                    decoration: const InputDecoration(
                      border: OutlineInputBorder(),
                      labelText: '币种',
                      filled: true,
                      fillColor: _inputFieldColor,
                    ),
                    items: _symbols
                        .map(
                          (symbol) => DropdownMenuItem<String>(
                            value: symbol,
                            child: Text(
                              symbol,
                              style: const TextStyle(color: _inputTextColor),
                            ),
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
      appBar: AppBar(
        title: const Text('交易'),
        centerTitle: true,
        actions: [
          IconButton(
            tooltip: '我的通讯录',
            onPressed: _openContactsPage,
            icon: SvgPicture.asset(
              'assets/icons/contact-round.svg',
              width: 20,
              height: 20,
            ),
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
            const SizedBox(height: 24),
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

class _ContactsPlaceholderPage extends StatelessWidget {
  const _ContactsPlaceholderPage();

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('我的通讯录'),
        centerTitle: true,
      ),
      body: const Center(
        child: Text('我的通讯录（开发中）'),
      ),
    );
  }
}
