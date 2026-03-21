import 'dart:async';
import 'dart:math';

import 'package:flutter/foundation.dart';
import 'package:polkadart/polkadart.dart'
    show HttpProvider, Hasher, RuntimeMetadata, RuntimeVersion;

class ChainRpc {
  ChainRpc({String? rpcUrl}) {
    if (rpcUrl != null && rpcUrl.isNotEmpty) {
      _nodes = [rpcUrl];
    } else {
      const fromDefine = String.fromEnvironment('WUMINAPP_RPC_URL');
      if (fromDefine.isNotEmpty) {
        _nodes = [fromDefine];
      } else {
        // 本地节点放最前，其余节点随机打乱
        final remote = List.of(_defaultNodes.skip(1))..shuffle(Random());
        _nodes = [_defaultNodes.first, ...remote];
      }
    }
  }

  late final List<String> _nodes;
  HttpProvider? _provider;
  int _currentIndex = 0;

  static const _defaultNodes = [
    'http://127.0.0.1:9944',
    'http://nrcgch.wuminapp.com:9944',
    'http://prczss.wuminapp.com:9944',
    'http://prclns.wuminapp.com:9944',
    'http://prcgds.wuminapp.com:9944',
    'http://prcgxs.wuminapp.com:9944',
    'http://prcfjs.wuminapp.com:9944',
    'http://prchns.wuminapp.com:9944',
    'http://prcyns.wuminapp.com:9944',
    'http://prcgzs.wuminapp.com:9944',
    'http://prchus.wuminapp.com:9944',
    'http://prcjxs.wuminapp.com:9944',
    'http://prczjs.wuminapp.com:9944',
    'http://prcjss.wuminapp.com:9944',
    'http://prcsds.wuminapp.com:9944',
    'http://prcsxs.wuminapp.com:9944',
    'http://prches.wuminapp.com:9944',
    'http://prchbs.wuminapp.com:9944',
    'http://prchis.wuminapp.com:9944',
    'http://prcsis.wuminapp.com:9944',
    'http://prccqs.wuminapp.com:9944',
    'http://prcscs.wuminapp.com:9944',
    'http://prcgss.wuminapp.com:9944',
    'http://prcbps.wuminapp.com:9944',
    'http://prchas.wuminapp.com:9944',
    'http://prcsjs.wuminapp.com:9944',
    'http://prcljs.wuminapp.com:9944',
    'http://prcjls.wuminapp.com:9944',
    'http://prclis.wuminapp.com:9944',
    'http://prcnxs.wuminapp.com:9944',
    'http://prcqhs.wuminapp.com:9944',
    'http://prcahs.wuminapp.com:9944',
    'http://prctws.wuminapp.com:9944',
    'http://prcxzs.wuminapp.com:9944',
    'http://prcxjs.wuminapp.com:9944',
    'http://prcxks.wuminapp.com:9944',
    'http://prcals.wuminapp.com:9944',
    'http://prccls.wuminapp.com:9944',
    'http://prctss.wuminapp.com:9944',
    'http://prchxs.wuminapp.com:9944',
    'http://prckls.wuminapp.com:9944',
    'http://prchts.wuminapp.com:9944',
    'http://prcrhs.wuminapp.com:9944',
    'http://prcxas.wuminapp.com:9944',
    'http://prchjs.wuminapp.com:9944',
  ];

  // twox_128("System") + twox_128("Account")
  static final Uint8List _systemAccountPrefix = _hexDecode(
    '26aa394eea5630e07c48ae0c9558cef7'
    'b99d880ec681799c0cf30e8886371da9',
  );
  static final Uint8List _sfidMainAccountKey =
      _buildStorageValueKey('SfidCodeAuth', 'SfidMainAccount');

  /// 当前活跃节点的 HTTP URL（可用于推导 WebSocket URL）。
  String get currentNodeUrl => _nodes[_currentIndex];

  // ──── 批量查询 ────

  /// 批量查询多个 storage key，一次 RPC 调用返回所有结果。
  /// 使用 state_queryStorageAt([keys]) RPC 方法。
  Future<Map<String, Uint8List?>> fetchStorageBatch(
      List<String> storageKeyHexList) async {
    if (storageKeyHexList.isEmpty) return {};
    final result = await _rpcCall('state_queryStorageAt', [storageKeyHexList]);
    // result 格式: [{ block: "0x...", changes: [["0xkey1", "0xvalue1"], ...] }]
    final map = <String, Uint8List?>{};
    // 先初始化所有 key 为 null（未返回的 key 表示不存在）
    for (final k in storageKeyHexList) {
      map[k] = null;
    }
    if (result is List && result.isNotEmpty) {
      final entry = result[0] as Map<String, dynamic>;
      final changes = entry['changes'] as List<dynamic>? ?? [];
      for (final change in changes) {
        final pair = change as List<dynamic>;
        final key = pair[0] as String;
        final value = pair.length > 1 ? pair[1] : null;
        if (value != null && value is String && value.length > 2) {
          map[key] = _hexDecode((value).substring(2));
        }
      }
    }
    return map;
  }

