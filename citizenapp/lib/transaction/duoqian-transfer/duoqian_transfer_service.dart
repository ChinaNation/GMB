import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:polkadart/scale_codec.dart' show CompactBigIntCodec, ByteOutput;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:citizenapp/governance/organization-manage/institution_manage_models.dart'
    as org_models;
import 'package:citizenapp/governance/organization-manage/institution_manage_service.dart';
import 'package:citizenapp/governance/personal-manage/personal_manage_models.dart';
import 'package:citizenapp/governance/personal-manage/personal_manage_service.dart';

import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/rpc/signed_extrinsic_builder.dart';
import 'package:citizenapp/rpc/smoldot_client.dart';
import 'package:citizenapp/governance/shared/institution_info.dart';
import 'package:citizenapp/governance/shared/proposal/proposal_cache.dart';
import 'package:citizenapp/governance/runtime-upgrade/runtime_upgrade_service.dart';
import 'package:citizenapp/governance/admins-change/models/admin_account.dart';
import 'package:citizenapp/governance/shared/proposal/proposal_models.dart';
import 'package:citizenapp/transaction/duoqian-transfer/duoqian_transfer_cache.dart';
import 'package:citizenapp/transaction/duoqian-transfer/duoqian_transfer_models.dart';
import 'package:citizenapp/votingengine/internal-vote/internal_vote_query_service.dart';

/// 机构转账提案链上交互服务。
///
/// 负责 extrinsic 编码/提交 和 storage 查询。
class DuoqianTransferService {
  DuoqianTransferService({ChainRpc? chainRpc})
      : _rpc = chainRpc ?? ChainRpc(),
        _internalVoteQuery = InternalVoteQueryService(chainRpc: chainRpc);

  final ChainRpc _rpc;
  final InternalVoteQueryService _internalVoteQuery;

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

  /// DuoqianTransfer::TransferProposed event_index=0。
  static const _transferProposedEventIndex = 0;

  /// DuoqianTransfer::SafetyFundTransferProposed event_index=3。
  /// 中文注释：Event enum 按声明顺序编号（无显式 codec index），顺序为
  /// TransferProposed(0)/ExecutionFailed(1)/Executed(2)/SafetyFundTransferProposed(3)/
  /// …/SweepToMainProposed(6)，与 runtime lib.rs 严格一致。
  static const _safetyFundProposedEventIndex = 3;

  /// DuoqianTransfer::SweepToMainProposed event_index=6。
  static const _sweepProposedEventIndex = 6;

  // ──── Extrinsic 提交 ────

