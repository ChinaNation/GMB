import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/util/amount_format.dart';
import 'package:wuminapp_mobile/wallet/capabilities/api_client.dart';
import 'package:wuminapp_mobile/wallet/models/server_tx_record.dart';

class TransactionHistoryPage extends StatefulWidget {
  const TransactionHistoryPage({
    super.key,
    required this.walletAddress,
  });

  final String walletAddress;

  @override
  State<TransactionHistoryPage> createState() =>
      _TransactionHistoryPageState();
}

class _TransactionHistoryPageState extends State<TransactionHistoryPage> {
  final ApiClient _api = ApiClient();
  final ScrollController _scrollController = ScrollController();

  List<ServerTxRecord> _records = [];
  bool _loading = true;
  bool _loadingMore = false;
  bool _hasMore = false;
  String? _error;

  @override
  void initState() {
    super.initState();
    _scrollController.addListener(_onScroll);
    _loadFirstPage();
  }

  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }

  void _onScroll() {
    if (_scrollController.position.pixels >=
        _scrollController.position.maxScrollExtent - 200) {
      _loadNextPage();
    }
  }

  Future<void> _loadFirstPage() async {
    setState(() {
      _loading = true;
      _error = null;
    });
    try {
      final page = await _api.fetchWalletTransactions(
        widget.walletAddress,
        limit: 20,
      );
      if (!mounted) return;
      setState(() {
        _records = page.records;
        _hasMore = page.hasMore;
        _loading = false;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _error = e.toString();
        _loading = false;
      });
    }
  }

  Future<void> _loadNextPage() async {
    if (_loadingMore || !_hasMore || _records.isEmpty) return;
    setState(() => _loadingMore = true);
    try {
      final page = await _api.fetchWalletTransactions(
        widget.walletAddress,
        limit: 20,
        beforeId: _records.last.id,
      );
      if (!mounted) return;
      setState(() {
        _records = [..._records, ...page.records];
        _hasMore = page.hasMore;
        _loadingMore = false;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() => _loadingMore = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('交易记录'),
        centerTitle: true,
      ),
      body: _buildBody(),
    );
  }

  Widget _buildBody() {
    if (_loading) {
      return const Center(child: CircularProgressIndicator());
    }
    if (_error != null) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Text('加载失败', style: TextStyle(color: AppTheme.textSecondary)),
            const SizedBox(height: 8),
            TextButton(onPressed: _loadFirstPage, child: const Text('重试')),
          ],
        ),
      );
    }
    if (_records.isEmpty) {
      return const Center(
        child: Text('暂无交易记录', style: TextStyle(color: AppTheme.textTertiary)),
      );
    }
    return RefreshIndicator(
      onRefresh: _loadFirstPage,
      child: ListView.separated(
        controller: _scrollController,
        itemCount: _records.length + (_hasMore ? 1 : 0),
        separatorBuilder: (_, __) => const Divider(height: 1),
        itemBuilder: (context, index) {
          if (index >= _records.length) {
            return const Padding(
              padding: EdgeInsets.symmetric(vertical: 16),
              child: Center(child: CircularProgressIndicator(strokeWidth: 2)),
            );
          }
          final record = _records[index];
          return ServerTxRecordTile(
            record: record,
            onTap: () {
              Navigator.of(context).push(
                MaterialPageRoute(
                  builder: (_) => ServerTxRecordDetailPage(record: record),
                ),
              );
            },
          );
        },
      ),
    );
  }
}

// ─── 交易记录列表项 ──────────────────────────────────────────

class ServerTxRecordTile extends StatelessWidget {
  const ServerTxRecordTile({
    super.key,
    required this.record,
    this.onTap,
  });

  final ServerTxRecord record;
  final VoidCallback? onTap;

  String _shortAddress(String? address) {
    if (address == null || address.length <= 12) return address ?? '-';
    return '${address.substring(0, 6)}...${address.substring(address.length - 6)}';
  }

  String _formatTime(DateTime? dt) {
    if (dt == null) return '-';
    final local = dt.toLocal();
    return '${local.year}-${_pad(local.month)}-${_pad(local.day)} ${_pad(local.hour)}:${_pad(local.minute)}';
  }

  String _pad(int n) => n.toString().padLeft(2, '0');

  Color get _iconColor {
    if (record.isExpense) return AppTheme.danger;
    if (record.isIncome) return AppTheme.primaryDark;
    return AppTheme.textTertiary;
  }

