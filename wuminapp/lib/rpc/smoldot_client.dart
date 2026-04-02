import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:smoldot/smoldot.dart';

/// 链健康状态。
enum ChainHealthStatus {
  /// 轻节点未初始化。
  uninitialized,

  /// 正在同步区块头。
  syncing,

  /// 链可用，读写正常。
  operational,

  /// 链暂不可用（storage proof 下载失败等瞬断场景）。
  degraded,
}

/// citizenchain 轻节点客户端管理器（全局单例）。
///
/// 基于 smoldot 轻客户端，App 启动时初始化，加载 chainspec 后
/// 加入 citizenchain P2P 网络。提供 JSON-RPC 请求和订阅接口。
///
/// 所有链上读操作内置瞬断重试（最多 2 次，间隔 1 秒），
/// 并维护 [healthStatus] 供 UI 层展示链状态。
class SmoldotClientManager {
  SmoldotClientManager._();

  /// 全局唯一实例。
  static final SmoldotClientManager instance = SmoldotClientManager._();

  SmoldotClient? _client;
  Chain? _chain;
  bool _initialized = false;
  bool _synced = false;
  Future<void>? _syncFuture;

  /// 当前链健康状态。
  ChainHealthStatus _healthStatus = ChainHealthStatus.uninitialized;
  ChainHealthStatus get healthStatus => _healthStatus;

  /// 最近一次链操作错误信息（仅 degraded 时有值）。
  String? _lastError;
  String? get lastError => _lastError;

  static const _readMaxRetries = 2;
  static const _readRetryDelay = Duration(seconds: 1);
  static const _defaultSyncTimeout = Duration(minutes: 3);

  /// 通用读操作包装：瞬断重试 + 健康状态更新。
  ///
  /// 所有链上读操作（余额、nonce、metadata、storage 等）统一走此方法，
  /// 避免每个调用点各自重复重试逻辑。
  Future<T> _withRetry<T>(
      String debugLabel, Future<T> Function() action) async {
    for (var attempt = 1; attempt <= _readMaxRetries; attempt++) {
      try {
        final result = await action();
        // 成功 → 恢复健康状态
        if (_healthStatus == ChainHealthStatus.degraded) {
          _healthStatus = ChainHealthStatus.operational;
          _lastError = null;
          debugPrint('[Smoldot] 链操作恢复正常');
        }
        return result;
      } catch (e) {
        final msg = e.toString().toLowerCase();
        final isTransient = msg.contains('timeout') ||
            msg.contains('proof') ||
            msg.contains('channel closed') ||
            msg.contains('no node') ||
            msg.contains('peers') ||
            msg.contains('inaccessible');
        if (!isTransient || attempt == _readMaxRetries) {
          _healthStatus = ChainHealthStatus.degraded;
          _lastError = '$debugLabel 失败: $e';
          debugPrint(
              '[Smoldot] $_lastError (attempt $attempt/$_readMaxRetries)');
          rethrow;
        }
        debugPrint(
          '[Smoldot] $debugLabel 瞬断，${_readRetryDelay.inSeconds}s 后重试 '
          '($attempt/$_readMaxRetries): $e',
        );
        await Future<void>.delayed(_readRetryDelay);
      }
    }
    // 不应到达
    throw StateError('$debugLabel 重试次数已用尽');
  }

  /// 轻节点是否已初始化并加入链。
  bool get isReady => _initialized && _chain != null;

  /// 打印当前轻节点诊断信息到 debugPrint，用于排查连接/同步/读取问题。
  Future<void> printDiagnostics() async {
    debugPrint('╔══════ Smoldot 诊断 ══════');
    debugPrint('║ initialized: $_initialized');
    debugPrint('║ chain: ${_chain != null ? "已加入" : "null"}');
    debugPrint('║ synced: $_synced');
    debugPrint('║ healthStatus: $_healthStatus');
    debugPrint('║ lastError: $_lastError');
    if (_chain != null) {
      try {
        final snapshot = await _chain!.getStatusSnapshot();
        debugPrint('║ peerCount: ${snapshot.peerCount}');
        debugPrint('║ isSyncing: ${snapshot.isSyncing}');
        debugPrint(
            '║ bestBlock: #${snapshot.bestBlockNumber} ${snapshot.bestBlockHash}');
        debugPrint(
            '║ finalizedBlock: #${snapshot.finalizedBlockNumber} ${snapshot.finalizedBlockHash}');
      } catch (e) {
        debugPrint('║ getStatusSnapshot 失败: $e');
      }
      try {
        final nonce = await _chain!.getAccountNextIndex(
            '0x0000000000000000000000000000000000000000000000000000000000000000');
        debugPrint('║ accountNextIndex(zero): $nonce');
      } catch (e) {
        debugPrint('║ accountNextIndex 失败: $e');
      }
    }
    debugPrint('╚══════════════════════════');
  }

  static const _dbCacheKey = 'smoldot_db_cache';

  /// 导出数据库的最大字节数（256 KB，足够存同步进度和已知 peer）。
  static const _dbExportMaxSize = 256 * 1024;

