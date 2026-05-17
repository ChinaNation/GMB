import 'dart:typed_data';

/// PersonalManage 创建个人多签提案详情（从链上 ProposalData 解码）。
///
/// 链上 SCALE 布局（`per-mgmt` + ACTION_CREATE=0 之后）：
///   duoqian_address: AccountId32(32) + proposer: AccountId32(32)
///   + amount: u128(16) + fee: u128(16)。
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

  /// 个人多签地址公钥 hex（32 字节，不含 0x 前缀）。
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

  /// 48 字节 SubjectId 兼容字段。
  ///
  /// 旧 UI 仍用该 getter 做投票上下文兼容；新个人多签 SubjectId 构造统一走
  /// `PersonalManageStorageCodec.subjectIdFromAccountHex`。
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

/// PersonalManage 关闭个人多签提案详情（从链上 ProposalData 解码）。
///
/// 链上 SCALE 布局（`per-mgmt` + ACTION_CLOSE=1 之后）：
///   duoqian_address: AccountId32(32) + beneficiary: AccountId32(32)
///   + proposer: AccountId32(32)。
class CloseDuoqianProposalInfo {
  const CloseDuoqianProposalInfo({
    required this.proposalId,
    required this.duoqianAddress,
    required this.beneficiary,
    required this.proposer,
    this.status,
  });

  final int proposalId;

  /// 个人多签地址公钥 hex（32 字节，不含 0x 前缀）。
  final String duoqianAddress;

  /// 受益人 SS58 地址。
  final String beneficiary;

  /// 发起人 SS58 地址。
  final String proposer;

  /// 0=voting, 1=passed, 2=rejected, null=unknown。
  final int? status;

  /// 48 字节 SubjectId 兼容字段。
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

/// 个人多签账户状态。
enum DuoqianStatus {
  /// 提案投票中，尚未激活。
  pending,

  /// 投票通过、入金完成，已激活。
  active,
}

/// 个人多签账户链上信息。
///
/// 个人状态来自 `PersonalManage::PersonalDuoqians`，
/// 管理员来自 `AdminsChange::Subjects`，动态阈值来自 `InternalVote`。
class DuoqianAccountInfo {
  const DuoqianAccountInfo({
    required this.adminCount,
    required this.threshold,
    required this.adminPubkeys,
    required this.status,
  });

  final int adminCount;
  final int? threshold;

  /// 管理员公钥列表（hex，不含 0x 前缀）。
  final List<String> adminPubkeys;

  final DuoqianStatus status;
}

Uint8List _hexDecode(String hex) {
  final h = hex.startsWith('0x') ? hex.substring(2) : hex;
  final result = Uint8List(h.length ~/ 2);
  for (var i = 0; i < result.length; i++) {
    result[i] = int.parse(h.substring(i * 2, i * 2 + 2), radix: 16);
  }
  return result;
}
