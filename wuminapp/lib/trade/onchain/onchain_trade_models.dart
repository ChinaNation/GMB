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