  Color get _iconBgColor {
    if (record.isExpense) return AppTheme.danger.withAlpha(20);
    if (record.isIncome) return AppTheme.success.withAlpha(20);
    return AppTheme.surfaceElevated;
  }

  IconData get _icon {
    switch (record.txType) {
      case 'block_reward':
        return Icons.token;
      case 'bank_interest':
        return Icons.account_balance;
      case 'gov_issuance':
        return Icons.gavel;
      case 'lightnode_reward':
        return Icons.verified;
      case 'fee_withdraw':
        return Icons.receipt_long;
      case 'fee_deposit':
        return Icons.receipt_long;
      case 'fund_destroy':
        return Icons.delete_forever;
      case 'dust':
        return Icons.auto_delete;
      default:
        return record.isExpense ? Icons.arrow_upward : Icons.arrow_downward;
    }
  }

  @override
  Widget build(BuildContext context) {
    final counterparty = record.isExpense
        ? _shortAddress(record.toAddress)
        : _shortAddress(record.fromAddress);

    return ListTile(
      onTap: onTap,
      leading: CircleAvatar(
        radius: 18,
        backgroundColor: _iconBgColor,
        child: Icon(_icon, size: 18, color: _iconColor),
      ),
      title: Text(
        record.txTypeLabel,
        style: const TextStyle(fontSize: 15, fontWeight: FontWeight.w600),
      ),
      subtitle: Text(
        '$counterparty\n${_formatTime(record.blockTimestamp)}',
        style: const TextStyle(fontSize: 12, height: 1.5),
      ),
      isThreeLine: true,
      trailing: Text(
        '${record.isExpense ? "-" : "+"}${AmountFormat.format(record.amountYuan, symbol: '')}',
        style: TextStyle(
          fontSize: 15,
          fontWeight: FontWeight.w700,
          color: _iconColor,
        ),
      ),
    );
  }
}

// ─── 交易详情页 ──────────────────────────────────────────────

class ServerTxRecordDetailPage extends StatelessWidget {
  const ServerTxRecordDetailPage({
    super.key,
    required this.record,
  });

  final ServerTxRecord record;

  String _formatTime(DateTime? dt) {
    if (dt == null) return '-';
    final local = dt.toLocal();
    return '${local.year}-${_pad(local.month)}-${_pad(local.day)} ${_pad(local.hour)}:${_pad(local.minute)}:${_pad(local.second)}';
  }

  String _pad(int n) => n.toString().padLeft(2, '0');

  void _copy(BuildContext context, String text) {
    Clipboard.setData(ClipboardData(text: text));
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('已复制')),
    );
  }

  Widget _buildRow(
    BuildContext context, {
    required String label,
    required String value,
    bool copyable = false,
  }) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 10),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 72,
            child: Text(
              label,
              style: const TextStyle(fontSize: 14, color: AppTheme.textSecondary),
            ),
          ),
          Expanded(
            child: Text(value, style: const TextStyle(fontSize: 14)),
          ),
          if (copyable)
            GestureDetector(
              onTap: () => _copy(context, value),
              child: const Padding(
                padding: EdgeInsets.only(left: 8),
                child: Icon(Icons.copy, size: 16, color: AppTheme.textTertiary),
              ),
            ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final amountColor = record.isExpense
        ? AppTheme.danger
        : record.isIncome
            ? AppTheme.primaryDark
            : AppTheme.textSecondary;

    return Scaffold(
      appBar: AppBar(
        title: const Text('交易详情'),
        centerTitle: true,
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Center(
            child: Text(
              '${record.isExpense ? "-" : "+"}${AmountFormat.format(record.amountYuan, symbol: 'GMB')}',
              style: TextStyle(
                fontSize: 28,
                fontWeight: FontWeight.w700,
                color: amountColor,
              ),
            ),
          ),
          const SizedBox(height: 24),
          const Divider(height: 1),
          _buildRow(context, label: '类型', value: record.txTypeLabel),
          _buildRow(context,
              label: '区块', value: record.blockNumber.toString()),
          if (record.fromAddress != null)
            _buildRow(context,
                label: '发送方', value: record.fromAddress!, copyable: true),
          if (record.toAddress != null)
            _buildRow(context,
                label: '接收方', value: record.toAddress!, copyable: true),
          _buildRow(context,
              label: '时间', value: _formatTime(record.blockTimestamp)),
          if (record.feeYuan != null)
            _buildRow(context,
                label: '手续费', value: '${record.feeYuan} GMB'),
        ],
      ),
    );
  }
}
