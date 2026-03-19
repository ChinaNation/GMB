import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:wuminapp_mobile/trade/onchain/onchain_trade_models.dart';
import 'package:wuminapp_mobile/trade/onchain/onchain_trade_repository.dart';

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
  final OnchainTradeRepository _repo = LocalOnchainTradeRepository();
  late Future<List<OnchainTxRecord>> _recordsFuture;

  @override
  void initState() {
    super.initState();
    _recordsFuture = _loadRecords();
  }

  Future<List<OnchainTxRecord>> _loadRecords() async {
    final all = await _repo.listRecent();
    final addr = widget.walletAddress.toLowerCase();
    return all
        .where((r) =>
            r.fromAddress.toLowerCase() == addr ||
            r.toAddress.toLowerCase() == addr)
        .toList(growable: false);
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('交易记录'),
        centerTitle: true,
      ),
      body: FutureBuilder<List<OnchainTxRecord>>(
        future: _recordsFuture,
        builder: (context, snapshot) {
          if (snapshot.connectionState == ConnectionState.waiting) {
            return const Center(child: CircularProgressIndicator());
          }
          final records = snapshot.data ?? [];
          if (records.isEmpty) {
            return const Center(
              child: Text('暂无交易记录', style: TextStyle(color: Colors.grey)),
            );
          }
          return ListView.separated(
            itemCount: records.length,
            separatorBuilder: (_, __) => const Divider(height: 1),
            itemBuilder: (context, index) {
              return TxRecordTile(
                record: records[index],
                selfAddress: widget.walletAddress,
                onTap: () {
                  Navigator.of(context).push(
                    MaterialPageRoute(
                      builder: (_) => TxRecordDetailPage(
                        record: records[index],
                        selfAddress: widget.walletAddress,
                      ),
                    ),
                  );
                },
              );
            },
          );
        },
      ),
    );
  }
}

class TxRecordTile extends StatelessWidget {
  const TxRecordTile({
    super.key,
    required this.record,
    required this.selfAddress,
    this.onTap,
  });

  final OnchainTxRecord record;
  final String selfAddress;
  final VoidCallback? onTap;

  bool get _isSend =>
      record.fromAddress.toLowerCase() == selfAddress.toLowerCase();

  String _shortAddress(String address) {
    if (address.length <= 12) return address;
    return '${address.substring(0, 6)}...${address.substring(address.length - 6)}';
  }

  String _formatTime(DateTime dt) {
    return '${dt.year}-${_pad(dt.month)}-${_pad(dt.day)} ${_pad(dt.hour)}:${_pad(dt.minute)}';
  }

  String _pad(int n) => n.toString().padLeft(2, '0');

  Widget _statusChip() {
    final (label, color) = switch (record.status) {
      OnchainTxStatus.pending => ('待确认', Colors.orange),
      OnchainTxStatus.confirmed => ('已确认', const Color(0xFF0B3D2E)),
      OnchainTxStatus.failed => ('失败', Colors.red),
    };
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
      decoration: BoxDecoration(
        color: color.withAlpha(25),
        borderRadius: BorderRadius.circular(4),
      ),
      child: Text(
        label,
        style:
            TextStyle(fontSize: 10, color: color, fontWeight: FontWeight.w600),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final counterparty = _isSend ? record.toAddress : record.fromAddress;
    return ListTile(
      onTap: onTap,
      leading: CircleAvatar(
        radius: 18,
        backgroundColor:
            _isSend ? Colors.red.shade50 : Colors.green.shade50,
        child: Icon(
          _isSend ? Icons.arrow_upward : Icons.arrow_downward,
          size: 18,
          color: _isSend ? Colors.red : const Color(0xFF0B3D2E),
        ),
      ),
      title: Row(
        children: [
          Text(
            _isSend ? '转出' : '转入',
            style: const TextStyle(fontSize: 15, fontWeight: FontWeight.w600),
          ),
          const SizedBox(width: 6),
          _statusChip(),
        ],
      ),
      subtitle: Text(
        '${_isSend ? "收" : "付"}：${_shortAddress(counterparty)}\n${_formatTime(record.createdAt)}',
        style: const TextStyle(fontSize: 12, height: 1.5),
      ),
      isThreeLine: true,
      trailing: Text(
        '${_isSend ? "-" : "+"}${record.amount.toStringAsFixed(2)}',
        style: TextStyle(
          fontSize: 15,
          fontWeight: FontWeight.w700,
          color: _isSend ? Colors.red : const Color(0xFF0B3D2E),
        ),
      ),
    );
  }
}

class TxRecordDetailPage extends StatelessWidget {
  const TxRecordDetailPage({
    super.key,
    required this.record,
    required this.selfAddress,
  });

  final OnchainTxRecord record;
  final String selfAddress;

  bool get _isSend =>
      record.fromAddress.toLowerCase() == selfAddress.toLowerCase();

  String _formatTime(DateTime dt) {
    return '${dt.year}-${_pad(dt.month)}-${_pad(dt.day)} ${_pad(dt.hour)}:${_pad(dt.minute)}:${_pad(dt.second)}';
  }

  String _pad(int n) => n.toString().padLeft(2, '0');

  String _statusLabel() {
    return switch (record.status) {
      OnchainTxStatus.pending => '待确认',
      OnchainTxStatus.confirmed => '已确认',
      OnchainTxStatus.failed => '失败',
    };
  }

  Color _statusColor() {
    return switch (record.status) {
      OnchainTxStatus.pending => Colors.orange,
      OnchainTxStatus.confirmed => const Color(0xFF0B3D2E),
      OnchainTxStatus.failed => Colors.red,
    };
  }

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
              style: const TextStyle(fontSize: 14, color: Colors.black54),
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
                child: Icon(Icons.copy, size: 16, color: Colors.black38),
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
        title: const Text('交易详情'),
        centerTitle: true,
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Center(
            child: Column(
              children: [
                Text(
                  '${_isSend ? "-" : "+"}${record.amount.toStringAsFixed(2)} ${record.symbol}',
                  style: TextStyle(
                    fontSize: 28,
                    fontWeight: FontWeight.w700,
                    color: _isSend ? Colors.red : const Color(0xFF0B3D2E),
                  ),
                ),
                const SizedBox(height: 6),
                Container(
                  padding:
                      const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                  decoration: BoxDecoration(
                    color: _statusColor().withAlpha(25),
                    borderRadius: BorderRadius.circular(6),
                  ),
                  child: Text(
                    _statusLabel(),
                    style: TextStyle(
                      fontSize: 13,
                      color: _statusColor(),
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 24),
          const Divider(height: 1),
          _buildRow(context, label: '类型', value: _isSend ? '转出' : '转入'),
          _buildRow(context,
              label: '发送方', value: record.fromAddress, copyable: true),
          _buildRow(context,
              label: '接收方', value: record.toAddress, copyable: true),
          _buildRow(context,
              label: '交易哈希', value: record.txHash, copyable: true),
          _buildRow(context,
              label: '时间', value: _formatTime(record.createdAt)),
          if (record.estimatedFee != null)
            _buildRow(context,
                label: '手续费',
                value: '${record.estimatedFee} ${record.symbol}'),
          if (record.failureReason != null &&
              record.failureReason!.trim().isNotEmpty)
            _buildRow(context, label: '失败原因', value: record.failureReason!),
        ],
      ),
    );
  }
}
