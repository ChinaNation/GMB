import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:polkadart/polkadart.dart'
    show ExtrinsicPayload, Hasher, SignatureType, SigningPayload;
import 'package:polkadart/scale_codec.dart' show CompactBigIntCodec, ByteOutput;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import '../rpc/chain_rpc.dart';
import '../rpc/nonce_manager.dart';
import 'duoqian_manage_models.dart';
import 'duoqian_manage_service.dart';
import 'institution_data.dart';
import 'proposal_cache.dart';
import 'runtime_upgrade_service.dart';

/// 机构转账提案链上交互服务。
///
/// 负责 extrinsic 编码/提交 和 storage 查询。
class TransferProposalService {
  TransferProposalService({ChainRpc? chainRpc}) : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;

  // ──── 常量 ────

  /// DuoqianTransfer pallet index（runtime pallet_index=19）。
  ///
  /// Phase 3(2026-04-22): 本 pallet 的所有 vote_X / finalize_X 已删除,
  /// 只保留 propose_X(0/1/2) 与 execute_X(3/4/5) 两组路径;
  /// 管理员投票一律走 VotingEngine(9).internal_vote(0)。
  static const _palletIndex = 19;

  /// propose_transfer call_index=0。
  static const _proposeCallIndex = 0;

  /// propose_safety_fund_transfer call_index=1。
  static const _proposeSafetyFundCallIndex = 1;

  /// propose_sweep_to_main call_index=2。
  static const _proposeSweepCallIndex = 2;

  /// Mortal era 周期。
  static const _eraPeriod = 64;

  // ──── Extrinsic 提交 ────

