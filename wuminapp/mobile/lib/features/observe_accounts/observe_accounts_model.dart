class ObservedAccount {
  const ObservedAccount({
    required this.id,
    required this.orgName,
    required this.publicKey,
    required this.address,
    required this.balance,
    required this.source,
  });

  final String id;
  final String orgName;
  final String publicKey;
  final String address;
  final double balance;
  final String source;

  ObservedAccount copyWith({
    String? id,
    String? orgName,
    String? publicKey,
    String? address,
    double? balance,
    String? source,
  }) {
    return ObservedAccount(
      id: id ?? this.id,
      orgName: orgName ?? this.orgName,
      publicKey: publicKey ?? this.publicKey,
      address: address ?? this.address,
      balance: balance ?? this.balance,
      source: source ?? this.source,
    );
  }
}
