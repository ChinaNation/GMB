import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:polkadart/polkadart.dart'
    show Hasher, RuntimeMetadata, RuntimeVersion;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import 'smoldot_client.dart';

/// citizenchain RPC 客户端。
///
/// 只使用 smoldot 轻节点（P2P 网络，无需远程 RPC 服务器）。
class ChainRpc {
  ChainRpc() {
    debugPrint('[ChainRpc] 使用 smoldot 轻节点模式');
  }

  static final _keyring = Keyring();
  static final Uint8List _sfidMainAccountKey =
      _buildStorageValueKey('SfidCodeAuth', 'SfidMainAccount');

  // ──── 批量查询 ────

  /// 批量查询多个 storage key，一次 RPC 调用返回所有结果。
  /// 使用 state_queryStorageAt([keys]) RPC 方法。
  Future<Map<String, Uint8List?>> fetchStorageBatch(
      List<String> storageKeyHexList) async {
    if (storageKeyHexList.isEmpty) return {};

    // 中文注释：轻节点模式改为走原生批量 storage 读取，避免继续依赖 `state_queryStorageAt`。
    final rawMap = await SmoldotClientManager.instance
        .getStorageValuesHex(storageKeyHexList);
    final result = <String, Uint8List?>{};
    for (final key in storageKeyHexList) {
      final valueHex = rawMap[key];
      result[key] = valueHex == null
          ? null
          : _hexDecode(valueHex.startsWith('0x') ? valueHex.substring(2) : valueHex);
    }
    return result;
  }

  // ──── 转账相关 RPC ────

  /// 查询账户下一个可用 nonce（含交易池中的 pending 交易）。
  Future<int> fetchNonce(String ss58Address) async {
    // 中文注释：轻节点模式先在 Dart 侧解出 accountId，再交给原生 runtime call，避免继续依赖 legacy `system_accountNextIndex`。
    final accountIdHex = '0x${_hexEncode(_keyring.decodeAddress(ss58Address))}';
    final result =
        await SmoldotClientManager.instance.getAccountNextIndex(accountIdHex);
    if (result == null) {
      throw StateError('smoldot 轻节点尚未提供 accountNextIndex');
    }
    return result;
  }

  /// 获取运行时版本（specVersion、transactionVersion）。
  Future<RuntimeVersion> fetchRuntimeVersion() async {
    // 中文注释：轻节点模式优先走原生 capability，避免业务层继续直接依赖裸 RPC 方法名。
    final result = await SmoldotClientManager.instance.getRuntimeVersionJson();
    if (result == null) {
      throw StateError('smoldot 轻节点尚未提供运行时版本');
    }
    return RuntimeVersion.fromJson(result);
  }

  /// 获取创世块哈希（32 字节）。结果缓存，同一实例只查一次。
  Future<Uint8List> fetchGenesisHash() async {
    if (_cachedGenesisHash != null) return _cachedGenesisHash!;

    final result = await SmoldotClientManager.instance.getBlockHash(0);
    if (result == null || result.isEmpty) {
      throw StateError('smoldot 轻节点尚未提供创世块哈希');
    }
    _cachedGenesisHash = _hexDecode(_stripHexPrefix(result));
    return _cachedGenesisHash!;
  }

  Uint8List? _cachedGenesisHash;

  /// 获取最新区块的哈希和块号（用于 mortal era 计算）。
  Future<({Uint8List blockHash, int blockNumber})> fetchLatestBlock() async {
    // 中文注释：轻节点模式直接复用原生状态快照，减少一次 `chain_getHeader` 往返。
    final snapshot = await SmoldotClientManager.instance.getStatusSnapshot();
    final hashHex = snapshot?.bestBlockHash;
    final blockNumber = snapshot?.bestBlockNumber;
    if (hashHex == null || hashHex.isEmpty || blockNumber == null) {
      throw StateError('smoldot 轻节点尚未提供最新区块快照');
    }
    return (
      blockHash: _hexDecode(_stripHexPrefix(hashHex)),
      blockNumber: blockNumber,
    );
  }

