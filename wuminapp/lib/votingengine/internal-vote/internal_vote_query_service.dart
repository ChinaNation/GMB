import 'dart:typed_data';

import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';

/// InternalVote 通用查询服务。
///
/// 中文注释：管理员投票记录属于投票引擎通用状态，不放进具体业务模块，
/// 避免 proposal 共享层依赖业务 service。
class InternalVoteQueryService {
  InternalVoteQueryService({ChainRpc? chainRpc})
      : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;

  /// 查询某管理员对某提案的投票记录。null=未投票，true=赞成，false=反对。
  Future<bool?> fetchAdminVote(int proposalId, String pubkeyHex) async {
    final data = await _rpc.fetchStorage(_adminVoteKey(proposalId, pubkeyHex));
    if (data == null || data.isEmpty) return null;
    return data[0] == 1;
  }

  /// 批量查询管理员投票记录。
  ///
  /// 中文注释：详情页和红点判断不能再按管理员逐条 RPC；这里统一拼好
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
    if (data == null || data.isEmpty) return null;
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
