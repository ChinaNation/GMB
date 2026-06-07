import 'package:flutter/material.dart';

import 'package:wuminapp_mobile/transaction/duoqian-transfer/duoqian_transfer_cache.dart';
import 'package:wuminapp_mobile/transaction/duoqian-transfer/duoqian_transfer_detail_page.dart';
import 'package:wuminapp_mobile/transaction/duoqian-transfer/duoqian_transfer_models.dart';
import 'package:wuminapp_mobile/transaction/duoqian-transfer/duoqian_transfer_service.dart';
import 'package:wuminapp_mobile/governance/shared/institution_info.dart';
import 'package:wuminapp_mobile/governance/shared/proposal/proposal_context.dart';
import 'package:wuminapp_mobile/governance/shared/proposal/proposal_models.dart';
import 'package:wuminapp_mobile/my/util/amount_format.dart';

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
    DuoqianTransferProposalFeed.clearCache();
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

  static const Duration _balanceCacheTtl = Duration(seconds: 10);
  static const Duration _proposalCacheTtl = Duration(seconds: 20);

  static final Map<String, _TimedValue<double>> _balanceCache = {};
  static final Map<String, Future<double>> _balanceInFlight = {};
  static final Map<String, int> _balanceFetchTokens = {};
  static final Map<String, _TimedValue<List<ProposalWithDetail>>>
      _visibleProposalCache = {};
  static final Map<String, Future<List<ProposalWithDetail>>>
      _visibleProposalInFlight = {};
  static final Map<String, int> _visibleProposalFetchTokens = {};
  static int _nextFetchToken = 0;

  final DuoqianTransferService _service;

  Future<double> fetchInstitutionBalance(
    InstitutionInfo institution, {
    bool forceRefresh = false,
  }) {
    final key = _balanceKey(institution);
    final cached = _balanceCache[key];
    if (!forceRefresh && cached != null && cached.isFresh(_balanceCacheTtl)) {
      return Future.value(cached.value);
    }

    final inFlight = _balanceInFlight[key];
    if (!forceRefresh && inFlight != null) return inFlight;

    final token = ++_nextFetchToken;
    _balanceFetchTokens[key] = token;
    final future = _service.fetchInstitutionBalance(institution).then((value) {
      if (_balanceFetchTokens[key] == token) {
        _balanceCache[key] = _TimedValue(value);
      }
      return value;
    });
    _balanceInFlight[key] = future;
    return future.whenComplete(() {
      if (_balanceInFlight[key] == future) {
        _balanceInFlight.remove(key);
      }
      if (_balanceFetchTokens[key] == token) {
        _balanceFetchTokens.remove(key);
      }
    });
  }

  Future<List<ProposalWithDetail>> fetchInstitutionVisibleProposals(
    InstitutionInfo institution, {
    bool forceRefresh = false,
  }) {
    final sfidNumber = institution.sfidNumber;
    final cached = _visibleProposalCache[sfidNumber];
    if (!forceRefresh && cached != null && cached.isFresh(_proposalCacheTtl)) {
      return Future.value(cached.value);
    }

    final inFlight = _visibleProposalInFlight[sfidNumber];
    if (!forceRefresh && inFlight != null) return inFlight;

    final token = ++_nextFetchToken;
    _visibleProposalFetchTokens[sfidNumber] = token;
    // 中文注释：机构页提案查询会读取机构索引和年度联合提案，短缓存只减少重复读取，不改变链上真值。
    final future = _service.fetchInstitutionVisibleProposals(institution).then(
      (value) {
        final immutableValue = List<ProposalWithDetail>.unmodifiable(value);
        if (_visibleProposalFetchTokens[sfidNumber] == token) {
          _visibleProposalCache[sfidNumber] = _TimedValue(immutableValue);
        }
        return immutableValue;
      },
    );
    _visibleProposalInFlight[sfidNumber] = future;
    return future.whenComplete(() {
      if (_visibleProposalInFlight[sfidNumber] == future) {
        _visibleProposalInFlight.remove(sfidNumber);
      }
      if (_visibleProposalFetchTokens[sfidNumber] == token) {
        _visibleProposalFetchTokens.remove(sfidNumber);
      }
    });
  }

  Future<List<int>> fetchProposalIdsByOrg(int org) {
    return _service.fetchProposalIdsByOrg(org);
  }

  Future<List<ProposalWithDetail>> fetchProposalsByIds(List<int> ids) {
    return _service.fetchProposalsByIds(ids);
  }

  static void clearCache() {
    _balanceCache.clear();
    _balanceInFlight.clear();
    _balanceFetchTokens.clear();
    _visibleProposalCache.clear();
    _visibleProposalInFlight.clear();
    _visibleProposalFetchTokens.clear();
  }

  static String _balanceKey(InstitutionInfo institution) {
    return '${institution.sfidNumber}:${institution.mainAddress}';
  }
}

class _TimedValue<T> {
  _TimedValue(this.value) : createdAt = DateTime.now();

  final T value;
  final DateTime createdAt;

  bool isFresh(Duration ttl) => DateTime.now().difference(createdAt) < ttl;
}