  /// 获取指定区块中所有 extrinsic 的 blake2_256 哈希。
  ///
  /// 用于交易确认：在链上区块中搜索指定 txHash 是否存在。
  Future<List<String>?> fetchBlockExtrinsicHashes(int blockNumber) async {
    try {
      final blockHashHex =
          await SmoldotClientManager.instance.getBlockHash(blockNumber);
      if (blockHashHex == null || blockHashHex.isEmpty) {
        return null;
      }
      final extrinsics =
          await SmoldotClientManager.instance.getBlockExtrinsics(blockHashHex);

      final hashes = <String>[];
      for (final ext in extrinsics) {
        final extBytes = _hexDecode(_stripHexPrefix(ext));
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

    // 中文注释：轻节点模式直接读取原生 metadata hex，避免 Dart 层再拼 `state_getMetadata`。
    final metadataHex = await SmoldotClientManager.instance.getMetadataHex();
    if (metadataHex == null || metadataHex.isEmpty) {
      throw StateError('smoldot 轻节点尚未提供 metadata');
    }
    _cachedMetadata = RuntimeMetadata.fromHex(metadataHex);
    return _cachedMetadata!;
  }

  RuntimeMetadata? _cachedMetadata;
  String? _cachedCurrentSfidMainPubkeyHex;

  /// 提交已签名的 extrinsic，返回交易哈希（32 字节）。
  ///
  /// 瞬断重试已由 `SmoldotClientManager._withRetry` 统一处理。
  Future<Uint8List> submitExtrinsic(Uint8List encoded) async {
    final hex = '0x${_hexEncode(encoded)}';
    final result =
        await SmoldotClientManager.instance.submitExtrinsicHex(hex);
    if (result == null || result.isEmpty) {
      throw StateError('smoldot 轻节点未返回交易哈希');
    }
    return _hexDecode(_stripHexPrefix(result));
  }

  // ──── 链上状态查询 ────

  /// 通用 storage 查询：传入完整的 storage key（含 0x 前缀），
  /// 返回原始 SCALE 编码字节。key 不存在时返回 null。
  Future<Uint8List?> fetchStorage(String storageKeyHex) async {
    // 中文注释：轻节点模式统一通过原生 storage 读取，逐步清理 Dart 层的裸 RPC。
    final valueHex = await SmoldotClientManager.instance
        .getStorageValueHex(storageKeyHex);
    if (valueHex == null) return null;
    return _hexDecode(
      valueHex.startsWith('0x') ? valueHex.substring(2) : valueHex,
    );
  }

  /// 查询链上已打包的 nonce（不含交易池），账户不存在返回 0。
  Future<int> fetchConfirmedNonce(String pubkeyHex) async {
    // 中文注释：轻节点模式优先走原生 `System.Account` 快照，避免 Dart 层继续拼 storage RPC。
    final snapshot = await SmoldotClientManager.instance
        .getSystemAccountSnapshot(_normalizeAccountHex(pubkeyHex));
    if (snapshot == null || !snapshot.exists) {
      return 0;
    }
    return snapshot.nonce ?? 0;
  }

  /// 查询链上余额，返回元（yuan）。账户不存在返回 0.0。
  Future<double> fetchBalance(String pubkeyHex) async {
    // 中文注释：钱包余额刷新先切到原生 `System.Account` 路径，后续再逐步迁移其他 storage 读取。
    final snapshot = await SmoldotClientManager.instance
        .getSystemAccountSnapshot(_normalizeAccountHex(pubkeyHex));
    if (snapshot == null || !snapshot.exists) {
      return 0.0;
    }
    return snapshot.freeYuan ?? 0.0;
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
    final result = await SmoldotClientManager.instance.getStorageValueHex(keyHex);
    if (result == null) {
      return null;
    }

    final data = _hexDecode(_stripHexPrefix(result));
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

  static Uint8List _buildStorageValueKey(
      String palletName, String storageName) {
    final palletHash = Hasher.twoxx128.hashString(palletName);
    final storageHash = Hasher.twoxx128.hashString(storageName);
    final key = Uint8List(palletHash.length + storageHash.length);
    key.setAll(0, palletHash);
    key.setAll(palletHash.length, storageHash);
    return key;
  }

  static String _normalizeAccountHex(String hex) {
    return hex.startsWith('0x') ? hex : '0x$hex';
  }

  static String _stripHexPrefix(String value) {
    return value.startsWith('0x') ? value.substring(2) : value;
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
