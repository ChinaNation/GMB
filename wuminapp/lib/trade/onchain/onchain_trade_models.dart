enum OnchainTxStatus {
  pending,
  confirmed,
  failed,
}

enum OnchainTradeErrorCode {
  walletMissing,
  walletMismatch,
  invalidDraft,
  broadcastFailed,
}

class OnchainTradeException implements Exception {
  const OnchainTradeException(this.code, this.message);

  final OnchainTradeErrorCode code;
  final String message;

  @override
  String toString() {
    return 'OnchainTradeException(${code.name}): $message';
  }
}

class OnchainTransferDraft {
  const OnchainTransferDraft({
    required this.toAddress,
    required this.amount,
    required this.symbol,
  });

  final String toAddress;
  final double amount;
  final String symbol;
}

/// 刚提交的 pending 交易记录（仅内存中，不持久化）。
class OnchainTxRecord {
  const OnchainTxRecord({
    required this.txHash,
    required this.fromAddress,
    required this.toAddress,
    required this.amount,
    required this.symbol,
    required this.createdAt,
    required this.status,
    this.failureReason,
    this.usedNonce,
    this.estimatedFee,
  });

  final String txHash;
  final String fromAddress;
  final String toAddress;
  final double amount;
  final String symbol;
  final DateTime createdAt;
  final OnchainTxStatus status;
  final String? failureReason;
  final int? usedNonce;
  final double? estimatedFee;
}
