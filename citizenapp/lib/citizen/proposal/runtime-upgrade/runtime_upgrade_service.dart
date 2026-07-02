import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:polkadart/scale_codec.dart' show ByteOutput;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/rpc/signed_extrinsic_builder.dart';
import 'package:citizenapp/citizen/shared/proposal/proposal_models.dart';

/// 协议升级提案链上交互服务。
///
/// 负责协议升级提案详情查询，并保留现有详情页投票提交能力。
class RuntimeUpgradeService {
  RuntimeUpgradeService({ChainRpc? chainRpc}) : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;

  // ──── 常量 ────

  /// JointVote sub-pallet pallet_index=23。
  static const _jointVotePalletIndex = 23;

  /// JointVote::cast_admin call_index=0。
  static const _jointVoteCallIndex = 0;

  /// runtime-upgrade 写入 VotingEngine::ProposalData 的业务前缀。
  static const _moduleTag = [0x72, 0x74, 0x2d, 0x75, 0x70, 0x67]; // rt-upg

  // ──── Extrinsic 提交 ────

  /// 提交机构管理员的联合投票。
  ///
  /// 联合投票必须等待入块，并回读 runtime JointVote storage。
  /// txHash 只代表交易提交，不能代表投票已经生效。
  Future<({String txHash, int usedNonce, String blockHashHex})>
      submitJointVote({
    required int proposalId,
    required Uint8List institutionAccountId,
    required bool approve,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    final callData = _buildJointVoteCall(
      proposalId: proposalId,
      institutionAccountId: institutionAccountId,
      approve: approve,
    );
    final result = await _signAndSubmit(
      callData: callData,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
    );
    await _confirmRuntimeJointVote(
      proposalId: proposalId,
      institutionAccountId: institutionAccountId,
      approve: approve,
      signerPubkey: signerPubkey,
      blockHashHex: result.blockHashHex,
    );
    return result;
  }

  // ──── 链上查询 ────

  /// 查询协议升级提案详情。返回 null 表示不存在。
  ///
  /// ProposalData 是 BoundedVec<u8>，SCALE 编码为 Compact 长度前缀 + 原始字节。
  /// 原始字节布局：
  ///   proposer: AccountId32(32) + reason: Vec<u8>(Compact len + bytes)
  ///   + code_hash: [u8;32]。真实状态只读取 VotingEngine::Proposals.status。
  Future<RuntimeUpgradeProposalInfo?> fetchRuntimeUpgradeProposal(
      int proposalId) async {
    final key = _buildStorageKey(
      'VotingEngine',
      'ProposalData',
      _u64ToLeBytes(proposalId),
    );
    final raw = await _rpc.fetchStorage('0x${_hexEncode(key)}');
    if (raw == null || raw.isEmpty) return null;
    return decodeRuntimeUpgradeStorageValue(proposalId, raw);
  }

  /// 解码 `ProposalData` 的原始 storage value（带 Compact 长度前缀）。
  ///
  /// 分页列表会批量读取 ProposalData，随后在内存里按提案类型解码；
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
      'JointVote',
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
  /// 双 map：blake2_128_concat(u64_le) + blake2_128_concat(机构 AccountId32)。
  /// Value: Option<bool> — null=未投票，true=赞成，false=反对。
  Future<bool?> fetchJointVoteByInstitution(
      int proposalId, Uint8List institutionAccountId) async {
    final fullKey = _buildDoubleStorageKey(
      'JointVote',
      'JointVotesByInstitution',
      _u64ToLeBytes(proposalId),
      institutionAccountId,
    );
    final data = await _rpc.fetchStorage('0x${_hexEncode(fullKey)}');
    if (data == null || data.isEmpty) return null;
    return data[0] == 1;
  }

  /// 查询某机构在联合投票阶段的管理员票数统计。
  Future<({int yes, int no})> fetchJointInstitutionTally(
      int proposalId, Uint8List institutionAccountId) async {
    final fullKey = _buildDoubleStorageKey(
      'JointVote',
      'JointInstitutionTallies',
      _u64ToLeBytes(proposalId),
      institutionAccountId,
    );
    final data = await _rpc.fetchStorage('0x${_hexEncode(fullKey)}');
    if (data == null || data.length < 8) return (yes: 0, no: 0);
    return (yes: _decodeU32(data, 0), no: _decodeU32(data, 4));
  }

  /// 查询某管理员在某机构联合投票中的投票记录。
  Future<bool?> fetchJointAdminVote(
    int proposalId,
    Uint8List institutionAccountId,
    String pubkeyHex,
  ) async {
    final key = _jointAdminVoteKey(proposalId, institutionAccountId, pubkeyHex);
    if (key == null) return null;
    final data = await _rpc.fetchStorage(key);
    if (data == null || data.isEmpty) return null;
    return data[0] == 1;
  }

  /// 批量查询联合投票管理员投票记录。
  ///
  /// 协议升级详情页最多会显示 43 个管理员，必须批量读取
  /// `JointVotesByAdmin`，不能逐管理员发起 RPC。
  Future<Map<String, bool?>> fetchJointAdminVotesBatch(
    int proposalId,
    Uint8List institutionAccountId,
    Iterable<String> pubkeysHex,
  ) async {
    final keyByPubkey = <String, String>{};
    for (final pubkey in pubkeysHex) {
      final clean = _normalizeHex(pubkey);
      if (clean.isEmpty) continue;
      final key = _jointAdminVoteKey(proposalId, institutionAccountId, clean);
      if (key == null) continue;
      keyByPubkey[clean] = key;
    }
    if (keyByPubkey.isEmpty) return const {};
    final values = await _rpc.fetchStorageBatchChunked(keyByPubkey.values);
    return {
      for (final entry in keyByPubkey.entries)
        entry.key: _decodeBoolVote(values[entry.value]),
    };
  }

  /// 跨提案批量查询联合投票:输入 `{proposalId: (机构account, [pubkeyHex])}`,
  /// 一次链查返回 `{proposalId: {pubkey: vote?}}`。
  ///
  /// (ADR-018 R2):与内部投票同理,广场上多个联合投票提案合并成单次
  /// 分块读取,避免每提案一次 RPC。
  Future<Map<int, Map<String, bool?>>> fetchJointAdminVotesForProposals(
    Map<int, ({Uint8List institutionAccountId, List<String> pubkeysHex})>
        byProposal,
  ) async {
    final keyToCoord = <String, ({int pid, String pk})>{};
    for (final entry in byProposal.entries) {
      for (final pubkey in entry.value.pubkeysHex) {
        final clean = _normalizeHex(pubkey);
        if (clean.isEmpty) continue;
        final key = _jointAdminVoteKey(
          entry.key,
          entry.value.institutionAccountId,
          clean,
        );
        if (key == null) continue;
        keyToCoord[key] = (pid: entry.key, pk: clean);
      }
    }
    if (keyToCoord.isEmpty) return const {};
    final values = await _rpc.fetchStorageBatchChunked(keyToCoord.keys);
    final result = <int, Map<String, bool?>>{};
    keyToCoord.forEach((key, coord) {
      (result[coord.pid] ??= <String, bool?>{})[coord.pk] =
          _decodeBoolVote(values[key]);
    });
    return result;
  }

  String? _jointAdminVoteKey(
    int proposalId,
    Uint8List institutionAccountId,
    String pubkeyHex,
  ) {
    final accountBytes = Uint8List.fromList(_hexDecode(pubkeyHex));
    if (institutionAccountId.length != 32 || accountBytes.length != 32) {
      return null;
    }
    final compositeKey =
        Uint8List(institutionAccountId.length + accountBytes.length)
          ..setAll(0, institutionAccountId)
          ..setAll(institutionAccountId.length, accountBytes);
    final fullKey = _buildDoubleStorageKey(
      'JointVote',
      'JointVotesByAdmin',
      _u64ToLeBytes(proposalId),
      compositeKey,
    );
    return '0x${_hexEncode(fullKey)}';
  }

  bool? _decodeBoolVote(Uint8List? data) {
    if (data == null || data.isEmpty) return null;
    return data[0] == 1;
  }

  /// 查询公民投票计数（CitizenTallies）。
  ///
  /// Value: VoteCountU64 { yes: u64, no: u64 } = 16 bytes。
  Future<({int yes, int no})> fetchReferendumTally(int proposalId) async {
    final key = _buildStorageKey(
      'JointVote',
      'ReferendumTallies',
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
      'VotingEngine',
      'Proposals',
      _u64ToLeBytes(proposalId),
    );
    final data = await _rpc.fetchStorage('0x${_hexEncode(key)}');
    if (data == null || data.length < 3) return null;

    final kind = data[0];
    final stage = data[1];
    final status = data[2];

    // internal_code: Option<[u8;4]>
    var offset = 3;
    String? internalCode;
    if (offset < data.length && data[offset] == 1) {
      offset++;
      if (offset + 4 <= data.length) {
        internalCode =
            _institutionCodeToString(data.sublist(offset, offset + 4));
        offset += 4;
      }
    } else {
      offset++; // skip 0x00 (None)
    }

    // account_context: Option<AccountId32>
    Uint8List? institutionBytes;
    if (offset < data.length && data[offset] == 1) {
      offset++;
      if (offset + 32 <= data.length) {
        institutionBytes =
            Uint8List.fromList(data.sublist(offset, offset + 32));
        offset += 32;
      }
    }
    final (subjectCidNumbers, _) = _decodeSubjectCidNumbers(data, offset);

    return ProposalMeta(
      proposalId: proposalId,
      kind: kind,
      stage: stage,
      status: status,
      internalCode: internalCode,
      institutionBytes: institutionBytes,
      subjectCidNumbers: subjectCidNumbers,
    );
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

  // ──── 内部：解码 ────

  /// 解码协议升级提案摘要 SCALE 数据。
  RuntimeUpgradeProposalInfo? _decodeRuntimeUpgradeAction(
      int proposalId, Uint8List data) {
    try {
      if (!_startsWith(data, _moduleTag)) return null;
      var offset = _moduleTag.length;

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

      // 协议升级摘要不保存业务状态，避免与投票引擎终态脱节。
      if (offset != data.length) return null;

      final proposerSs58 =
          Keyring().encodeAddress(Uint8List.fromList(proposerBytes), 2027);
      final codeHashHex = _hexEncode(Uint8List.fromList(codeHashBytes));

      return RuntimeUpgradeProposalInfo(
        proposalId: proposalId,
        proposer: proposerSs58,
        reason: reason,
        codeHashHex: codeHashHex,
      );
    } catch (_) {
      return null;
    }
  }

  bool _startsWith(Uint8List data, List<int> prefix) {
    if (data.length < prefix.length) return false;
    for (var i = 0; i < prefix.length; i++) {
      if (data[i] != prefix[i]) return false;
    }
    return true;
  }

  // ──── 内部：extrinsic 编码 ────

  Uint8List _buildJointVoteCall({
    required int proposalId,
    required Uint8List institutionAccountId,
    required bool approve,
  }) {
    if (institutionAccountId.length != 32) {
      throw ArgumentError('institutionAccountId 必须为 32 字节');
    }

    final output = ByteOutput();
    output.pushByte(_jointVotePalletIndex);
    output.pushByte(_jointVoteCallIndex);
    output.write(_u64ToLeBytes(proposalId));
    output.write(institutionAccountId);
    output.pushByte(approve ? 1 : 0);

    return output.toBytes();
  }

  /// 签名、提交并等待交易进入区块。
  ///
  /// 返回交易哈希、runtime nonce 和入块哈希。
  Future<({String txHash, int usedNonce, String blockHashHex})> _signAndSubmit({
    required Uint8List callData,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    return SignedExtrinsicBuilder(
      chainRpc: _rpc,
      logLabel: 'ProtocolUpgrade',
    ).signAndSubmitInBlock(
      callData: callData,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
      onTrace: (trace) {
        debugPrint(
            '[ProtocolUpgrade] encoded extrinsic hex: ${_hexEncode(trace.encoded)}');
      },
    );
  }

  /// 入块后回读 JointVote storage，确认该管理员投票已经由 runtime 记录。
  Future<void> _confirmRuntimeJointVote({
    required int proposalId,
    required Uint8List institutionAccountId,
    required bool approve,
    required Uint8List signerPubkey,
    required String blockHashHex,
  }) async {
    final pubkeyHex = _hexEncode(signerPubkey);
    for (var attempt = 0; attempt < 6; attempt++) {
      final chainVote = await fetchJointAdminVote(
        proposalId,
        institutionAccountId,
        pubkeyHex,
      );
      if (chainVote == approve) return;
      if (chainVote != null && chainVote != approve) {
        throw StateError('runtime 联合投票记录与本次投票方向不一致');
      }
      if (attempt < 5) {
        await Future<void>.delayed(const Duration(milliseconds: 500));
      }
    }

    final events = await _rpc.fetchSystemEventsAtBlock(blockHashHex);
    final failure =
        events == null ? null : _rpc.findExtrinsicFailureInEvents(events);
    if (failure != null) {
      throw StateError('runtime 拒绝联合投票：${failure.description}');
    }
    throw StateError('交易已入块，但 runtime JointVote 未记录该管理员投票');
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

  static Uint8List _u64ToLeBytes(int value) {
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

  String _institutionCodeToString(List<int> bytes) {
    return String.fromCharCodes(bytes.where((b) => b != 0)).toUpperCase();
  }

  (List<String>, int) _decodeSubjectCidNumbers(Uint8List data, int offset) {
    if (offset >= data.length) return (const [], offset);
    final (count, lenSize) = _decodeCompact(data, offset);
    var cursor = offset + lenSize;
    final result = <String>[];
    for (var i = 0; i < count && cursor < data.length; i++) {
      final (cidLen, cidLenSize) = _decodeCompact(data, cursor);
      cursor += cidLenSize;
      if (cursor + cidLen > data.length) {
        return (List.unmodifiable(result), cursor);
      }
      result.add(
        utf8.decode(data.sublist(cursor, cursor + cidLen),
            allowMalformed: true),
      );
      cursor += cidLen;
    }
    return (List.unmodifiable(result), cursor);
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
    final clean = _normalizeHex(hex);
    final out = <int>[];
    for (var i = 0; i + 1 < clean.length; i += 2) {
      out.add(int.parse(clean.substring(i, i + 2), radix: 16));
    }
    return out;
  }

  static String _normalizeHex(String hex) {
    final clean = hex.startsWith('0x') ? hex.substring(2) : hex;
    return clean.toLowerCase();
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
