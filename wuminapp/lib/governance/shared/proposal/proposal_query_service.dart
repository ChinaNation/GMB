import 'dart:typed_data';

import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:wuminapp_mobile/governance/shared/institution_info.dart';
import 'package:wuminapp_mobile/votingengine/internal-vote/internal_vote_query_service.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';

/// VotingEngine / InternalVote 通用查询服务。
///
/// 中文注释：提案状态、投票计数、快照和 NextProposalId 都是投票引擎
/// 通用状态，不能借用具体业务 service 暴露给其他模块。
class ProposalQueryService {
  ProposalQueryService({ChainRpc? chainRpc})
      : _rpc = chainRpc ?? ChainRpc(),
        _internalVoteQuery = InternalVoteQueryService(chainRpc: chainRpc);

  final ChainRpc _rpc;
  final InternalVoteQueryService _internalVoteQuery;

  /// 查询 NextProposalId（投票引擎全局递增 ID）。
  Future<int> fetchNextProposalId() async {
    final key = _buildStorageValueKey('VotingEngine', 'NextProposalId');
    final data = await _rpc.fetchStorage('0x${_hexEncode(key)}');
    if (data == null || data.length < 8) return 0;
    return _decodeU64(data);
  }

  /// 查询提案状态。返回 status（0=voting, 1=passed, 2=rejected），null 表示不存在。
  Future<int?> fetchProposalStatus(int proposalId) async {
    final key = _buildStorageKey(
      'VotingEngine',
      'Proposals',
      _u64ToLeBytes(proposalId),
    );
    final data = await _rpc.fetchStorage('0x${_hexEncode(key)}');
    if (data == null || data.length < 3) return null;
    return data[2];
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
    return (yes: _decodeU32(data, 0), no: _decodeU32(data, 4));
  }

  /// 查询内部投票阈值快照。
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
  Future<List<String>> fetchAdminSnapshot(
    int proposalId,
    String institutionIdentity,
  ) async {
    final key = _buildDoubleStorageKey(
      'VotingEngine',
      'AdminSnapshot',
      _u64ToLeBytes(proposalId),
      Uint8List.fromList(institutionIdentityToPalletId(institutionIdentity)),
    );
    final data = await _rpc.fetchStorage('0x${_hexEncode(key)}');
    if (data == null || data.isEmpty) return const [];
    final (count, lenSize) = _decodeCompact(data, 0);
    final admins = <String>[];
    var offset = lenSize;
    for (var i = 0; i < count && offset + 32 <= data.length; i++) {
      admins.add(
        _hexEncode(Uint8List.fromList(data.sublist(offset, offset + 32))),
      );
      offset += 32;
    }
    return admins;
  }

  /// 查询某管理员对某提案的投票记录。
  Future<bool?> fetchAdminVote(int proposalId, String pubkeyHex) {
    return _internalVoteQuery.fetchAdminVote(proposalId, pubkeyHex);
  }

  Uint8List _buildStorageValueKey(String palletName, String storageName) {
    final palletHash = Hasher.twoxx128.hashString(palletName);
    final storageHash = Hasher.twoxx128.hashString(storageName);
    final key = Uint8List(palletHash.length + storageHash.length);
    key.setAll(0, palletHash);
    key.setAll(palletHash.length, storageHash);
    return key;
  }

  Uint8List _buildStorageKey(
    String palletName,
    String storageName,
    Uint8List keyData,
  ) {
    final palletHash = Hasher.twoxx128.hashString(palletName);
    final storageHash = Hasher.twoxx128.hashString(storageName);
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

  Uint8List _buildDoubleStorageKey(
    String palletName,
    String storageName,
    Uint8List key1Data,
    Uint8List key2Data,
  ) {
    final palletHash = Hasher.twoxx128.hashString(palletName);
    final storageHash = Hasher.twoxx128.hashString(storageName);
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

  Uint8List _blake2128Concat(Uint8List data) {
    final hash = Hasher.blake2b128.hash(data);
    final result = Uint8List(hash.length + data.length);
    result.setAll(0, hash);
    result.setAll(hash.length, data);
    return result;
  }

  (int, int) _decodeCompact(Uint8List data, int offset) {
    final first = data[offset];
    final mode = first & 0x03;
    if (mode == 0) return (first >> 2, 1);
    if (mode == 1) {
      final val = (data[offset] | (data[offset + 1] << 8)) >> 2;
      return (val, 2);
    }
    if (mode == 2) {
      final val = (data[offset] |
              (data[offset + 1] << 8) |
              (data[offset + 2] << 16) |
              (data[offset + 3] << 24)) >>
          2;
      return (val, 4);
    }
    final lenBytes = (first >> 2) + 4;
    var val = 0;
    for (var i = lenBytes - 1; i >= 0; i--) {
      val = (val << 8) | data[offset + 1 + i];
    }
    return (val, 1 + lenBytes);
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
}
