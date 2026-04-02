import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/util/amount_format.dart';
import 'package:wuminapp_mobile/trade/local_tx_store.dart';
import 'package:wuminapp_mobile/isar/wallet_isar.dart';

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
  static const int _pageSize = 20;
  final ScrollController _scrollController = ScrollController();

  List<LocalTxEntity> _records = [];
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
      final records = await LocalTxStore.queryByWallet(
        widget.walletAddress,
        limit: _pageSize,
        offset: 0,
      );
      if (!mounted) return;
      setState(() {
        _records = records;
        _hasMore = records.length >= _pageSize;
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
      final records = await LocalTxStore.queryByWallet(
        widget.walletAddress,
        limit: _pageSize,
        offset: _records.length,
      );
      if (!mounted) return;
      setState(() {
        _records = [..._records, ...records];
        _hasMore = records.length >= _pageSize;
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
          return _LocalTxRecordTile(
            record: record,
            onTap: () {
              Navigator.of(context).push(
                MaterialPageRoute(
                  builder: (_) => _LocalTxRecordDetailPage(record: record),
                ),
              );
            },
          );
        },
      ),
    );
  }
}

// ─── 交易类型中文标签 ─────────────────────────────────────────

String _txTypeLabel(String txType, String direction) {
  switch (txType) {
    case 'transfer':
      return direction == 'out' ? '转账支出' : '转账收入';
    case 'offchain_pay':
      return direction == 'out' ? '扫码支付' : '扫码收款';
    case 'proposal_transfer':
      return direction == 'out' ? '提案转出' : '提案转入';
    case 'fee_withdraw':
      return '手续费';
    case 'fee_deposit':
      return '手续费分成';
    case 'block_reward':
      return '出块奖励';
    case 'bank_interest':
      return '银行利息';
    case 'gov_issuance':
      return '治理增发';
    case 'lightnode_reward':
      return '认证奖励';
    case 'duoqian_create':
      return '多签出资';
    case 'duoqian_close':
      return direction == 'out' ? '多签关闭' : '多签收款';
    case 'fund_destroy':
      return '资金销毁';
    default:
      return txType;
  }
}

String _statusLabel(String status) {
  switch (status) {
    case 'pending':
      return '待确认';
    case 'confirmed':
      return '已确认';
    case 'onchain':
      return '已上链';
    default:
      return status;
  }
}

Color _statusColor(String status) {
  switch (status) {
    case 'pending':
      return AppTheme.warning;
    case 'confirmed':
      return AppTheme.success;
    case 'onchain':
      return AppTheme.primary;
    default:
      return AppTheme.textTertiary;
  }
}

String _pad(int n) => n.toString().padLeft(2, '0');

String _formatMillis(int millis) {
  final dt = DateTime.fromMillisecondsSinceEpoch(millis).toLocal();
  return '${dt.year}-${_pad(dt.month)}-${_pad(dt.day)} ${_pad(dt.hour)}:${_pad(dt.minute)}';
}

String _formatMillisFull(int millis) {
  final dt = DateTime.fromMillisecondsSinceEpoch(millis).toLocal();
  return '${dt.year}-${_pad(dt.month)}-${_pad(dt.day)} ${_pad(dt.hour)}:${_pad(dt.minute)}:${_pad(dt.second)}';
}

// ─── 交易记录列表项 ──────────────────────────────────────────

class _LocalTxRecordTile extends StatelessWidget {
  const _LocalTxRecordTile({
    required this.record,
    this.onTap,
  });

  final LocalTxEntity record;
  final VoidCallback? onTap;

  String _shortAddress(String? address) {
    if (address == null || address.length <= 12) return address ?? '-';
    return '${address.substring(0, 6)}...${address.substring(address.length - 6)}';
  }

  bool get _isExpense => record.direction == 'out';
  bool get _isIncome => record.direction == 'in';

  Color get _iconColor {
    if (_isExpense) return AppTheme.danger;
    if (_isIncome) return AppTheme.primaryDark;
    return AppTheme.textTertiary;
  }

