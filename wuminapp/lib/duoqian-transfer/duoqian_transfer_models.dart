import 'dart:typed_data';

/// 多签转账模块写入 ProposalWithDetail.businessDetails 的键名。
class DuoqianTransferProposalDetailKeys {
  DuoqianTransferProposalDetailKeys._();

  static const transfer = 'duoqian-transfer.transfer';
  static const safetyFund = 'duoqian-transfer.safety-fund';
  static const sweep = 'duoqian-transfer.sweep';
}

/// 多签转账提案链上数据。
class TransferProposalInfo {
  const TransferProposalInfo({
    required this.proposalId,
    required this.institutionBytes,
    required this.beneficiary,
    required this.amountFen,
    required this.remark,
    required this.proposer,
    this.status,
  });

  final int proposalId;
  final Uint8List institutionBytes;
  final String beneficiary; // SS58
  final BigInt amountFen;
  final String remark;
  final String proposer; // SS58

  /// 0=voting, 1=passed, 2=rejected, null=unknown
  final int? status;

  double get amountYuan => amountFen.toDouble() / 100;

  TransferProposalInfo copyWithStatus(int? newStatus) {
    return TransferProposalInfo(
      proposalId: proposalId,
      institutionBytes: institutionBytes,
      beneficiary: beneficiary,
      amountFen: amountFen,
      remark: remark,
      proposer: proposer,
      status: newStatus,
    );
  }
}

/// 安全基金转账提案详情（从 SafetyFundProposalActions 存储解码）。
class SafetyFundProposalInfo {
  const SafetyFundProposalInfo({
    required this.proposalId,
    required this.beneficiary,
    required this.amountFen,
    required this.remark,
    required this.proposer,
    this.status,
  });

  final int proposalId;
  final String beneficiary; // SS58
  final BigInt amountFen;
  final String remark;
  final String proposer; // SS58
  final int? status;

  double get amountYuan => amountFen.toDouble() / 100;
}

/// 手续费划转提案详情（从 SweepProposalActions 存储解码）。
class SweepProposalInfo {
  const SweepProposalInfo({
    required this.proposalId,
    required this.institutionBytes,
    required this.amountFen,
    this.status,
  });

  final int proposalId;
  final Uint8List institutionBytes;
  final BigInt amountFen;
  final int? status;

  double get amountYuan => amountFen.toDouble() / 100;
}
