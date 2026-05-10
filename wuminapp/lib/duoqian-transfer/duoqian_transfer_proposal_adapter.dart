import 'package:flutter/material.dart';

import 'package:wuminapp_mobile/duoqian-transfer/duoqian_transfer_cache.dart';
import 'package:wuminapp_mobile/duoqian-transfer/duoqian_transfer_detail_page.dart';
import 'package:wuminapp_mobile/duoqian-transfer/duoqian_transfer_models.dart';
import 'package:wuminapp_mobile/duoqian-transfer/duoqian_transfer_service.dart';
import 'package:wuminapp_mobile/common/institution_info.dart';
import 'package:wuminapp_mobile/common/proposal/proposal_context.dart';
import 'package:wuminapp_mobile/common/proposal/proposal_models.dart';
import 'package:wuminapp_mobile/util/amount_format.dart';

/// 多签转账给外部列表页使用的只读适配器。
///
/// 外部页面只调用这里的标题、摘要、图标、跳转和缓存清理能力，
/// 不直接判断多签转账模型，也不直接打开多签转账详情页。
class DuoqianTransferProposalAdapter {
  DuoqianTransferProposalAdapter._();

  static TransferProposalInfo? _transfer(ProposalWithDetail proposal) {
    final detail =
        proposal.businessDetails[DuoqianTransferProposalDetailKeys.transfer];
    return detail is TransferProposalInfo ? detail : null;
  }

  static SafetyFundProposalInfo? _safetyFund(ProposalWithDetail proposal) {
    final detail =
        proposal.businessDetails[DuoqianTransferProposalDetailKeys.safetyFund];
    return detail is SafetyFundProposalInfo ? detail : null;
  }

  static SweepProposalInfo? _sweep(ProposalWithDetail proposal) {
    final detail =
        proposal.businessDetails[DuoqianTransferProposalDetailKeys.sweep];
    return detail is SweepProposalInfo ? detail : null;
  }

  static bool matches(ProposalWithDetail proposal) {
    return _transfer(proposal) != null ||
        _safetyFund(proposal) != null ||
        _sweep(proposal) != null;
  }

  static String? title(ProposalWithDetail proposal, String proposalId) {
    if (_transfer(proposal) != null) {
      return '转账提案 $proposalId';
    }
    if (_safetyFund(proposal) != null) {
      return '安全基金转账 $proposalId';
    }
    if (_sweep(proposal) != null) {
      return '手续费划转 $proposalId';
    }
    return null;
  }

  static String? subtitle(ProposalWithDetail proposal, String status) {
    final transfer = _transfer(proposal);
    if (transfer != null) {
      return '${AmountFormat.format(transfer.amountYuan, symbol: '')} 元 · $status';
    }
    final safetyFund = _safetyFund(proposal);
    if (safetyFund != null) {
      return '安全基金转账 ${AmountFormat.format(safetyFund.amountYuan, symbol: '')} 元 · $status';
    }
    final sweep = _sweep(proposal);
    if (sweep != null) {
      return '手续费划转 ${AmountFormat.format(sweep.amountYuan, symbol: '')} 元 · $status';
    }
    return null;
  }

  static String? listSummary(ProposalWithDetail proposal) {
    final transfer = _transfer(proposal);
    if (transfer != null) {
      return '转账 ${AmountFormat.format(transfer.amountYuan, symbol: '')} 元';
    }
    final safetyFund = _safetyFund(proposal);
    if (safetyFund != null) {
      return '安全基金转账 ${AmountFormat.format(safetyFund.amountYuan, symbol: '')} 元';
    }
    final sweep = _sweep(proposal);
    if (sweep != null) {
      return '手续费划转 ${AmountFormat.format(sweep.amountYuan, symbol: '')} 元';
    }
    return null;
  }

  static IconData? icon(ProposalWithDetail proposal) {
    if (_transfer(proposal) != null) {
      return Icons.send_outlined;
    }
    if (_safetyFund(proposal) != null) {
      return Icons.health_and_safety_outlined;
    }
    if (_sweep(proposal) != null) {
      return Icons.account_balance_wallet_outlined;
    }
    return null;
  }

  static Future<bool> openDetail(
    BuildContext context, {
    required ProposalWithDetail proposal,
    required InstitutionInfo? institution,
    required ProposalContext proposalContext,
  }) async {
    if (institution == null) {
      return false;
    }

    final proposalId = proposal.meta.proposalId;
    final kind = _detailKind(proposal);
    if (kind == null) {
      return false;
    }

    await Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => DuoqianTransferDetailPage(
          institution: institution,
          proposalId: proposalId,
          proposalContext: proposalContext,
          kind: kind,
        ),
      ),
    );
    return true;
  }

  static void clearCache() {
    DuoqianTransferCache.clear();
  }

  static DuoqianTransferKind? _detailKind(ProposalWithDetail proposal) {
    if (_transfer(proposal) != null) {
      return DuoqianTransferKind.transfer;
    }
    if (_safetyFund(proposal) != null) {
      return DuoqianTransferKind.safetyFund;
    }
    if (_sweep(proposal) != null) {
      return DuoqianTransferKind.sweep;
    }
    return null;
  }
}

/// 多签转账模块导出的提案数据源适配器。
class DuoqianTransferProposalFeed {
  DuoqianTransferProposalFeed({DuoqianTransferService? service})
      : _service = service ?? DuoqianTransferService();

  final DuoqianTransferService _service;

  Future<double> fetchInstitutionBalance(InstitutionInfo institution) {
    return _service.fetchInstitutionBalance(institution);
  }

  Future<List<ProposalWithDetail>> fetchInstitutionVisibleProposals(
      String sfidNumber) {
    return _service.fetchInstitutionVisibleProposals(sfidNumber);
  }

  Future<List<int>> fetchProposalIdsByOrg(int org) {
    return _service.fetchProposalIdsByOrg(org);
  }

  Future<List<ProposalWithDetail>> fetchProposalsByIds(List<int> ids) {
    return _service.fetchProposalsByIds(ids);
  }
}