  /// 将内部诊断状态转换为用户可理解的链路错误提示。
  ///
  /// 原始错误细节仍保留在日志与 [lastError] 中，UI 层只展示统一文案，
  /// 避免把底层 FFI / JSON-RPC 细节直接暴露给最终用户。
  String buildUserFacingError([Object? error]) {
    final raw = '${error ?? ''} ${_lastError ?? ''}'.toLowerCase();
    if (!_initialized ||
        raw.contains('未初始化') ||
        raw.contains('failed to initialize smoldot client') ||
        raw.contains('failed to add chain')) {
      return '轻节点初始化失败，请检查网络后重试';
    }
    if ((_healthStatus == ChainHealthStatus.degraded &&
            (raw.contains('waituntilsynced') ||
                raw.contains('timeout') ||
                raw.contains('timed out') ||
                raw.contains('同步失败'))) ||
        raw.contains('轻节点同步失败')) {
      return '轻节点同步超时，请检查网络后重试';
    }
    if (_healthStatus == ChainHealthStatus.syncing ||
        raw.contains('waituntilsynced') ||
        raw.contains('timeout') ||
        raw.contains('timed out')) {
      return '轻节点正在同步区块头，请稍后再试';
    }
    if (_healthStatus == ChainHealthStatus.degraded ||
        raw.contains('proof') ||
        raw.contains('channel closed') ||
        raw.contains('peers') ||
        raw.contains('inaccessible')) {
      return '区块链暂不可用，请检查网络连接后重试';
    }
    return '区块链读取失败，请稍后再试';
  }

  /// 初始化 smoldot 轻客户端并加入 citizenchain。
  ///
  /// 从 assets/chainspec.json 加载链规格文件。
  /// 如果上次运行有缓存的同步数据库，会通过 `databaseContent` 恢复，
  /// 大幅缩短区块头同步时间。
  /// 如果已初始化则直接返回。
  Future<void> initialize() async {
    if (_initialized) return;

    _lastError = null;
    _healthStatus = ChainHealthStatus.syncing;

    try {
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

      // 3. 优先恢复上次导出的 finalized database，避免每次冷启动都从零同步
      final cachedDatabase = await _loadCachedDatabase();
      if (cachedDatabase != null && cachedDatabase.isNotEmpty) {
        try {
          _chain = await _addChain(
            chainSpec,
            databaseContent: cachedDatabase,
          );
          debugPrint('[Smoldot] 已从同步缓存恢复轻节点 (${cachedDatabase.length} bytes)');
        } catch (e) {
          // 中文注释：缓存与当前链状态不兼容时，清掉缓存并回退到无缓存重连，
          // 避免一次坏缓存把后续所有启动都卡死。
          debugPrint('[Smoldot] 同步缓存失效，清理后重试: $e');
          await _clearCachedDatabase();
          _chain = await _addChain(chainSpec);
        }
      } else {
        _chain = await _addChain(chainSpec);
      }

      _initialized = true;
      _synced = false;
      _syncFuture = null;
      _healthStatus = ChainHealthStatus.syncing;
      debugPrint('[Smoldot] 轻节点已启动，正在同步区块头...');

      // 中文注释：App 启动后立刻在后台预热同步，后续页面读链只需等待同一个 Future。
      unawaited(
        ensureSynced(timeout: _defaultSyncTimeout).catchError((Object e) {
          debugPrint('[Smoldot] 后台同步失败: $e');
        }),
      );
    } catch (e) {
      _healthStatus = ChainHealthStatus.degraded;
      _lastError = '轻节点初始化失败: $e';
      debugPrint('[Smoldot] $_lastError');
      try {
        _chain?.dispose();
      } catch (_) {}
      try {
        _client?.dispose();
      } catch (_) {}
      _chain = null;
      _client = null;
      _initialized = false;
      _synced = false;
      _syncFuture = null;
      rethrow;
    }
  }

  Future<Chain> _addChain(
    String chainSpec, {
    String? databaseContent,
  }) {
    return _client!.addChain(
      AddChainConfig(
        chainSpec: chainSpec,
        databaseContent: databaseContent,
      ),
    );
  }

  /// 从 SharedPreferences 加载缓存的 smoldot 同步数据库。
  Future<String?> _loadCachedDatabase() async {
    try {
      final prefs = await SharedPreferences.getInstance();
      return prefs.getString(_dbCacheKey);
    } catch (e) {
      debugPrint('[Smoldot] 加载同步缓存失败: $e');
      return null;
    }
  }

  Future<void> _clearCachedDatabase() async {
    try {
      final prefs = await SharedPreferences.getInstance();
      await prefs.remove(_dbCacheKey);
      debugPrint('[Smoldot] 已清除失效同步缓存');
    } catch (e) {
      debugPrint('[Smoldot] 清除同步缓存失败: $e');
    }
  }

