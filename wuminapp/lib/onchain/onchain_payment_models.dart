enum OnchainPaymentErrorCode {
  walletMissing,
  walletMismatch,
  invalidDraft,
  broadcastFailed,
}

class OnchainPaymentException implements Exception {
  const OnchainPaymentException(this.code, this.message);

  final OnchainPaymentErrorCode code;
  final String message;

  @override
  String toString() {
    return 'OnchainPaymentException(${code.name}): $message';
  }
}

class OnchainPaymentDraft {
  const OnchainPaymentDraft({
    required this.toAddress,
    required this.amount,
    required this.symbol,
  });

  final String toAddress;
  final double amount;
  final String symbol;
}