  // ──── 转账相关 RPC ────

  /// 查询账户下一个可用 nonce（含交易池中的 pending 交易）。
  Future<int> fetchNonce(String ss58Address) async {
    final result = await _rpcCall('system_accountNextIndex', [ss58Address]);
    return result as int;
  }

  /// 获取运行时版本（specVersion、transactionVersion）。
  Future<RuntimeVersion> fetchRuntimeVersion() async {
    final result = await _rpcCall('state_getRuntimeVersion', []);
    return RuntimeVersion.fromJson(result as Map<String, dynamic>);
  }

  /// 获取创世块哈希（32 字节）。结果缓存，同一实例只查一次。
  Future<Uint8List> fetchGenesisHash() async {
    if (_cachedGenesisHash != null) return _cachedGenesisHash!;
    final result = await _rpcCall('chain_getBlockHash', [0]);
    _cachedGenesisHash = _hexDecode((result as String).substring(2));
    return _cachedGenesisHash!;
  }

  Uint8List? _cachedGenesisHash;

  /// 获取最新区块的哈希和块号（用于 mortal era 计算）。
  Future<({Uint8List blockHash, int blockNumber})> fetchLatestBlock() async {
    final hashHex = await _rpcCall('chain_getBlockHash', []) as String;
    final blockHash = _hexDecode(hashHex.substring(2));
    final header =
        await _rpcCall('chain_getHeader', [hashHex]) as Map<String, dynamic>;
    final blockNumber = int.parse(header['number'] as String);
    return (blockHash: blockHash, blockNumber: blockNumber);
  }

  /// 获取运行时 metadata（含 registry，用于 extrinsic 编码）。结果缓存。
  Future<RuntimeMetadata> fetchMetadata() async {
    if (_cachedMetadata != null) return _cachedMetadata!;
    final result = await _rpcCall('state_getMetadata', []) as String;
    _cachedMetadata = RuntimeMetadata.fromHex(result);
    return _cachedMetadata!;
  }

  RuntimeMetadata? _cachedMetadata;
  String? _cachedCurrentSfidMainPubkeyHex;

  /// 提交已签名的 extrinsic，返回交易哈希（32 字节）。
  Future<Uint8List> submitExtrinsic(Uint8List encoded) async {
    final hex = '0x${_hexEncode(encoded)}';
    final result = await _rpcCall('author_submitExtrinsic', [hex]);
    return _hexDecode((result as String).substring(2));
  }

  // ──── 链上状态查询 ────

  /// 通用 storage 查询：传入完整的 storage key（含 0x 前缀），
  /// 返回原始 SCALE 编码字节。key 不存在时返回 null。
  Future<Uint8List?> fetchStorage(String storageKeyHex) async {
    final result = await _rpcCall('state_getStorage', [storageKeyHex]);
    if (result == null) return null;
    return _hexDecode((result as String).substring(2));
  }

  /// 查询链上已打包的 nonce（不含交易池），账户不存在返回 0。
  Future<int> fetchConfirmedNonce(String pubkeyHex) async {
    final accountId = _pubkeyHexToBytes(pubkeyHex);
    final storageKey = _buildSystemAccountKey(accountId);
    final keyHex = '0x${_hexEncode(storageKey)}';

    final result = await _rpcCall('state_getStorage', [keyHex]);
    if (result == null) return 0;

    final bytes = _hexDecode((result as String).substring(2));
    if (bytes.length < 4) return 0;
    // AccountInfo[0..3] = nonce u32 LE
    return bytes[0] | (bytes[1] << 8) | (bytes[2] << 16) | (bytes[3] << 24);
  }

  /// 查询链上余额，返回元（yuan）。账户不存在返回 0.0。
  Future<double> fetchBalance(String pubkeyHex) async {
    final accountId = _pubkeyHexToBytes(pubkeyHex);
    final storageKey = _buildSystemAccountKey(accountId);
    final keyHex = '0x${_hexEncode(storageKey)}';

    final result = await _rpcCall('state_getStorage', [keyHex]);
    if (result == null) {
      return 0.0;
    }

    final bytes = _hexDecode((result as String).substring(2));
    return _decodeFreeBalance(bytes);
  }

  /// 读取链上当前 SFID 主验签公钥（32 字节 AccountId）。
  ///
  /// 存储项：`SfidCodeAuth::SfidMainAccount`，类型为 `Option<AccountId32>`。
  Future<String?> fetchCurrentSfidMainPubkeyHex() async {
    final cached = _cachedCurrentSfidMainPubkeyHex;
    if (cached != null && cached.isNotEmpty) {
      return cached;
    }

    final keyHex = '0x${_hexEncode(_sfidMainAccountKey)}';
    final result = await _rpcCall('state_getStorage', [keyHex]);
    if (result == null) {
      return null;
    }

    final data = _hexDecode((result as String).substring(2));
    if (data.isEmpty) {
      return null;
    }

    Uint8List pubkeyBytes;
    if (data.length == 33 && data.first == 0x01) {
      pubkeyBytes = Uint8List.sublistView(data, 1, 33);
    } else if (data.length == 32) {
      pubkeyBytes = data;
    } else {
      throw Exception('SfidMainAccount 存储格式异常');
    }

    final pubkeyHex = '0x${_hexEncode(pubkeyBytes)}';
    _cachedCurrentSfidMainPubkeyHex = pubkeyHex;
    return pubkeyHex;
  }

