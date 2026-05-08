import 'dart:typed_data';

/// 创建多签账户提案详情（从链上 ProposalData 解码）。
///
/// 链上 SCALE 布局（PersonalManage ACTION_CREATE = 0 前缀之后）：
///   duoqian_address: AccountId32(32) + proposer: AccountId32(32)
///   + amount: u128(16) + fee: u128(16)
class CreateDuoqianProposalInfo {
  const CreateDuoqianProposalInfo({
    required this.proposalId,
    required this.duoqianAddress,
    required this.proposer,
    required this.amountFen,
    required this.feeFen,
    this.status,
  });

  final int proposalId;

  /// 多签地址公钥 hex（32 字节，不含 0x 前缀）。
  final String duoqianAddress;

  /// 发起人 SS58 地址。
  final String proposer;

  /// 初始资金（分）。
  final BigInt amountFen;

  /// 创建手续费快照（分）。
  final BigInt feeFen;

  /// 0=voting, 1=passed, 2=rejected, null=unknown。
  final int? status;

  double get amountYuan => amountFen.toDouble() / 100;
  double get feeYuan => feeFen.toDouble() / 100;

  /// 48 字节 SubjectId。
  ///
  /// 历史上该字节数组曾使用过旧称；当前统一命名为 SubjectId。
  /// 本 getter 保留原编码逻辑，只作为旧 ProposalData 的
  /// 兼容解码辅助，不参与新 SubjectKind 协议构造。
  Uint8List get institutionBytes {
    final bytes = Uint8List(48);
    final addrBytes = _hexDecode(duoqianAddress);
    bytes.setAll(0, addrBytes);
    return bytes;
  }

  CreateDuoqianProposalInfo copyWithStatus(int? newStatus) {
    return CreateDuoqianProposalInfo(
      proposalId: proposalId,
      duoqianAddress: duoqianAddress,
      proposer: proposer,
      amountFen: amountFen,
      feeFen: feeFen,
      status: newStatus,
    );
  }
}

/// 关闭多签账户提案详情（从链上 ProposalData 解码）。
///
/// 链上 SCALE 布局（ACTION_CLOSE = 2 前缀之后）：
///   duoqian_address: AccountId32(32) + beneficiary: AccountId32(32)
///   + proposer: AccountId32(32)
class CloseDuoqianProposalInfo {
  const CloseDuoqianProposalInfo({
    required this.proposalId,
    required this.duoqianAddress,
    required this.beneficiary,
    required this.proposer,
    this.status,
  });

  final int proposalId;

  /// 多签地址公钥 hex（32 字节，不含 0x 前缀）。
  final String duoqianAddress;

  /// 受益人 SS58 地址。
  final String beneficiary;

  /// 发起人 SS58 地址。
  final String proposer;

  /// 0=voting, 1=passed, 2=rejected, null=unknown。
  final int? status;

  /// 48 字节 SubjectId。
  ///
  /// 历史上该字节数组曾使用过旧称；当前统一命名为 SubjectId。
  /// 本 getter 保留原编码逻辑，只作为旧 ProposalData 的
  /// 兼容解码辅助，不参与新 SubjectKind 协议构造。
  Uint8List get institutionBytes {
    final bytes = Uint8List(48);
    final addrBytes = _hexDecode(duoqianAddress);
    bytes.setAll(0, addrBytes);
    return bytes;
  }

  CloseDuoqianProposalInfo copyWithStatus(int? newStatus) {
    return CloseDuoqianProposalInfo(
      proposalId: proposalId,
      duoqianAddress: duoqianAddress,
      beneficiary: beneficiary,
      proposer: proposer,
      status: newStatus,
    );
  }
}

/// 多签账户状态。
enum DuoqianStatus {
  /// 提案投票中，尚未激活。
  pending,

  /// 投票通过、入金完成，已激活。
  active,
}

/// 多签账户链上信息。
///
/// 注册机构来自 `OrganizationManage::Institutions / InstitutionAccounts`，
/// 个人多签状态来自 `PersonalManage::PersonalDuoqians`，
/// 管理员和阈值来自 `AdminsChange::Subjects`。
class DuoqianAccountInfo {
  const DuoqianAccountInfo({
    required this.adminCount,
    required this.threshold,
    required this.adminPubkeys,
    required this.status,
  });

  final int adminCount;
  final int threshold;

  /// 管理员公钥列表（hex，不含 0x 前缀）。
  final List<String> adminPubkeys;

  final DuoqianStatus status;
}

// ──── 工具函数 ────

Uint8List _hexDecode(String hex) {
  final h = hex.startsWith('0x') ? hex.substring(2) : hex;
  final result = Uint8List(h.length ~/ 2);
  for (var i = 0; i < result.length; i++) {
    result[i] = int.parse(h.substring(i * 2, i * 2 + 2), radix: 16);
  }
  return result;
}
