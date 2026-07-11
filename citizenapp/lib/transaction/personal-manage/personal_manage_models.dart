import 'dart:typed_data';

/// PersonalAdmins 创建个人多签提案详情（从链上 ProposalData 解码）。
///
/// 链上 SCALE 布局（`per-mgmt` + ACTION_CREATE=0 之后）：
///   account: AccountId32(32) + proposer: AccountId32(32)
///   + amount: u128(16) + fee: u128(16)。
class CreateProposalInfo {
  const CreateProposalInfo({
    required this.proposalId,
    required this.account,
    required this.proposer,
    required this.amountFen,
    required this.feeFen,
    this.status,
  });

  final int proposalId;

  /// 个人多签账户公钥 hex（32 字节，不含 0x 前缀）。
  final String account;

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

  /// 个人多签治理 AccountId。
  Uint8List get institutionBytes {
    return Uint8List.fromList(_hexDecode(account));
  }

  CreateProposalInfo copyWithStatus(int? newStatus) {
    return CreateProposalInfo(
      proposalId: proposalId,
      account: account,
      proposer: proposer,
      amountFen: amountFen,
      feeFen: feeFen,
      status: newStatus,
    );
  }
}

/// PersonalAdmins 关闭个人多签提案详情（从链上 ProposalData 解码）。
///
/// 链上 SCALE 布局（`per-mgmt` + ACTION_CLOSE=1 之后）：
///   account: AccountId32(32) + beneficiary: AccountId32(32)
///   + proposer: AccountId32(32)。
class CloseProposalInfo {
  const CloseProposalInfo({
    required this.proposalId,
    required this.account,
    required this.beneficiary,
    required this.proposer,
    this.status,
  });

  final int proposalId;

  /// 个人多签账户公钥 hex（32 字节，不含 0x 前缀）。
  final String account;

  /// 受益人 SS58 地址。
  final String beneficiary;

  /// 发起人 SS58 地址。
  final String proposer;

  /// 0=voting, 1=passed, 2=rejected, null=unknown。
  final int? status;

  /// 个人多签治理 AccountId。
  Uint8List get institutionBytes {
    return Uint8List.fromList(_hexDecode(account));
  }

  CloseProposalInfo copyWithStatus(int? newStatus) {
    return CloseProposalInfo(
      proposalId: proposalId,
      account: account,
      beneficiary: beneficiary,
      proposer: proposer,
      status: newStatus,
    );
  }
}

/// 个人多签账户状态。
enum MultisigStatus {
  /// 提案投票中，尚未激活。
  pending,

  /// 投票通过、入金完成，已激活。
  active,
}

/// 个人多签账户链上信息。
///
/// 个人状态和管理员来自 `PersonalAdmins`，动态阈值来自 `InternalVote`。
class AccountInfo {
  const AccountInfo({
    required this.adminsLen,
    required this.threshold,
    required this.admins,
    required this.status,
  });

  final int adminsLen;
  final int? threshold;

  /// 管理员公钥列表（hex，不含 0x 前缀）。
  final List<String> admins;

  final MultisigStatus status;
}

Uint8List _hexDecode(String hex) {
  final h = hex.startsWith('0x') ? hex.substring(2) : hex;
  final result = Uint8List(h.length ~/ 2);
  for (var i = 0; i < result.length; i++) {
    result[i] = int.parse(h.substring(i * 2, i * 2 + 2), radix: 16);
  }
  return result;
}
