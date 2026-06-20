import 'dart:typed_data';

import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:wuminapp_mobile/governance/shared/institution_info.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';

/// 提案活跃数量限制查询。
///
/// 中文注释：活跃提案上限属于 VotingEngine 的通用规则，因此放在
/// proposal 共享层，避免提案入口依赖具体业务模块。
class ProposalLimitService {
  ProposalLimitService({ChainRpc? chainRpc}) : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;

  /// 每个机构最多同时 10 个活跃提案（全局，不区分提案类型）。
  static const maxActiveProposalsPerInstitution = 10;

  /// 查询机构活跃的提案 ID 列表（从 VotingEngine 全局存储读取）。
  Future<List<int>> fetchActiveProposalIds(InstitutionInfo institution) async {
    final key = _buildStorageKey(
      'VotingEngine',
      'ActiveProposalsByInstitution',
      Uint8List.fromList(
        institutionIdentityToAccountId(
          institution.sfidNumber,
          mainAccount: institution.mainAccount,
        ),
      ),
    );
    final data = await _rpc.fetchStorage('0x${_hexEncode(key)}');
    if (data == null || data.isEmpty) return const [];
    final (count, lenSize) = _decodeCompact(data, 0);
    final ids = <int>[];
    var offset = lenSize;
    for (var i = 0; i < count && offset + 8 <= data.length; i++) {
      ids.add(_decodeU64(data.sublist(offset, offset + 8)));
      offset += 8;
    }
    return ids;
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
    }
    final lenBytes = (first >> 2) + 4;
    var val = 0;
    for (var i = lenBytes - 1; i >= 0; i--) {
      val = (val << 8) | data[offset + 1 + i];
    }
    return (val, 1 + lenBytes);
  }

  int _decodeU64(Uint8List data) {
    final bd = ByteData.sublistView(data);
    return bd.getUint64(0, Endian.little);
  }

  static String _hexEncode(Uint8List bytes) {
    return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  }
}
