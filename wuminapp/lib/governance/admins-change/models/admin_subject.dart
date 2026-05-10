class AdminSubjectState {
  const AdminSubjectState({
    required this.subjectIdHex,
    required this.org,
    required this.kind,
    required this.admins,
    required this.threshold,
    required this.creatorHex,
    required this.createdAt,
    required this.updatedAt,
    required this.status,
  });

  final String subjectIdHex;
  final int org;
  final int kind;
  final List<String> admins;
  final int threshold;
  final String creatorHex;
  final int createdAt;
  final int updatedAt;
  final int status;

  bool get isActive => status == 1;

  String get kindLabel => switch (kind) {
        0 => '内置治理机构',
        1 => 'SFID机构归属',
        2 => '个人多签',
        3 => '机构账户',
        _ => '未知主体',
      };

  String get orgLabel => switch (org) {
        0 => '国储会',
        1 => '省储会',
        2 => '省储行',
        3 => '个人多签',
        4 => '公权机构账户',
        5 => '其他机构账户',
        _ => '未知组织',
      };

  String get statusLabel => switch (status) {
        0 => '待激活',
        1 => '已激活',
        2 => '已关闭',
        _ => '未知状态',
      };
}
