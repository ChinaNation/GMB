class AdminSetChangeDraft {
  const AdminSetChangeDraft({
    required this.accountHex,
    required this.org,
    required this.currentAdmins,
    required this.admins,
  });

  final String accountHex;
  final int org;
  final List<String> currentAdmins;
  final List<String> admins;
}
