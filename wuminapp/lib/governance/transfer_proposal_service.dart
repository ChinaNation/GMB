import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:polkadart/polkadart.dart'
    show ExtrinsicPayload, Hasher, SignatureType, SigningPayload;
import 'package:polkadart/scale_codec.dart' show CompactBigIntCodec, ByteOutput;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import '../rpc/chain_rpc.dart';
import 'institution_data.dart';
import 'proposal_cache.dart';

/// 机构转账提案链上交互服务。
///
/// 负责 extrinsic 编码/提交 和 storage 查询。
class TransferProposalService {
  TransferProposalService({ChainRpc? chainRpc})
      : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;

  /// 当前 RPC 节点的 HTTP URL（用于推导 WebSocket URL）。
  String get rpcNodeUrl => _rpc.currentNodeUrl;

  // ──── 常量 ────

  /// DuoqianTransferPow pallet index（runtime pallet_index=19）。
  static const _palletIndex = 19;

  /// propose_transfer call_index=0。
  static const _proposeCallIndex = 0;

  /// vote_transfer call_index=1。
  static const _voteCallIndex = 1;

  /// Mortal era 周期。
  static const _eraPeriod = 64;

  // ──── Extrinsic 提交 ────

  /// 提交 propose_transfer extrinsic。
  ///
  /// 返回交易哈希 hex（含 0x 前缀）。
  Future<String> submitProposeTransfer({
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
      shenfenId: institution.shenfenId,
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

  /// 提交 vote_transfer extrinsic。
  ///
  /// 返回交易哈希 hex（含 0x 前缀）。
  Future<String> submitVoteTransfer({
    required int proposalId,
    required bool approve,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    final callData = _buildVoteTransferCall(
      proposalId: proposalId,
      approve: approve,
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

  /// 查询机构活跃的提案 ID 列表（从 VotingEngineSystem 全局存储读取）。
  Future<List<int>> fetchActiveProposalIds(String shenfenId) async {
    final key = _buildStorageKey(
      'VotingEngineSystem',
      'ActiveProposalsByInstitution',
      _shenfenIdToFixed48(shenfenId),
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
      'VotingEngineSystem',
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
      'VotingEngineSystem',
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
      'VotingEngineSystem',
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
        institutionBytes = Uint8List.fromList(data.sublist(offset, offset + 48));
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
      // 只获取转账提案详情；runtime 升级的 ProposalData 包含完整 WASM（数 MB），
      // 列表展示不需要下载，进入详情页时再加载。
      TransferProposalInfo? transferDetail;
      if (meta.kind == 0) {
        try {
          transferDetail = await fetchProposalAction(meta.proposalId);
        } catch (_) {}
      }

      results.add(ProposalWithDetail(
        meta: meta,
        transferDetail: transferDetail?.copyWithStatus(meta.status),
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

    for (var id = startId; id > startId - count && id >= 0; id--) {
      final cached = ProposalCache.getMeta(id);
      if (cached != null) {
        cachedMetas[id] = cached;
      } else {
        final keyBytes = _buildStorageKey(
            'VotingEngineSystem', 'Proposals', _u64ToLeBytes(id));
        uncachedMetaKeys.add('0x${_hexEncode(keyBytes)}');
        uncachedMetaIds.add(id);
      }
    }

    // 批量查询未命中的 meta
    if (uncachedMetaKeys.isNotEmpty) {
      debugPrint('[ProposalPage] batch query ${uncachedMetaKeys.length} metas');
      final batchResult = await _rpc.fetchStorageBatch(uncachedMetaKeys);
      for (var i = 0; i < uncachedMetaIds.length; i++) {
        final id = uncachedMetaIds[i];
        final data = batchResult[uncachedMetaKeys[i]];
        debugPrint('[ProposalPage] id=$id, meta data=${data != null ? "len=${data.length} bytes=[${data.take(10).join(",")}]" : "null"}');
        if (data != null && data.length >= 3) {
          final meta = _decodeProposalMeta(id, data);
          debugPrint('[ProposalPage] id=$id, decoded meta: kind=${meta?.kind}, stage=${meta?.stage}, status=${meta?.status}');
          if (meta != null) {
            cachedMetas[id] = meta;
            ProposalCache.putMeta(id, meta);
          }
        }
      }
    }
    debugPrint('[ProposalPage] total metas found: ${cachedMetas.length}, kinds: ${cachedMetas.values.map((m) => m.kind).toList()}');

    // 对有 meta 的提案，批量查询 ProposalData（先查缓存）
    // 注意：只查 kind==0（内部/转账）的 ProposalData；
    // kind==1（联合/runtime升级）的 ProposalData 包含完整 WASM 二进制（可能数 MB），
    // 列表展示不需要下载这些数据。
    final uncachedDetailKeys = <String>[];
    final uncachedDetailIds = <int>[];
    final cachedDetails = <int, TransferProposalInfo>{};

    for (final entry in cachedMetas.entries) {
      if (entry.value.kind != 0) continue; // 只查转账提案的详情
      final cachedDetail = ProposalCache.getDetail(entry.key);
      if (cachedDetail != null) {
        cachedDetails[entry.key] = cachedDetail;
      } else {
        final keyBytes = _buildStorageKey(
            'VotingEngineSystem', 'ProposalData', _u64ToLeBytes(entry.key));
        uncachedDetailKeys.add('0x${_hexEncode(keyBytes)}');
        uncachedDetailIds.add(entry.key);
      }
    }

    if (uncachedDetailKeys.isNotEmpty) {
      final batchResult = await _rpc.fetchStorageBatch(uncachedDetailKeys);
      for (var i = 0; i < uncachedDetailIds.length; i++) {
        final id = uncachedDetailIds[i];
        final raw = batchResult[uncachedDetailKeys[i]];
        if (raw != null && raw.isNotEmpty) {
          final detail = _decodeProposalData(id, raw);
          if (detail != null) {
            cachedDetails[id] = detail;
            ProposalCache.putDetail(id, detail);
          }
        }
      }
    }

    // 组装结果
    for (var id = startId; id > startId - count && id >= 0; id--) {
      final meta = cachedMetas[id];
      if (meta == null) continue;

      TransferProposalInfo? transferDetail;
      if (meta.kind == 0) {
        // 内部投票提案 → 转账详情
        transferDetail = cachedDetails[id]?.copyWithStatus(meta.status);
      }
      // kind==1（runtime 升级）的详情在进入详情页时才加载，
      // 列表展示只需 meta.kind 即可区分提案类型。

      results.add(ProposalWithDetail(
        meta: meta,
        transferDetail: transferDetail,
      ));
    }

    return results;
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

  /// 从原始 SCALE 字节解码 ProposalData（BoundedVec<u8> → TransferAction）。
  TransferProposalInfo? _decodeProposalData(int proposalId, Uint8List raw) {
    try {
      int offset = 0;
      final (vecLen, lenBytes) = _decodeCompact(raw, offset);
      offset += lenBytes;
      if (offset + vecLen > raw.length) return null;
      final data = raw.sublist(offset, offset + vecLen);
      if (data.length < 48 + 32 + 16 + 1 + 32) return null;
      return _decodeTransferAction(proposalId, data);
    } catch (_) {
      return null;
    }
  }

  /// 查询某管理员对某提案的投票记录。null=未投票，true=赞成，false=反对。
  Future<bool?> fetchAdminVote(int proposalId, String pubkeyHex) async {
    final proposalIdBytes = _u64ToLeBytes(proposalId);
    final accountBytes = _hexDecode(pubkeyHex);

    // 双 key：blake2_128_concat(proposal_id) + blake2_128_concat(account)
    final palletHash =
        _twoxx128String('VotingEngineSystem');
    final storageHash =
        _twoxx128String('InternalVotesByAccount');
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
    final palletHash = _twoxx128String('VotingEngineSystem');
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
      'VotingEngineSystem',
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

    if (data.length < 48 + 32 + 16 + 1 + 32) return null;
    return _decodeTransferAction(proposalId, data);
  }

  /// 查询指定机构相关的所有提案（包括转账和 runtime 升级等），按 ID 倒序。
  ///
  /// 对于内部提案（kind==0）：按 institution 匹配。
  /// 对于联合提案（kind==1，如 runtime 升级）：由国储会管理员发起，
  ///   通过查 ProposalData 中的 proposer 是否属于该机构的管理员来匹配。
  ///   但联合提案属于全链，所以只要 shenfenId 是国储会就显示所有联合提案。
  Future<List<ProposalWithDetail>> fetchAllInstitutionProposals(
      String shenfenId) async {
    final nextId = await fetchNextProposalId();
    debugPrint('[ProposalQuery] nextProposalId=$nextId');
    if (nextId == 0) return const [];

    final institutionBytes = _shenfenIdToFixed48(shenfenId);

    // 提案 ID 格式：year * 1000000 + counter，从当年起始 ID 开始查询
    final year = nextId ~/ 1000000;
    final startId = year * 1000000;

    // 第一步：批量查询所有提案的 meta
    final metaKeys = <String>[];
    final metaIds = <int>[];
    for (var id = startId; id < nextId; id++) {
      final keyBytes = _buildStorageKey(
          'VotingEngineSystem', 'Proposals', _u64ToLeBytes(id));
      metaKeys.add('0x${_hexEncode(keyBytes)}');
      metaIds.add(id);
    }

    final allMetas = <int, ProposalMeta>{};
    if (metaKeys.isNotEmpty) {
      final batchResult = await _rpc.fetchStorageBatch(metaKeys);
      for (var i = 0; i < metaIds.length; i++) {
        final data = batchResult[metaKeys[i]];
        debugPrint('[InstProposal] id=${metaIds[i]}, meta data=${data != null ? "len=${data.length} bytes=[${data.take(10).join(",")}]" : "null"}');
        if (data != null && data.length >= 3) {
          final meta = _decodeProposalMeta(metaIds[i], data);
          debugPrint('[InstProposal] id=${metaIds[i]}, kind=${meta?.kind}, stage=${meta?.stage}, status=${meta?.status}');
          if (meta != null) allMetas[metaIds[i]] = meta;
        }
      }
    }
    // 判断当前机构是否是国储会（国储会显示所有联合提案）
    final isNrc = _isNrcInstitution(shenfenId);
    debugPrint('[InstProposal] total metas: ${allMetas.length}, isNrc=$isNrc');

    // 第二步：分类处理
    // kind==0：查 ProposalData 按 institution 匹配
    // kind==1：联合提案（runtime 升级等），国储会页面直接显示
    final results = <ProposalWithDetail>[];

    // 收集 kind==0 的 ID，批量查 ProposalData
    final internalIds = <int>[];
    for (final entry in allMetas.entries) {
      if (entry.value.kind == 0) {
        internalIds.add(entry.key);
      } else if (entry.value.kind == 1 && isNrc) {
        // 联合提案：国储会页面直接显示
        results.add(ProposalWithDetail(meta: entry.value));
      }
    }

    // 查 kind==0 的转账详情
    final futures = <Future<TransferProposalInfo?>>[];
    for (final id in internalIds) {
      futures.add(fetchProposalAction(id));
    }
    final detailResults = await Future.wait(futures);

    for (var i = 0; i < internalIds.length; i++) {
      final info = detailResults[i];
      if (info == null) continue;
      if (_bytesEqual(info.institutionBytes, institutionBytes)) {
        final meta = allMetas[internalIds[i]]!;
        results.add(ProposalWithDetail(
          meta: meta,
          transferDetail: info.copyWithStatus(meta.status),
        ));
      }
    }

    // 按 ID 倒序（最新在上）
    results.sort((a, b) => b.meta.proposalId.compareTo(a.meta.proposalId));
    return results;
  }

  /// 判断 shenfenId 对应的机构是否是国储会。
  bool _isNrcInstitution(String shenfenId) {
    final inst = findInstitutionByShenfenId(shenfenId);
    return inst != null && inst.orgType == OrgType.nrc;
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

      final beneficiarySs58 = Keyring()
          .encodeAddress(Uint8List.fromList(beneficiaryBytes), 2027);
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
    required String shenfenId,
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
    output.write(_shenfenIdToFixed48(shenfenId));

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

  /// 构造 vote_transfer call data。
  ///
  /// 格式：[0x13][0x01][proposal_id:u64_le][approve:bool]
  Uint8List _buildVoteTransferCall({
    required int proposalId,
    required bool approve,
  }) {
    final output = ByteOutput();
    output.pushByte(_palletIndex);
    output.pushByte(_voteCallIndex);

    // proposal_id: u64 little-endian
    output.write(_u64ToLeBytes(proposalId));

    // approve: bool
    output.pushByte(approve ? 1 : 0);

    return output.toBytes();
  }

  /// 签名并提交 extrinsic（复用 onchain.dart 的流程）。
  Future<String> _signAndSubmit({
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

    debugPrint('[TransferProposal] 步骤3: 并行获取 runtimeVersion/nonce/latestBlock...');
    final results = await Future.wait([
      _rpc.fetchRuntimeVersion(),
      _rpc.fetchNonce(fromAddress),
      _rpc.fetchLatestBlock(),
    ]);
    final runtimeVersion = results[0] as dynamic;
    final nonce = results[1] as int;
    final latestBlock =
        results[2] as ({Uint8List blockHash, int blockNumber});
    debugPrint('[TransferProposal] nonce=$nonce, block=${latestBlock.blockNumber}');

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
    final encoded =
        extrinsicPayload.encode(registry, SignatureType.sr25519);
    debugPrint('[TransferProposal] extrinsic 编码完成 (${encoded.length} bytes)');

    debugPrint('[TransferProposal] 步骤7: 提交到链...');
    debugPrint('[TransferProposal] call data hex: ${_hexEncode(callData)}');
    debugPrint('[TransferProposal] encoded extrinsic hex: ${_hexEncode(encoded)}');
    try {
      final txHash = await _rpc.submitExtrinsic(encoded);
      debugPrint('[TransferProposal] 提交成功: 0x${_hexEncode(txHash)}');
      return '0x${_hexEncode(txHash)}';
    } catch (e) {
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

  Uint8List _shenfenIdToFixed48(String shenfenId) {
    final raw = utf8.encode(shenfenId);
    if (raw.isEmpty || raw.length > 48) {
      throw ArgumentError('shenfenId 长度必须在 1..48 字节');
    }
    final out = Uint8List(48);
    out.setAll(0, raw);
    return out;
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
  final int kind;   // 0=internal, 1=joint
  final int stage;  // 0=internal, 1=joint, 2=citizen
  final int status; // 0=voting, 1=passed, 2=rejected
  final int? internalOrg;
  final Uint8List? institutionBytes;
}

/// 提案 + 业务详情（用于全局提案列表展示）。
class ProposalWithDetail {
  const ProposalWithDetail({
    required this.meta,
    this.transferDetail,
  });

  final ProposalMeta meta;
  /// 转账提案详情（非转账提案为 null）。
  final TransferProposalInfo? transferDetail;
}