  /// 提交 propose_transfer extrinsic。
  ///
  /// 返回交易哈希 hex（含 0x 前缀）和使用的 nonce。
  Future<({String txHash, int usedNonce})> submitProposeTransfer({
    required InstitutionInfo institution,
    required String beneficiaryAddress,
    required double amountYuan,
    required String remark,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    final callData = _buildProposeTransferCall(
      org: institution.orgType,
      institutionIdentity: institution.shenfenId,
      beneficiaryAddress: beneficiaryAddress,
      amountFen: BigInt.from((amountYuan * 100).round()),
      remark: remark,
    );
    return _signAndSubmit(
      callData: callData,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
    );
  }

  /// 提交 propose_safety_fund_transfer extrinsic（安全基金转账提案）。
  Future<({String txHash, int usedNonce})> submitProposeSafetyFund({
    required String beneficiaryAddress,
    required double amountYuan,
    required String remark,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    final callData = _buildProposeSafetyFundCall(
      beneficiaryAddress: beneficiaryAddress,
      amountYuan: amountYuan,
      remark: remark,
    );
    return _signAndSubmit(
      callData: callData,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
    );
  }

  /// 提交 propose_sweep_to_main extrinsic（手续费划转提案）。
  Future<({String txHash, int usedNonce})> submitProposeSweep({
    required InstitutionInfo institution,
    required double amountYuan,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    final callData = _buildProposeSweepCall(
      institutionIdentity: institution.shenfenId,
      amountYuan: amountYuan,
    );
    return _signAndSubmit(
      callData: callData,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
    );
  }

  // ──── 链上查询 ────

  /// 查询机构 duoqian_address 的可用余额（元）。
  Future<double> fetchInstitutionBalance(InstitutionInfo institution) {
    return _rpc.fetchBalance(institution.duoqianAddress);
  }

  /// 每个机构最多同时 10 个活跃提案（全局，不区分提案类型）。
  static const maxActiveProposalsPerInstitution = 10;

  /// 查询机构活跃的提案 ID 列表（从 VotingEngine 全局存储读取）。
  Future<List<int>> fetchActiveProposalIds(String shenfenId) async {
    final key = _buildStorageKey(
      'VotingEngine',
      'ActiveProposalsByInstitution',
      _institutionIdentityToFixed48(shenfenId),
    );
    final data = await _rpc.fetchStorage('0x${_hexEncode(key)}');
    if (data == null || data.isEmpty) return const [];
    // SCALE: BoundedVec<u64> = Compact<u32> length + N × u64_le
    final (count, lenSize) = _decodeCompact(data, 0);
    final ids = <int>[];
    var offset = lenSize;
    for (var i = 0; i < count && offset + 8 <= data.length; i++) {
      ids.add(_decodeU64(data.sublist(offset, offset + 8)));
      offset += 8;
    }
    return ids;
  }

  /// 查询投票计数。
  Future<({int yes, int no})> fetchVoteTally(int proposalId) async {
    final key = _buildStorageKey(
      'VotingEngine',
      'InternalTallies',
      _u64ToLeBytes(proposalId),
    );
    final data = await _rpc.fetchStorage('0x${_hexEncode(key)}');
    if (data == null || data.length < 8) return (yes: 0, no: 0);
    // VoteCountU32: { yes: u32, no: u32 } — 4+4 bytes little-endian
    final yes = _decodeU32(data, 0);
    final no = _decodeU32(data, 4);
    return (yes: yes, no: no);
  }

  /// 查询提案状态。返回 status（0=voting, 1=passed, 2=rejected），null 表示不存在。
  Future<int?> fetchProposalStatus(int proposalId) async {
    final key = _buildStorageKey(
      'VotingEngine',
      'Proposals',
      _u64ToLeBytes(proposalId),
    );
    final data = await _rpc.fetchStorage('0x${_hexEncode(key)}');
    if (data == null) return null;
    // Proposal 结构：kind(u8) + stage(u8) + status(u8) + ...
    // status 在第 3 字节（offset=2）
    if (data.length < 3) return null;
    return data[2];
  }

  /// 查询提案完整元数据（status + institution bytes）。
  /// 返回 null 表示提案不存在。
  Future<ProposalMeta?> fetchProposalMeta(int proposalId) async {
    final key = _buildStorageKey(
      'VotingEngine',
      'Proposals',
      _u64ToLeBytes(proposalId),
    );
    final data = await _rpc.fetchStorage('0x${_hexEncode(key)}');
    if (data == null || data.length < 3) return null;

    final kind = data[0];
    final stage = data[1];
    final status = data[2];

    // internal_org: Option<u8>
    var offset = 3;
    int? internalOrg;
    if (offset < data.length && data[offset] == 1) {
      offset++;
      if (offset < data.length) {
        internalOrg = data[offset];
        offset++;
      }
    } else {
      offset++; // skip 0x00 (None)
    }

    // internal_institution: Option<[u8;48]>
    Uint8List? institutionBytes;
    if (offset < data.length && data[offset] == 1) {
      offset++;
      if (offset + 48 <= data.length) {
        institutionBytes =
            Uint8List.fromList(data.sublist(offset, offset + 48));
        offset += 48;
      }
    }

    return ProposalMeta(
      proposalId: proposalId,
      kind: kind,
      stage: stage,
      status: status,
      internalOrg: internalOrg,
      institutionBytes: institutionBytes,
    );
  }

  /// 查询全链所有活跃提案（status=0 投票中），按 ID 倒序。
  Future<List<ProposalWithDetail>> fetchAllActiveProposals() async {
    final nextId = await fetchNextProposalId();
    if (nextId == 0) return const [];

    // 计算当前年份的起始 ID
    final year = nextId ~/ 1000000;
    final startId = year * 1000000;

    // 并行查询所有提案元数据
    final metaFutures = <Future<ProposalMeta?>>[];
    for (var id = startId; id < nextId; id++) {
      metaFutures.add(fetchProposalMeta(id));
    }
    final metas = await Future.wait(metaFutures);

    // 收集所有存在的提案（包括已完成的，供历史查看）
    final results = <ProposalWithDetail>[];
    for (final meta in metas) {
      if (meta == null) continue;

      // 尝试获取业务详情（ProposalData）
      TransferProposalInfo? transferDetail;
      RuntimeUpgradeProposalInfo? runtimeUpgradeDetail;
      CreateDuoqianProposalInfo? createDuoqianDetail;
      CloseDuoqianProposalInfo? closeDuoqianDetail;
      SafetyFundProposalInfo? safetyFundDetail;
      SweepProposalInfo? sweepDetail;
      if (meta.kind == 0) {
        // 内部投票提案 → 先尝试管理提案,再尝试转账提案,最后尝试安全基金/手续费划转
        try {
          final manageService = DuoqianManageService(chainRpc: _rpc);
          final key = _buildStorageKey(
            'VotingEngine',
            'ProposalData',
            _u64ToLeBytes(meta.proposalId),
          );
          final raw = await _rpc.fetchStorage('0x${_hexEncode(key)}');
          if (raw != null && raw.isNotEmpty) {
            final manageDetail =
                manageService.decodeManageProposalData(meta.proposalId, raw);
            if (manageDetail is CreateDuoqianProposalInfo) {
              createDuoqianDetail = manageDetail;
            } else if (manageDetail is CloseDuoqianProposalInfo) {
              closeDuoqianDetail = manageDetail;
            } else {
              transferDetail = await fetchProposalAction(meta.proposalId);
            }
          }
        } catch (_) {}
        // 如果仍无匹配，尝试安全基金 / 手续费划转提案。
        // 原"省储行费率提案"(`RateProposalActions`)在 Step 2b-iv-b 随老
        // 省储行 pallet 一起下线,此处不再枚举。
        if (transferDetail == null &&
            createDuoqianDetail == null &&
            closeDuoqianDetail == null) {
          try {
            safetyFundDetail = await fetchSafetyFundAction(meta.proposalId);
          } catch (_) {}
          if (safetyFundDetail == null) {
            try {
              sweepDetail = await fetchSweepAction(meta.proposalId);
            } catch (_) {}
          }
        }
      } else if (meta.kind == 1) {
        // 联合投票提案 → 尝试解码为 runtime 升级提案
        try {
          final upgradeService = RuntimeUpgradeService(chainRpc: _rpc);
          runtimeUpgradeDetail =
              await upgradeService.fetchRuntimeUpgradeProposal(meta.proposalId);
        } catch (_) {}
      }

      // 如果联合提案且不是 runtime 升级，尝试检测决议发行/销毁 TAG
      String? resIssuanceSummary;
      String? resDestroySummary;
      if (meta.kind == 1 && runtimeUpgradeDetail == null) {
        try {
          final raw = await fetchProposalDataRaw(meta.proposalId);
          if (raw != null) {
            final tag = _detectJointProposalTag(raw);
            if (tag == 'res-iss') {
              resIssuanceSummary = '决议发行提案';
            } else if (tag == 'res-dst') {
              resDestroySummary = '决议销毁提案';
            }
          }
        } catch (_) {}
      }

      // 多签管理提案不在治理列表中显示
      if (createDuoqianDetail != null || closeDuoqianDetail != null) {
        continue;
      }

      results.add(ProposalWithDetail(
        meta: meta,
        transferDetail: transferDetail?.copyWithStatus(meta.status),
        runtimeUpgradeDetail: runtimeUpgradeDetail,
        safetyFundDetail: safetyFundDetail,
        sweepDetail: sweepDetail,
        resolutionIssuanceSummary: resIssuanceSummary,
        resolutionDestroySummary: resDestroySummary,
      ));
    }

    // 按 ID 倒序
    results.sort((a, b) => b.meta.proposalId.compareTo(a.meta.proposalId));
    return results;
  }

  // ──── 分页 + 缓存 + 批量查询 ────

  /// 分页查询提案：从 [startId] 往前（含 startId）加载 [count] 个。
  ///
  /// 优先读缓存，未命中的用 [fetchStorageBatch] 批量查询。
  /// 返回结果按 ID 倒序。
  Future<List<ProposalWithDetail>> fetchProposalPage(
      int startId, int count) async {
    final results = <ProposalWithDetail>[];
    final uncachedMetaKeys = <String>[];
    final uncachedMetaIds = <int>[];
    final cachedMetas = <int, ProposalMeta>{};
    final runtimeUpgradeService = RuntimeUpgradeService(chainRpc: _rpc);

    for (var id = startId; id > startId - count && id >= 0; id--) {
      final cached = ProposalCache.getMeta(id);
      if (cached != null) {
        cachedMetas[id] = cached;
      } else {
        final keyBytes =
            _buildStorageKey('VotingEngine', 'Proposals', _u64ToLeBytes(id));
        uncachedMetaKeys.add('0x${_hexEncode(keyBytes)}');
        uncachedMetaIds.add(id);
      }
    }

    // 批量查询未命中的 meta
    if (uncachedMetaKeys.isNotEmpty) {
      final batchResult = await _rpc.fetchStorageBatch(uncachedMetaKeys);
      for (var i = 0; i < uncachedMetaIds.length; i++) {
        final id = uncachedMetaIds[i];
        final data = batchResult[uncachedMetaKeys[i]];
        if (data != null && data.length >= 3) {
          final meta = _decodeProposalMeta(id, data);
          if (meta != null) {
            cachedMetas[id] = meta;
            ProposalCache.putMeta(id, meta);
          }
        }
      }
    }

    // 对有 meta 的提案，批量查询 ProposalData（先查缓存）。
    // 中文注释：联合投票提案不能再走“统一按转账解码”的旧逻辑，
    // 否则 runtime 升级这类提案会被漏掉或误判类型。
    final uncachedDetailKeys = <String>[];
    final uncachedDetailIds = <int>[];
    final cachedTransferDetails = <int, TransferProposalInfo>{};
    final cachedRuntimeUpgradeDetails = <int, RuntimeUpgradeProposalInfo>{};
    final cachedCreateDuoqianDetails = <int, CreateDuoqianProposalInfo>{};
    final cachedCloseDuoqianDetails = <int, CloseDuoqianProposalInfo>{};

    for (final entry in cachedMetas.entries) {
      final meta = entry.value;
      if (meta.kind == 1) {
        final cachedDetail = ProposalCache.getRuntimeUpgradeDetail(entry.key);
        if (cachedDetail != null) {
          cachedRuntimeUpgradeDetails[entry.key] = cachedDetail;
          continue;
        }
      } else {
        // 内部投票提案：先查转账缓存，再查管理缓存
        final cachedTransfer = ProposalCache.getTransferDetail(entry.key);
        if (cachedTransfer != null) {
          cachedTransferDetails[entry.key] = cachedTransfer;
          continue;
        }
        final cachedCreate = ProposalCache.getCreateDuoqianDetail(entry.key);
        if (cachedCreate != null) {
          cachedCreateDuoqianDetails[entry.key] = cachedCreate;
          continue;
        }
        final cachedClose = ProposalCache.getCloseDuoqianDetail(entry.key);
        if (cachedClose != null) {
          cachedCloseDuoqianDetails[entry.key] = cachedClose;
          continue;
        }
      }
      final keyBytes = _buildStorageKey(
          'VotingEngine', 'ProposalData', _u64ToLeBytes(entry.key));
      uncachedDetailKeys.add('0x${_hexEncode(keyBytes)}');
      uncachedDetailIds.add(entry.key);
    }

    if (uncachedDetailKeys.isNotEmpty) {
      final manageService = DuoqianManageService(chainRpc: _rpc);
      final batchResult = await _rpc.fetchStorageBatch(uncachedDetailKeys);
      for (var i = 0; i < uncachedDetailIds.length; i++) {
        final id = uncachedDetailIds[i];
        final meta = cachedMetas[id];
        final raw = batchResult[uncachedDetailKeys[i]];
        if (meta == null || raw == null || raw.isEmpty) {
          continue;
        }
        if (meta.kind == 1) {
          final runtimeDetail =
              runtimeUpgradeService.decodeRuntimeUpgradeStorageValue(id, raw);
          if (runtimeDetail != null) {
            cachedRuntimeUpgradeDetails[id] = runtimeDetail;
            ProposalCache.putRuntimeUpgradeDetail(id, runtimeDetail);
          }
          continue;
        }

        // 内部投票提案：先尝试解码为多签管理提案，失败再尝试转账提案。
        // 管理提案的 ProposalData 首字节（BoundedVec 内容）为 ACTION_CREATE(1) 或 ACTION_CLOSE(2)。
        final manageDetail = manageService.decodeManageProposalData(id, raw);
        if (manageDetail is CreateDuoqianProposalInfo) {
          cachedCreateDuoqianDetails[id] = manageDetail;
          ProposalCache.putCreateDuoqianDetail(id, manageDetail);
          continue;
        }
        if (manageDetail is CloseDuoqianProposalInfo) {
          cachedCloseDuoqianDetails[id] = manageDetail;
          ProposalCache.putCloseDuoqianDetail(id, manageDetail);
          continue;
        }

        final transferDetail = _decodeProposalData(id, raw);
        if (transferDetail != null) {
          cachedTransferDetails[id] = transferDetail;
          ProposalCache.putTransferDetail(id, transferDetail);
        }
      }
    }

    // 组装结果（跳过多签管理提案，这些在多签账户详情页单独展示）
    for (var id = startId; id > startId - count && id >= 0; id--) {
      final meta = cachedMetas[id];
      if (meta == null) continue;
      // 多签管理提案不在治理列表中显示
      if (cachedCreateDuoqianDetails.containsKey(id) ||
          cachedCloseDuoqianDetails.containsKey(id)) {
        continue;
      }
      final transferDetail = cachedTransferDetails[id];
      final runtimeUpgradeDetail = cachedRuntimeUpgradeDetails[id];
      final createDuoqianDetail = cachedCreateDuoqianDetails[id];
      final closeDuoqianDetail = cachedCloseDuoqianDetails[id];
      // 如果都没匹配到，尝试安全基金 / 手续费划转提案。
      // 原"省储行费率提案"(`RateProposalActions`)在 Step 2b-iv-b 随老省储
      // 行 pallet 一起下线,此处不再枚举 feeRateDetail。
      SafetyFundProposalInfo? safetyFundDetail;
      SweepProposalInfo? sweepDetail;
      if (transferDetail == null &&
          runtimeUpgradeDetail == null &&
          createDuoqianDetail == null &&
          closeDuoqianDetail == null &&
          meta.kind == 0) {
        try {
          safetyFundDetail = await fetchSafetyFundAction(id);
        } catch (_) {}
        if (safetyFundDetail == null) {
          try {
            sweepDetail = await fetchSweepAction(id);
          } catch (_) {}
        }
      }
      // 联合提案且不是 runtime 升级，尝试检测决议发行/销毁
      String? resIssuanceSummary;
      String? resDestroySummary;
      if (meta.kind == 1 && runtimeUpgradeDetail == null) {
        try {
          final raw = await fetchProposalDataRaw(id);
          if (raw != null) {
            final tag = _detectJointProposalTag(raw);
            if (tag == 'res-iss') {
              resIssuanceSummary = '决议发行提案';
            } else if (tag == 'res-dst') {
              resDestroySummary = '决议销毁提案';
            }
          }
        } catch (_) {}
      }
      results.add(ProposalWithDetail(
        meta: meta,
        transferDetail: transferDetail?.copyWithStatus(meta.status),
        runtimeUpgradeDetail: runtimeUpgradeDetail,
        createDuoqianDetail: createDuoqianDetail?.copyWithStatus(meta.status),
        closeDuoqianDetail: closeDuoqianDetail?.copyWithStatus(meta.status),
        safetyFundDetail: safetyFundDetail,
        sweepDetail: sweepDetail,
        resolutionIssuanceSummary: resIssuanceSummary,
        resolutionDestroySummary: resDestroySummary,
      ));
    }

    return results;
  }

  /// 查询当前年份内，对指定机构用户可见的提案事件。
  ///
  /// 中文注释：机构页除了显示本机构内部提案，也必须显示所有用户都可见的联合投票提案，
  /// 否则 runtime 升级这类联合投票提案无法在各机构入口被发现。
  Future<List<ProposalWithDetail>> fetchInstitutionVisibleProposals(
      String shenfenId) async {
    final nextId = await fetchNextProposalId();
    if (nextId == 0) return const <ProposalWithDetail>[];

    final yearStartId = _currentYearStartId(nextId);
    final institutionBytes = _institutionIdentityToFixed48(shenfenId);
    final visibleProposals = <ProposalWithDetail>[];

    const pageSize = 100;
    for (var startId = nextId - 1;
        startId >= yearStartId;
        startId -= pageSize) {
      final remaining = startId - yearStartId + 1;
      final count = remaining < pageSize ? remaining : pageSize;
      final page = await fetchProposalPage(startId, count);
      for (final proposal in page) {
        final transferDetail = proposal.transferDetail;
        if (transferDetail != null &&
            _bytesEqual(transferDetail.institutionBytes, institutionBytes)) {
          visibleProposals.add(proposal);
          continue;
        }
        // 中文注释：多签管理提案已在 fetchProposalPage 中过滤，此处不再匹配。
        // 内部投票提案（费率设置等）：通过 institutionBytes 匹配
        if (proposal.meta.kind == 0 &&
            proposal.meta.institutionBytes != null &&
            _bytesEqual(proposal.meta.institutionBytes!, institutionBytes)) {
          visibleProposals.add(proposal);
          continue;
        }
        if (proposal.meta.kind == 1) {
          visibleProposals.add(proposal);
        }
      }
    }

    return visibleProposals;
  }

  int _currentYearStartId(int nextId) {
    final year = nextId ~/ 1000000;
    return year * 1000000;
  }

  /// 查询指定机构的所有转账提案（包括已完成的），按 ID 倒序。
  Future<List<TransferProposalInfo>> fetchAllInstitutionProposals(
      String shenfenId) async {
    final visibleProposals = await fetchInstitutionVisibleProposals(shenfenId);
    final institutionBytes = _institutionIdentityToFixed48(shenfenId);
    final proposals = <TransferProposalInfo>[];

    for (final proposal in visibleProposals) {
      final detail = proposal.transferDetail;
      if (detail == null) {
        continue;
      }
      if (_bytesEqual(detail.institutionBytes, institutionBytes)) {
        proposals.add(detail.copyWithStatus(proposal.meta.status));
      }
    }

    proposals.sort((a, b) => b.proposalId.compareTo(a.proposalId));
    return proposals;
  }

  /// 从原始 SCALE 字节解码 ProposalMeta（与 fetchProposalMeta 相同逻辑）。
  ProposalMeta? _decodeProposalMeta(int proposalId, Uint8List data) {
    if (data.length < 3) return null;
    final kind = data[0];
    final stage = data[1];
    final status = data[2];

    var offset = 3;
    int? internalOrg;
    if (offset < data.length && data[offset] == 1) {
      offset++;
      if (offset < data.length) {
        internalOrg = data[offset];
        offset++;
      }
    } else {
      offset++;
    }

    Uint8List? institutionBytes;
    if (offset < data.length && data[offset] == 1) {
      offset++;
      if (offset + 48 <= data.length) {
        institutionBytes =
            Uint8List.fromList(data.sublist(offset, offset + 48));
      }
    }

    return ProposalMeta(
      proposalId: proposalId,
      kind: kind,
      stage: stage,
      status: status,
      internalOrg: internalOrg,
      institutionBytes: institutionBytes,
    );
  }

  /// 读取原始 ProposalData 存储字节。
  Future<Uint8List?> fetchProposalDataRaw(int proposalId) async {
    final key = _buildStorageKey(
      'VotingEngine',
      'ProposalData',
      _u64ToLeBytes(proposalId),
    );
    return _rpc.fetchStorage('0x${_hexEncode(key)}');
  }

  /// 检测联合提案 ProposalData 的 MODULE_TAG 前缀。
  /// 返回 'rt-upg'、'res-iss'、'res-dst' 或 null。
  static String? _detectJointProposalTag(Uint8List raw) {
    if (raw.length < 2) return null;
    // BoundedVec<u8>：Compact<len> + bytes
    final first = raw[0];
    final mode = first & 0x03;
    int offset;
    if (mode == 0) {
      offset = 1;
    } else if (mode == 1) {
      offset = 2;
    } else if (mode == 2) {
      offset = 4;
    } else {
      return null;
    }
    if (offset + 7 > raw.length) return null;
    final tag = String.fromCharCodes(raw.sublist(offset, offset + 7));
    if (tag == 'res-iss') return 'res-iss';
    if (tag == 'res-dst') return 'res-dst';
    if (offset + 6 <= raw.length) {
      final tag6 = String.fromCharCodes(raw.sublist(offset, offset + 6));
      if (tag6 == 'rt-upg') return 'rt-upg';
    }
    return null;
  }

  /// 从原始 SCALE 字节解码 ProposalData（BoundedVec<u8> → TransferAction）。
  TransferProposalInfo? _decodeProposalData(int proposalId, Uint8List raw) {
    try {
      int offset = 0;
      final (vecLen, lenBytes) = _decodeCompact(raw, offset);
      offset += lenBytes;
      if (offset + vecLen > raw.length) return null;
      final data = raw.sublist(offset, offset + vecLen);
      // 跳过 MODULE_TAG 前缀（"dq-xfer" = 7 字节）
      const tag = [0x64, 0x71, 0x2d, 0x78, 0x66, 0x65, 0x72]; // "dq-xfer"
      if (data.length < tag.length + 48 + 32 + 16 + 1 + 32) return null;
      for (var i = 0; i < tag.length; i++) {
        if (data[i] != tag[i]) return null;
      }
      return _decodeTransferAction(proposalId, data.sublist(tag.length));
    } catch (_) {
      return null;
    }
  }

  /// 查询某管理员对某提案的投票记录。null=未投票，true=赞成，false=反对。
  Future<bool?> fetchAdminVote(int proposalId, String pubkeyHex) async {
    final proposalIdBytes = _u64ToLeBytes(proposalId);
    final accountBytes = _hexDecode(pubkeyHex);

    // 双 key：blake2_128_concat(proposal_id) + blake2_128_concat(account)
    final palletHash = _twoxx128String('VotingEngine');
    final storageHash = _twoxx128String('InternalVotesByAccount');
    final key1 = _blake2128Concat(proposalIdBytes);
    final key2 = _blake2128Concat(accountBytes);

    final fullKey = Uint8List(
        palletHash.length + storageHash.length + key1.length + key2.length);
    var offset = 0;
    fullKey.setAll(offset, palletHash);
    offset += palletHash.length;
    fullKey.setAll(offset, storageHash);
    offset += storageHash.length;
    fullKey.setAll(offset, key1);
    offset += key1.length;
    fullKey.setAll(offset, key2);

    final data = await _rpc.fetchStorage('0x${_hexEncode(fullKey)}');
    if (data == null || data.isEmpty) return null;
    return data[0] == 1;
  }

  /// 查询 NextProposalId（投票引擎全局递增 ID）。
  Future<int> fetchNextProposalId() async {
    final palletHash = _twoxx128String('VotingEngine');
    final storageHash = _twoxx128String('NextProposalId');
    final key = Uint8List(palletHash.length + storageHash.length);
    key.setAll(0, palletHash);
    key.setAll(palletHash.length, storageHash);
    final data = await _rpc.fetchStorage('0x${_hexEncode(key)}');
    if (data == null || data.length < 8) return 0;
    return _decodeU64(data);
  }

  /// 查询单个转账提案详情。返回 null 表示不存在。
  ///
  /// ProposalData 是 BoundedVec<u8>，SCALE 编码为 Compact 长度前缀 + 原始字节。
  /// 原始字节为 TransferAction SCALE 布局：
  ///   institution: [u8;48] + beneficiary: AccountId32(32) + amount: u128(16)
  ///   + remark: Vec<u8>(Compact len + bytes) + proposer: AccountId32(32)
  Future<TransferProposalInfo?> fetchProposalAction(int proposalId) async {
    final key = _buildStorageKey(
      'VotingEngine',
      'ProposalData',
      _u64ToLeBytes(proposalId),
    );
    final raw = await _rpc.fetchStorage('0x${_hexEncode(key)}');
    if (raw == null || raw.isEmpty) return null;

    // ProposalData 存储为 BoundedVec<u8>，SCALE 编码：Compact<len> + bytes
    int offset = 0;
    final (vecLen, lenBytes) = _decodeCompact(raw, offset);
    offset += lenBytes;
    if (offset + vecLen > raw.length) return null;
    final data = raw.sublist(offset, offset + vecLen);

    // 跳过 MODULE_TAG 前缀（"dq-xfer" = 7 字节）
    const tag = [0x64, 0x71, 0x2d, 0x78, 0x66, 0x65, 0x72]; // "dq-xfer"
    if (data.length < tag.length + 48 + 32 + 16 + 1 + 32) return null;
    for (var i = 0; i < tag.length; i++) {
      if (data[i] != tag[i]) return null;
    }
    return _decodeTransferAction(proposalId, data.sublist(tag.length));
  }

  /// 解码 TransferAction SCALE 数据。
  TransferProposalInfo? _decodeTransferAction(int proposalId, Uint8List data) {
    try {
      var offset = 0;

      // institution: [u8; 48]
      final institutionBytes = data.sublist(offset, offset + 48);
      offset += 48;

      // beneficiary: AccountId32 (32 bytes)
      final beneficiaryBytes = data.sublist(offset, offset + 32);
      offset += 32;

      // amount: u128 little-endian (16 bytes)
      final amountBytes = data.sublist(offset, offset + 16);
      var amountBig = BigInt.zero;
      for (var i = 15; i >= 0; i--) {
        amountBig = (amountBig << 8) | BigInt.from(amountBytes[i]);
      }
      offset += 16;

      // remark: Vec<u8> (Compact length + bytes)
      final (remarkLen, remarkLenSize) = _decodeCompact(data, offset);
      offset += remarkLenSize;
      final remarkBytes = data.sublist(offset, offset + remarkLen);
      final remark = utf8.decode(remarkBytes, allowMalformed: true);
      offset += remarkLen;

      // proposer: AccountId32 (32 bytes)
      final proposerBytes = data.sublist(offset, offset + 32);

      final beneficiarySs58 =
          Keyring().encodeAddress(Uint8List.fromList(beneficiaryBytes), 2027);
      final proposerSs58 =
          Keyring().encodeAddress(Uint8List.fromList(proposerBytes), 2027);

      return TransferProposalInfo(
        proposalId: proposalId,
        institutionBytes: Uint8List.fromList(institutionBytes),
        beneficiary: beneficiarySs58,
        amountFen: amountBig,
        remark: remark,
        proposer: proposerSs58,
      );
    } catch (_) {
      return null;
    }
  }

  /// 解码 SCALE Compact<u32>，返回 (value, bytesConsumed)。
  (int, int) _decodeCompact(Uint8List data, int offset) {
    final first = data[offset];
    final mode = first & 0x03;
    if (mode == 0) {
      return (first >> 2, 1);
    } else if (mode == 1) {
      final val = (data[offset] | (data[offset + 1] << 8)) >> 2;
      return (val, 2);
    } else if (mode == 2) {
      final val = (data[offset] |
              (data[offset + 1] << 8) |
              (data[offset + 2] << 16) |
              (data[offset + 3] << 24)) >>
          2;
      return (val, 4);
    } else {
      // big integer mode — 简单处理，假设不超过 256
      final lenBytes = (first >> 2) + 4;
      var val = 0;
      for (var i = lenBytes - 1; i >= 0; i--) {
        val = (val << 8) | data[offset + 1 + i];
      }
      return (val, 1 + lenBytes);
    }
  }

  static bool _bytesEqual(Uint8List a, Uint8List b) {
    if (a.length != b.length) return false;
    for (var i = 0; i < a.length; i++) {
      if (a[i] != b[i]) return false;
    }
    return true;
  }

  // ──── 内部：extrinsic 编码 ────

  /// 构造 propose_transfer call data。
  ///
  /// 格式：[0x13][0x00][org:u8][institution:48bytes][0x00+beneficiary:32bytes][Compact amount][Vec remark]
  Uint8List _buildProposeTransferCall({
    required int org,
    required String institutionIdentity,
    required String beneficiaryAddress,
    required BigInt amountFen,
    required String remark,
  }) {
    final output = ByteOutput();
    output.pushByte(_palletIndex);
    output.pushByte(_proposeCallIndex);

    // org: u8
    output.pushByte(org);

    // institution: [u8; 48]
    output.write(_institutionIdentityToFixed48(institutionIdentity));

    // beneficiary: AccountId32 = 32 bytes（不是 MultiAddress，无 0x00 前缀）
    final beneficiaryId = Keyring().decodeAddress(beneficiaryAddress);
    output.write(beneficiaryId);

    // amount: u128 little-endian（16 字节，非 Compact）
    output.write(_u128ToLeBytes(amountFen));

    // remark: Vec<u8> = Compact<u32> length + bytes
    final remarkBytes = utf8.encode(remark);
    output.write(
        CompactBigIntCodec.codec.encode(BigInt.from(remarkBytes.length)));
    if (remarkBytes.isNotEmpty) {
      output.write(Uint8List.fromList(remarkBytes));
    }

    return output.toBytes();
  }

  /// 构造 propose_sweep_to_main call data。
  ///
  /// 格式：[0x13][0x02][institution:48][amount:u128_le]
  Uint8List _buildProposeSweepCall({
    required String institutionIdentity,
    required double amountYuan,
  }) {
    final output = ByteOutput();
    output.pushByte(_palletIndex);
    output.pushByte(_proposeSweepCallIndex);
    output.write(_institutionIdentityToFixed48(institutionIdentity));
    final amountFen = BigInt.from((amountYuan * 100).round());
    final amountBytes = Uint8List(16);
    var rem = amountFen;
    for (var i = 0; i < 16; i++) {
      amountBytes[i] = (rem & BigInt.from(0xFF)).toInt();
      rem = rem >> 8;
    }
    output.write(amountBytes);
    return output.toBytes();
  }

  /// 构造 propose_safety_fund_transfer call data。
  ///
  /// 格式：[0x13][0x01][beneficiary:32][amount:u128_le][remark:Vec<u8>]
  Uint8List _buildProposeSafetyFundCall({
    required String beneficiaryAddress,
    required double amountYuan,
    required String remark,
  }) {
    final output = ByteOutput();
    output.pushByte(_palletIndex);
    output.pushByte(_proposeSafetyFundCallIndex);

    // beneficiary: 32 bytes
    final beneficiaryId = Keyring().decodeAddress(beneficiaryAddress);
    output.write(beneficiaryId);

    // amount: u128 LE
    final amountFen = BigInt.from((amountYuan * 100).round());
    final amountBytes = Uint8List(16);
    var rem = amountFen;
    for (var i = 0; i < 16; i++) {
      amountBytes[i] = (rem & BigInt.from(0xFF)).toInt();
      rem = rem >> 8;
    }
    output.write(amountBytes);

    // remark: Vec<u8>
    final remarkBytes = utf8.encode(remark);
    output.write(
        CompactBigIntCodec.codec.encode(BigInt.from(remarkBytes.length)));
    if (remarkBytes.isNotEmpty) {
      output.write(Uint8List.fromList(remarkBytes));
    }

    return output.toBytes();
  }

  /// 从链上 SafetyFundProposalActions 存储查询安全基金转账提案详情。
  Future<SafetyFundProposalInfo?> fetchSafetyFundAction(int proposalId) async {
    final key = _buildStorageKey(
      'DuoqianTransfer',
      'SafetyFundProposalActions',
      _u64ToLeBytes(proposalId),
    );
    final raw = await _rpc.fetchStorage('0x${_hexEncode(key)}');
    // SafetyFundAction: beneficiary(32) + amount(u128=16) + remark(BoundedVec) + proposer(32)
    if (raw == null || raw.length < 32 + 16 + 1 + 32) return null;
    try {
      var offset = 0;
      final beneficiaryBytes = raw.sublist(offset, offset + 32);
      offset += 32;
      var amountBig = BigInt.zero;
      for (var i = 15; i >= 0; i--) {
        amountBig = (amountBig << 8) | BigInt.from(raw[offset + i]);
      }
      offset += 16;
      final (remarkLen, remarkLenSize) = _decodeCompact(raw, offset);
      offset += remarkLenSize;
      final remark = utf8.decode(
        raw.sublist(offset, offset + remarkLen),
        allowMalformed: true,
      );
      offset += remarkLen;
      final proposerBytes = raw.sublist(offset, offset + 32);
      return SafetyFundProposalInfo(
        proposalId: proposalId,
        beneficiary:
            Keyring().encodeAddress(Uint8List.fromList(beneficiaryBytes), 2027),
        amountFen: amountBig,
        remark: remark,
        proposer:
            Keyring().encodeAddress(Uint8List.fromList(proposerBytes), 2027),
      );
    } catch (_) {
      return null;
    }
  }

  /// 从链上 SweepProposalActions 存储查询手续费划转提案详情。
  Future<SweepProposalInfo?> fetchSweepAction(int proposalId) async {
    final key = _buildStorageKey(
      'DuoqianTransfer',
      'SweepProposalActions',
      _u64ToLeBytes(proposalId),
    );
    final raw = await _rpc.fetchStorage('0x${_hexEncode(key)}');
    // SweepAction: institution([u8;48]) + amount(u128=16)
    if (raw == null || raw.length < 48 + 16) return null;
    try {
      final institutionBytes = Uint8List.fromList(raw.sublist(0, 48));
      var amountBig = BigInt.zero;
      for (var i = 15; i >= 0; i--) {
        amountBig = (amountBig << 8) | BigInt.from(raw[48 + i]);
      }
      return SweepProposalInfo(
        proposalId: proposalId,
        institutionBytes: institutionBytes,
        amountFen: amountBig,
      );
    } catch (_) {
      return null;
    }
  }

  /// 签名并提交 extrinsic（复用 onchain.dart 的流程）。
  ///
  /// 返回交易哈希和使用的 nonce（用于链上确认跟踪）。
  Future<({String txHash, int usedNonce})> _signAndSubmit({
    required Uint8List callData,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    debugPrint('[TransferProposal] 步骤1: 获取 metadata...');
    final metadata = await _rpc.fetchMetadata();
    debugPrint('[TransferProposal] 步骤2: 获取 genesisHash...');
    final genesisHash = await _rpc.fetchGenesisHash();
    final registry = metadata.chainInfo.scaleCodec.registry;

    debugPrint(
        '[TransferProposal] 步骤3: 并行获取 runtimeVersion/nonce/latestBlock...');
    final results = await Future.wait([
      _rpc.fetchRuntimeVersion(),
      NonceManager.instance.getNextNonce(
        address: fromAddress,
        fetchChainNonce: _rpc.fetchNonce,
      ),
      _rpc.fetchLatestBlock(),
    ]);
    final runtimeVersion = results[0] as dynamic;
    final nonce = results[1] as int;
    final latestBlock = results[2] as ({Uint8List blockHash, int blockNumber});
    debugPrint(
        '[TransferProposal] nonce=$nonce, block=${latestBlock.blockNumber}');

    debugPrint('[TransferProposal] 步骤4: 构造签名载荷...');
    final signingPayload = SigningPayload(
      method: callData,
      specVersion: runtimeVersion.specVersion,
      transactionVersion: runtimeVersion.transactionVersion,
      genesisHash: '0x${_hexEncode(genesisHash)}',
      blockHash: '0x${_hexEncode(latestBlock.blockHash)}',
      blockNumber: latestBlock.blockNumber,
      eraPeriod: _eraPeriod,
      nonce: nonce,
      tip: 0,
    );
    final payloadBytes = signingPayload.encode(registry);

    debugPrint('[TransferProposal] 步骤5: 签名 (${payloadBytes.length} bytes)...');
    final signature = await sign(payloadBytes);
    debugPrint('[TransferProposal] 签名完成 (${signature.length} bytes)');

    debugPrint('[TransferProposal] 步骤6: 编码 extrinsic...');
    final extrinsicPayload = ExtrinsicPayload(
      signer: signerPubkey,
      method: callData,
      signature: signature,
      eraPeriod: _eraPeriod,
      blockNumber: latestBlock.blockNumber,
      nonce: nonce,
      tip: 0,
    );
    final encoded = extrinsicPayload.encode(registry, SignatureType.sr25519);
    debugPrint('[TransferProposal] extrinsic 编码完成 (${encoded.length} bytes)');

    // ──── 诊断：逐字节打印 extrinsic 结构 ────
    debugPrint('[TransferProposal] ════════ EXTRINSIC 诊断 ════════');
    debugPrint(
        '[TransferProposal] signing payload hex (${payloadBytes.length} bytes): ${_hexEncode(payloadBytes)}');
    debugPrint(
        '[TransferProposal] signature hex (${signature.length} bytes): ${_hexEncode(signature)}');
    debugPrint(
        '[TransferProposal] signer pubkey hex: ${_hexEncode(signerPubkey)}');
    debugPrint(
        '[TransferProposal] call data hex (${callData.length} bytes): ${_hexEncode(callData)}');
    debugPrint(
        '[TransferProposal] nonce=$nonce, eraPeriod=$_eraPeriod, blockNumber=${latestBlock.blockNumber}');
    debugPrint(
        '[TransferProposal] specVersion=${runtimeVersion.specVersion}, txVersion=${runtimeVersion.transactionVersion}');
    debugPrint('[TransferProposal] genesisHash=0x${_hexEncode(genesisHash)}');
    debugPrint(
        '[TransferProposal] blockHash=0x${_hexEncode(latestBlock.blockHash)}');
    debugPrint(
        '[TransferProposal] registry.extrinsicVersion=${registry.extrinsicVersion}');
    // 打印 registry 中的 signedExtension 键列表（按序）
    try {
      final extKeys =
          (registry.getSignedExtensionTypes() as Map<String, dynamic>)
              .keys
              .toList();
      debugPrint(
          '[TransferProposal] signedExtension keys (${extKeys.length}): $extKeys');
      final addExtKeys =
          (registry.getAdditionalSignedExtensionTypes() as Map<String, dynamic>)
              .keys
              .toList();
      debugPrint(
          '[TransferProposal] additionalSignedExtension keys (${addExtKeys.length}): $addExtKeys');
    } catch (e) {
      debugPrint('[TransferProposal] 获取 extension keys 失败: $e');
    }
    debugPrint(
        '[TransferProposal] encoded extrinsic hex (${encoded.length} bytes): ${_hexEncode(encoded)}');
    // 手工拆解 encoded extrinsic：compact_length + [0x84][0x00+signer(32)][0x01+sig(64)][extensions][calldata]
    {
      int pos = 0;
      // 解析 compact length prefix
      final firstByte = encoded[0];
      int compactLen;
      if (firstByte & 0x03 == 0x00) {
        compactLen = firstByte >> 2;
        pos = 1;
      } else if (firstByte & 0x03 == 0x01) {
        compactLen = ((encoded[1] << 8 | firstByte) >> 2);
        pos = 2;
      } else if (firstByte & 0x03 == 0x02) {
        compactLen = ((encoded[3] << 24 |
                encoded[2] << 16 |
                encoded[1] << 8 |
                firstByte) >>
            2);
        pos = 4;
      } else {
        compactLen = -1;
        pos = 0;
      }
      debugPrint(
          '[TransferProposal] compact length prefix: $compactLen, body starts at byte $pos');
      if (pos < encoded.length) {
        debugPrint(
            '[TransferProposal] version byte: 0x${encoded[pos].toRadixString(16).padLeft(2, '0')}');
        final bodyHex = _hexEncode(encoded.sublist(pos));
        debugPrint(
            '[TransferProposal] extrinsic body hex ($compactLen bytes): $bodyHex');
      }
    }
    debugPrint('[TransferProposal] ════════ 诊断结束 ════════');

    debugPrint('[TransferProposal] 步骤7: 提交到链...');
    try {
      final txHash = await _rpc.submitExtrinsic(encoded);
      debugPrint('[TransferProposal] 提交成功: 0x${_hexEncode(txHash)}');
      return (txHash: '0x${_hexEncode(txHash)}', usedNonce: nonce);
    } catch (e) {
      NonceManager.instance.rollback(fromAddress);
      debugPrint('[TransferProposal] 提交失败，原始错误: $e');
      rethrow;
    }
  }

  // ──── 内部：storage key 构造 ────

  /// 构造 storage key：twox128(pallet) + twox128(storage) + blake2_128_concat(keyData)。
  Uint8List _buildStorageKey(
    String palletName,
    String storageName,
    Uint8List keyData,
  ) {
    final palletHash = _twoxx128String(palletName);
    final storageHash = _twoxx128String(storageName);
    final keyHash = _blake2128Concat(keyData);

    final result =
        Uint8List(palletHash.length + storageHash.length + keyHash.length);
    var offset = 0;
    result.setAll(offset, palletHash);
    offset += palletHash.length;
    result.setAll(offset, storageHash);
    offset += storageHash.length;
    result.setAll(offset, keyHash);
    return result;
  }

  // ──── 内部：编码工具 ────

  Uint8List _institutionIdentityToFixed48(String institutionIdentity) {
    return Uint8List.fromList(
        institutionIdentityToPalletId(institutionIdentity));
  }

  /// 将 BigInt 编码为 u128 little-endian（16 字节）。
  Uint8List _u128ToLeBytes(BigInt value) {
    final bytes = Uint8List(16);
    var v = value;
    for (var i = 0; i < 16; i++) {
      bytes[i] = (v & BigInt.from(0xFF)).toInt();
      v >>= 8;
    }
    return bytes;
  }

  Uint8List _u64ToLeBytes(int value) {
    final bytes = Uint8List(8);
    final bd = ByteData.sublistView(bytes);
    bd.setUint64(0, value, Endian.little);
    return bytes;
  }

  int _decodeU64(Uint8List data) {
    final bd = ByteData.sublistView(data);
    return bd.getUint64(0, Endian.little);
  }

  int _decodeU32(Uint8List data, int offset) {
    final bd = ByteData.sublistView(data);
    return bd.getUint32(offset, Endian.little);
  }

  static String _hexEncode(Uint8List bytes) {
    return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  }

  Uint8List _hexDecode(String hex) {
    final h = hex.startsWith('0x') ? hex.substring(2) : hex;
    final result = Uint8List(h.length ~/ 2);
    for (var i = 0; i < result.length; i++) {
      result[i] = int.parse(h.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return result;
  }

  // ──── 内部：哈希（直接使用 polkadart Hasher）────

  Uint8List _twoxx128String(String input) {
    return Hasher.twoxx128.hashString(input);
  }

  Uint8List _blake2128Concat(Uint8List data) {
    final hash = Hasher.blake2b128.hash(data);
    final result = Uint8List(hash.length + data.length);
    result.setAll(0, hash);
    result.setAll(hash.length, data);
    return result;
  }
}

/// 转账提案链上数据。
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

/// 提案链上元数据（从 Proposals Storage 解码）。
class ProposalMeta {
  const ProposalMeta({
    required this.proposalId,
    required this.kind,
    required this.stage,
    required this.status,
    this.internalOrg,
    this.institutionBytes,
  });

  final int proposalId;
  final int kind; // 0=internal, 1=joint
  final int stage; // 0=internal, 1=joint, 2=citizen
  final int status; // 0=voting, 1=passed, 2=rejected
  final int? internalOrg;
  final Uint8List? institutionBytes;
}

/// 提案 + 业务详情（用于全局提案列表展示）。
class ProposalWithDetail {
  const ProposalWithDetail({
    required this.meta,
    this.transferDetail,
    this.runtimeUpgradeDetail,
    this.createDuoqianDetail,
    this.closeDuoqianDetail,
    this.safetyFundDetail,
    this.sweepDetail,
    this.resolutionIssuanceSummary,
    this.resolutionDestroySummary,
  });

  final ProposalMeta meta;

  /// 转账提案详情（非转账提案为 null）。
  final TransferProposalInfo? transferDetail;

  /// Runtime 升级提案详情（非升级提案为 null）。
  final RuntimeUpgradeProposalInfo? runtimeUpgradeDetail;

  /// 创建多签账户提案详情。
  final CreateDuoqianProposalInfo? createDuoqianDetail;

  /// 关闭多签账户提案详情。
  final CloseDuoqianProposalInfo? closeDuoqianDetail;

  /// 安全基金转账提案详情。
  final SafetyFundProposalInfo? safetyFundDetail;

  /// 手续费划转提案详情。
  final SweepProposalInfo? sweepDetail;

  /// 决议发行提案摘要（仅列表展示用）。
  final String? resolutionIssuanceSummary;

  /// 决议销毁提案摘要（仅列表展示用）。
  final String? resolutionDestroySummary;
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
