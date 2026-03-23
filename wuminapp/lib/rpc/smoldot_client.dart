import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:smoldot/smoldot.dart';

/// citizenchain 轻节点客户端管理器（全局单例）。
///
/// 基于 smoldot 轻客户端，App 启动时初始化，加载 chainspec 后
/// 加入 citizenchain P2P 网络。提供 JSON-RPC 请求和订阅接口。
class SmoldotClientManager {
  SmoldotClientManager._();

  /// 全局唯一实例。
  static final SmoldotClientManager instance = SmoldotClientManager._();

  SmoldotClient? _client;
  Chain? _chain;
  bool _initialized = false;
  bool _synced = false;
  Future<void>? _syncFuture;

  /// 轻节点是否已初始化并加入链。
  bool get isReady => _initialized && _chain != null;

  /// 初始化 smoldot 轻客户端并加入 citizenchain。
  ///
  /// 从 assets/chainspec.json 加载链规格文件。
  /// 如果已初始化则直接返回。
  Future<void> initialize() async {
    if (_initialized) return;

    // 1. 创建 smoldot 客户端
    _client = SmoldotClient(
      config: const SmoldotConfig(
        maxLogLevel: kDebugMode ? 3 : 1, // debug 模式输出 info，release 仅 error
        maxChains: 1,
      ),
    );
    await _client!.initialize();

    // 2. 从 assets 加载 citizenchain 链规格文件
    final chainSpec = await rootBundle.loadString('assets/chainspec.json');

    // 3. 加入 citizenchain P2P 网络
    _chain = await _client!.addChain(
      AddChainConfig(chainSpec: chainSpec),
    );

    _initialized = true;
    _synced = false;
    _syncFuture = null;
    debugPrint('[Smoldot] 轻节点已启动，正在同步区块头...');
  }

  static const _peerWaitInterval = Duration(milliseconds: 500);
  static const _peerWaitMaxAttempts = 12; // 最多等 6 秒

  /// 发送 JSON-RPC 请求，返回 result 字段。
  ///
  /// 如果当前 peers=0，先等待 peer 重连后再发请求（最多 6 秒），
  /// 避免在短暂断连期间直接报错。
  Future<dynamic> request(
    String method,
    List<dynamic> params, {
    bool requireSynced = true,
  }) async {
    _ensureReady();
    if (requireSynced) {
      await ensureSynced();
    }

    // 等待至少有 1 个 peer 连接
    await _waitForPeer();

    final response = await _chain!.request(method, params);
    if (response.isError) {
      throw Exception('smoldot RPC 请求失败: $method, error=${response.error}');
    }
    return response.result;
  }

  /// 等待至少有 1 个 peer 连接。如果当前 peers=0，轮询等待。
  Future<void> _waitForPeer() async {
    for (var i = 0; i < _peerWaitMaxAttempts; i++) {
      final peers = await getPeerCount();
      if (peers > 0) return;
      if (i == 0) {
        debugPrint('[Smoldot] peers=0，等待 P2P 重连...');
      }
      await Future<void>.delayed(_peerWaitInterval);
    }
    // 超时后仍然发请求（让 smoldot 返回具体错误，由上层重试处理）
  }

  /// 创建轻节点订阅，返回事件流。
  ///
  /// 当前用于接收 `chain_subscribeNewHeads` 等链事件。
  Stream<dynamic> subscribe(String method, List<dynamic> params) {
    _ensureReady();
    return _chain!.subscribe(method, params);
  }

  /// 等待轻节点同步到最新区块。
  Future<void> waitUntilSynced({
    Duration timeout = const Duration(minutes: 2),
  }) async {
    await ensureSynced(timeout: timeout);
  }

  /// 在首次链上读写前等待轻节点同步完成，避免把未同步状态误判为链上空数据。
  Future<void> ensureSynced({
    Duration timeout = const Duration(minutes: 2),
  }) async {
    if (!isReady || _synced) return;

    final current = _syncFuture;
    if (current != null) {
      await current;
      return;
    }

    final future = _waitForSync(timeout);
    _syncFuture = future;
    try {
      await future;
    } finally {
      if (!_synced) {
        _syncFuture = null;
      }
    }
  }

  Future<void> _waitForSync(Duration timeout) async {
    debugPrint('[Smoldot] 等待轻节点同步完成...');
    await _chain!.waitUntilSynced(timeout: timeout);
    _synced = true;
    debugPrint('[Smoldot] 区块头同步完成');
  }

  /// 获取当前连接的 P2P 节点数。
  Future<int> getPeerCount() async {
    if (!isReady) return 0;
    return await _chain!.getPeerCount();
  }

  /// 获取轻节点状态快照，供后续业务层逐步替代裸 JSON-RPC 读状态。
  Future<LightClientStatusSnapshot?> getStatusSnapshot() async {
    if (!isReady) return null;
    return await _chain!.getStatusSnapshot();
  }

  /// 原生读取运行时版本 JSON。
  Future<Map<String, dynamic>?> getRuntimeVersionJson() async {
    if (!isReady) return null;
    return await _chain!.getRuntimeVersionJson();
  }

  /// 原生读取 metadata hex。
  Future<String?> getMetadataHex() async {
    if (!isReady) return null;
    return await _chain!.getMetadataHex();
  }

  /// 原生读取账户下一个可用 nonce。
  Future<int?> getAccountNextIndex(String accountIdHex) async {
    if (!isReady) return null;
    return await _chain!.getAccountNextIndex(accountIdHex);
  }

  /// 原生读取指定块高的 block hash。
  Future<String?> getBlockHash(int blockNumber) async {
    if (!isReady) return null;
    return await _chain!.getBlockHash(blockNumber);
  }

  /// 原生读取指定区块中的 extrinsics。
  Future<List<String>> getBlockExtrinsics(String blockHashHex) async {
    if (!isReady) return const [];
    return await _chain!.getBlockExtrinsics(blockHashHex);
  }

  /// 原生提交已编码 extrinsic。
  Future<String?> submitExtrinsicHex(String extrinsicHex) async {
    if (!isReady) return null;
    return await _chain!.submitExtrinsicHex(extrinsicHex);
  }

  /// 原生读取 `System.Account` 快照，供钱包余额迁移使用。
  Future<SystemAccountSnapshot?> getSystemAccountSnapshot(String accountIdHex) async {
    if (!isReady) return null;
    return await _chain!.getSystemAccount(accountIdHex);
  }

  /// 原生读取单个 storage value（hex）。
  Future<String?> getStorageValueHex(String storageKeyHex) async {
    if (!isReady) return null;
    return await _chain!.getStorageValueHex(storageKeyHex);
  }

  /// 原生批量读取多个 storage value（hex）。
  Future<Map<String, String?>> getStorageValuesHex(List<String> storageKeyHexList) async {
    if (!isReady || storageKeyHexList.isEmpty) {
      return const {};
    }
    return await _chain!.getStorageValuesHex(storageKeyHexList);
  }

  /// 释放资源。App 退出时调用。
  void dispose() {
    _chain?.dispose();
    _client?.dispose();
    _chain = null;
    _client = null;
    _initialized = false;
    _synced = false;
    _syncFuture = null;
    debugPrint('[Smoldot] 轻节点已关闭');
  }

  void _ensureReady() {
    if (!isReady) {
      throw StateError('smoldot 轻节点未初始化，请先调用 initialize()');
    }
  }
}
