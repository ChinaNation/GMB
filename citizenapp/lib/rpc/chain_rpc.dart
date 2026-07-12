import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:polkadart/polkadart.dart'
    show Hasher, RuntimeMetadata, RuntimeVersion;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:smoldot/smoldot.dart' show LightClientStatusSnapshot;

import 'chain_read_cache.dart';
import 'signed_extrinsic_relay_api.dart';
import 'smoldot_client.dart';

/// 交易池观察状态。
///
/// submitExtrinsic 返回 txHash 只代表已提交到 RPC，不能代表已出块。
/// 业务页面可订阅这里的状态，把 future/invalid/dropped 等失败原因回显给用户。
enum TxPoolWatchKind {
  ready,
  broadcast,
  inBlock,
  finalized,
  future,
  invalid,
  dropped,
  usurped,
  retracted,
  finalityTimeout,
  timeout,
  error,
  unknown,
}

class TxPoolWatchEvent {
  const TxPoolWatchEvent({
    required this.kind,
    required this.description,
    required this.raw,
    this.blockHashHex,
  });

  final TxPoolWatchKind kind;
  final String description;
  final String raw;
  final String? blockHashHex;

  bool get isFailure {
    switch (kind) {
      case TxPoolWatchKind.future:
      case TxPoolWatchKind.invalid:
      case TxPoolWatchKind.dropped:
      case TxPoolWatchKind.usurped:
      case TxPoolWatchKind.retracted:
      case TxPoolWatchKind.finalityTimeout:
      case TxPoolWatchKind.timeout:
      case TxPoolWatchKind.error:
        return true;
      case TxPoolWatchKind.ready:
      case TxPoolWatchKind.broadcast:
      case TxPoolWatchKind.inBlock:
      case TxPoolWatchKind.finalized:
      case TxPoolWatchKind.unknown:
        return false;
    }
  }

  bool get isIncluded =>
      kind == TxPoolWatchKind.inBlock || kind == TxPoolWatchKind.finalized;
}

class ChainExtrinsicFailure {
  const ChainExtrinsicFailure({
    required this.moduleIndex,
    required this.errorIndex,
    required this.description,
  });

  final int moduleIndex;
  final int errorIndex;
  final String description;
}

typedef TxPoolWatchCallback = void Function(TxPoolWatchEvent event);

/// citizenchain RPC 客户端。
///
/// 只使用 smoldot 轻节点（P2P 网络，无需远程 RPC 服务器）。
class ChainRpc {
  ChainRpc() {
    debugPrint('[ChainRpc] 使用 smoldot 轻节点模式');
  }

  static final _keyring = Keyring();

  // ──── 批量查询 ────

  /// 批量查询多个 storage key，一次原生调用返回所有结果。
  ///
  /// (ADR-017 全端 finalized 单一口径)：本入口及所有 fetch* 状态
  /// 读取一律返回 finalized 块上的状态——业务/展示读取禁止使用 best，
  /// 平名即 finalized 是全 App 唯一约定。交易构造与提交管线的豁免接口
  /// (nonce/runtime version/genesis/metadata/fetchLatestBlock)另列。
  ///
  /// (ADR-018 §九 / 卡⑤)：所有 finalized 状态读取都汇入本入口,
  /// 故缓存挂在这里即覆盖余额 / storage / 反查 / 多签扫描全部读取(`ChainReadCache`,
  /// 按 finalizedBlockHash 命名空间,块内复用)。豁免管线走各自原生调用不经本入口,
  /// 结构性免于被缓存;需绝对最新的场景传 [forceFresh]。
  Future<Map<String, Uint8List?>> fetchStorageBatch(
    List<String> storageKeyHexList, {
    bool forceFresh = false,
  }) async {
    if (storageKeyHexList.isEmpty) return {};
    return ChainReadCache.instance.read(
      storageKeyHexList,
      finalizedHashProvider: () async =>
          (await SmoldotClientManager.instance.getStatusSnapshot())
              .currentVerifiedFinalizedBlockHash,
      fetchMissing: _rawFetchFinalizedStorage,
      forceFresh: forceFresh,
    );
  }

  /// 真正下沉到 smoldot 的 finalized 批量读取(仅 [ChainReadCache] 未命中时调用)。
  Future<Map<String, Uint8List?>> _rawFetchFinalizedStorage(
      List<String> storageKeyHexList) async {
    final rawMap = await SmoldotClientManager.instance
        .getFinalizedStorageValuesHex(storageKeyHexList);
    final result = <String, Uint8List?>{};
    for (final key in storageKeyHexList) {
      final valueHex = rawMap[key];
      result[key] = valueHex == null
          ? null
          : _hexDecode(
              valueHex.startsWith('0x') ? valueHex.substring(2) : valueHex);
    }
    return result;
  }

  /// 分块批量查询多个 storage key，避免单次请求 payload 过大。
  ///
  /// 多签列表会按 storage 依赖分阶段拼出大量 key；这里统一限制
  /// 每批大小，让业务层得到批量收益，同时不给 smoldot / 全节点制造尖峰。
  Future<Map<String, Uint8List?>> fetchStorageBatchChunked(
    Iterable<String> storageKeyHexList, {
    int chunkSize = 100,
  }) async {
    final keys = storageKeyHexList.toSet().toList(growable: false);
    if (keys.isEmpty) return {};
    if (chunkSize <= 0) {
      throw ArgumentError.value(chunkSize, 'chunkSize', '必须大于 0');
    }

    final result = <String, Uint8List?>{};
    for (var start = 0; start < keys.length; start += chunkSize) {
      final end = (start + chunkSize).clamp(0, keys.length);
      final chunk = keys.sublist(start, end);
      result.addAll(await fetchStorageBatch(chunk));
    }
    return result;
  }

