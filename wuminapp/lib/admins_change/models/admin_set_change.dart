class AdminSetChangeDraft {
  const AdminSetChangeDraft({
    required this.subjectIdHex,
    required this.org,
    required this.currentAdmins,
    required this.newAdmins,
  });

  final String subjectIdHex;
  final int org;
  final List<String> currentAdmins;
  final List<String> newAdmins;
}
