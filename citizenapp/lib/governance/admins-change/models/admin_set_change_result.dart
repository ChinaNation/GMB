class AdminSetChangeSubmitResult {
  const AdminSetChangeSubmitResult({
    required this.txHash,
    required this.usedNonce,
  });

  final String txHash;
  final int usedNonce;
}