  // ──── 转账相关 RPC ────

  /// 查询 runtime `frame_system::Account.nonce` 给出的账户 nonce。
  ///
  /// CitizenApp 不缓存、不预占、不自增 nonce；每次签名前都
  /// 通过原生 runtime call 读取当前 nonce，并把该值交给 signed extrinsic。
  Future<int> fetchNonce(String ss58Address) async {
    // 轻节点模式先在 Dart 侧解出 accountId，再交给原生 runtime call，避免继续依赖 legacy `system_accountNextIndex`。
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
    // 轻节点模式优先走原生 capability，避免业务层继续直接依赖裸 RPC 方法名。
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

  /// 获取轻节点当前同步进度（同步中也可读）。
  ///
  /// 仅用于 UI 展示和诊断，不应用于交易构造或需要最新状态一致性的逻辑。
  Future<LightClientStatusSnapshot> fetchChainProgress() async {
    return SmoldotClientManager.instance.getStatusSnapshotRaw();
  }

  /// 获取最新(best)区块的哈希和块号。
  ///
  /// (ADR-017 豁免区)：仅限交易/支付凭证构造内部使用(如离线支付
  /// payload 携带当前块高)，业务/展示读取禁止调用本方法——那些一律走
  /// finalized 口径。signed extrinsic 构造按 P-SIGN-001 固定 immortal era，
  /// 不得用最新块参与 CheckEra。
  Future<({Uint8List blockHash, int blockNumber})> fetchLatestBlock() async {
    // 轻节点模式直接复用原生状态快照，减少一次 `chain_getHeader` 往返。
    final snapshot = await SmoldotClientManager.instance.getStatusSnapshot();
    final hashHex = snapshot.bestBlockHash;
    final blockNumber = snapshot.bestBlockNumber;
    if (hashHex == null || hashHex.isEmpty || blockNumber == null) {
      throw StateError('smoldot 轻节点尚未提供最新区块快照');
    }
    return (
      blockHash: _hexDecode(_stripHexPrefix(hashHex)),
      blockNumber: blockNumber,
    );
  }

  /// 获取最新 finalized 区块的哈希和块号。
  ///
  /// 钱包交易流水的“已确认”只能来自 finalized 高度；best/latest
  /// 只代表当前最优链头，不能用来升级 `finalized` 状态。普通订阅视图可能在
  /// warp 收口前已经显示 F，因此这里只接受原生完整 chain information 对应的
  /// `currentVerifiedFinalized`。
  Future<({Uint8List blockHash, int blockNumber})> fetchFinalizedBlock() async {
    final snapshot = await SmoldotClientManager.instance.getStatusSnapshot();
    if (!snapshot.isUsable) {
      throw StateError('smoldot 轻节点尚未提供完整验证的 finalized 区块快照');
    }
    final hashHex = snapshot.currentVerifiedFinalizedBlockHash;
    final blockNumber = snapshot.currentVerifiedFinalizedBlockNumber;
    return (
      blockHash: _hexDecode(_stripHexPrefix(hashHex)),
      blockNumber: blockNumber,
    );
  }

  // 钱包交易流水由区块事件监听写入,不逐块拉 body 求 extrinsic 哈希
  // (逐块拉 body 会触发 substrate block-request 反滥用 ban 把轻节点打死)。

  /// 获取运行时 metadata（含 registry，用于 extrinsic 编码）。结果缓存。
  Future<RuntimeMetadata> fetchMetadata() async {
    if (_cachedMetadata != null) return _cachedMetadata!;

    // 轻节点模式直接读取原生 metadata hex，避免 Dart 层再拼 `state_getMetadata`。
    final metadataHex = await SmoldotClientManager.instance.getMetadataHex();
    if (metadataHex == null || metadataHex.isEmpty) {
      throw StateError('smoldot 轻节点尚未提供 metadata');
    }
    _cachedMetadata = RuntimeMetadata.fromHex(metadataHex);
    return _cachedMetadata!;
  }

  RuntimeMetadata? _cachedMetadata;

  /// 提交已签名的 extrinsic,返回交易哈希(32 字节)。
  ///
  /// **设计**(submit-only + 后台监听):
  /// - 主流程仅调原生 `submitExtrinsicHex`(底层走 `author_submitExtrinsic`),
  ///   拿到 txHash 立即返回,UI 永不卡住。
  /// - 后台 fire-and-forget 启一条 `author_submitAndWatchExtrinsic` 订阅,
  ///   8 秒内观察到 invalid/dropped/usurped/future 仅打印日志,不回灌 UI。
  ///
  /// 客户端不拦截 mempool reject(smoldot native binding 转发首条 event 有调度
  /// 延迟,在 6 分钟出块节奏下经常拿不到 txHash 误判失败);reject 排查走
  /// polkadot.js apps + 终端日志。
  Future<Uint8List> submitExtrinsic(
    Uint8List encoded, {
    TxPoolWatchCallback? onWatchEvent,
  }) async {
    final hex = '0x${_hexEncode(encoded)}';
    // 完整 extrinsic hex 用 debugPrint 输出,便于直接复制到 polkadot.js apps
    // "Tools → Decode" 验证编码(call/signer/nonce/era 等);旧版仅前 80 字符,
    // 排查"提交看似成功但链上没出块"时不够用。
    debugPrint(
        '[ChainRpc.submitExtrinsic] 提交 extrinsic (${encoded.length} bytes)');
    debugPrint('[ChainRpc.submitExtrinsic] full hex: $hex');

    try {
      final txHashHex =
          await SmoldotClientManager.instance.submitExtrinsicHex(hex);
      if (txHashHex == null || txHashHex.isEmpty) {
        throw StateError('smoldot 未返回交易哈希');
      }
      debugPrint('[ChainRpc.submitExtrinsic] smoldot 返回 txHash: $txHashHex');

      unawaited(_watchTxRejectInBackground(hex, txHashHex, onWatchEvent));
      return _hexDecode(_stripHexPrefix(txHashHex));
    } catch (error) {
      if (!_shouldRelaySignedExtrinsic(error)) {
        rethrow;
      }
      return _relaySignedExtrinsicAfterSmoldotFailure(
        hex,
        error,
        onWatchEvent,
      );
    }
  }

  Future<Uint8List> _relaySignedExtrinsicAfterSmoldotFailure(
    String signedExtrinsicHex,
    Object smoldotError,
    TxPoolWatchCallback? onWatchEvent,
  ) async {
    final manifest = SmoldotClientManager.instance.lastBootstrapManifest;
    if (manifest?.services.signedExtrinsicRelayEnabled != true ||
        manifest?.services.signedExtrinsicRelayPath !=
            SignedExtrinsicRelayApi.relayPath) {
      throw smoldotError;
    }

    debugPrint(
      '[ChainRpc.submitExtrinsic] 轻节点提交失败，尝试已签名交易受控广播兜底: $smoldotError',
    );
    final api = SignedExtrinsicRelayApi();
    try {
      final result = await api.relaySignedExtrinsic(
        signedExtrinsicHex: signedExtrinsicHex,
      );
      onWatchEvent?.call(TxPoolWatchEvent(
        kind: TxPoolWatchKind.broadcast,
        description: '已通过受控 API 广播，最终成功以 finalized 链状态或事件为准',
        raw: 'signed_extrinsic_relay:${result.relayId}',
      ));
      debugPrint(
        '[ChainRpc.submitExtrinsic] 受控广播返回 txHash=${result.txHash}, relay=${result.relayId}',
      );
      return _hexDecode(_stripHexPrefix(result.txHash));
    } catch (relayError) {
      throw StateError(
        '轻节点提交失败且受控广播兜底失败: $relayError; 原始错误: $smoldotError',
      );
    } finally {
      api.close();
    }
  }

  bool _shouldRelaySignedExtrinsic(Object error) {
    final manifest = SmoldotClientManager.instance.lastBootstrapManifest;
    if (manifest?.services.signedExtrinsicRelayEnabled != true) {
      return false;
    }
    final raw = error.toString().toLowerCase();
    if (raw.contains('invalid transaction') ||
        raw.contains('bad proof') ||
        raw.contains('exhausts resources') ||
        raw.contains('payment') ||
        raw.contains('future') ||
        raw.contains('stale')) {
      return false;
    }
    final status = SmoldotClientManager.instance.healthStatus;
    if (status == ChainHealthStatus.offline ||
        status == ChainHealthStatus.degraded) {
      return true;
    }
    return raw.contains('timeout') ||
        raw.contains('timed out') ||
        raw.contains('socketexception') ||
        raw.contains('failed host lookup') ||
        raw.contains('network is unreachable') ||
        raw.contains('connection refused') ||
        raw.contains('channel closed') ||
        raw.contains('no node') ||
        raw.contains('peers') ||
        raw.contains('inaccessible');
  }

  /// 提交已签名 extrinsic，并阻塞等待交易真正进入区块。
  ///
  /// 提案类交易不能把 txHash 当成功；只有 `inBlock/finalized`
  /// 状态携带区块哈希后，业务层才能继续读取 `System.Events` 核对事件。
  Future<({Uint8List txHash, TxPoolWatchEvent included})>
      submitExtrinsicAndWaitForInBlock(
    Uint8List encoded, {
    TxPoolWatchCallback? onWatchEvent,
    Duration timeout = const Duration(minutes: 20),
  }) async {
    final hex = '0x${_hexEncode(encoded)}';
    final txHash = Hasher.blake2b256.hash(encoded);
    final txHashHex = '0x${_hexEncode(txHash)}';
    debugPrint(
        '[ChainRpc.submitExtrinsicAndWaitForInBlock] 提交 extrinsic (${encoded.length} bytes), txHash=$txHashHex');
    debugPrint('[ChainRpc.submitExtrinsicAndWaitForInBlock] full hex: $hex');

    StreamSubscription? sub;
    Timer? bailTimer;
    final done = Completer<TxPoolWatchEvent>();
    try {
      final stream = SmoldotClientManager.instance
          .subscribe('author_submitAndWatchExtrinsic', [hex]);
      bailTimer = Timer(timeout, () {
        if (!done.isCompleted) {
          done.completeError(TimeoutException(
            '交易 $txHashHex 在 ${timeout.inMinutes} 分钟内未进入区块',
            timeout,
          ));
        }
      });
      sub = stream.listen(
        (event) {
          try {
            final dynamic raw = (event as dynamic).result;
            final watchEvent = _toWatchEvent(raw);
            onWatchEvent?.call(watchEvent);
            debugPrint(
                '[ChainRpc.submitExtrinsicAndWaitForInBlock] $txHashHex status=$raw');
            if (watchEvent.isFailure && !done.isCompleted) {
              done.completeError(StateError(watchEvent.description));
              return;
            }
            if (watchEvent.isIncluded && !done.isCompleted) {
              final blockHashHex = watchEvent.blockHashHex;
              if (blockHashHex == null || blockHashHex.isEmpty) {
                done.completeError(StateError('交易已入块，但订阅状态未返回区块哈希'));
                return;
              }
              done.complete(watchEvent);
            }
          } catch (e) {
            if (!done.isCompleted) done.completeError(e);
          }
        },
        onError: (Object e) {
          onWatchEvent?.call(TxPoolWatchEvent(
            kind: TxPoolWatchKind.error,
            description: '交易池订阅异常：$e',
            raw: '$e',
          ));
          if (!done.isCompleted) done.completeError(e);
        },
        onDone: () {
          if (!done.isCompleted) {
            done.completeError(StateError('交易池订阅已结束，但交易未进入区块'));
          }
        },
      );

      final included = await done.future;
      return (txHash: txHash, included: included);
    } finally {
      bailTimer?.cancel();
      if (sub != null) unawaited(sub.cancel());
    }
  }

  /// 后台观察一条交易的交易池状态,**所有状态都打日志**。
  ///
  /// 普通钱包转账需要用 inBlock 回调把本机流水从 pending 升级为
  /// inBlock，因此监听窗口要覆盖正常出块周期；被拒、已入块或超时都会结束。
  Future<void> _watchTxRejectInBackground(
    String hex,
    String txHashHex,
    TxPoolWatchCallback? onWatchEvent,
  ) async {
    StreamSubscription? sub;
    Timer? bailTimer;
    try {
      final stream = SmoldotClientManager.instance
          .subscribe('author_submitAndWatchExtrinsic', [hex]);
      final done = Completer<void>();
      var sawAnyStatus = false;
      bailTimer = Timer(const Duration(minutes: 20), () {
        if (!done.isCompleted) {
          if (!sawAnyStatus) {
            onWatchEvent?.call(const TxPoolWatchEvent(
              kind: TxPoolWatchKind.timeout,
              description: '20 分钟内未收到交易池状态，可能转发失败或交易被静默丢弃',
              raw: 'timeout',
            ));
            debugPrint(
                '[ChainRpc.bgWatch] $txHashHex 20m timeout 未收到任何状态,可能 smoldot 转发失败或全节点静默 drop');
          } else {
            debugPrint('[ChainRpc.bgWatch] $txHashHex 20m 后结束后台监听,后续交由业务真源确认');
          }
          done.complete();
        }
      });
      sub = stream.listen(
        (event) {
          try {
            final dynamic raw = (event as dynamic).result;
            sawAnyStatus = true;
            final cls = _classifyTxStatus(raw);
            final watchEvent = _toWatchEvent(raw);
            onWatchEvent?.call(watchEvent);
            debugPrint(
                '[ChainRpc.bgWatch] $txHashHex status=$raw classify=$cls');
            if (cls == _TxResult.failure) {
              debugPrint(
                  '[ChainRpc.bgWatch] $txHashHex 被拒绝: ${_describeTxStatus(raw)}');
              if (!done.isCompleted) done.complete();
            } else if (watchEvent.isIncluded) {
              debugPrint('[ChainRpc.bgWatch] $txHashHex 已入块，结束后台监听');
              if (!done.isCompleted) done.complete();
            }
          } catch (e) {
            debugPrint('[ChainRpc.bgWatch] event 解析异常: $e');
          }
        },
        onError: (Object e) {
          onWatchEvent?.call(TxPoolWatchEvent(
            kind: TxPoolWatchKind.error,
            description: '交易池订阅异常：$e',
            raw: '$e',
          ));
          debugPrint('[ChainRpc.bgWatch] $txHashHex stream error: $e');
          if (!done.isCompleted) done.complete();
        },
      );
      await done.future;
    } catch (e) {
      debugPrint('[ChainRpc.bgWatch] 整体异常: $e');
    } finally {
      bailTimer?.cancel();
      // sub.cancel() 不能 await(smoldot native binding 在持续推送 events 期间
      // 可能阻塞调用线程),fire-and-forget 让本协程立即结束。
      if (sub != null) unawaited(sub.cancel());
    }
  }

  /// 把 TransactionStatus(JSON 形式)归三类:成功 / 失败 / 仍在等待。
  ///
  /// 仅 [_watchTxRejectInBackground] 使用:主流程只关心 failure 一种。
  _TxResult _classifyTxStatus(dynamic status) {
    if (status is String) {
      switch (status) {
        case 'ready':
        case 'broadcast':
          return _TxResult.success;
        case 'future':
        case 'invalid':
        case 'dropped':
        case 'finalityTimeout':
          return _TxResult.failure;
        default:
          return _TxResult.pending;
      }
    }
    if (status is Map) {
      if (status.containsKey('inBlock') ||
          status.containsKey('finalized') ||
          status.containsKey('broadcast')) {
        return _TxResult.success;
      }
      if (status.containsKey('future') ||
          status.containsKey('invalid') ||
          status.containsKey('dropped') ||
          status.containsKey('usurped') ||
          status.containsKey('retracted') ||
          status.containsKey('finalityTimeout')) {
        return _TxResult.failure;
      }
    }
    return _TxResult.pending;
  }

  TxPoolWatchEvent _toWatchEvent(dynamic status) {
    if (status is String) {
      final kind = switch (status) {
        'ready' => TxPoolWatchKind.ready,
        'broadcast' => TxPoolWatchKind.broadcast,
        'future' => TxPoolWatchKind.future,
        'invalid' => TxPoolWatchKind.invalid,
        'dropped' => TxPoolWatchKind.dropped,
        'finalityTimeout' => TxPoolWatchKind.finalityTimeout,
        _ => TxPoolWatchKind.unknown,
      };
      return TxPoolWatchEvent(
        kind: kind,
        description: _describeTxStatus(status),
        raw: status,
      );
    }
    if (status is Map) {
      TxPoolWatchKind kind = TxPoolWatchKind.unknown;
      String? blockHashHex;
      if (status.containsKey('inBlock')) {
        kind = TxPoolWatchKind.inBlock;
        blockHashHex = _statusHashHex(status['inBlock']);
      } else if (status.containsKey('finalized')) {
        kind = TxPoolWatchKind.finalized;
        blockHashHex = _statusHashHex(status['finalized']);
      } else if (status.containsKey('broadcast')) {
        kind = TxPoolWatchKind.broadcast;
      } else if (status.containsKey('future')) {
        kind = TxPoolWatchKind.future;
      } else if (status.containsKey('invalid')) {
        kind = TxPoolWatchKind.invalid;
      } else if (status.containsKey('dropped')) {
        kind = TxPoolWatchKind.dropped;
      } else if (status.containsKey('usurped')) {
        kind = TxPoolWatchKind.usurped;
      } else if (status.containsKey('retracted')) {
        kind = TxPoolWatchKind.retracted;
      } else if (status.containsKey('finalityTimeout')) {
        kind = TxPoolWatchKind.finalityTimeout;
      }
      return TxPoolWatchEvent(
        kind: kind,
        description: _describeTxStatus(status),
        raw: '$status',
        blockHashHex: blockHashHex,
      );
    }
    return TxPoolWatchEvent(
      kind: TxPoolWatchKind.unknown,
      description: '$status',
      raw: '$status',
    );
  }

  /// 把 TransactionStatus 转成可读 reject 原因(仅后台日志使用)。
  String _describeTxStatus(dynamic status) {
    if (status is String) {
      switch (status) {
        case 'invalid':
          return '交易无效(可能 nonce 重复 / 余额不足 / 签名无效 / SignedExtension 校验失败)';
        case 'dropped':
          return '被交易池剔除(mempool 已满或优先级过低)';
        case 'future':
          return 'nonce 大于链上已确认值,交易暂留(等待前序交易确认)';
        case 'finalityTimeout':
          return '最终化超时';
      }
      return status;
    }
    if (status is Map) {
      if (status.containsKey('usurped')) {
        return '被同 nonce 的另一笔交易顶替(usurped)';
      }
      if (status.containsKey('retracted')) {
        return '所在区块被 retracted';
      }
    }
    return '$status';
  }

  String? _statusHashHex(dynamic value) {
    if (value == null) return null;
    final text = '$value';
    if (text.isEmpty) return null;
    return text.startsWith('0x') ? text : '0x$text';
  }

  // ──── 链上状态查询 ────

  /// 通用 storage 查询：传入完整的 storage key（含 0x 前缀），
  /// 返回 finalized 块上的原始 SCALE 编码字节。key 不存在时返回 null。
  ///
  /// (ADR-017)：业务/展示读取唯一口径 = finalized，禁止 best。
  Future<Uint8List?> fetchStorage(String storageKeyHex) async {
    final valueHex = await SmoldotClientManager.instance
        .getFinalizedStorageValueHex(storageKeyHex);
    if (valueHex == null) return null;
    return _hexDecode(
      valueHex.startsWith('0x') ? valueHex.substring(2) : valueHex,
    );
  }

  /// 链是否可达且已同步到 finalized 区块。
  ///
  /// 读 `System.Number`（任意已同步链上必然存在的 plain StorageValue）：能读到即
  /// 证明链可用。用于把「链不可达 / 未同步」与「链上确认不存在」区分开——离线时
  /// 一律返回 false，避免把本机记录误判成链上幽灵而误删。
  Future<bool> isFinalizedChainReachable() async {
    try {
      final keyHex =
          '0x${_hexEncode(_buildStorageValueKey('System', 'Number'))}';
      final data = await fetchStorage(keyHex);
      return data != null && data.isNotEmpty;
    } catch (_) {
      return false;
    }
  }

  /// 查询指定区块的 `System.Events` 原始 SCALE 数据。
  Future<Uint8List?> fetchSystemEventsAtBlock(String blockHashHex) async {
    final keyHex = '0x${_hexEncode(_buildStorageValueKey('System', 'Events'))}';
    final valueHex = await SmoldotClientManager.instance.request(
      'state_getStorage',
      [keyHex, blockHashHex],
    ) as String?;
    if (valueHex == null || valueHex.isEmpty) return null;
    return _hexDecode(_stripHexPrefix(valueHex));
  }

  /// 从 `System.Events` 中提取本区块的 `System.ExtrinsicFailed` 模块错误。
  ///
  /// 创建类交易已经入块但业务事件缺失时，必须优先回显真实
  /// DispatchError，不能再把“未找到成功事件”误报成原因。
  ChainExtrinsicFailure? findExtrinsicFailureInEvents(Uint8List data) {
    final (_, countSize) = _decodeCompact(data, 0);
    if (countSize <= 0 || countSize >= data.length) return null;

    for (var offset = countSize; offset + 9 <= data.length; offset++) {
      final phase = data[offset];
      var eventOffset = -1;
      if (phase == 0x00) {
        // Phase::ApplyExtrinsic(u32)
        eventOffset = offset + 5;
      } else if (phase == 0x01 || phase == 0x02) {
        // Phase::Finalization / Initialization
        eventOffset = offset + 1;
      }
      if (eventOffset < 0 || eventOffset + 8 > data.length) continue;
      final palletIndex = data[eventOffset];
      final eventIndex = data[eventOffset + 1];
      if (palletIndex != 0 || eventIndex != 1) continue;

      final payloadOffset = eventOffset + 2;
      // DispatchError::Module = 0x03 + pallet_index + [error; 4]
      if (payloadOffset + 6 > data.length || data[payloadOffset] != 0x03) {
        continue;
      }
      final moduleIndex = data[payloadOffset + 1];
      final errorIndex = data[payloadOffset + 2];
      return ChainExtrinsicFailure(
        moduleIndex: moduleIndex,
        errorIndex: errorIndex,
        description: _describeRuntimeModuleError(moduleIndex, errorIndex),
      );
    }
    return null;
  }

  /// 查询 finalized 块上的链上余额(free，yuan)。账户不存在返回 0.0。
  ///
  /// (ADR-018 卡⑤)：改走 [fetchFinalizedBalances] 单元素路径,与批量
  /// 余额共用 System.Account storage 读 + ChainReadCache 块内缓存;口径仍是 free,
  /// 与原 getFinalizedSystemAccountSnapshot().freeYuan 等价。[forceFresh] 旁路缓存
  /// (转账前余额守卫用,确保拿到最新 finalized 状态)。
  Future<double> fetchFinalizedBalance(
    String pubkeyHex, {
    bool forceFresh = false,
  }) async {
    final balances =
        await fetchFinalizedBalances([pubkeyHex], forceFresh: forceFresh);
    return balances[pubkeyHex] ?? 0.0;
  }

  /// 查询链上真实余额 = free + reserved,best 视图。
  ///
  ///
  /// - 对齐 polkadot.js apps 的 total 余额口径;钱包详情页第 3 张卡片展示
  ///   的就是这个值,不能只取 free,否则锁仓 / 质押的 reserved 部分会漏算。
  /// - 走通用 `fetchStorageBatch` 取 `System.Account` 原始 bytes,在 Dart 侧
  ///   自行解码 AccountData 的 free + reserved,绕过原生 SystemAccountSnapshot
  ///   当前只暴露 freeFen 字段的限制。
  /// - 账户不存在或数据不完整均返回 0.0。
  /// 查询 finalized 块上的真实余额 = free + reserved。
  Future<double> fetchFinalizedTotalBalance(String pubkeyHex) async {
    final accountId = _hexDecode(
        pubkeyHex.startsWith('0x') ? pubkeyHex.substring(2) : pubkeyHex);
    final blake2 = Hasher.blake2b128.hash(accountId);
    final fullKey = Uint8List(
        _systemAccountPrefix.length + blake2.length + accountId.length);
    fullKey.setAll(0, _systemAccountPrefix);
    fullKey.setAll(_systemAccountPrefix.length, blake2);
    fullKey.setAll(_systemAccountPrefix.length + blake2.length, accountId);
    final keyHex = '0x${_hexEncode(fullKey)}';

    final batchResult = await fetchStorageBatch([keyHex]);
    final data = batchResult[keyHex];
    return _decodeTotalBalanceFromAccountData(data);
  }

  /// 从 System.Account 的 SCALE 编码数据中解码 free + reserved 总余额(yuan)。
  ///
  ///
  /// AccountInfo 布局:
  ///   nonce(u32, 4 字节) + consumers(u32, 4 字节) + providers(u32, 4 字节)
  ///   + sufficients(u32, 4 字节) = 16 字节头;
  /// 紧接着 AccountData:
  ///   free(u128, offset 16, 16 字节 little-endian)
  ///   reserved(u128, offset 32, 16 字节 little-endian)
  /// data 为 null 或长度 < 48 返回 0.0(账户不存在 / 数据不完整)。
  static double _decodeTotalBalanceFromAccountData(Uint8List? data) {
    if (data == null || data.length < 48) return 0.0;
    BigInt readU128(int offset) {
      var value = BigInt.zero;
      for (var i = 0; i < 16; i++) {
        value += BigInt.from(data[offset + i]) << (i * 8);
      }
      return value;
    }

    final free = readU128(16);
    final reserved = readU128(32);
    final totalFen = free + reserved;
    return totalFen.toDouble() / 100.0;
  }

  /// System.Account storage key 前缀（twox128("System") + twox128("Account")）。
  static final Uint8List _systemAccountPrefix = _hexDecode(
      '26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da9');

  /// 批量查询多个账户在 best 视图上的链上余额，返回 pubkeyHex → yuan 的映射。
  ///
  /// 一次 storage proof 请求查询所有账户，比逐个调用 [fetchBalance] 更高效。
  /// 账户不存在时对应值为 0.0。
  /// 批量查询多个账户在 finalized 块上的链上余额。
  Future<Map<String, double>> fetchFinalizedBalances(
    List<String> pubkeyHexList, {
    bool forceFresh = false,
  }) async {
    if (pubkeyHexList.isEmpty) return {};

    final keyToPubkey = <String, String>{};
    final storageKeys = <String>[];
    for (final pubkeyHex in pubkeyHexList) {
      final accountId = _hexDecode(
          pubkeyHex.startsWith('0x') ? pubkeyHex.substring(2) : pubkeyHex);
      final blake2 = Hasher.blake2b128.hash(accountId);
      final fullKey = Uint8List(
          _systemAccountPrefix.length + blake2.length + accountId.length);
      fullKey.setAll(0, _systemAccountPrefix);
      fullKey.setAll(_systemAccountPrefix.length, blake2);
      fullKey.setAll(_systemAccountPrefix.length + blake2.length, accountId);
      final keyHex = '0x${_hexEncode(fullKey)}';
      storageKeys.add(keyHex);
      keyToPubkey[keyHex] = pubkeyHex;
    }

    final batchResult =
        await fetchStorageBatch(storageKeys, forceFresh: forceFresh);

    final balances = <String, double>{};
    for (final entry in keyToPubkey.entries) {
      final data = batchResult[entry.key];
      balances[entry.value] = _decodeBalanceFromAccountData(data);
    }
    return balances;
  }

  /// 从 System.Account 的 SCALE 编码数据中解码 free 余额（yuan）。
  ///
  /// AccountInfo 布局：nonce(u32) + consumers(u32) + providers(u32) + sufficients(u32) + free(u128) + ...
  /// free 在 offset 16，长度 16 字节，little-endian u128。
  static double _decodeBalanceFromAccountData(Uint8List? data) {
    if (data == null || data.length < 32) return 0.0;
    // 读 u128 little-endian at offset 16
    var fen = BigInt.zero;
    for (var i = 0; i < 16; i++) {
      fen += BigInt.from(data[16 + i]) << (i * 8);
    }
    return fen.toDouble() / 100.0;
  }

  static (BigInt value, int size) _decodeCompact(Uint8List data, int offset) {
    if (offset >= data.length) return (BigInt.zero, 0);
    final first = data[offset];
    final mode = first & 0x03;
    if (mode == 0) {
      return (BigInt.from(first >> 2), 1);
    }
    if (mode == 1) {
      if (offset + 2 > data.length) return (BigInt.zero, 0);
      final raw = data[offset] | (data[offset + 1] << 8);
      return (BigInt.from(raw >> 2), 2);
    }
    if (mode == 2) {
      if (offset + 4 > data.length) return (BigInt.zero, 0);
      var raw = 0;
      for (var i = 0; i < 4; i++) {
        raw |= data[offset + i] << (8 * i);
      }
      return (BigInt.from(raw >> 2), 4);
    }
    final byteLen = (first >> 2) + 4;
    if (offset + 1 + byteLen > data.length) return (BigInt.zero, 0);
    var value = BigInt.zero;
    for (var i = 0; i < byteLen; i++) {
      value += BigInt.from(data[offset + 1 + i]) << (8 * i);
    }
    return (value, 1 + byteLen);
  }

  static String _describeRuntimeModuleError(int moduleIndex, int errorIndex) {
    final moduleName = switch (moduleIndex) {
      7 => 'PersonalManage',
      27 => 'PublicAdmins',
      28 => 'PrivateAdmins',
      29 => 'PersonalAdmins',
      30 => 'PublicManage',
      31 => 'PrivateManage',
      _ => 'Module($moduleIndex)',
    };
    final errorName = switch (moduleIndex) {
      7 => _personalManageErrorName(errorIndex),
      27 || 28 || 29 => _adminSetChangeErrorName(errorIndex),
      30 || 31 => _institutionManageErrorName(errorIndex),
      _ => null,
    };
    final hint = switch (moduleIndex) {
      7 => _personalManageErrorHint(errorIndex),
      27 || 28 || 29 => _adminSetChangeErrorHint(errorIndex),
      30 || 31 => _institutionManageErrorHint(errorIndex),
      _ => null,
    };
    final code = errorName == null
        ? '$moduleName.error_$errorIndex'
        : '$moduleName.$errorName';
    return hint == null ? '链上执行失败：$code' : '链上执行失败：$code，$hint';
  }

  static String? _personalManageErrorName(int index) => switch (index) {
        0 => 'IncompleteParameters',
        1 => 'InvalidAccount',
        2 => 'AccountReserved',
        3 => 'DuplicateAdmin',
        4 => 'InvalidThreshold',
        5 => 'InsufficientAmount',
        6 => 'CreateAmountBelowMinimum',
        7 => 'CloseBalanceBelowMinimum',
        8 => 'PermissionDenied',
        9 => 'InvalidAdminsLen',
        11 => 'PersonalNotFound',
        12 => 'PersonalNotActive',
        16 => 'ReservedBalanceRemaining',
        18 => 'ProposalActionNotFound',
        20 => 'EmptyPersonalName',
        21 => 'PersonalAlreadyExists',
        22 => 'CloseAlreadyPending',
        24 => 'ReserveFailed',
        26 => 'FeeWithdrawFailed',
        27 => 'CloseTransferBelowED',
        28 => 'NotPersonalAccount',
        29 => 'AdminSetUnchanged',
        _ => null,
      };

  static String? _personalManageErrorHint(int index) => switch (index) {
        4 => '普通提案阈值必须严格过半且不能超过管理员数量',
        5 => '发起钱包余额不足，不能覆盖初始资金和链上手续费',
        6 => '初始资金低于链上最低创建金额',
        8 => '发起人不是该多签账户管理员',
        9 => '管理员数量不符合链上规则',
        11 => '个人多签账户不存在',
        12 => '个人多签账户不是激活状态',
        16 => '账户仍有保留余额，不能注销',
        18 => '提案业务数据不存在或不属于个人多签模块',
        20 => '账户名称不能为空',
        21 => '个人多签账户当前已存在',
        22 => '该账户已有注销提案正在进行',
        24 => '创建资金锁定失败，通常是可用余额不足',
        26 => '链上手续费扣除失败',
        27 => '注销转出金额低于链上最小存活余额',
        29 => '新管理员集合与当前管理员集合没有变化',
        _ => null,
      };

  static String? _adminSetChangeErrorName(int index) => switch (index) {
        0 => 'InvalidInstitution',
        1 => 'InstitutionOrgMismatch',
        2 => 'InvalidAdminsLen',
        3 => 'UnauthorizedAdmin',
        4 => 'AdminSetUnchanged',
        10 => 'ProposalOrgMismatch',
        11 => 'InstitutionAlreadyExists',
        12 => 'AdminAccountNotPending',
        13 => 'AdminAccountNotActive',
        14 => 'BuiltinAdminAccountCannotClose',
        15 => 'InvalidAdminAccountKind',
        16 => 'InvalidThreshold',
        17 => 'DuplicateAdmin',
        18 => 'InvalidAdminAccountLifecycleScope',
        _ => null,
      };

  static String? _adminSetChangeErrorHint(int index) => switch (index) {
        11 => '管理员主体当前状态已存在；如果是已注销账户，说明链上当前状态还没有完成清理',
        12 => '管理员主体不是待激活状态',
        13 => '管理员主体不是激活状态',
        15 => '管理员主体类型和组织类型不匹配',
        16 => '动态阈值必须严格过半且不能超过管理员数量',
        17 => '管理员列表存在重复账户',
        _ => null,
      };

  static String? _institutionManageErrorName(int index) => switch (index) {
        0 => 'IncompleteParameters',
        1 => 'InvalidAccount',
        2 => 'AccountReserved',
        3 => 'AccountAlreadyExists',
        4 => 'DuplicateAdmin',
        5 => 'InvalidThreshold',
        6 => 'InsufficientAmount',
        7 => 'CreateAmountBelowMinimum',
        8 => 'AccountInitialAmountBelowMinimum',
        9 => 'CloseBalanceBelowMinimum',
        10 => 'PermissionDenied',
        11 => 'InvalidAdminsLen',
        13 => 'InvalidOrg',
        14 => 'MultisigNotFound',
        15 => 'MultisigNotActive',
        16 => 'InvalidBeneficiary',
        18 => 'InstitutionNotRegistered',
        20 => 'CidAlreadyRegistered',
        21 => 'EmptyCidNumber',
        22 => 'RegisterNonceAlreadyUsed',
        25 => 'ReservedBalanceRemaining',
        30 => 'TransferFailed',
        32 => 'EmptyAccountName',
        33 => 'MissingMainAccount',
        34 => 'MissingFeeAccount',
        35 => 'DuplicateAccountName',
        36 => 'InstitutionAlreadyExists',
        37 => 'NotInstitutionMultisig',
        38 => 'EmptyInstitutionAccounts',
        39 => 'TooManyInstitutionAccounts',
        40 => 'InitialAmountOverflow',
        41 => 'ReserveFailed',
        42 => 'ReserveReleaseFailed',
        _ => null,
      };

  static String? _institutionManageErrorHint(int index) => switch (index) {
        3 => '机构账户地址当前已存在',
        5 => '普通提案阈值必须严格过半且不能超过管理员数量',
        6 => '发起钱包余额不足，不能覆盖初始资金和链上手续费',
        7 => '初始资金低于链上最低创建金额',
        8 => '机构账户初始余额低于链上最低金额',
        10 => '发起人不是该机构账户管理员',
        11 => '管理员数量不符合链上规则',
        13 => '机构账户管理员必须使用注册机构码',
        18 => 'CID 机构尚未登记',
        20 => '该 CID 账户名已登记',
        25 => '账户仍有保留余额，不能注销',
        32 => '账户名称不能为空',
        33 => '机构创建必须包含主账户',
        34 => '机构创建必须包含费用账户',
        35 => '机构账户名称重复',
        36 => '机构当前已存在',
        41 => '创建资金锁定失败，通常是可用余额不足',
        _ => null,
      };

  static Uint8List _buildStorageValueKey(
      String palletName, String storageName) {
    final palletHash = Hasher.twoxx128.hashString(palletName);
    final storageHash = Hasher.twoxx128.hashString(storageName);
    final key = Uint8List(palletHash.length + storageHash.length);
    key.setAll(0, palletHash);
    key.setAll(palletHash.length, storageHash);
    return key;
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

/// `submitExtrinsic` 内部状态归类:成功 / 失败 / 仍在等待。
enum _TxResult { success, failure, pending }