  /// 通过 JSON-RPC 导出当前同步数据库并写入 SharedPreferences。
  Future<void> _saveDatabaseCache() async {
    if (!isReady) return;
    try {
      final result = await _chain!.request(
        'chainHead_unstable_finalizedDatabase',
        [_dbExportMaxSize],
      );
      if (result.isError || result.result == null) {
        debugPrint('[Smoldot] 导出同步数据库失败: ${result.error}');
        return;
      }
      final dbContent = result.result as String;
      if (dbContent.isEmpty) return;
      final prefs = await SharedPreferences.getInstance();
      await prefs.setString(_dbCacheKey, dbContent);
      debugPrint('[Smoldot] 同步缓存已保存 (${dbContent.length} bytes)');
    } catch (e) {
      debugPrint('[Smoldot] 保存同步缓存失败: $e');
    }
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
    Duration timeout = _defaultSyncTimeout,
  }) async {
    await ensureSynced(timeout: timeout);
  }

  /// 在首次链上读写前等待轻节点同步完成，避免把未同步状态误判为链上空数据。
  Future<void> ensureSynced({
    Duration timeout = _defaultSyncTimeout,
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
    try {
      await _chain!.waitUntilSynced(timeout: timeout);
      _synced = true;
      _healthStatus = ChainHealthStatus.operational;
      _lastError = null;
      debugPrint('[Smoldot] 区块头同步完成');

      // 同步完成后异步保存数据库缓存，下次启动可快速恢复
      unawaited(_saveDatabaseCache());
    } catch (e) {
      _healthStatus = ChainHealthStatus.degraded;
      _lastError = '轻节点同步失败: $e';
      debugPrint('[Smoldot] $_lastError');
      rethrow;
    }
  }

  /// 获取当前连接的 P2P 节点数。
  Future<int> getPeerCount() async {
    if (!isReady) return 0;
    return await _chain!.getPeerCount();
  }

  /// 获取轻节点状态快照，供后续业务层逐步替代裸 JSON-RPC 读状态。
  Future<LightClientStatusSnapshot?> getStatusSnapshot() async {
    _ensureReady();
    await ensureSynced();
    return _withRetry('getStatusSnapshot', () => _chain!.getStatusSnapshot());
  }

  /// 原生读取运行时版本 JSON。
  Future<Map<String, dynamic>?> getRuntimeVersionJson() async {
    _ensureReady();
    await ensureSynced();
    return _withRetry(
        'getRuntimeVersion', () => _chain!.getRuntimeVersionJson());
  }

  /// 原生读取 metadata hex。
  Future<String?> getMetadataHex() async {
    _ensureReady();
    await ensureSynced();
    return _withRetry('getMetadata', () => _chain!.getMetadataHex());
  }

  /// 原生读取账户下一个可用 nonce。
  Future<int?> getAccountNextIndex(String accountIdHex) async {
    _ensureReady();
    await ensureSynced();
    await _waitForPeer();
    return _withRetry(
        'getAccountNextIndex', () => _chain!.getAccountNextIndex(accountIdHex));
  }

  /// 原生读取指定块高的 block hash。
  Future<String?> getBlockHash(int blockNumber) async {
    _ensureReady();
    await ensureSynced();
    return _withRetry('getBlockHash', () => _chain!.getBlockHash(blockNumber));
  }

  /// 原生读取指定区块中的 extrinsics。
  Future<List<String>> getBlockExtrinsics(String blockHashHex) async {
    _ensureReady();
    await ensureSynced();
    return _withRetry(
        'getBlockExtrinsics', () => _chain!.getBlockExtrinsics(blockHashHex));
  }

  /// 原生提交已编码 extrinsic。
  Future<String?> submitExtrinsicHex(String extrinsicHex) async {
    _ensureReady();
    await ensureSynced();
    await _waitForPeer();
    return _withRetry(
        'submitExtrinsic', () => _chain!.submitExtrinsicHex(extrinsicHex));
  }

  /// 原生读取 `System.Account` 快照，供钱包余额迁移使用。
  Future<SystemAccountSnapshot?> getSystemAccountSnapshot(
      String accountIdHex) async {
    _ensureReady();
    await ensureSynced();
    return _withRetry(
        'getSystemAccount', () => _chain!.getSystemAccount(accountIdHex));
  }

  /// 原生读取单个 storage value（hex）。
  Future<String?> getStorageValueHex(String storageKeyHex) async {
    _ensureReady();
    await ensureSynced();
    return _withRetry(
        'getStorageValue', () => _chain!.getStorageValueHex(storageKeyHex));
  }

  /// 原生批量读取多个 storage value（hex）。
  Future<Map<String, String?>> getStorageValuesHex(
      List<String> storageKeyHexList) async {
    if (storageKeyHexList.isEmpty) {
      return const {};
    }
    _ensureReady();
    await ensureSynced();
    return _withRetry('getStorageValues',
        () => _chain!.getStorageValuesHex(storageKeyHexList));
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
    _healthStatus = ChainHealthStatus.uninitialized;
    _lastError = null;
    debugPrint('[Smoldot] 轻节点已关闭');
  }

  void _ensureReady() {
    if (!isReady) {
      throw StateError('smoldot 轻节点未初始化，请先调用 initialize()');
    }
  }
}
