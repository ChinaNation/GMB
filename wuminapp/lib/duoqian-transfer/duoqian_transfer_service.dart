import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:polkadart/scale_codec.dart' show CompactBigIntCodec, ByteOutput;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:wuminapp_mobile/organization-manage/shared/duoqian_manage_models.dart'
    as org_models;
import 'package:wuminapp_mobile/organization-manage/shared/duoqian_manage_service.dart';
import 'package:wuminapp_mobile/personal-manage/personal_manage_models.dart';
import 'package:wuminapp_mobile/personal-manage/personal_manage_service.dart';

import 'package:wuminapp_mobile/rpc/chain_rpc.dart';
import 'package:wuminapp_mobile/rpc/signed_extrinsic_builder.dart';
import 'package:wuminapp_mobile/rpc/smoldot_client.dart';
import 'package:wuminapp_mobile/institution/institution_data.dart';
import 'package:wuminapp_mobile/proposal/shared/proposal_cache.dart';
import 'package:wuminapp_mobile/proposal/runtime_upgrade/runtime_upgrade_service.dart';
import 'package:wuminapp_mobile/proposal/shared/proposal_models.dart';
import 'package:wuminapp_mobile/duoqian-transfer/duoqian_transfer_cache.dart';
import 'package:wuminapp_mobile/duoqian-transfer/duoqian_transfer_models.dart';

/// 机构转账提案链上交互服务。
///
/// 负责 extrinsic 编码/提交 和 storage 查询。
class DuoqianTransferService {
  DuoqianTransferService({ChainRpc? chainRpc}) : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;

  // ──── 常量 ────

  /// DuoqianTransfer pallet index（runtime pallet_index=19）。
  ///
  /// 本 pallet 只保留 propose_X(0/1/2)；管理员投票一律走
  /// InternalVote(22).cast(0)，手动重试走 VotingEngine(9).retry_passed_proposal(4)。
  static const _palletIndex = 19;

  /// propose_transfer call_index=0。
  static const _proposeCallIndex = 0;

  /// propose_safety_fund_transfer call_index=1。
  static const _proposeSafetyFundCallIndex = 1;

  /// propose_sweep_to_main call_index=2。
  static const _proposeSweepCallIndex = 2;

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
      institutionIdentity: institution.sfidNumber,
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
      institutionIdentity: institution.sfidNumber,
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

  /// 查询转出主账户的可用余额（元）。
  ///
  /// 中文注释：治理机构按主账户、费用账户、安全基金账户、质押账户分别建模，转账提案固定从主账户支出；
  /// 个人/注册多签账户通过 InstitutionInfo.mainAddress 继续映射到账户地址。
  Future<double> fetchInstitutionBalance(InstitutionInfo institution) {
    return _rpc.fetchBalance(institution.mainAddress);
  }

  // ──── 双层 ID 与反向索引(spec_version v1)────

  /// 查询提案展示号:`ProposalDisplayId[proposal_id] = ProposalDisplayMeta { year, seq_in_year }`。
  ///
  /// 返回 null 表示链上不存在该展示号(理论上不应该发生 — 创建提案时
  /// 同事务一定写入)。
  Future<ProposalDisplayMeta?> fetchProposalDisplayId(int proposalId) async {
    final key = _buildStorageKey(
      'VotingEngine',
      'ProposalDisplayId',
      _u64ToLeBytes(proposalId),
    );
    final data = await _rpc.fetchStorage('0x${_hexEncode(key)}');
    if (data == null || data.length < 6) return null;
    // SCALE: u16 LE (year) + u32 LE (seq_in_year) = 6 bytes
    final bd = ByteData.sublistView(data);
    return ProposalDisplayMeta(
      year: bd.getUint16(0, Endian.little),
      seqInYear: bd.getUint32(2, Endian.little),
    );
  }

