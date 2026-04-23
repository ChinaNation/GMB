import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:polkadart/polkadart.dart'
    show ExtrinsicPayload, Hasher, SignatureType, SigningPayload;
import 'package:polkadart/scale_codec.dart' show CompactBigIntCodec, ByteOutput;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import '../rpc/chain_rpc.dart';
import '../rpc/nonce_manager.dart';
import 'transfer_proposal_service.dart' show ProposalMeta;

/// Runtime upgrade 提案链上交互服务。
///
/// 负责 extrinsic 编码/提交 和 storage 查询。
class RuntimeUpgradeService {
  RuntimeUpgradeService({ChainRpc? chainRpc}) : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;

  // ──── 常量 ────

  /// runtime-root-upgrade pallet index=13。
  static const _palletIndex = 13;

  /// VotingEngineSystem pallet index=9。
  static const _jointVotePalletIndex = 9;

  /// propose_runtime_upgrade call_index=0。
  static const _proposeCallIndex = 0;

  /// joint_vote call_index=1（Phase 2 重排后,投票引擎内部 call_index 序列
  /// 为 0=internal_vote / 1=joint_vote / 2=citizen_vote / 3=finalize_proposal）。
  static const _jointVoteCallIndex = 1;

  /// Mortal era 周期。
  static const _eraPeriod = 64;

  // ──── Extrinsic 提交 ────