  /// 构造 System.Account(accountId) 的 storage key。
  Uint8List _buildSystemAccountKey(Uint8List accountId) {
    // blake2_128_concat = blake2b_128(data) + data
    final blake2Hash = Hasher.blake2b128.hash(accountId);

    final key = Uint8List(
      _systemAccountPrefix.length + blake2Hash.length + accountId.length,
    );
    var offset = 0;
    key.setAll(offset, _systemAccountPrefix);
    offset += _systemAccountPrefix.length;
    key.setAll(offset, blake2Hash);
    offset += blake2Hash.length;
    key.setAll(offset, accountId);
    return key;
  }

  static Uint8List _buildStorageValueKey(
      String palletName, String storageName) {
    final palletHash = Hasher.twoxx128.hashString(palletName);
    final storageHash = Hasher.twoxx128.hashString(storageName);
    final key = Uint8List(palletHash.length + storageHash.length);
    key.setAll(0, palletHash);
    key.setAll(palletHash.length, storageHash);
    return key;
  }

  /// 从 SCALE 编码的 AccountInfo 中提取 free 余额，转换为元。
  double _decodeFreeBalance(Uint8List data) {
    // AccountInfo 布局：
    // [0..3]   nonce u32
    // [4..7]   consumers u32
    // [8..11]  providers u32
    // [12..15] sufficients u32
    // [16..31] free u128 (LE)
    if (data.length < 32) {
      return 0.0;
    }
    final freeFen = _readU128LE(data, 16);
    // 100 fen = 1 yuan, TOKEN_DECIMALS = 2
    return freeFen.toDouble() / 100.0;
  }

  /// 读取 16 字节小端序 u128。
  BigInt _readU128LE(Uint8List bytes, int offset) {
    var value = BigInt.zero;
    for (var i = 15; i >= 0; i--) {
      value = (value << 8) | BigInt.from(bytes[offset + i]);
    }
    return value;
  }

  static const _maxRetries = 3;
  static const _requestTimeout = Duration(seconds: 8);

  /// 发送 JSON-RPC 请求，自动故障切换（最多尝试 3 个节点）。
  ///
  /// 区分两类错误：
  /// - 网络错误（超时、连接拒绝）：切换节点重试
  /// - RPC 业务错误（链返回 error）：直接抛出，不重试
  Future<dynamic> _rpcCall(String method, List<dynamic> params) async {
    final maxAttempts =
        _nodes.length < _maxRetries ? _nodes.length : _maxRetries;
    final tried = <int>{};
    while (tried.length < maxAttempts) {
      final provider = _getProvider();
      tried.add(_currentIndex);
      try {
        final response =
            await provider.send(method, params).timeout(_requestTimeout);
        if (response.error != null) {
          // RPC 业务错误：链收到了请求但拒绝了，不需要重试其他节点
          throw Exception('${response.error}');
        }
        return response.result;
      } on Exception catch (e) {
        final msg = e.toString();
        // 如果是链返回的业务错误（非网络问题），直接抛出不重试
        if (msg.contains('code') ||
            msg.contains('1010') ||
            msg.contains('1012') ||
            msg.contains('Extrinsic') ||
            msg.contains('decode')) {
          debugPrint('RPC business error on ${_nodes[_currentIndex]}: $e');
          rethrow;
        }
        // 网络错误：切换节点重试
        debugPrint('RPC node ${_nodes[_currentIndex]} network error: $e');
        _provider = null;
        _currentIndex = (_currentIndex + 1) % _nodes.length;
      }
    }
    throw Exception('RPC 节点不可达（已尝试 $maxAttempts 个节点）');
  }

  HttpProvider _getProvider() {
    _provider ??= HttpProvider(Uri.parse(_nodes[_currentIndex]));
    return _provider!;
  }

  static Uint8List _pubkeyHexToBytes(String hex) {
    final clean = hex.startsWith('0x') ? hex.substring(2) : hex;
    if (clean.length != 64) {
      throw ArgumentError('pubkeyHex 应为 32 字节（64 hex），实际: ${clean.length}');
    }
    return _hexDecode(clean);
  }

  static Uint8List _hexDecode(String hex) {
    final result = Uint8List(hex.length ~/ 2);
    for (var i = 0; i < result.length; i++) {
      result[i] = int.parse(hex.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return result;
  }

  static String _hexEncode(Uint8List bytes) {
    return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  }
}
