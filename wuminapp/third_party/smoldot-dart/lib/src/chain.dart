import 'dart:convert';
import 'dart:async';
import 'bindings.dart';
import 'types.dart';
import 'json_rpc.dart';

/// Represents a blockchain chain managed by smoldot
///
/// A [Chain] instance provides methods for interacting with a specific
/// blockchain through JSON-RPC calls and subscriptions.
class Chain {
  /// Chain identifier (handle from Rust)
  final int chainId;

  /// Parent client instance (kept as reference)
  final Object client;

  /// FFI bindings
  final SmoldotBindings bindings;

  /// Native client handle (u64 from Rust)
  final int clientHandle;

  /// JSON-RPC handler for this chain
  late final JsonRpcHandler _jsonRpc;

  /// Whether the chain has been disposed
  bool _isDisposed = false;

  /// Creates a new Chain instance
  ///
  /// This is typically called internally by [SmoldotClient.addChain].
  Chain({
    required this.chainId,
    required this.client,
    required this.bindings,
    required this.clientHandle,
  }) {
    _jsonRpc = JsonRpcHandler(
      chainId: chainId,
      bindings: bindings,
      clientHandle: clientHandle,
    );
  }

  /// Whether this chain has been disposed
  bool get isDisposed => _isDisposed;

  /// Send a JSON-RPC request to the chain
  ///
  /// [method] is the RPC method name (e.g., 'system_chain').
  /// [params] is a list of parameters for the method.
  ///
  /// Returns a [Future] that completes with the response.
  /// Throws [JsonRpcException] if the request fails.
  ///
  /// Example:
  /// ```dart
  /// final response = await chain.request('system_chain', []);
  /// print(response.result);
  /// ```
  Future<JsonRpcResponse> request(String method, List<dynamic> params) async {
    _ensureNotDisposed();
    return _jsonRpc.request(method, params);
  }

  /// Subscribe to JSON-RPC notifications
  ///
  /// [method] is the subscription method name (e.g., 'chain_subscribeNewHeads').
  /// [params] is a list of parameters for the subscription.
  ///
  /// Returns a [Stream] of responses.
  /// The stream will emit [JsonRpcException] if errors occur.
  ///
  /// Example:
  /// ```dart
  /// final subscription = await chain.subscribe('chain_subscribeNewHeads', []);
  /// await for (final response in subscription) {
  ///   print('New block: ${response.result}');
  /// }
  /// ```
  Stream<JsonRpcResponse> subscribe(String method, List<dynamic> params) {
    _ensureNotDisposed();
    return _jsonRpc.subscribe(method, params);
  }

  /// Unsubscribe from a JSON-RPC subscription
  ///
  /// [subscriptionId] is the ID returned by the subscribe method.
  Future<void> unsubscribe(String subscriptionId) async {
    _ensureNotDisposed();
    await _jsonRpc.unsubscribe(subscriptionId);
  }

  /// Get information about this chain
  Future<ChainInfo> getInfo() async {
    _ensureNotDisposed();

    final chainName = await request('system_chain', []);
    final snapshot = await getStatusSnapshot();

    return ChainInfo(
      chainId: chainId,
      name: chainName.result as String,
      status: snapshot.isSyncing ? ChainStatus.syncing : ChainStatus.synced,
      peerCount: snapshot.peerCount,
      bestBlockNumber: snapshot.bestBlockNumber,
      bestBlockHash: snapshot.bestBlockHash,
    );
  }

  /// Get the current best block number
  Future<int?> getBestBlockNumber() async {
    _ensureNotDisposed();
    final snapshot = await getStatusSnapshot();
    return snapshot.bestBlockNumber;
  }

  /// Get the current best block hash
  Future<String?> getBestBlockHash() async {
    _ensureNotDisposed();
    final snapshot = await getStatusSnapshot();
    return snapshot.bestBlockHash;
  }

  /// Get the number of connected peers
  Future<int> getPeerCount() async {
    _ensureNotDisposed();
    final snapshot = await getStatusSnapshot();
    return snapshot.peerCount;
  }

  /// Get the chain status
  Future<ChainStatus> getStatus() async {
    _ensureNotDisposed();
    final snapshot = await getStatusSnapshot();
    return snapshot.isSyncing ? ChainStatus.syncing : ChainStatus.synced;
  }

