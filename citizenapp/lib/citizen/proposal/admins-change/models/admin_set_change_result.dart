class AdminsChangeSubmitResult {
  const AdminsChangeSubmitResult({
    required this.txHash,
    required this.usedNonce,
  });

  final String txHash;
  final int usedNonce;
}
