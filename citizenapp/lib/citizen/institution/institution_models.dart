import 'dart:typed_data';

/// 公权/私权机构 关闭机构多签账户提案详情（从链上 ProposalData 解码）。
///
/// 链上 SCALE 布局（`pub-mgmt`/`pri-mgmt` + ACTION_CLOSE = 2 前缀之后）：
///   account: AccountId32(32) + beneficiary: AccountId32(32)
///   + proposer: AccountId32(32)
class CloseMultisigProposalInfo {
  const CloseMultisigProposalInfo({
    required this.proposalId,
    required this.account,
    required this.beneficiary,
    required this.proposer,
    this.status,
  });

  final int proposalId;

  /// 多签账户公钥 hex（32 字节，不含 0x 前缀）。
  final String account;

  /// 受益人 SS58 地址。
  final String beneficiary;

  /// 发起人 SS58 地址。
  final String proposer;

  /// 0=voting, 1=passed, 2=rejected, null=unknown。
  final int? status;

  /// 机构多签 AccountId32。
  Uint8List get institutionBytes {
    final addrBytes = _hexDecode(account);
    return Uint8List.fromList(addrBytes);
  }

  CloseMultisigProposalInfo copyWithStatus(int? newStatus) {
    return CloseMultisigProposalInfo(
      proposalId: proposalId,
      account: account,
      beneficiary: beneficiary,
      proposer: proposer,
      status: newStatus,
    );
  }
}

/// 多签账户状态。
enum InstitutionStatus {
  /// 提案投票中，尚未激活。
  pending,

  /// 投票通过、入金完成，已激活。
  active,
}

/// 多签账户链上信息。
///
/// 机构状态来自 `PublicManage/PrivateManage::InstitutionAccounts`，
/// 管理员来自对应机构管理员 pallet 的 `AdminAccounts`，动态阈值来自 `InternalVote`。
class InstitutionAccountInfo {
  const InstitutionAccountInfo({
    required this.adminsLen,
    required this.threshold,
    required this.admins,
    required this.status,
  });

  final int adminsLen;
  final int? threshold;

  /// 管理员公钥列表（hex，不含 0x 前缀）。
  final List<String> admins;

  final InstitutionStatus status;
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
