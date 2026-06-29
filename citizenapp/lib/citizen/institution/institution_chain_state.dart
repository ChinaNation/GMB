// 统一机构链态读服务(ADR-028 决策 2)——合并公权 `LivePublicInstitutionChainData`
// 与治理侧 admins 读取为一套:按机构主账户读 余额 / 管理员 / 提案,公权治理同路径。
//
// 中文注释:
// - 管理员身份路由按机构码统一:储备治理三档走 governanceInstitution,
//   其它 GenesisAdmins 创世机构走 genesisInstitution,
//   其余注册机构走 institutionAccount 并用**真实机构码**(修复公权侧旧链读路径对所有
//   公权机构硬编码 'CGOV' 的 bug —— 见 ADR-028 决策 2 / 风险点 5)。
// - 读取遵守 ADR-018:余额走精确整键批量,提案走当年共享缓存客户端过滤,不长前缀扫描。

import 'dart:typed_data';

import 'package:flutter/foundation.dart' show listEquals;

import 'package:citizenapp/citizen/institution/institution.dart';
import 'package:citizenapp/citizen/institution/institution_classification.dart';
import 'package:citizenapp/citizen/proposal/admins-change/models/admin_account.dart';
import 'package:citizenapp/citizen/proposal/admins-change/services/institution_admin_service.dart';
import 'package:citizenapp/citizen/shared/admin_profile.dart';
import 'package:citizenapp/citizen/shared/institution_code_label.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/transaction/multisig-transfer/multisig_transfer_proposal_adapter.dart';

/// 机构提案摘要(详情页提案列表用)。
class InstitutionProposalSummary {
  const InstitutionProposalSummary({
    required this.proposalId,
    required this.idLabel,
    required this.status,
  });

  final int proposalId;
  final String idLabel;

  /// 0=表决中 1=通过 2=否决 3=已执行 4=执行失败。
  final int status;

  String get statusLabel => switch (status) {
        1 => '已通过',
        2 => '已否决',
        3 => '已执行',
        4 => '执行失败',
        _ => '表决中',
      };
}

/// 统一机构链态读服务接口(可注入 fake 单测)。
abstract interface class InstitutionChainState {
  /// 批量余额(hex→元);精确整键 + ChainReadCache。
  Future<Map<String, double>> balances(List<String> pubkeyHexes);

  /// 机构主账户管理员公钥列表(按机构码读取 Genesis/Public/Private Admins)。
  Future<List<String>> admins(Institution institution);

  /// 机构主账户管理员**完整资料**(cid/姓名/职务/任期/来源,A2;个人多签仅 account)。
  Future<List<AdminProfile>> adminProfiles(Institution institution);

  /// 该机构当年提案(按 institutionBytes==主账户 id 过滤当年缓存)。
  Future<List<InstitutionProposalSummary>> proposals(Uint8List mainAccountId);
}

/// 生产实现:复用既有链读基础设施。链读需联网,真机验证。
class LiveInstitutionChainState implements InstitutionChainState {
  LiveInstitutionChainState({
    ChainRpc? chainRpc,
    InstitutionAdminService? adminService,
    MultisigTransferProposalFeed? feed,
  })  : _chainRpc = chainRpc ?? ChainRpc(),
        _adminService = adminService ?? InstitutionAdminService(),
        _feed = feed ?? MultisigTransferProposalFeed();

  final ChainRpc _chainRpc;
  final InstitutionAdminService _adminService;
  final MultisigTransferProposalFeed _feed;

  @override
  Future<Map<String, double>> balances(List<String> pubkeyHexes) {
    if (pubkeyHexes.isEmpty) return Future.value(const {});
    return _chainRpc.fetchFinalizedBalances(pubkeyHexes);
  }

  @override
  Future<List<String>> admins(Institution institution) {
    return _adminService.fetchAdmins(adminIdentityOf(institution));
  }

  @override
  Future<List<AdminProfile>> adminProfiles(Institution institution) {
    return _adminService.fetchAdminProfiles(adminIdentityOf(institution));
  }

  @override
  Future<List<InstitutionProposalSummary>> proposals(
    Uint8List mainAccountId,
  ) async {
    final all = await _feed.currentYearProposals();
    final out = <InstitutionProposalSummary>[];
    for (final p in all) {
      final ib = p.meta.institutionBytes;
      if (ib != null && listEquals(ib, mainAccountId)) {
        out.add(InstitutionProposalSummary(
          proposalId: p.meta.proposalId,
          idLabel: '提案 #${p.meta.proposalId}',
          status: p.meta.status,
        ));
      }
    }
    return out;
  }
}

/// 机构 → 管理员账户身份(单一路由,机构码决定):
/// - 储备治理三档(NRC/PRC/PRB)→ governanceInstitution;
/// - 其它 GenesisAdmins 创世机构 → genesisInstitution;
/// - 其余注册机构 → institutionAccount(用真实机构码,修复旧 'CGOV' 硬编码)。
AdminAccountIdentity adminIdentityOf(Institution institution) {
  if (InstitutionClassification.isGovernance(institution.institutionCode)) {
    return AdminAccountIdentity.governanceInstitution(
      accountHex: institution.mainAccountHex,
      orgType: institution.orgType,
      accountLabel: institution.cidFullName,
    );
  }
  if (InstitutionCodeLabel.isGenesisAdminCode(institution.institutionCode)) {
    return AdminAccountIdentity.genesisInstitution(
      accountHex: institution.mainAccountHex,
      institutionCode: institution.institutionCode,
      accountLabel: institution.cidFullName,
    );
  }
  return AdminAccountIdentity.institutionAccount(
    accountHex: institution.mainAccountHex,
    institutionCode: institution.institutionCode,
    accountLabel: institution.cidFullName,
  );
}
