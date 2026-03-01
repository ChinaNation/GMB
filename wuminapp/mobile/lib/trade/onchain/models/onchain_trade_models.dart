enum OnchainTxStatus {
  pending,
  confirmed,
  failed,
}

OnchainTxStatus onchainTxStatusFromString(String raw) {
  switch (raw.toLowerCase()) {
    case 'confirmed':
      return OnchainTxStatus.confirmed;
    case 'failed':
      return OnchainTxStatus.failed;
    case 'pending':
    default:
      return OnchainTxStatus.pending;
  }
}

String onchainTxStatusToString(OnchainTxStatus status) {
  switch (status) {
    case OnchainTxStatus.pending:
      return 'pending';
    case OnchainTxStatus.confirmed:
      return 'confirmed';
    case OnchainTxStatus.failed:
      return 'failed';
  }
}

bool onchainTxStatusIsFinal(OnchainTxStatus status) {
  return status == OnchainTxStatus.confirmed ||
      status == OnchainTxStatus.failed;
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

class OnchainSignedTransfer {
  const OnchainSignedTransfer({
    required this.fromAddress,
    required this.pubkeyHex,
    required this.toAddress,
    required this.amount,
    required this.symbol,
    required this.nonce,
    required this.signedAt,
    required this.signMessage,
    required this.signatureHex,
  });

  final String fromAddress;
  final String pubkeyHex;
  final String toAddress;
  final double amount;
  final String symbol;
  final String nonce;
  final int signedAt;
  final String signMessage;
  final String signatureHex;
}

class OnchainSubmitResult {
  const OnchainSubmitResult({
    required this.txHash,
    required this.status,
    this.failureReason,
  });

  final String txHash;
  final OnchainTxStatus status;
  final String? failureReason;
}

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
  });

  final String txHash;
  final String fromAddress;
  final String toAddress;
  final double amount;
  final String symbol;
  final DateTime createdAt;
  final OnchainTxStatus status;
  final String? failureReason;

  OnchainTxRecord copyWith({
    String? txHash,
    String? fromAddress,
    String? toAddress,
    double? amount,
    String? symbol,
    DateTime? createdAt,
    OnchainTxStatus? status,
    String? failureReason,
    bool clearFailureReason = false,
  }) {
    return OnchainTxRecord(
      txHash: txHash ?? this.txHash,
      fromAddress: fromAddress ?? this.fromAddress,
      toAddress: toAddress ?? this.toAddress,
      amount: amount ?? this.amount,
      symbol: symbol ?? this.symbol,
      createdAt: createdAt ?? this.createdAt,
      status: status ?? this.status,
      failureReason:
          clearFailureReason ? null : (failureReason ?? this.failureReason),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'txHash': txHash,
      'fromAddress': fromAddress,
      'toAddress': toAddress,
      'amount': amount,
      'symbol': symbol,
      'createdAtMillis': createdAt.millisecondsSinceEpoch,
      'status': onchainTxStatusToString(status),
      'failureReason': failureReason,
    };
  }

  factory OnchainTxRecord.fromJson(Map<String, dynamic> json) {
    return OnchainTxRecord(
      txHash: json['txHash']?.toString() ?? '',
      fromAddress: json['fromAddress']?.toString() ?? '',
      toAddress: json['toAddress']?.toString() ?? '',
      amount: (json['amount'] as num?)?.toDouble() ?? 0,
      symbol: json['symbol']?.toString() ?? 'GMB',
      createdAt: DateTime.fromMillisecondsSinceEpoch(
        (json['createdAtMillis'] as num?)?.toInt() ??
            DateTime.now().millisecondsSinceEpoch,
      ),
      status:
          onchainTxStatusFromString(json['status']?.toString() ?? 'pending'),
      failureReason: json['failureReason']?.toString(),
    );
  }
}
