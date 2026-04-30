import 'dart:convert';
import 'dart:typed_data';

import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';
import 'package:wuminapp_mobile/rpc/sfid_public.dart';

/// 清算行节点链上声明。
///
/// 中文注释:
/// - 该结构来自链上 `OffchainTransaction::ClearingBankNodes[sfid_id]`。
/// - SFID 只提供机构资料;节点端点必须以链上 storage 为准。
class ClearingBankNodeEndpoint {
  const ClearingBankNodeEndpoint({
    required this.sfidId,
    required this.peerId,
    required this.rpcDomain,
    required this.rpcPort,
    required this.registeredAt,
    required this.registeredBy,
  });

  final String sfidId;
  final String peerId;
  final String rpcDomain;
  final int rpcPort;
  final int registeredAt;
  final String registeredBy;

  String get wssUrl {
    final isLocal = rpcDomain == '127.0.0.1' || rpcDomain == 'localhost';
    final scheme = isLocal ? 'ws' : 'wss';
    return '$scheme://$rpcDomain:$rpcPort';
  }
}

/// SFID 清算行资料 + 链上节点端点的组合结果。
class ClearingBankCandidate {
  const ClearingBankCandidate({
    required this.info,
    required this.endpoint,
  });

  final ClearingBankInfo info;
  final ClearingBankNodeEndpoint? endpoint;

  bool get canBind =>
      endpoint != null &&
      info.mainAccount != null &&
      info.mainAccount!.isNotEmpty;
}

/// wuminapp 清算行目录服务。
///
/// 中文注释:
/// - 搜索入口使用 SFID 公开 API。
/// - 端点入口读取链上 `ClearingBankNodes`,避免继续依赖固定启动参数。
/// - 用户绑定状态读取链上 `UserBank[user]`,本地缓存只做 UI 快照。
class ClearingBankDirectory {
  ClearingBankDirectory({
    required this.sfidBaseUrl,
    ChainRpc? chainRpc,
  }) : _chainRpc = chainRpc ?? ChainRpc();

  final String sfidBaseUrl;
  final ChainRpc _chainRpc;

  Future<List<ClearingBankCandidate>> search(String query) async {
    final api = SfidPublicApi(baseUrl: sfidBaseUrl);
    try {
      final result = await api.searchClearingBanks(keyword: query, size: 20);
      final out = <ClearingBankCandidate>[];
      for (final item in result.items) {
        final endpoint = await fetchEndpoint(item.sfidId);
        out.add(ClearingBankCandidate(info: item, endpoint: endpoint));
      }
      return out;
    } finally {
      api.close();
    }
  }

  Future<ClearingBankNodeEndpoint?> fetchEndpoint(String sfidId) async {
    final key = _clearingBankNodesKey(sfidId);
    final raw = await _chainRpc.fetchStorage(key);
    if (raw == null || raw.isEmpty) return null;
    return _decodeEndpoint(sfidId, raw);
  }

  /// 查询链上 `UserBank[user]`,返回用户当前绑定清算行主账户 SS58。
  Future<String?> fetchUserBank(String userAddress) async {
    final account = Uint8List.fromList(Keyring().decodeAddress(userAddress));
    final key = _userBankKey(account);
    final raw = await _chainRpc.fetchStorage(key);
    if (raw == null || raw.length < 32) return null;
    return Keyring().encodeAddress(raw.sublist(0, 32).toList(), 2027);
  }

  static ClearingBankNodeEndpoint? _decodeEndpoint(
    String sfidId,
    Uint8List raw,
  ) {
    var offset = 0;
    final (peerId, peerNext) = _readUtf8Vec(raw, offset);
    if (peerId == null) return null;
    offset = peerNext;
    final (domain, domainNext) = _readUtf8Vec(raw, offset);
    if (domain == null) return null;
    offset = domainNext;
    if (offset + 2 + 4 + 32 > raw.length) return null;
    final port = raw[offset] | (raw[offset + 1] << 8);
    offset += 2;
    final registeredAt = _readU32Le(raw, offset);
    offset += 4;
    final registeredBy = Keyring().encodeAddress(
      raw.sublist(offset, offset + 32).toList(),
      2027,
    );
    return ClearingBankNodeEndpoint(
      sfidId: sfidId,
      peerId: peerId,
      rpcDomain: domain,
      rpcPort: port,
      registeredAt: registeredAt,
      registeredBy: registeredBy,
    );
  }

  static String _clearingBankNodesKey(String sfidId) {
    final keyData = _encodeBytes(utf8.encode(sfidId));
    return _mapKey('OffchainTransaction', 'ClearingBankNodes', keyData);
  }

  static String _userBankKey(Uint8List accountId) {
    return _mapKey('OffchainTransaction', 'UserBank', accountId);
  }

  static String _mapKey(String pallet, String storage, Uint8List keyData) {
    final bytes = BytesBuilder()
      ..add(Hasher.twoxx128.hashString(pallet))
      ..add(Hasher.twoxx128.hashString(storage))
      ..add(Hasher.blake2b128.hash(keyData))
      ..add(keyData);
    return '0x${_hex(bytes.toBytes())}';
  }

  static Uint8List _encodeBytes(List<int> raw) {
    return Uint8List.fromList([..._compactU32(raw.length), ...raw]);
  }

  static List<int> _compactU32(int value) {
    if (value < 1 << 6) return [value << 2];
    if (value < 1 << 14) {
      final v = (value << 2) | 0x01;
      return [v & 0xff, (v >> 8) & 0xff];
    }
    final v = (value << 2) | 0x02;
    return [v & 0xff, (v >> 8) & 0xff, (v >> 16) & 0xff, (v >> 24) & 0xff];
  }

  static (String?, int) _readUtf8Vec(Uint8List bytes, int offset) {
    final (len, lenSize) = _decodeCompactU32(bytes, offset);
    if (lenSize == 0) return (null, offset);
    offset += lenSize;
    if (offset + len > bytes.length) return (null, offset);
    return (
      utf8.decode(bytes.sublist(offset, offset + len), allowMalformed: true),
      offset + len,
    );
  }

  static (int, int) _decodeCompactU32(Uint8List bytes, int offset) {
    if (offset >= bytes.length) return (0, 0);
    final mode = bytes[offset] & 0x03;
    if (mode == 0) return (bytes[offset] >> 2, 1);
    if (mode == 1) {
      if (offset + 2 > bytes.length) return (0, 0);
      return ((((bytes[offset + 1] << 8) | bytes[offset]) >> 2), 2);
    }
    if (mode == 2) {
      if (offset + 4 > bytes.length) return (0, 0);
      final raw = bytes[offset] |
          (bytes[offset + 1] << 8) |
          (bytes[offset + 2] << 16) |
          (bytes[offset + 3] << 24);
      return (raw >> 2, 4);
    }
    return (0, 0);
  }

  static int _readU32Le(Uint8List bytes, int offset) {
    return bytes[offset] |
        (bytes[offset + 1] << 8) |
        (bytes[offset + 2] << 16) |
        (bytes[offset + 3] << 24);
  }

  static String _hex(List<int> bytes) {
    return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  }
}
