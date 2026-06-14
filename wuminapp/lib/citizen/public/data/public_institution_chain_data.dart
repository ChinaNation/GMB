// 公权机构详情页的链上数据源抽象(ADR-018 §九 卡C)。
//
// 中文注释:把"余额/管理员/提案"三类链读隔离在可注入接口后,详情页逻辑(派生地址、
// 订阅、展示)可用 fake 单测;生产用 [LivePublicInstitutionChainData] 走既有
// ChainRpc(ChainReadCache 卡⑤)/ InstitutionAdminService / ProposalFeed(卡①)。
// R1/R2:余额走精确整键批量,提案走年缓存客户端过滤,均不长前缀扫描。

import 'dart:typed_data';

import 'package:flutter/foundation.dart' show listEquals;

import 'package:wuminapp_mobile/governance/admins-change/models/admin_account.dart';
import 'package:wuminapp_mobile/governance/admins-change/services/institution_admin_service.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';
import 'package:wuminapp_mobile/transaction/duoqian-transfer/duoqian_transfer_proposal_adapter.dart';

/// 提案摘要(详情页列表用)。
class PublicProposalSummary {
  const PublicProposalSummary({required this.idLabel, required this.status});

  final String idLabel;

  /// 0=表决中 1=通过 2=否决。
  final int status;

  String get statusLabel => switch (status) {
        1 => '已通过',
        2 => '已否决',
        _ => '表决中',
      };
}

/// 公权机构详情页链上数据源。
abstract interface class PublicInstitutionChainData {
  /// 批量余额(hex→元);走精确整键 + ChainReadCache。
  Future<Map<String, double>> balances(List<String> pubkeyHexes);

  /// 机构主账户管理员公钥列表(AdminsChange::AdminAccounts)。
  Future<List<String>> admins({
    required String mainAccountHex,
    required String displayName,
  });

  /// 该机构当年提案(按 institutionBytes==主账户 id 过滤年缓存)。
  Future<List<PublicProposalSummary>> proposals(Uint8List mainAccountId);
}

/// 生产实现:复用既有链读基础设施。链读需联网,真机验证。
class LivePublicInstitutionChainData implements PublicInstitutionChainData {
  LivePublicInstitutionChainData({
    ChainRpc? chainRpc,
    InstitutionAdminService? adminService,
    DuoqianTransferProposalFeed? feed,
  })  : _chainRpc = chainRpc ?? ChainRpc(),
        _adminService = adminService ?? InstitutionAdminService(),
        _feed = feed ?? DuoqianTransferProposalFeed();

  final ChainRpc _chainRpc;
  final InstitutionAdminService _adminService;
  final DuoqianTransferProposalFeed _feed;

  @override
  Future<Map<String, double>> balances(List<String> pubkeyHexes) {
    if (pubkeyHexes.isEmpty) return Future.value(const {});
    return _chainRpc.fetchFinalizedBalances(pubkeyHexes);
  }

  @override
  Future<List<String>> admins({
    required String mainAccountHex,
    required String displayName,
  }) {
    // 公权机构主账户走 institutionAccount org=4(公权机构账户)。
    final identity = AdminAccountIdentity.institutionAccount(
      accountHex: mainAccountHex,
      org: 4,
      displayName: displayName,
    );
    return _adminService.fetchAdmins(identity);
  }

  @override
  Future<List<PublicProposalSummary>> proposals(
    Uint8List mainAccountId,
  ) async {
    final all = await _feed.currentYearProposals();
    final out = <PublicProposalSummary>[];
    for (final p in all) {
      final ib = p.meta.institutionBytes;
      if (ib != null && listEquals(ib, mainAccountId)) {
        out.add(PublicProposalSummary(
          idLabel: '提案 #${p.meta.proposalId}',
          status: p.meta.status,
        ));
      }
    }
    return out;
  }
}
