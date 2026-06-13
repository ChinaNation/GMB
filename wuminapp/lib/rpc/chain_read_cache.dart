import 'dart:async';

import 'package:flutter/foundation.dart';

/// finalized storage 读取的进程级共享缓存(ADR-018 §三-2 / 卡⑤)。
///
/// 设计:
/// - **命名空间 = finalizedBlockHash**。同一 finalized 块内链上状态不可变,故块内
///   缓存零陈旧;换块即整体失效。GMB 约 6 分钟出块,块内复用收益很大。
/// - **单例**:ChainRpc 在各处 `new`,缓存必须进程级共享才能跨页面/跨服务去重。
/// - **in-flight 合并**:同一 key 的并发请求只下沉一次。
/// - **负缓存**:key 缺失(账户不存在)同样缓存,块内不重复下沉。
/// - **失效驱动**:① ChainTxMonitor 收到新 finalized 头时调 [invalidate]()(即时);
///   ② [read] 内 finalizedHash 门控复查([recheckInterval] 兜底,防订阅缺位)。
/// - **豁免**:本缓存只服务 finalized 状态读;交易提交管线(nonce/dry-run/submit/
///   runtimeVersion/genesis)走各自原生调用,本就不经此层(见 ChainRpc 注释)。
///   余额守卫等需绝对最新的场景用 `forceFresh` 旁路。
class ChainReadCache {
  ChainReadCache._();

  /// 进程级唯一实例。
  static final ChainReadCache instance = ChainReadCache._();

  /// finalizedHash 门控复查间隔(兜底;ChainTxMonitor 即时 [invalidate] 为主)。
  @visibleForTesting
  Duration recheckInterval = const Duration(seconds: 15);

  String? _namespaceHash;
  DateTime? _namespaceCheckedAt;
  final Map<String, Uint8List?> _entries = {};
  final Map<String, Future<Uint8List?>> _inflight = {};

  /// 经缓存读取一批 storage key,返回 key → 解码后的值(缺失为 null)。
  ///
  /// - [finalizedHashProvider] 返回当前 finalized 块哈希(命名空间)。返回 null
  ///   时沿用上次命名空间(不误清缓存)。门控间隔内不会重复调用。
  /// - [fetchMissing] 只会收到未命中的 key,返回解码后的值(key 缺失→null)。
  /// - [forceFresh] 跳过缓存强制下沉,并刷新这批 key 的缓存。
  Future<Map<String, Uint8List?>> read(
    List<String> keys, {
    required Future<String?> Function() finalizedHashProvider,
    required Future<Map<String, Uint8List?>> Function(List<String> misses)
        fetchMissing,
    bool forceFresh = false,
    @visibleForTesting DateTime? now,
  }) async {
    if (keys.isEmpty) return {};
    await _refreshNamespace(finalizedHashProvider, now ?? DateTime.now());

    final uniqueKeys = keys.toSet();
    if (forceFresh) {
      for (final k in uniqueKeys) {
        _entries.remove(k);
        _inflight.remove(k);
      }
    }

    // 既不在 entries 也不在 inflight 的 key 才需要新发起一次批量下沉。
    final toFetch = uniqueKeys
        .where((k) => !_entries.containsKey(k) && !_inflight.containsKey(k))
        .toList(growable: false);

    if (toFetch.isNotEmpty) {
      final batch = fetchMissing(toFetch);
      for (final k in toFetch) {
        _inflight[k] = batch.then((m) => m[k]);
      }
      // 批量完成后落地 entries 并清 inflight;失败只清 inflight(下次可重试)。
      unawaited(batch.then(
        (m) {
          for (final k in toFetch) {
            _entries[k] = m[k];
            _inflight.remove(k);
          }
        },
        onError: (Object _) {
          for (final k in toFetch) {
            _inflight.remove(k);
          }
        },
      ));
    }

    final result = <String, Uint8List?>{};
    await Future.wait(uniqueKeys.map((k) async {
      if (_entries.containsKey(k)) {
        result[k] = _entries[k];
        return;
      }
      final pending = _inflight[k];
      result[k] = pending == null ? _entries[k] : await pending;
    }));
    return result;
  }

  Future<void> _refreshNamespace(
    Future<String?> Function() finalizedHashProvider,
    DateTime now,
  ) async {
    final checkedAt = _namespaceCheckedAt;
    if (checkedAt != null && now.difference(checkedAt) < recheckInterval) {
      return;
    }
    String? hash;
    try {
      hash = await finalizedHashProvider();
    } catch (_) {
      hash = null;
    }
    _namespaceCheckedAt = now;
    if (hash == null || hash.isEmpty) return; // 取不到 → 沿用上次命名空间
    if (hash != _namespaceHash) {
      _namespaceHash = hash;
      _entries.clear();
      _inflight.clear();
    }
  }

  /// 立即失效整层缓存。ChainTxMonitor 收到新 finalized 头时调用,
  /// 让换块后的读取立刻拿到最新 finalized 状态(无需等门控复查)。
  void invalidate() {
    _entries.clear();
    _inflight.clear();
    _namespaceHash = null;
    _namespaceCheckedAt = null;
  }
}
