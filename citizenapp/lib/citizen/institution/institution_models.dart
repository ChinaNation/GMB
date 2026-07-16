import 'dart:typed_data';

/// 公权/私权机构 关闭机构多签账户提案详情（从链上 ProposalData 解码）。
///
/// 链上 SCALE 布局（`pub-mgmt`/`pri-mgmt` + ACTION_CLOSE = 2 前缀之后）：
///   actor_cid_number + institution_account: AccountId32(32)
///   + beneficiary: AccountId32(32) + proposer: AccountId32(32)
class CloseProposalInfo {
  const CloseProposalInfo({
    required this.proposalId,
    required this.actorCidNumber,
    required this.institutionAccount,
    required this.beneficiary,
    required this.proposer,
    this.status,
  });

  final int proposalId;
  final String actorCidNumber;

  /// 多签账户公钥 hex（32 字节，不含 0x 前缀）。
  final String institutionAccount;

  /// 受益人 SS58 地址。
  final String beneficiary;

  /// 发起人 SS58 地址。
  final String proposer;

  /// 0=voting, 1=passed, 2=rejected, null=unknown。
  final int? status;

  /// 机构多签 AccountId32。
  Uint8List get institutionBytes {
    final addrBytes = _hexDecode(institutionAccount);
    return Uint8List.fromList(addrBytes);
  }

  CloseProposalInfo copyWithStatus(int? newStatus) {
    return CloseProposalInfo(
      proposalId: proposalId,
      actorCidNumber: actorCidNumber,
      institutionAccount: institutionAccount,
      beneficiary: beneficiary,
      proposer: proposer,
      status: newStatus,
    );
  }
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