  /// 提交 propose_transfer extrinsic。
  ///
  /// 返回真实创建出的 proposal_id、交易哈希、nonce 和入块哈希。
  Future<
      ({
        String txHash,
        int usedNonce,
        int proposalId,
        String blockHashHex,
      })> submitProposeTransfer({
    required InstitutionInfo institution,
    required String beneficiaryAddress,
    required double amountYuan,
    required String remark,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    final identity = AdminAccountIdentity.fromInstitution(institution);
    final amountFen = BigInt.from((amountYuan * 100).round());
    final institutionBytes = _institutionAccountId(institution);
    final beneficiaryPubkey =
        _ss58AddressToAccountId(beneficiaryAddress, '收款地址');
    // 中文注释：InstitutionInfo.mainAccount 是 App 内部 AccountId hex，
    // 不能按 SS58/Base58 解码，否则 hex 中的 0 会被当成非法 Base58 字符。
    final fromPubkey = _accountHexToAccountId(institution.mainAccount, '转出主账户');
    final callData = _buildProposeTransferCall(
      org: identity.org,
      institutionIdentity: institution.sfidNumber,
      mainAccount: institution.mainAccount,
      beneficiaryAddress: beneficiaryAddress,
      amountFen: amountFen,
      remark: remark,
    );
    final submitResult = await _signAndSubmitInBlock(
      callData: callData,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
    );
    final proposalId = await _confirmTransferProposedEvent(
      blockHashHex: submitResult.blockHashHex,
      org: identity.org,
      institutionBytes: institutionBytes,
      proposerPubkey: signerPubkey,
      fromPubkey: fromPubkey,
      beneficiaryPubkey: beneficiaryPubkey,
      amountFen: amountFen,
    );
    return (
      txHash: submitResult.txHash,
      usedNonce: submitResult.usedNonce,
      proposalId: proposalId,
      blockHashHex: submitResult.blockHashHex,
    );
  }

  /// 提交 propose_safety_fund_transfer extrinsic（安全基金转账提案）。
  ///
  /// 中文注释：提案创建类交易必须等真正入块并核对事件后才算业务成功
  /// （与 submitProposeTransfer 同标准），返回事件中的 proposal_id
  /// 供后续投票跟踪；submit-only 给不出 proposal_id，也无法区分
  /// "已接受"与"已上链"。
  Future<({String txHash, int usedNonce, int proposalId, String blockHashHex})>
      submitProposeSafetyFund({
    required String beneficiaryAddress,
    required double amountYuan,
    required String remark,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    final amountFen = BigInt.from((amountYuan * 100).round());
    final beneficiaryPubkey =
        _ss58AddressToAccountId(beneficiaryAddress, '收款地址');
    final callData = _buildProposeSafetyFundCall(
      beneficiaryAddress: beneficiaryAddress,
      amountYuan: amountYuan,
      remark: remark,
    );
    final submitResult = await _signAndSubmitInBlock(
      callData: callData,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
    );
    final proposalId = await _confirmProposalEvent(
      blockHashHex: submitResult.blockHashHex,
      eventLabel: 'SafetyFundTransferProposed',
      eventIndex: _safetyFundProposedEventIndex,
      decode: (data, offset) => _decodeSafetyFundProposedEvent(
        data,
        offset,
        proposerPubkey: signerPubkey,
        beneficiaryPubkey: beneficiaryPubkey,
        amountFen: amountFen,
      ),
    );
    return (
      txHash: submitResult.txHash,
      usedNonce: submitResult.usedNonce,
      proposalId: proposalId,
      blockHashHex: submitResult.blockHashHex,
    );
  }

  /// 提交 propose_sweep_to_main extrinsic（手续费划转提案）。
  ///
  /// 中文注释：同 submitProposeSafetyFund，提案类必须入块+核对事件。
  Future<({String txHash, int usedNonce, int proposalId, String blockHashHex})>
      submitProposeSweep({
    required InstitutionInfo institution,
    required double amountYuan,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    final amountFen = BigInt.from((amountYuan * 100).round());
    final institutionBytes = _institutionAccountId(institution);
    final toPubkey = _accountHexToAccountId(institution.mainAccount, '机构主账户');
    final callData = _buildProposeSweepCall(
      institutionIdentity: institution.sfidNumber,
      mainAccount: institution.mainAccount,
      amountYuan: amountYuan,
    );
    final submitResult = await _signAndSubmitInBlock(
      callData: callData,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
    );
    final proposalId = await _confirmProposalEvent(
      blockHashHex: submitResult.blockHashHex,
      eventLabel: 'SweepToMainProposed',
      eventIndex: _sweepProposedEventIndex,
      decode: (data, offset) => _decodeSweepProposedEvent(
        data,
        offset,
        institutionBytes: institutionBytes,
        proposerPubkey: signerPubkey,
        toPubkey: toPubkey,
        amountFen: amountFen,
      ),
    );
    return (
      txHash: submitResult.txHash,
      usedNonce: submitResult.usedNonce,
      proposalId: proposalId,
      blockHashHex: submitResult.blockHashHex,
    );
  }

  // ──── 链上查询 ────

  /// 查询转出主账户的 finalized 可用余额（元）。
  ///
  /// 中文注释：治理机构按主账户、费用账户、安全基金、永久质押分别建模，转账提案固定从主账户支出；
  /// 个人/注册多签账户通过 InstitutionInfo.mainAccount 继续映射到账户地址。
  Future<double> fetchInstitutionBalance(InstitutionInfo institution) {
    return _rpc.fetchFinalizedBalance(institution.mainAccount);
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

  // ADR-018:`fetchProposalIdsByOrg` / `fetchProposalIdsByInstitution` /
  // `fetchProposalIdsByOwner` 已删除。前两者的 `ProposalsByOrg` 走统一的
  // 按年取 + 客户端 org 过滤(filterGovernanceIds);`ProposalsByInstitution`
  // 是嵌 32 字节 account 的长前缀扫描,轻节点静默返回空,改走按年取 +
  // 客户端 institution 过滤(filterInstitutionVisible)。`ProposalsByOwner`
  // 无调用方,一并移除。`ProposalsByYear`(短 key)是唯一保留的反向索引入口。

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

    // 一次拉到底(开发期数据量小);生产 1000 万级时再分页。
    // 必须走 finalized 钉块入口,否则轻节点追块窗口内会拿到旧状态空列表。
    final keysHex =
        await SmoldotClientManager.instance.getKeysPagedFinalized(prefixHex);
    if (keysHex.isEmpty) return const [];

    // 每条 key 末 8 字节 = proposal_id u64 LE(twox64_concat 的 raw 部分)
    final ids = <int>[];
    for (final keyHex in keysHex) {
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
  Future<List<int>> fetchActiveProposalIds(InstitutionInfo institution) async {
    final key = _buildStorageKey(
      'VotingEngine',
      'ActiveProposalsByInstitution',
      _institutionAccountId(institution),
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
    InstitutionInfo institution,
  ) async {
    final key = _buildDoubleStorageKey(
      'VotingEngine',
      'AdminSnapshot',
      _u64ToLeBytes(proposalId),
      _institutionAccountId(institution),
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

    // internal_institution: Option<AccountId32>
    Uint8List? institutionBytes;
    if (offset < data.length && data[offset] == 1) {
      offset++;
      if (offset + 32 <= data.length) {
        institutionBytes =
            Uint8List.fromList(data.sublist(offset, offset + 32));
        offset += 32;
      }
    }

    // 双层 ID v1:同时查 ProposalDisplayId 反查表(失败不阻塞,fallback null)
    ProposalDisplayMeta? displayMeta;
    try {
      displayMeta = await fetchProposalDisplayId(proposalId);
    } catch (e) {
      // 展示号缺失不阻断主流程，但必须留痕，否则 RPC 故障会伪装成"无展示号"。
      debugPrint(
          '[DuoqianTransfer] fetchProposalDisplayId($proposalId) 失败: $e');
    }

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
      final manageService = InstitutionManageService(chainRpc: _rpc);
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
            duoqianAccount: orgManageDetail.duoqianAccount,
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
        } catch (e) {
          // 查询失败必须留痕，否则与"确实不是安全基金提案"无法区分。
          debugPrint('[DuoqianTransfer] fetchSafetyFundAction($id) 失败: $e');
        }
        if (safetyFundDetail == null) {
          try {
            sweepDetail = await fetchSweepAction(id);
          } catch (e) {
            debugPrint('[DuoqianTransfer] fetchSweepAction($id) 失败: $e');
          }
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
        } catch (e) {
          debugPrint('[DuoqianTransfer] 联合提案类型检测($id) 失败: $e');
        }
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

  /// ADR-018 统一提案查询:按年取当前年全部提案一次。
  ///
  /// 中文注释:`ProposalsByYear[year]` 是短 key 索引,轻节点可正常前缀扫描;
  /// 链端对每个提案无条件写入该索引、终态清理时移除,所以按年取得到的就是
  /// "全部存活提案"。广场 / 机构详情 / 个人多签统一从这一份结果客户端按
  /// 已解码字段过滤,替代会在轻节点静默返回空的 `ProposalsByInstitution[account]`
  /// 长前缀扫描,并让多页面共用一次查询(见 ProposalFeedCache,降全节点负载)。
  Future<List<ProposalWithDetail>> fetchCurrentYearProposals() async {
    final currentYear = await _resolveCurrentYear();
    if (currentYear == null) return const [];
    final ids = await _fetchProposalIdsByYearTwox(currentYear);
    return _fetchProposalsForIds(ids);
  }

  /// 从当前年提案全集过滤出指定机构页可见的提案(纯客户端,不联网):
  /// 本机构内部提案(`internal_institution == 机构 AccountId`)∪ 全部联合投票
  /// (`kind == 1`,runtime 升级 / 决议在所有机构页可见)。
  List<ProposalWithDetail> filterInstitutionVisible(
    List<ProposalWithDetail> all,
    InstitutionInfo institution,
  ) {
    final account = _institutionAccountId(institution);
    final visible = all.where((p) {
      final inst = p.meta.institutionBytes;
      final mine = inst != null && _bytesEqual(inst, account);
      return mine || p.meta.kind == 1;
    }).toList();
    visible.sort((a, b) => b.meta.proposalId.compareTo(a.meta.proposalId));
    return visible;
  }

  /// 从当前年提案全集过滤出指定治理类型(org)的提案 id(广场用,纯客户端)。
  List<int> filterGovernanceIds(
    List<ProposalWithDetail> all,
    Set<int> orgs,
  ) {
    final ids = all
        .where((p) =>
            p.meta.internalOrg != null && orgs.contains(p.meta.internalOrg))
        .map((p) => p.meta.proposalId)
        .toList();
    ids.sort((a, b) => b.compareTo(a));
    return ids;
  }

  /// 查询对指定机构用户可见的提案事件(ADR-018:按年取 + 客户端过滤)。
  Future<List<ProposalWithDetail>> fetchInstitutionVisibleProposals(
      InstitutionInfo institution) async {
    final all = await fetchCurrentYearProposals();
    return filterInstitutionVisible(all, institution);
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
      InstitutionInfo institution) async {
    final visibleProposals =
        await fetchInstitutionVisibleProposals(institution);
    final institutionBytes = _institutionAccountId(institution);
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
      if (offset + 32 <= data.length) {
        institutionBytes =
            Uint8List.fromList(data.sublist(offset, offset + 32));
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

  /// 测试入口：暴露批量解码路径（_decodeProposalData），用链上真实
  /// payload 字节做布局回归，防止再出现"守卫长度与链端布局漂移"导致
  /// 提案静默消失（2026-06-11 旧 48 字节主体残留事故）。
  @visibleForTesting
  TransferProposalInfo? debugDecodeProposalData(int proposalId, Uint8List raw) {
    return _decodeProposalData(proposalId, raw);
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
      // 最小长度 = tag(7) + institution(32) + beneficiary(32) + amount(16)
      //          + remark Compact(≥1) + proposer(32) = 120（空备注下限）。
      if (data.length < tag.length + 32 + 32 + 16 + 1 + 32) return null;
      for (var i = 0; i < tag.length; i++) {
        if (data[i] != tag[i]) return null;
      }
      return _decodeTransferAction(proposalId, data.sublist(tag.length));
    } catch (e) {
      // SCALE 解码失败必须留痕：runtime 升级布局变更时提案会"凭空消失"，
      // 没有日志就无从排查。
      debugPrint('[DuoqianTransfer] 提案 $proposalId ProposalData 解码失败: $e');
      return null;
    }
  }

  /// 查询某管理员对某提案的投票记录。null=未投票，true=赞成，false=反对。
  Future<bool?> fetchAdminVote(int proposalId, String pubkeyHex) async {
    return _internalVoteQuery.fetchAdminVote(proposalId, pubkeyHex);
  }

  /// 批量查询某提案下多名管理员的投票记录。
  ///
  /// 中文注释：转账/多签管理详情页展示全管理员投票时走批量读取，
  /// 避免 43 名管理员造成 43 次 storage RPC。
  Future<Map<String, bool?>> fetchAdminVotesBatch(
    int proposalId,
    Iterable<String> pubkeysHex,
  ) {
    return _internalVoteQuery.fetchAdminVotesBatch(proposalId, pubkeysHex);
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
  ///   institution: AccountId32(32) + beneficiary: AccountId32(32) + amount: u128(16)
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
    if (data.length < tag.length + 32 + 32 + 16 + 1 + 32) return null;
    for (var i = 0; i < tag.length; i++) {
      if (data[i] != tag[i]) return null;
    }
    return _decodeTransferAction(proposalId, data.sublist(tag.length));
  }

  /// 解码 TransferAction SCALE 数据。
  TransferProposalInfo? _decodeTransferAction(int proposalId, Uint8List data) {
    try {
      var offset = 0;

      // institution: AccountId32
      final institutionBytes = data.sublist(offset, offset + 32);
      offset += 32;

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
    } catch (e) {
      // 字段级解码失败同样要留痕，与上层"提案不存在"区分开。
      debugPrint('[DuoqianTransfer] 提案 $proposalId TransferAction 解码失败: $e');
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
  /// 格式：[0x13][0x00][org:u8][institution:AccountId32][beneficiary:32bytes][amount:u128][Vec remark]
  Uint8List _buildProposeTransferCall({
    required int org,
    required String institutionIdentity,
    required String mainAccount,
    required String beneficiaryAddress,
    required BigInt amountFen,
    required String remark,
  }) {
    final output = ByteOutput();
    output.pushByte(_palletIndex);
    output.pushByte(_proposeCallIndex);

    // org: u8
    output.pushByte(org);

    // institution: AccountId32
    output.write(
        _institutionIdentityToAccountId(institutionIdentity, mainAccount));

    // beneficiary: AccountId32 = 32 bytes（不是 MultiAddress，无 0x00 前缀）
    final beneficiaryId = _ss58AddressToAccountId(beneficiaryAddress, '收款地址');
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
  /// 格式：[0x13][0x02][institution:AccountId32][amount:u128_le]
  Uint8List _buildProposeSweepCall({
    required String institutionIdentity,
    required String mainAccount,
    required double amountYuan,
  }) {
    final output = ByteOutput();
    output.pushByte(_palletIndex);
    output.pushByte(_proposeSweepCallIndex);
    output.write(
        _institutionIdentityToAccountId(institutionIdentity, mainAccount));
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
    final beneficiaryId = _ss58AddressToAccountId(beneficiaryAddress, '收款地址');
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
    } catch (e) {
      // SCALE 解码失败必须留痕，与"确实不是安全基金提案"区分开。
      debugPrint('[DuoqianTransfer] 提案 $proposalId SafetyFundAction 解码失败: $e');
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
    // SweepAction: institution(AccountId32) + amount(u128=16)
    if (raw == null || raw.length < 32 + 16) return null;
    try {
      final institutionBytes = Uint8List.fromList(raw.sublist(0, 32));
      var amountBig = BigInt.zero;
      for (var i = 15; i >= 0; i--) {
        amountBig = (amountBig << 8) | BigInt.from(raw[32 + i]);
      }
      return SweepProposalInfo(
        proposalId: proposalId,
        institutionBytes: institutionBytes,
        amountFen: amountBig,
      );
    } catch (e) {
      // SCALE 解码失败必须留痕，与"确实不是手续费划转提案"区分开。
      debugPrint('[DuoqianTransfer] 提案 $proposalId SweepAction 解码失败: $e');
      return null;
    }
  }

  Future<({String txHash, int usedNonce, String blockHashHex})>
      _signAndSubmitInBlock({
    required Uint8List callData,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    return SignedExtrinsicBuilder(
      chainRpc: _rpc,
      logLabel: 'TransferProposal',
    ).signAndSubmitInBlock(
      callData: callData,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
      onTrace: _logSignedExtrinsicTrace,
    );
  }

  Future<int> _confirmTransferProposedEvent({
    required String blockHashHex,
    required int org,
    required Uint8List institutionBytes,
    required Uint8List proposerPubkey,
    required Uint8List fromPubkey,
    required Uint8List beneficiaryPubkey,
    required BigInt amountFen,
  }) {
    return _confirmProposalEvent(
      blockHashHex: blockHashHex,
      eventLabel: 'TransferProposed',
      eventIndex: _transferProposedEventIndex,
      decode: (data, offset) => _decodeTransferProposedEvent(
        data,
        offset,
        org: org,
        institutionBytes: institutionBytes,
        proposerPubkey: proposerPubkey,
        fromPubkey: fromPubkey,
        beneficiaryPubkey: beneficiaryPubkey,
        amountFen: amountFen,
      ),
    );
  }

  /// 入块后读取 System.Events 核对提案事件并提取 proposal_id。
  ///
  /// 中文注释：三类提案（转账/安全基金/手续费划转）共用本入口，
  /// 事件不存在=业务失败，必须抛错而不是静默放过。
  Future<int> _confirmProposalEvent({
    required String blockHashHex,
    required String eventLabel,
    required int eventIndex,
    required ({int proposalId, bool matches})? Function(
      Uint8List data,
      int offset,
    ) decode,
  }) async {
    final events = await _rpc.fetchSystemEventsAtBlock(blockHashHex);
    if (events == null || events.isEmpty) {
      throw StateError('交易已入块，但未读取到 System.Events，不能确认提案创建成功');
    }
    final proposalId = _findProposalIdInEvents(
      events,
      eventIndex: eventIndex,
      decode: decode,
    );
    if (proposalId == null) {
      throw StateError(
        '交易已入块，但未找到 DuoqianTransfer.$eventLabel 事件，提案创建失败',
      );
    }
    return proposalId;
  }

  /// 在 System.Events 原始字节中扫描本 pallet 指定事件并提取 proposal_id。
  ///
  /// 中文注释：三类提案共用扫描骨架，事件字段解码与匹配由 decode 回调完成。
  int? _findProposalIdInEvents(
    Uint8List data, {
    required int eventIndex,
    required ({int proposalId, bool matches})? Function(
      Uint8List data,
      int offset,
    ) decode,
  }) {
    final (_, countSize) = _decodeCompact(data, 0);
    if (countSize <= 0) return null;
    for (var scanOffset = countSize; scanOffset < data.length; scanOffset++) {
      var offset = scanOffset;
      final phase = data[offset];
      offset += 1;
      if (phase == 0x00) {
        if (offset + 4 > data.length) continue;
        offset += 4;
      } else if (phase != 0x01 && phase != 0x02) {
        continue;
      }

      if (offset + 2 > data.length) continue;
      final palletIndex = data[offset];
      final evtIndex = data[offset + 1];
      offset += 2;

      if (palletIndex == _palletIndex && evtIndex == eventIndex) {
        final decoded = decode(data, offset);
        if (decoded == null) continue;
        if (decoded.matches) return decoded.proposalId;
      }
    }
    return null;
  }

  ({
    int proposalId,
    bool matches,
  })? _decodeTransferProposedEvent(
    Uint8List data,
    int offset, {
    required int org,
    required Uint8List institutionBytes,
    required Uint8List proposerPubkey,
    required Uint8List fromPubkey,
    required Uint8List beneficiaryPubkey,
    required BigInt amountFen,
  }) {
    // 中文注释：TransferProposed 事件字段顺序必须与 runtime Event enum 完全一致。
    const fixedBytes = 8 + 1 + 32 + 32 + 32 + 32 + 16;
    if (offset + fixedBytes > data.length) return null;
    var pos = offset;
    final proposalId = _readU64LE(data, pos);
    pos += 8;
    final eventOrg = data[pos];
    pos += 1;
    final eventInstitution = Uint8List.fromList(data.sublist(pos, pos + 32));
    pos += 32;
    final eventProposer = Uint8List.fromList(data.sublist(pos, pos + 32));
    pos += 32;
    final eventFrom = Uint8List.fromList(data.sublist(pos, pos + 32));
    pos += 32;
    final eventBeneficiary = Uint8List.fromList(data.sublist(pos, pos + 32));
    pos += 32;
    final eventAmount = _readU128LE(data, pos);
    pos += 16;

    final (remarkLen, remarkLenSize) = _decodeCompact(data, pos);
    if (remarkLenSize <= 0 || pos + remarkLenSize + remarkLen > data.length) {
      return null;
    }
    pos += remarkLenSize + remarkLen;

    // runtime BlockNumber = u32，expires_at 紧跟 remark 之后。
    if (pos + 4 > data.length) return null;
    pos += 4;

    if (_skipTopics(data, pos) == null) return null;
    final matches = eventOrg == org &&
        _bytesEqual(eventInstitution, institutionBytes) &&
        _bytesEqual(eventProposer, proposerPubkey) &&
        _bytesEqual(eventFrom, fromPubkey) &&
        _bytesEqual(eventBeneficiary, beneficiaryPubkey) &&
        eventAmount == amountFen;
    return (
      proposalId: proposalId,
      matches: matches,
    );
  }

  ({
    int proposalId,
    bool matches,
  })? _decodeSafetyFundProposedEvent(
    Uint8List data,
    int offset, {
    required Uint8List proposerPubkey,
    required Uint8List beneficiaryPubkey,
    required BigInt amountFen,
  }) {
    // 中文注释：SafetyFundTransferProposed 字段顺序必须与 runtime Event enum
    // 完全一致：proposal_id u64 | proposer 32B | from 32B | beneficiary 32B
    // | amount u128 | remark Compact+bytes | expires_at u32 | topics。
    const fixedBytes = 8 + 32 + 32 + 32 + 16;
    if (offset + fixedBytes > data.length) return null;
    var pos = offset;
    final proposalId = _readU64LE(data, pos);
    pos += 8;
    final eventProposer = Uint8List.fromList(data.sublist(pos, pos + 32));
    pos += 32;
    // from = NRC 安全基金常量地址，调用方不持有，不参与匹配。
    pos += 32;
    final eventBeneficiary = Uint8List.fromList(data.sublist(pos, pos + 32));
    pos += 32;
    final eventAmount = _readU128LE(data, pos);
    pos += 16;

    final (remarkLen, remarkLenSize) = _decodeCompact(data, pos);
    if (remarkLenSize <= 0 || pos + remarkLenSize + remarkLen > data.length) {
      return null;
    }
    pos += remarkLenSize + remarkLen;

    // runtime BlockNumber = u32，expires_at 紧跟 remark 之后。
    if (pos + 4 > data.length) return null;
    pos += 4;

    if (_skipTopics(data, pos) == null) return null;
    final matches = _bytesEqual(eventProposer, proposerPubkey) &&
        _bytesEqual(eventBeneficiary, beneficiaryPubkey) &&
        eventAmount == amountFen;
    return (
      proposalId: proposalId,
      matches: matches,
    );
  }

  ({
    int proposalId,
    bool matches,
  })? _decodeSweepProposedEvent(
    Uint8List data,
    int offset, {
    required Uint8List institutionBytes,
    required Uint8List proposerPubkey,
    required Uint8List toPubkey,
    required BigInt amountFen,
  }) {
    // 中文注释：SweepToMainProposed 字段顺序必须与 runtime Event enum 完全
    // 一致：proposal_id u64 | institution 32B | proposer 32B | from 32B
    // | to 32B | amount u128 | expires_at u32 | topics（无 remark 字段）。
    const fixedBytes = 8 + 32 + 32 + 32 + 32 + 16 + 4;
    if (offset + fixedBytes > data.length) return null;
    var pos = offset;
    final proposalId = _readU64LE(data, pos);
    pos += 8;
    final eventInstitution = Uint8List.fromList(data.sublist(pos, pos + 32));
    pos += 32;
    final eventProposer = Uint8List.fromList(data.sublist(pos, pos + 32));
    pos += 32;
    // from = 机构费用账户，调用方不直接持有，不参与匹配。
    pos += 32;
    final eventTo = Uint8List.fromList(data.sublist(pos, pos + 32));
    pos += 32;
    final eventAmount = _readU128LE(data, pos);
    pos += 16;
    // expires_at: u32。
    pos += 4;

    if (_skipTopics(data, pos) == null) return null;
    final matches = _bytesEqual(eventInstitution, institutionBytes) &&
        _bytesEqual(eventProposer, proposerPubkey) &&
        _bytesEqual(eventTo, toPubkey) &&
        eventAmount == amountFen;
    return (
      proposalId: proposalId,
      matches: matches,
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

  Uint8List _institutionAccountId(InstitutionInfo institution) {
    return _institutionIdentityToAccountId(
      institution.sfidNumber,
      institution.mainAccount,
    );
  }

  Uint8List _institutionIdentityToAccountId(
    String institutionIdentity,
    String mainAccount,
  ) {
    return Uint8List.fromList(
      institutionIdentityToAccountId(
        institutionIdentity,
        mainAccount: mainAccount,
      ),
    );
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

  int _readU64LE(Uint8List data, int offset) {
    final bd = ByteData.sublistView(data, offset, offset + 8);
    return bd.getUint64(0, Endian.little);
  }

  BigInt _readU128LE(Uint8List data, int offset) {
    var value = BigInt.zero;
    for (var i = 15; i >= 0; i--) {
      value = (value << 8) | BigInt.from(data[offset + i]);
    }
    return value;
  }

  int _decodeU32(Uint8List data, int offset) {
    final bd = ByteData.sublistView(data);
    return bd.getUint32(offset, Endian.little);
  }

  int? _skipTopics(Uint8List data, int offset) {
    if (offset >= data.length) return null;
    final (count, size) = _decodeCompact(data, offset);
    if (size <= 0) return null;
    final next = offset + size + count * 32;
    return next <= data.length ? next : null;
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

  Uint8List _accountHexToAccountId(String hex, String fieldName) {
    final clean = hex.startsWith('0x') ? hex.substring(2) : hex;
    if (clean.length != 64 || !RegExp(r'^[0-9a-fA-F]+$').hasMatch(clean)) {
      throw FormatException('$fieldName必须是32字节账户hex');
    }
    return _hexDecode(clean);
  }

  Uint8List _ss58AddressToAccountId(String address, String fieldName) {
    try {
      return Uint8List.fromList(Keyring().decodeAddress(address));
    } catch (_) {
      throw FormatException('$fieldName必须是SS58地址');
    }
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