  /// 中文注释：把轻节点可观察状态收口成结构化对象，避免业务层继续直接拼裸 RPC。
  Future<LightClientStatusSnapshot> getStatusSnapshot() async {
    _ensureNotDisposed();
    final snapshotJson = bindings.getStatusSnapshotJson(chainId);
    return LightClientStatusSnapshot.fromJson(
      jsonDecode(snapshotJson) as Map<String, dynamic>,
    );
  }

  /// 原生读取运行时版本。
  Future<Map<String, dynamic>> getRuntimeVersionJson() async {
    _ensureNotDisposed();
    final snapshotJson = bindings.getRuntimeVersionJson(chainId);
    return jsonDecode(snapshotJson) as Map<String, dynamic>;
  }

  /// 原生读取 metadata hex。
  Future<String> getMetadataHex() async {
    _ensureNotDisposed();
    return bindings.getMetadataHex(chainId);
  }

  /// 原生读取账户下一个可用 nonce。
  Future<int> getAccountNextIndex(String accountIdHex) async {
    _ensureNotDisposed();
    return int.parse(bindings.getAccountNextIndex(chainId, accountIdHex));
  }

  /// 原生读取指定块高的 block hash。
  Future<String> getBlockHash(int blockNumber) async {
    _ensureNotDisposed();
    return bindings.getBlockHash(chainId, blockNumber);
  }

  /// 原生读取指定区块的 extrinsics 列表。
  Future<List<String>> getBlockExtrinsics(String blockHashHex) async {
    _ensureNotDisposed();
    final responseJson = bindings.getBlockExtrinsicsJson(chainId, blockHashHex);
    final response = jsonDecode(responseJson) as List<dynamic>;
    return response.cast<String>();
  }

  /// 原生提交已编码 extrinsic。
  Future<String> submitExtrinsicHex(String extrinsicHex) async {
    _ensureNotDisposed();
    return bindings.submitExtrinsicHex(chainId, extrinsicHex);
  }

  /// 原生读取 `System.Account`。
  Future<SystemAccountSnapshot> getSystemAccount(String accountIdHex) async {
    _ensureNotDisposed();
    final snapshotJson = bindings.getSystemAccountJson(chainId, accountIdHex);
    return SystemAccountSnapshot.fromJson(
      jsonDecode(snapshotJson) as Map<String, dynamic>,
    );
  }

  /// 原生读取单个 storage value。
  Future<String?> getStorageValueHex(String storageKeyHex) async {
    _ensureNotDisposed();
    final responseJson = bindings.getStorageValueJson(chainId, storageKeyHex);
    final response = jsonDecode(responseJson) as Map<String, dynamic>;
    if (response['exists'] != true) {
      return null;
    }
    return response['valueHex'] as String?;
  }

  /// 原生批量读取多个 storage value。
  Future<Map<String, String?>> getStorageValuesHex(List<String> storageKeys) async {
    _ensureNotDisposed();
    final responseJson =
        bindings.getStorageValuesJson(chainId, jsonEncode(storageKeys));
    final response = jsonDecode(responseJson) as Map<String, dynamic>;
    return response.map(
      (key, value) => MapEntry(key, value == null ? null : value as String),
    );
  }

  /// Wait until the chain is synced
  Future<void> waitUntilSynced({
    Duration timeout = const Duration(minutes: 5),
    Duration pollInterval = const Duration(seconds: 1),
  }) async {
    _ensureNotDisposed();

    final stopwatch = Stopwatch()..start();

    while (stopwatch.elapsed < timeout) {
      final status = await getStatus();

      if (status == ChainStatus.synced) {
        return;
      }

      await Future<void>.delayed(pollInterval);
    }

    throw TimeoutException(
      'Chain did not sync within ${timeout.inSeconds} seconds',
      timeout,
    );
  }

  /// Get the database content for this chain
  /// Note: This is not yet implemented in smoldot FFI
  Future<String?> getDatabaseContent() async {
    _ensureNotDisposed();
    // This would require additional FFI support from smoldot
    // For now, return null
    return null;
  }

  /// Dispose of this chain and free resources
  Future<void> dispose() async {
    if (_isDisposed) {
      return;
    }

    _jsonRpc.dispose();
    _isDisposed = true;
  }

  /// Ensure the chain is not disposed
  void _ensureNotDisposed() {
    if (_isDisposed) {
      throw SmoldotException('Chain $chainId has been disposed');
    }
  }
  @override
  String toString() => 'Chain(chainId: $chainId, isDisposed: $_isDisposed)';
}
