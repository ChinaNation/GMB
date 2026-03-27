/// 来自 SFID indexer 的交易记录。
class ServerTxRecord {
  const ServerTxRecord({
    required this.id,
    required this.blockNumber,
    required this.txType,
    required this.direction,
    this.fromAddress,
    this.toAddress,
    required this.amountYuan,
    this.feeYuan,
    this.blockTimestamp,
  });

  final int id;
  final int blockNumber;
  final String txType;
  final String direction; // "in" / "out" / "info"
  final String? fromAddress;
  final String? toAddress;
  final double amountYuan;
  final double? feeYuan;
  final DateTime? blockTimestamp;

  factory ServerTxRecord.fromJson(Map<String, dynamic> json) {
    return ServerTxRecord(
      id: (json['id'] as num).toInt(),
      blockNumber: (json['block_number'] as num).toInt(),
      txType: json['tx_type'] as String? ?? '',
      direction: json['direction'] as String? ?? 'info',
      fromAddress: json['from_address'] as String?,
      toAddress: json['to_address'] as String?,
      amountYuan: (json['amount_yuan'] as num?)?.toDouble() ?? 0,
      feeYuan: (json['fee_yuan'] as num?)?.toDouble(),
      blockTimestamp: json['block_timestamp'] != null
          ? DateTime.tryParse(json['block_timestamp'] as String)
          : null,
    );
  }

  /// 交易类型的中文标签。
  String get txTypeLabel {
    switch (txType) {
      case 'transfer':
        return direction == 'out' ? '转账支出' : '转账收入';
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
      case 'proposal_transfer':
        return direction == 'out' ? '提案转出' : '提案转入';
      case 'duoqian_create':
        return '多签出资';
      case 'duoqian_close':
        return direction == 'out' ? '多签关闭' : '多签收款';
      case 'fund_destroy':
        return '资金销毁';
      case 'dust':
        return 'Dust回收';
      default:
        return txType;
    }
  }

  /// 是否为支出类。
  bool get isExpense => direction == 'out';

  /// 是否为收入类。
  bool get isIncome => direction == 'in';
}

/// SFID 分页响应。
class ServerTxPage {
  const ServerTxPage({
    required this.records,
    required this.hasMore,
  });

  final List<ServerTxRecord> records;
  final bool hasMore;

  factory ServerTxPage.fromJson(Map<String, dynamic> json) {
    final rawRecords = json['records'] as List? ?? [];
    return ServerTxPage(
      records: rawRecords
          .whereType<Map<String, dynamic>>()
          .map((e) => ServerTxRecord.fromJson(e))
          .toList(),
      hasMore: json['has_more'] as bool? ?? false,
    );
  }
}
