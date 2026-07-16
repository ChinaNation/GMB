import 'dart:typed_data';

import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:citizenapp/rpc/chain_rpc.dart';

/// InternalVote 通用查询服务。
///
/// 管理员投票记录属于投票引擎通用状态，不放进具体业务模块，
/// 避免 proposal 共享层依赖业务 service。
class InternalVoteQueryService {
  InternalVoteQueryService({ChainRpc? chainRpc})
      : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;

  /// 查询某管理员对某提案的投票记录。null=未投票，true=赞成，false=反对。
  Future<bool?> fetchAdminVote(int proposalId, String pubkeyHex) async {
    final data = await _rpc.fetchStorage(_adminVoteKey(proposalId, pubkeyHex));
    return _decodeVote(data);
  }

  /// 批量查询管理员投票记录。
  ///
  /// 详情页和红点判断不能再按管理员逐条 RPC；这里统一拼好
  /// `InternalVotesByAccount` storage key 后分块读取。
  Future<Map<String, bool?>> fetchAdminVotesBatch(
    int proposalId,
    Iterable<String> pubkeysHex,
  ) async {
    final keyByPubkey = <String, String>{};
    for (final pubkey in pubkeysHex) {
      final clean = _normalizeHex(pubkey);
      if (clean.isEmpty) continue;
      keyByPubkey[clean] = _adminVoteKey(proposalId, clean);
    }
    if (keyByPubkey.isEmpty) return const {};

    final values = await _rpc.fetchStorageBatchChunked(keyByPubkey.values);
    return {
      for (final entry in keyByPubkey.entries)
        entry.key: _decodeVote(values[entry.value]),
    };
  }

  /// 跨提案批量查询内部投票:输入 `{proposalId: [pubkeyHex]}`,一次链查返回
  /// `{proposalId: {pubkey: vote?}}`。
  ///
  /// (ADR-018 R2):公民-提案列表原来每个提案各发一次批量 RPC(P 个提案
  /// = P 次往返),这里把所有 (proposalId, admin) 的 storage key 一次拼齐、单次
  /// 分块读取,P 次往返降为 1 次。
  Future<Map<int, Map<String, bool?>>> fetchAdminVotesForProposals(
    Map<int, List<String>> pubkeysByProposal,
  ) async {
    final keyToCoord = <String, ({int pid, String pk})>{};
    for (final entry in pubkeysByProposal.entries) {
      for (final pubkey in entry.value) {
        final clean = _normalizeHex(pubkey);
        if (clean.isEmpty) continue;
        keyToCoord[_adminVoteKey(entry.key, clean)] =
            (pid: entry.key, pk: clean);
      }
    }
    if (keyToCoord.isEmpty) return const {};
    final values = await _rpc.fetchStorageBatchChunked(keyToCoord.keys);
    final result = <int, Map<String, bool?>>{};
    keyToCoord.forEach((key, coord) {
      (result[coord.pid] ??= <String, bool?>{})[coord.pk] =
          _decodeVote(values[key]);
    });
    return result;
  }

  String _adminVoteKey(int proposalId, String pubkeyHex) {
    final proposalIdBytes = _u64ToLeBytes(proposalId);
    final accountBytes = _hexDecode(pubkeyHex);
    final palletHash = Hasher.twoxx128.hashString('InternalVote');
    final storageHash = Hasher.twoxx128.hashString('InternalVotesByAccount');
    final key1 = _blake2128Concat(proposalIdBytes);
    final key2 = _blake2128Concat(accountBytes);
    final fullKey = Uint8List(
      palletHash.length + storageHash.length + key1.length + key2.length,
    );
    var offset = 0;
    fullKey.setAll(offset, palletHash);
    offset += palletHash.length;
    fullKey.setAll(offset, storageHash);
    offset += storageHash.length;
    fullKey.setAll(offset, key1);
    offset += key1.length;
    fullKey.setAll(offset, key2);
    return '0x${_hexEncode(fullKey)}';
  }

  bool? _decodeVote(Uint8List? data) {
    if (data == null) return null;
    if (data.length != 1 || (data[0] != 0 && data[0] != 1)) {
      throw const FormatException(
        'InternalVotesByAccount 必须是严格的 SCALE bool',
      );
    }
    return data[0] == 1;
  }

  Uint8List _u64ToLeBytes(int value) {
    final bytes = Uint8List(8);
    final bd = ByteData.sublistView(bytes);
    bd.setUint64(0, value, Endian.little);
    return bytes;
  }

  Uint8List _blake2128Concat(Uint8List data) {
    final hash = Hasher.blake2b128.hash(data);
    final result = Uint8List(hash.length + data.length);
    result.setAll(0, hash);
    result.setAll(hash.length, data);
    return result;
  }

  Uint8List _hexDecode(String hex) {
    final h = _normalizeHex(hex);
    final result = Uint8List(h.length ~/ 2);
    for (var i = 0; i < result.length; i++) {
      result[i] = int.parse(h.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return result;
  }

  String _normalizeHex(String hex) {
    final h = hex.startsWith('0x') ? hex.substring(2) : hex;
    return h.toLowerCase();
  }

  static String _hexEncode(Uint8List bytes) {
    return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  }
}