  /// 批量查询展示号(列表页一次性 batch fetch,避免 N 次 RPC)。
  Future<Map<int, ProposalDisplayMeta>> fetchProposalDisplayIdBatch(
      List<int> proposalIds) async {
    if (proposalIds.isEmpty) return const {};
    final keyHexList = proposalIds
        .map((id) => '0x${_hexEncode(_buildStorageKey(
              'VotingEngine',
              'ProposalDisplayId',
              _u64ToLeBytes(id),
            ))}')
        .toList();
    final batchResult = await _rpc.fetchStorageBatch(keyHexList);
    final result = <int, ProposalDisplayMeta>{};
    for (var i = 0; i < proposalIds.length; i++) {
      final data = batchResult[keyHexList[i]];
      if (data == null || data.length < 6) continue;
      final bd = ByteData.sublistView(data);
      result[proposalIds[i]] = ProposalDisplayMeta(
        year: bd.getUint16(0, Endian.little),
        seqInYear: bd.getUint32(2, Endian.little),
      );
    }
    return result;
  }

  /// 反向索引:`ProposalsByOrg[org]` 下的所有 proposal_id。
  ///
  /// org 取值:0=NRC, 1=PRC, 2=PRB, 3=DUOQIAN(详见
  /// `votingengine::internal_vote::ORG_*` 常量)。
  Future<List<int>> fetchProposalIdsByOrg(int org) async {
    return _fetchProposalIdsByDoubleMap(
      'ProposalsByOrg',
      Uint8List.fromList([org]),
    );
  }

  /// 反向索引:`ProposalsByInstitution[subject_id(48 bytes)]` 下的所有 proposal_id。
  Future<List<int>> fetchProposalIdsByInstitution(String sfidNumber) async {
    return _fetchProposalIdsByDoubleMap(
      'ProposalsByInstitution',
      _institutionIdentityToFixed48(sfidNumber),
    );
  }

  /// 反向索引:`ProposalsByOwner[module_tag]` 下的所有 proposal_id。
  /// `module_tag` 是业务模块的 BoundedVec<u8, MaxModuleTagLen>,SCALE 编码后传入。
  Future<List<int>> fetchProposalIdsByOwner(Uint8List moduleTag) async {
    // BoundedVec<u8> 的 SCALE 编码 = Compact<len> + bytes;作为 storage 的 K1 键时,
    // 链上对该编码后的字节做 twox64,然后 concat 原编码字节。
    final encoded = ByteOutput();
    encoded
        .write(CompactBigIntCodec.codec.encode(BigInt.from(moduleTag.length)));
    encoded.write(moduleTag);
    return _fetchProposalIdsByDoubleMap('ProposalsByOwner', encoded.toBytes());
  }

  /// 通用反向索引迭代器。
  ///
  /// `StorageDoubleMap<_, Twox64Concat, K1, Twox64Concat, u64, ()>` 的 key 结构:
  ///   `twox128(pallet) ++ twox128(storage) ++ twox64(K1)(8B) ++ K1(原值) ++ twox64(u64)(8B) ++ u64(8B LE)`
  ///
  /// 取所有 key 的前缀:
  ///   `twox128(pallet) ++ twox128(storage) ++ twox64(K1) ++ K1`
  ///
  /// `state_getKeysPaged(prefix, count, startKey)` 返回前缀下所有完整 key,
  /// 每个 key 末 8 字节 = u64 LE = proposal_id。
  Future<List<int>> _fetchProposalIdsByDoubleMap(
      String storageName, Uint8List firstKeyRaw) async {
    final palletHash = _twoxx128String('VotingEngine');
    final storageHash = _twoxx128String(storageName);
    final firstKeyHashed = Hasher.twoxx64.hash(firstKeyRaw);
    final prefixLen = palletHash.length +
        storageHash.length +
        firstKeyHashed.length +
        firstKeyRaw.length;
    final prefix = Uint8List(prefixLen);
    var off = 0;
    prefix.setAll(off, palletHash);
    off += palletHash.length;
    prefix.setAll(off, storageHash);
    off += storageHash.length;
    prefix.setAll(off, firstKeyHashed);
    off += firstKeyHashed.length;
    prefix.setAll(off, firstKeyRaw);
    final prefixHex = '0x${_hexEncode(prefix)}';

    // 一次拉到底(开发期数据量小);生产 1000 万级时再分页
    final keysHex = await SmoldotClientManager.instance.request(
      'state_getKeysPaged',
      [prefixHex, 1000, null],
    ) as List<dynamic>?;
    if (keysHex == null || keysHex.isEmpty) return const [];

    // 每条 key 末 8 字节 = proposal_id u64 LE(twox64_concat 的 raw 部分)
    final ids = <int>[];
    for (final keyHex in keysHex) {
      if (keyHex is! String) continue;
      final keyBytes = _hexDecode(keyHex);
      if (keyBytes.length < 8) continue;
      final tail = keyBytes.sublist(keyBytes.length - 8);
      ids.add(_decodeU64(tail));
    }
    return ids;
  }

  /// 每个机构最多同时 10 个活跃提案（全局，不区分提案类型）。
  static const maxActiveProposalsPerInstitution = 10;

  /// 查询机构活跃的提案 ID 列表（从 VotingEngine 全局存储读取）。
  Future<List<int>> fetchActiveProposalIds(String sfidNumber) async {
    final key = _buildStorageKey(
      'VotingEngine',
      'ActiveProposalsByInstitution',
      _institutionIdentityToFixed48(sfidNumber),
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
      'InternalVote',
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

  /// 查询内部投票阈值快照。
  ///
  /// 中文注释：个人多签创建提案的创建阈值是提案级快照，
  /// 不能使用 InstitutionInfo.internalThreshold 的多签默认值。
  Future<int?> fetchInternalThresholdSnapshot(int proposalId) async {
    final key = _buildStorageKey(
      'InternalVote',
      'InternalThresholdSnapshot',
      _u64ToLeBytes(proposalId),
    );
    final data = await _rpc.fetchStorage('0x${_hexEncode(key)}');
    if (data == null || data.length < 4) return null;
    return _decodeU32(data, 0);
  }

  /// 查询提案创建时锁定的管理员快照。
  ///
  /// 中文注释：投票资格由 VotingEngine::AdminSnapshot 判定，详情页展示和
  /// 可投钱包筛选也应优先使用同一份快照，避免当前管理员列表变化影响旧提案。
  Future<List<String>> fetchAdminSnapshot(
    int proposalId,
    String institutionIdentity,
  ) async {
    final key = _buildDoubleStorageKey(
      'VotingEngine',
      'AdminSnapshot',
      _u64ToLeBytes(proposalId),
      _institutionIdentityToFixed48(institutionIdentity),
    );
    final data = await _rpc.fetchStorage('0x${_hexEncode(key)}');
    if (data == null || data.isEmpty) return const [];

    final (count, lenSize) = _decodeCompact(data, 0);
    final admins = <String>[];
    var offset = lenSize;
    for (var i = 0; i < count && offset + 32 <= data.length; i++) {
      admins.add(
          _hexEncode(Uint8List.fromList(data.sublist(offset, offset + 32))));
      offset += 32;
    }
    return admins;
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

    // 双层 ID v1:同时查 ProposalDisplayId 反查表(失败不阻塞,fallback null)
    ProposalDisplayMeta? displayMeta;
    try {
      displayMeta = await fetchProposalDisplayId(proposalId);
    } catch (_) {}

    return ProposalMeta(
      proposalId: proposalId,
      kind: kind,
      stage: stage,
      status: status,
      internalOrg: internalOrg,
      institutionBytes: institutionBytes,
      displayMeta: displayMeta,
    );
  }

  // ──── 分页 + 缓存 + 批量查询 ────

  /// 给定一组 proposal_id,batch 查 meta + 业务详情 + 展示号,
  /// 返回 `ProposalWithDetail` 列表(顺序按 ids 入参,**不再重排**)。
  ///
  /// 双层 ID v1 模式下,客户端先通过反向索引拿 ID 列表,再调本方法取详情。
  /// 多签管理提案(ACTION_CREATE_PERSONAL / ACTION_CLOSE)按既有规则过滤掉。
  Future<List<ProposalWithDetail>> fetchProposalsByIds(List<int> ids) async {
    if (ids.isEmpty) return const [];
    return _fetchProposalsForIds(ids);
  }

  /// 分页查询提案:从 [startId] 往前(含 startId)加载 [count] 个。
  ///
  /// 优先读缓存,未命中的用 [fetchStorageBatch] 批量查询。
  /// 返回结果按 ID 倒序。
  Future<List<ProposalWithDetail>> fetchProposalPage(
      int startId, int count) async {
    // 中文注释:把范围转成显式 ids 列表,然后委派给 _fetchProposalsForIds。
    final ids = <int>[
      for (var id = startId; id > startId - count && id >= 0; id--) id,
    ];
    return _fetchProposalsForIds(ids);
  }

  /// 给定 [ids] 一组提案 ID,batch 拉 meta + 业务详情 + 展示号,
  /// 返回 `ProposalWithDetail` 列表,**顺序与入参一致**。
  /// 多签管理类提案(ACTION_CREATE_PERSONAL / ACTION_CLOSE)在装配阶段过滤掉。
  Future<List<ProposalWithDetail>> _fetchProposalsForIds(List<int> ids) async {
    final results = <ProposalWithDetail>[];
    final uncachedMetaKeys = <String>[];
    final uncachedMetaIds = <int>[];
    final cachedMetas = <int, ProposalMeta>{};
    final runtimeUpgradeService = RuntimeUpgradeService(chainRpc: _rpc);

    for (final id in ids) {
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
      // 双层 ID v1:为这一批 meta 同步 batch 拉 ProposalDisplayId 并 patch 进 meta
      final displayMap = await fetchProposalDisplayIdBatch(uncachedMetaIds);
      for (final id in uncachedMetaIds) {
        final meta = cachedMetas[id];
        final dm = displayMap[id];
        if (meta != null && dm != null) {
          final patched = ProposalMeta(
            proposalId: meta.proposalId,
            kind: meta.kind,
            stage: meta.stage,
            status: meta.status,
            internalOrg: meta.internalOrg,
            institutionBytes: meta.institutionBytes,
            displayMeta: dm,
          );
          cachedMetas[id] = patched;
          ProposalCache.putMeta(id, patched);
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
        final cachedTransfer =
            DuoqianTransferCache.getTransferDetail(entry.key);
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
      final personalManageService = PersonalManageService(chainRpc: _rpc);
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

        // 内部投票提案：先按 PersonalManage 解码，再按 OrganizationManage 解码，
        // 失败后才尝试普通多签转账提案。
        final personalDetail =
            personalManageService.decodePersonalProposalData(id, raw);
        if (personalDetail is CreateDuoqianProposalInfo) {
          cachedCreateDuoqianDetails[id] = personalDetail;
          ProposalCache.putCreateDuoqianDetail(id, personalDetail);
          continue;
        }
        if (personalDetail is CloseDuoqianProposalInfo) {
          cachedCloseDuoqianDetails[id] = personalDetail;
          ProposalCache.putCloseDuoqianDetail(id, personalDetail);
          continue;
        }
        final orgManageDetail = manageService.decodeManageProposalData(id, raw);
        if (orgManageDetail is org_models.CloseDuoqianProposalInfo) {
          final detail = CloseDuoqianProposalInfo(
            proposalId: orgManageDetail.proposalId,
            duoqianAddress: orgManageDetail.duoqianAddress,
            beneficiary: orgManageDetail.beneficiary,
            proposer: orgManageDetail.proposer,
            status: orgManageDetail.status,
          );
          cachedCloseDuoqianDetails[id] = detail;
          ProposalCache.putCloseDuoqianDetail(id, detail);
          continue;
        }

        final transferDetail = _decodeProposalData(id, raw);
        if (transferDetail != null) {
          cachedTransferDetails[id] = transferDetail;
          DuoqianTransferCache.putTransferDetail(id, transferDetail);
        }
      }
    }

    // 组装结果(跳过多签管理提案,这些在多签账户详情页单独展示)
    for (final id in ids) {
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
        runtimeUpgradeDetail: runtimeUpgradeDetail,
        createDuoqianDetail: createDuoqianDetail?.copyWithStatus(meta.status),
        closeDuoqianDetail: closeDuoqianDetail?.copyWithStatus(meta.status),
        businessDetails: {
          if (transferDetail != null)
            DuoqianTransferProposalDetailKeys.transfer:
                transferDetail.copyWithStatus(meta.status),
          if (safetyFundDetail != null)
            DuoqianTransferProposalDetailKeys.safetyFund: safetyFundDetail,
          if (sweepDetail != null)
            DuoqianTransferProposalDetailKeys.sweep: sweepDetail,
        },
        resolutionIssuanceSummary: resIssuanceSummary,
        resolutionDestroySummary: resDestroySummary,
      ));
    }

    return results;
  }

  /// 查询对指定机构用户可见的提案事件。
  ///
  /// **v1 双层 ID 模式**:
  /// - 走 `ProposalsByInstitution[sfidNumber]` 反向索引拿本机构所有提案 ID
  ///   (含内部投票如转账/费率设置/管理员变更等)
  /// - 联合投票提案(runtime 升级 / 决议)的 internal_institution = None,
  ///   不会落 `ProposalsByInstitution`,所以这里**额外**取本年所有 kind=1
  ///   提案,并入结果(机构页要让所有用户都能看到联合投票)
  Future<List<ProposalWithDetail>> fetchInstitutionVisibleProposals(
      String sfidNumber) async {
    // 1) 本机构所有提案(含内部投票)
    final institutionIds = await fetchProposalIdsByInstitution(sfidNumber);
    final institutionProposals = await _fetchProposalsForIds(institutionIds);

    // 2) 联合投票提案(kind=1)在所有机构页可见 — 取本年所有 ProposalsByYear[当前年]
    //    再筛 kind=1。开发期数据量小;PR-Z 之后产品如果要"机构页只看本机构"
    //    可以删掉这段。
    final currentYear = await _resolveCurrentYear();
    final yearIds = currentYear == null
        ? const <int>[]
        : await _fetchProposalIdsByYearTwox(currentYear);
    final extraJointIds =
        yearIds.where((id) => !institutionIds.contains(id)).toList();
    final extraJointProposals = extraJointIds.isEmpty
        ? const <ProposalWithDetail>[]
        : await _fetchProposalsForIds(extraJointIds);
    final jointOnly =
        extraJointProposals.where((p) => p.meta.kind == 1).toList();

    final all = <ProposalWithDetail>[...institutionProposals, ...jointOnly];
    all.sort((a, b) => b.meta.proposalId.compareTo(a.meta.proposalId));
    return all;
  }

  /// 从 `CurrentProposalYear` 拿当前提案分配年份(用于反向索引按年迭代)。
  Future<int?> _resolveCurrentYear() async {
    final palletHash = _twoxx128String('VotingEngine');
    final storageHash = _twoxx128String('CurrentProposalYear');
    final key = Uint8List(palletHash.length + storageHash.length);
    key.setAll(0, palletHash);
    key.setAll(palletHash.length, storageHash);
    final data = await _rpc.fetchStorage('0x${_hexEncode(key)}');
    if (data == null || data.length < 2) return null;
    return ByteData.sublistView(data).getUint16(0, Endian.little);
  }

  /// `ProposalsByYear[year]` 反向索引迭代(year 是 u16,内部不暴露给业务层)。
  Future<List<int>> _fetchProposalIdsByYearTwox(int year) async {
    final yearBytes = Uint8List(2);
    ByteData.sublistView(yearBytes).setUint16(0, year, Endian.little);
    return _fetchProposalIdsByDoubleMap('ProposalsByYear', yearBytes);
  }

  /// 查询指定机构的所有转账提案（包括已完成的），按 ID 倒序。
  Future<List<TransferProposalInfo>> fetchAllInstitutionProposals(
      String sfidNumber) async {
    final visibleProposals = await fetchInstitutionVisibleProposals(sfidNumber);
    final institutionBytes = _institutionIdentityToFixed48(sfidNumber);
    final proposals = <TransferProposalInfo>[];

    for (final proposal in visibleProposals) {
      final detailObject =
          proposal.businessDetails[DuoqianTransferProposalDetailKeys.transfer];
      if (detailObject is! TransferProposalInfo) {
        continue;
      }
      if (_bytesEqual(detailObject.institutionBytes, institutionBytes)) {
        proposals.add(detailObject.copyWithStatus(proposal.meta.status));
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
    final palletHash = _twoxx128String('InternalVote');
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
  /// 格式：[0x13][0x00][org:u8][institution:48bytes][beneficiary:32bytes][amount:u128][Vec remark]
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
    return SignedExtrinsicBuilder(
      chainRpc: _rpc,
      logLabel: 'TransferProposal',
    ).signAndSubmit(
      callData: callData,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
      onTrace: _logSignedExtrinsicTrace,
    );
  }

  void _logSignedExtrinsicTrace(SignedExtrinsicTrace trace) {
    debugPrint('[TransferProposal] ════════ EXTRINSIC 诊断 ════════');
    debugPrint(
        '[TransferProposal] signing payload hex (${trace.payloadBytes.length} bytes): ${_hexEncode(trace.payloadBytes)}');
    debugPrint(
        '[TransferProposal] signature hex (${trace.signature.length} bytes): ${_hexEncode(trace.signature)}');
    debugPrint(
        '[TransferProposal] signer pubkey hex: ${_hexEncode(trace.signerPubkey)}');
    debugPrint(
        '[TransferProposal] call data hex (${trace.callData.length} bytes): ${_hexEncode(trace.callData)}');
    debugPrint(
        '[TransferProposal] nonce=${trace.nonce}, eraPeriod=${trace.eraPeriod}, blockNumber=${trace.blockNumber}');
    debugPrint(
        '[TransferProposal] specVersion=${trace.runtimeVersion.specVersion}, txVersion=${trace.runtimeVersion.transactionVersion}');
    debugPrint(
        '[TransferProposal] genesisHash=0x${_hexEncode(trace.genesisHash)}');
    debugPrint(
        '[TransferProposal] CheckEra blockHash=0x${_hexEncode(trace.genesisHash)}');
    debugPrint(
        '[TransferProposal] registry.extrinsicVersion=${trace.registry.extrinsicVersion}');
    try {
      final extKeys =
          (trace.registry.getSignedExtensionTypes() as Map<String, dynamic>)
              .keys
              .toList();
      debugPrint(
          '[TransferProposal] signedExtension keys (${extKeys.length}): $extKeys');
      final addExtKeys = (trace.registry.getAdditionalSignedExtensionTypes()
              as Map<String, dynamic>)
          .keys
          .toList();
      debugPrint(
          '[TransferProposal] additionalSignedExtension keys (${addExtKeys.length}): $addExtKeys');
    } catch (e) {
      debugPrint('[TransferProposal] 获取 extension keys 失败: $e');
    }
    debugPrint(
        '[TransferProposal] encoded extrinsic hex (${trace.encoded.length} bytes): ${_hexEncode(trace.encoded)}');
    _logExtrinsicBody(trace.encoded);
    debugPrint('[TransferProposal] ════════ 诊断结束 ════════');
  }

  void _logExtrinsicBody(Uint8List encoded) {
    var pos = 0;
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

  /// 构造 StorageDoubleMap key。
  Uint8List _buildDoubleStorageKey(
    String palletName,
    String storageName,
    Uint8List key1Data,
    Uint8List key2Data,
  ) {
    final palletHash = _twoxx128String(palletName);
    final storageHash = _twoxx128String(storageName);
    final key1Hash = _blake2128Concat(key1Data);
    final key2Hash = _blake2128Concat(key2Data);

    final result = Uint8List(
      palletHash.length +
          storageHash.length +
          key1Hash.length +
          key2Hash.length,
    );
    var offset = 0;
    result.setAll(offset, palletHash);
    offset += palletHash.length;
    result.setAll(offset, storageHash);
    offset += storageHash.length;
    result.setAll(offset, key1Hash);
    offset += key1Hash.length;
    result.setAll(offset, key2Hash);
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