  /// 提交 propose_runtime_upgrade extrinsic。
  ///
  /// 返回交易哈希 hex（含 0x 前缀）和使用的 nonce。
  Future<({String txHash, int usedNonce})> submitProposeRuntimeUpgrade({
    required String reason,
    required Uint8List wasmCode,
    required int eligibleTotal,
    required Uint8List snapshotNonce,
    required Uint8List signature,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    final callData = _buildProposeRuntimeUpgradeCall(
      reason: reason,
      wasmCode: wasmCode,
      eligibleTotal: eligibleTotal,
      snapshotNonce: snapshotNonce,
      signature: signature,
    );
    return _signAndSubmit(
      callData: callData,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
    );
  }

  /// 提交机构管理员的联合投票。
  ///
  /// 返回交易哈希 hex（含 0x 前缀）和使用的 nonce。
  Future<({String txHash, int usedNonce})> submitJointVote({
    required int proposalId,
    required Uint8List institutionId48,
    required bool approve,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    final callData = _buildJointVoteCall(
      proposalId: proposalId,
      institutionId48: institutionId48,
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

  /// 查询 runtime upgrade 提案详情。返回 null 表示不存在。
  ///
  /// ProposalData 是 BoundedVec<u8>，SCALE 编码为 Compact 长度前缀 + 原始字节。
  /// 原始字节布局：
  ///   proposer: AccountId32(32) + reason: Vec<u8>(Compact len + bytes)
  ///   + code_hash: [u8;32] + status: u8 enum (0=Voting, 1=Passed, 2=Rejected, 3=ExecutionFailed)
  Future<RuntimeUpgradeProposalInfo?> fetchRuntimeUpgradeProposal(
      int proposalId) async {
    final key = _buildStorageKey(
      'VotingEngineSystem',
      'ProposalData',
      _u64ToLeBytes(proposalId),
    );
    final raw = await _rpc.fetchStorage('0x${_hexEncode(key)}');
    if (raw == null || raw.isEmpty) return null;
    return decodeRuntimeUpgradeStorageValue(proposalId, raw);
  }

  /// 解码 `ProposalData` 的原始 storage value（带 Compact 长度前缀）。
  ///
  /// 中文注释：分页列表会批量读取 ProposalData，随后在内存里按提案类型解码；
  /// 这里提供公共入口，避免不同页面各自复制一套 runtime 提案解码逻辑。
  RuntimeUpgradeProposalInfo? decodeRuntimeUpgradeStorageValue(
      int proposalId, Uint8List raw) {
    try {
      int offset = 0;
      final (vecLen, lenBytes) = _decodeCompact(raw, offset);
      offset += lenBytes;
      if (offset + vecLen > raw.length) return null;
      final data = raw.sublist(offset, offset + vecLen);
      return _decodeRuntimeUpgradeAction(proposalId, data);
    } catch (_) {
      return null;
    }
  }

  /// 查询联合投票计数（JointTallies）。
  ///
  /// Value: VoteCountU32 { yes: u32, no: u32 } = 8 bytes。
  Future<({int yes, int no})> fetchJointTally(int proposalId) async {
    final key = _buildStorageKey(
      'VotingEngineSystem',
      'JointTallies',
      _u64ToLeBytes(proposalId),
    );
    final data = await _rpc.fetchStorage('0x${_hexEncode(key)}');
    if (data == null || data.length < 8) return (yes: 0, no: 0);
    // VoteCountU32: { yes: u32, no: u32 } — 4+4 bytes little-endian
    final yes = _decodeU32(data, 0);
    final no = _decodeU32(data, 4);
    return (yes: yes, no: no);
  }

  /// 查询联合投票中某机构的投票记录。
  ///
  /// 双 map：blake2_128_concat(u64_le) + blake2_128_concat(48 bytes)。
  /// Value: Option<bool> — null=未投票，true=赞成，false=反对。
  Future<bool?> fetchJointVoteByInstitution(
      int proposalId, Uint8List institutionId48) async {
    final fullKey = _buildDoubleStorageKey(
      'VotingEngineSystem',
      'JointVotesByInstitution',
      _u64ToLeBytes(proposalId),
      institutionId48,
    );
    final data = await _rpc.fetchStorage('0x${_hexEncode(fullKey)}');
    if (data == null || data.isEmpty) return null;
    return data[0] == 1;
  }

  /// 查询某机构在联合投票阶段的管理员票数统计。
  Future<({int yes, int no})> fetchJointInstitutionTally(
      int proposalId, Uint8List institutionId48) async {
    final fullKey = _buildDoubleStorageKey(
      'VotingEngineSystem',
      'JointInstitutionTallies',
      _u64ToLeBytes(proposalId),
      institutionId48,
    );
    final data = await _rpc.fetchStorage('0x${_hexEncode(fullKey)}');
    if (data == null || data.length < 8) return (yes: 0, no: 0);
    return (yes: _decodeU32(data, 0), no: _decodeU32(data, 4));
  }

  /// 查询某管理员在某机构联合投票中的投票记录。
  Future<bool?> fetchJointAdminVote(
    int proposalId,
    Uint8List institutionId48,
    String pubkeyHex,
  ) async {
    final accountBytes = Uint8List.fromList(_hexDecode(pubkeyHex));
    if (institutionId48.length != 48 || accountBytes.length != 32) return null;
    final compositeKey = Uint8List(institutionId48.length + accountBytes.length)
      ..setAll(0, institutionId48)
      ..setAll(institutionId48.length, accountBytes);
    final fullKey = _buildDoubleStorageKey(
      'VotingEngineSystem',
      'JointVotesByAdmin',
      _u64ToLeBytes(proposalId),
      compositeKey,
    );
    final data = await _rpc.fetchStorage('0x${_hexEncode(fullKey)}');
    if (data == null || data.isEmpty) return null;
    return data[0] == 1;
  }

  /// 查询公民投票计数（CitizenTallies）。
  ///
  /// Value: VoteCountU64 { yes: u64, no: u64 } = 16 bytes。
  Future<({int yes, int no})> fetchCitizenTally(int proposalId) async {
    final key = _buildStorageKey(
      'VotingEngineSystem',
      'CitizenTallies',
      _u64ToLeBytes(proposalId),
    );
    final data = await _rpc.fetchStorage('0x${_hexEncode(key)}');
    if (data == null || data.length < 16) return (yes: 0, no: 0);
    // VoteCountU64: { yes: u64, no: u64 } — 8+8 bytes little-endian
    final yes = _decodeU64(data.sublist(0, 8));
    final no = _decodeU64(data.sublist(8, 16));
    return (yes: yes, no: no);
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

  // ──── 内部：解码 ────

  /// 解码 RuntimeUpgradeAction SCALE 数据。
  RuntimeUpgradeProposalInfo? _decodeRuntimeUpgradeAction(
      int proposalId, Uint8List data) {
    try {
      var offset = 0;

      // proposer: AccountId32 (32 bytes)
      if (offset + 32 > data.length) return null;
      final proposerBytes = data.sublist(offset, offset + 32);
      offset += 32;

      // reason: Vec<u8> (Compact length + bytes)
      final (reasonLen, reasonLenSize) = _decodeCompact(data, offset);
      offset += reasonLenSize;
      if (offset + reasonLen > data.length) return null;
      final reasonBytes = data.sublist(offset, offset + reasonLen);
      final reason = utf8.decode(reasonBytes, allowMalformed: true);
      offset += reasonLen;

      // code_hash: [u8; 32]
      if (offset + 32 > data.length) return null;
      final codeHashBytes = data.sublist(offset, offset + 32);
      offset += 32;

      // status: u8 enum (0=Voting, 1=Passed, 2=Rejected, 3=ExecutionFailed)
      if (offset >= data.length) return null;
      final status = data[offset];

      final proposerSs58 =
          Keyring().encodeAddress(Uint8List.fromList(proposerBytes), 2027);
      final codeHashHex = _hexEncode(Uint8List.fromList(codeHashBytes));

      return RuntimeUpgradeProposalInfo(
        proposalId: proposalId,
        proposer: proposerSs58,
        reason: reason,
        codeHashHex: codeHashHex,
        status: status,
      );
    } catch (_) {
      return null;
    }
  }

  // ──── 内部：extrinsic 编码 ────

  /// 构造 propose_runtime_upgrade call data。
  ///
  /// 格式：[13][0][compact_len+reason_utf8][compact_len+wasm_bytes]
  ///       [u64_le:eligible_total][compact_len+nonce_bytes][compact_len+signature_bytes]
  Uint8List _buildProposeRuntimeUpgradeCall({
    required String reason,
    required Uint8List wasmCode,
    required int eligibleTotal,
    required Uint8List snapshotNonce,
    required Uint8List signature,
  }) {
    final output = ByteOutput();
    output.pushByte(_palletIndex);
    output.pushByte(_proposeCallIndex);

    // reason: Vec<u8> = Compact<u32> length + utf8 bytes
    final reasonBytes = utf8.encode(reason);
    output.write(
        CompactBigIntCodec.codec.encode(BigInt.from(reasonBytes.length)));
    if (reasonBytes.isNotEmpty) {
      output.write(Uint8List.fromList(reasonBytes));
    }

    // wasm_code: Vec<u8> = Compact<u32> length + bytes
    output.write(CompactBigIntCodec.codec.encode(BigInt.from(wasmCode.length)));
    if (wasmCode.isNotEmpty) {
      output.write(wasmCode);
    }

    // eligible_total: u64 little-endian
    output.write(_u64ToLeBytes(eligibleTotal));

    // snapshot_nonce: Vec<u8> = Compact<u32> length + bytes
    output.write(
        CompactBigIntCodec.codec.encode(BigInt.from(snapshotNonce.length)));
    if (snapshotNonce.isNotEmpty) {
      output.write(snapshotNonce);
    }

    // signature: Vec<u8> = Compact<u32> length + bytes
    output
        .write(CompactBigIntCodec.codec.encode(BigInt.from(signature.length)));
    if (signature.isNotEmpty) {
      output.write(signature);
    }

    return output.toBytes();
  }

  Uint8List _buildJointVoteCall({
    required int proposalId,
    required Uint8List institutionId48,
    required bool approve,
  }) {
    if (institutionId48.length != 48) {
      throw ArgumentError('institutionId48 必须为 48 字节');
    }

    final output = ByteOutput();
    output.pushByte(_jointVotePalletIndex);
    output.pushByte(_jointVoteCallIndex);
    output.write(_u64ToLeBytes(proposalId));
    output.write(institutionId48);
    output.pushByte(approve ? 1 : 0);

    return output.toBytes();
  }

  /// 签名并提交 extrinsic。
  ///
  /// 返回交易哈希和使用的 nonce（用于链上确认跟踪）。
  Future<({String txHash, int usedNonce})> _signAndSubmit({
    required Uint8List callData,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    debugPrint('[RuntimeUpgrade] 步骤1: 获取 metadata...');
    final metadata = await _rpc.fetchMetadata();
    debugPrint('[RuntimeUpgrade] 步骤2: 获取 genesisHash...');
    final genesisHash = await _rpc.fetchGenesisHash();
    final registry = metadata.chainInfo.scaleCodec.registry;

    debugPrint(
        '[RuntimeUpgrade] 步骤3: 并行获取 runtimeVersion/nonce/latestBlock...');
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
        '[RuntimeUpgrade] nonce=$nonce, block=${latestBlock.blockNumber}');

    debugPrint('[RuntimeUpgrade] 步骤4: 构造签名载荷...');
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

    debugPrint('[RuntimeUpgrade] 步骤5: 签名 (${payloadBytes.length} bytes)...');
    final signature = await sign(payloadBytes);
    debugPrint('[RuntimeUpgrade] 签名完成 (${signature.length} bytes)');

    debugPrint('[RuntimeUpgrade] 步骤6: 编码 extrinsic...');
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
    debugPrint('[RuntimeUpgrade] extrinsic 编码完成 (${encoded.length} bytes)');

    debugPrint('[RuntimeUpgrade] 步骤7: 提交到链...');
    debugPrint('[RuntimeUpgrade] call data hex: ${_hexEncode(callData)}');
    debugPrint(
        '[RuntimeUpgrade] encoded extrinsic hex: ${_hexEncode(encoded)}');
    try {
      final txHash = await _rpc.submitExtrinsic(encoded);
      debugPrint('[RuntimeUpgrade] 提交成功: 0x${_hexEncode(txHash)}');
      return (txHash: '0x${_hexEncode(txHash)}', usedNonce: nonce);
    } catch (e) {
      NonceManager.instance.rollback(fromAddress);
      debugPrint('[RuntimeUpgrade] 提交失败，原始错误: $e');
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

  /// 构造双 map storage key：twox128(pallet) + twox128(storage)
  /// + blake2_128_concat(key1) + blake2_128_concat(key2)。
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

    final result = Uint8List(palletHash.length +
        storageHash.length +
        key1Hash.length +
        key2Hash.length);
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

  static String _hexEncode(Uint8List bytes) {
    return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  }

  static List<int> _hexDecode(String hex) {
    final clean = hex.startsWith('0x') ? hex.substring(2) : hex;
    final out = <int>[];
    for (var i = 0; i + 1 < clean.length; i += 2) {
      out.add(int.parse(clean.substring(i, i + 2), radix: 16));
    }
    return out;
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

/// Runtime upgrade 提案链上数据。
class RuntimeUpgradeProposalInfo {
  const RuntimeUpgradeProposalInfo({
    required this.proposalId,
    required this.proposer,
    required this.reason,
    required this.codeHashHex,
    required this.status,
  });

  final int proposalId;
  final String proposer; // SS58 (ss58Format 2027)
  final String reason; // UTF-8 decoded
  final String codeHashHex; // 32-byte hash as hex
  final int status; // 0=Voting, 1=Passed, 2=Rejected, 3=ExecutionFailed
}

// ProposalMeta 来自 transfer_proposal_service.dart（复用同一定义）。
