/// 多签账户在手机端的入口分流类型。
///
/// 这里只区分产品入口与本地列表归属；链上仍统一走
/// DuoqianManage / DuoqianTransfer / VotingEngine。
enum DuoqianAccountType {
  /// 机构多签账户。
  institution,

  /// 个人多签账户。
  personal,
}

extension DuoqianAccountTypeText on DuoqianAccountType {
  String get title {
    switch (this) {
      case DuoqianAccountType.institution:
        return '机构多签';
      case DuoqianAccountType.personal:
        return '个人多签';
    }
  }

  String get emptyTitle {
    switch (this) {
      case DuoqianAccountType.institution:
        return '暂无机构多签账户';
      case DuoqianAccountType.personal:
        return '暂无个人多签账户';
    }
  }

  String get createTitle {
    switch (this) {
      case DuoqianAccountType.institution:
        return '创建机构多签账户';
      case DuoqianAccountType.personal:
        return '创建个人多签账户';
    }
  }
}
