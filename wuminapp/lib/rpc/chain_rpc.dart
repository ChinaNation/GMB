import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:polkadart/polkadart.dart'
    show HttpProvider, Hasher, RuntimeMetadata, RuntimeVersion;

import 'smoldot_client.dart';

/// citizenchain RPC 客户端。
///
/// 默认使用 smoldot 轻节点（P2P 网络，无需远程 RPC 服务器）。
/// 如果设置了 WUMINAPP_RPC_URL 环境变量或构造时传入 rpcUrl，
/// 则回退到传统 HTTP RPC 模式（仅用于开发调试）。
class ChainRpc {
  ChainRpc({String? rpcUrl}) {
    // 如果显式指定了 RPC URL 或环境变量中有配置，使用传统 HTTP RPC 模式
    final explicitUrl =
        rpcUrl ?? const String.fromEnvironment('WUMINAPP_RPC_URL');
    if (explicitUrl.isNotEmpty) {
      _useSmoldot = false;
      _rpcProvider = HttpProvider(Uri.parse(explicitUrl));
      debugPrint('[ChainRpc] 使用 HTTP RPC 模式: $explicitUrl');
    } else {
      _useSmoldot = true;
      debugPrint('[ChainRpc] 使用 smoldot 轻节点模式');
    }
  }

  /// 是否使用 smoldot 轻节点模式。
  late final bool _useSmoldot;

  /// HTTP RPC 模式下的 provider（仅在非 smoldot 模式下使用）。
  HttpProvider? _rpcProvider;

  // twox_128("System") + twox_128("Account")
  static final Uint8List _systemAccountPrefix = _hexDecode(
    '26aa394eea5630e07c48ae0c9558cef7'
    'b99d880ec681799c0cf30e8886371da9',
  );
  static final Uint8List _sfidMainAccountKey =
      _buildStorageValueKey('SfidCodeAuth', 'SfidMainAccount');

  /// 当前是否为轻节点模式。
  bool get isLightClient => _useSmoldot;

  /// 当前活跃节点的 HTTP URL（WebSocket 回退模式使用）。
  ///
  /// smoldot 模式下返回空字符串（ChainEventSubscription 会自动
  /// 检测 smoldot 状态，不依赖此值）。
  String get currentNodeUrl {
    if (_useSmoldot) return '';
    return _rpcProvider?.url.toString() ?? '';
  }

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

  /// 获取指定区块中所有 extrinsic 的 blake2_256 哈希。
  ///
  /// 用于交易确认：在链上区块中搜索指定 txHash 是否存在。
  Future<List<String>?> fetchBlockExtrinsicHashes(int blockNumber) async {
    try {
      final blockHashHex =
          await _rpcCall('chain_getBlockHash', [blockNumber]) as String;
      final block = await _rpcCall('chain_getBlock', [blockHashHex])
          as Map<String, dynamic>;
      final extrinsics =
          (block['block'] as Map<String, dynamic>)['extrinsics'] as List<dynamic>;

      final hashes = <String>[];
      for (final ext in extrinsics) {
        final extHex = ext as String;
        final extBytes = _hexDecode(extHex.substring(2));
        // blake2_256 哈希
        final hash = _blake2b256(extBytes);
        hashes.add('0x${_hexEncode(hash)}');
      }
      return hashes;
    } catch (_) {
      return null;
    }
  }

  static Uint8List _blake2b256(Uint8List data) {
    // 使用 polkadart 的 Hasher
    return Uint8List.fromList(Hasher.blake2b256.hash(data));
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

  static const _smoldotRpcTimeout = Duration(minutes: 2);
  static const _httpRpcTimeout = Duration(seconds: 30);

  static const _smoldotMaxRetries = 3;
  static const _smoldotRetryDelay = Duration(seconds: 2);

  /// 发送 JSON-RPC 请求。
  ///
  /// - smoldot 模式：通过轻节点 P2P 网络发送，遇到"No node available"自动重试
  /// - HTTP RPC 模式：通过 HTTP 发送到远程节点（开发用）
  Future<dynamic> _rpcCall(String method, List<dynamic> params) async {
    if (_useSmoldot) {
      // smoldot P2P 连接可能短暂中断，遇到 -32000/-32800 错误时自动重试。
      for (var attempt = 1; attempt <= _smoldotMaxRetries; attempt++) {
        try {
          return await SmoldotClientManager.instance
              .request(method, params)
              .timeout(_smoldotRpcTimeout);
        } on Exception catch (e) {
          final msg = e.toString();
          final isTransient = msg.contains('No node available') ||
              msg.contains('-32000') ||
              msg.contains('-32800') ||
              msg.contains('Failed to retrieve');
          if (!isTransient || attempt == _smoldotMaxRetries) {
            rethrow;
          }
          debugPrint('[ChainRpc] $method 失败（peers 暂时为 0），${_smoldotRetryDelay.inSeconds}s 后重试 ($attempt/$_smoldotMaxRetries)');
          await Future<void>.delayed(_smoldotRetryDelay);
        }
      }
      // 不应到达此处
      throw Exception('RPC 重试次数已用尽');
    } else {
      // 传统 HTTP RPC 模式（开发调试用）
      final response =
          await _rpcProvider!.send(method, params).timeout(_httpRpcTimeout);
      if (response.error != null) {
        throw Exception('${response.error}');
      }
      return response.result;
    }
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