  Color get _iconBgColor {
    if (_isExpense) return AppTheme.danger.withAlpha(20);
    if (_isIncome) return AppTheme.success.withAlpha(20);
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
      case 'offchain_pay':
        return Icons.qr_code_scanner;
      default:
        return _isExpense ? Icons.arrow_upward : Icons.arrow_downward;
    }
  }

  @override
  Widget build(BuildContext context) {
    final label = _txTypeLabel(record.txType, record.direction);
    final counterparty = _isExpense
        ? _shortAddress(record.toAddress)
        : _shortAddress(record.fromAddress);
    final timeStr = _formatMillis(record.createdAtMillis);

    return ListTile(
      onTap: onTap,
      leading: CircleAvatar(
        radius: 18,
        backgroundColor: _iconBgColor,
        child: Icon(_icon, size: 18, color: _iconColor),
      ),
      title: Row(
        children: [
          Text(
            label,
            style: const TextStyle(fontSize: 15, fontWeight: FontWeight.w600),
          ),
          if (record.status == 'pending') ...[
            const SizedBox(width: 6),
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 1),
              decoration: BoxDecoration(
                color: AppTheme.warning.withAlpha(30),
                borderRadius: BorderRadius.circular(4),
              ),
              child: Text(
                _statusLabel(record.status),
                style: TextStyle(
                  fontSize: 10,
                  color: _statusColor(record.status),
                  fontWeight: FontWeight.w600,
                ),
              ),
            ),
          ],
        ],
      ),
      subtitle: Text(
        '$counterparty\n$timeStr',
        style: const TextStyle(fontSize: 12, height: 1.5),
      ),
      isThreeLine: true,
      trailing: Text(
        '${_isExpense ? "-" : "+"}${AmountFormat.format(record.amountYuan, symbol: '')}',
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

class _LocalTxRecordDetailPage extends StatelessWidget {
  const _LocalTxRecordDetailPage({
    required this.record,
  });

  final LocalTxEntity record;

  bool get _isExpense => record.direction == 'out';
  bool get _isIncome => record.direction == 'in';

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
    final amountColor = _isExpense
        ? AppTheme.danger
        : _isIncome
            ? AppTheme.primaryDark
            : AppTheme.textSecondary;

    final label = _txTypeLabel(record.txType, record.direction);

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
              '${_isExpense ? "-" : "+"}${AmountFormat.format(record.amountYuan, symbol: 'GMB')}',
              style: TextStyle(
                fontSize: 28,
                fontWeight: FontWeight.w700,
                color: amountColor,
              ),
            ),
          ),
          const SizedBox(height: 8),
          Center(
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
              decoration: BoxDecoration(
                color: _statusColor(record.status).withAlpha(20),
                borderRadius: BorderRadius.circular(4),
              ),
              child: Text(
                _statusLabel(record.status),
                style: TextStyle(
                  fontSize: 12,
                  color: _statusColor(record.status),
                  fontWeight: FontWeight.w600,
                ),
              ),
            ),
          ),
          const SizedBox(height: 24),
          const Divider(height: 1),
          _buildRow(context, label: '类型', value: label),
          if (record.blockNumber != null)
            _buildRow(context,
                label: '区块', value: record.blockNumber.toString()),
          if (record.fromAddress != null)
            _buildRow(context,
                label: '发送方', value: record.fromAddress!, copyable: true),
          if (record.toAddress != null)
            _buildRow(context,
                label: '接收方', value: record.toAddress!, copyable: true),
          _buildRow(context,
              label: '时间', value: _formatMillisFull(record.createdAtMillis)),
          if (record.confirmedAtMillis != null)
            _buildRow(context,
                label: '确认时间',
                value: _formatMillisFull(record.confirmedAtMillis!)),
          if (record.feeYuan != null)
            _buildRow(context,
                label: '手续费', value: '${record.feeYuan} GMB'),
          if (record.txHash != null)
            _buildRow(context,
                label: '交易哈希', value: record.txHash!, copyable: true),
          if (record.bankShenfenId != null)
            _buildRow(context,
                label: '清算行', value: record.bankShenfenId!),
        ],
      ),
    );
  }
}
