import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:polkadart/polkadart.dart'
    show Hasher, RuntimeMetadata, RuntimeVersion;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:smoldot/smoldot.dart' show LightClientStatusSnapshot;

import 'smoldot_client.dart';

/// 交易池观察状态。
///
/// 中文注释：submitExtrinsic 返回 txHash 只代表已提交到 RPC，不能代表已出块。
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

typedef TxPoolWatchCallback = void Function(TxPoolWatchEvent event);

/// citizenchain RPC 客户端。
///
/// 只使用 smoldot 轻节点（P2P 网络，无需远程 RPC 服务器）。
class ChainRpc {
  ChainRpc() {
    debugPrint('[ChainRpc] 使用 smoldot 轻节点模式');
  }

  static final _keyring = Keyring();
  static final Uint8List _sfidMainAccountKey =
      _buildStorageValueKey('SfidSystem', 'SfidMainAccount');

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
          : _hexDecode(
              valueHex.startsWith('0x') ? valueHex.substring(2) : valueHex);
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

  /// 获取轻节点当前同步进度（同步中也可读）。
  ///
  /// 仅用于 UI 展示和诊断，不应用于交易构造或需要最新状态一致性的逻辑。
  Future<LightClientStatusSnapshot> fetchChainProgress() async {
    return SmoldotClientManager.instance.getStatusSnapshotRaw();
  }

  /// 获取最新区块的哈希和块号。
  ///
  /// 仅用于 UI 展示、事件查询和诊断；signed extrinsic 构造按 P-SIGN-001
  /// 固定使用 immortal era，不得用最新块参与 CheckEra。
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

  // 2026-04-23 整改:`fetchBlockExtrinsicHashes` 已删除。
  //
  // 原实现包装 `getBlockExtrinsics`(smoldot `chainHead_v1_body`)逐块拉 body
  // 并用 blake2_256 求每笔 extrinsic 哈希。因触发 substrate block-request
  // 反滥用 ban 把轻节点打死,已整体下线。交易确认现走 nonce-only,见
  // `pending_tx_reconciler.dart`。

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

  /// 提交已签名的 extrinsic,返回交易哈希(32 字节)。
  ///
  /// **设计**(2026-05-03 改为 submit-only + 后台监听):
  /// - 主流程仅调原生 `submitExtrinsicHex`(底层走 `author_submitExtrinsic`),
  ///   拿到 txHash 立即返回,UI 永不卡住。
  /// - 后台 fire-and-forget 启一条 `author_submitAndWatchExtrinsic` 订阅,
  ///   8 秒内观察到 invalid/dropped/usurped/future 仅打印日志,不再回灌 UI。
  ///
  /// 历史:曾尝试在主流程内 watch + 1 秒 timeout(参见 git 2026-05-03 早些时候的提交),
  /// 但 smoldot 通过 native binding 转发 broadcast stream 的首条 event 存在调度延迟,
  /// 在 GMB 链 6 分钟出块的节奏下经常 1 秒内拿不到 txHash 导致 `completeError` 抛出,
  /// UI 反而误判失败并继续转圈。最终回到 submit-only,放弃在客户端拦截 mempool reject,
  /// reject 排查改走 polkadot.js apps + 终端日志。
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

    final txHashHex =
        await SmoldotClientManager.instance.submitExtrinsicHex(hex);
    if (txHashHex == null || txHashHex.isEmpty) {
      throw StateError('smoldot 未返回交易哈希');
    }
    debugPrint('[ChainRpc.submitExtrinsic] smoldot 返回 txHash: $txHashHex');

    unawaited(_watchTxRejectInBackground(hex, txHashHex, onWatchEvent));
    return _hexDecode(_stripHexPrefix(txHashHex));
  }

  /// 提交已签名 extrinsic，并阻塞等待交易真正进入区块。
  ///
  /// 中文注释：提案类交易不能把 txHash 当成功；只有 `inBlock/finalized`
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

  /// 后台观察一条交易的 mempool 状态,**所有状态都打日志**,被拒时立即结束。
  ///
  /// 60 秒内未收到任何状态视为 timeout(smoldot 转发失败 / 全节点完全不响应),
  /// 也打日志退出 — 这是排查"提交成功但链上没出块"的核心诊断输入。
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
      bailTimer = Timer(const Duration(seconds: 60), () {
        if (!done.isCompleted) {
          if (!sawAnyStatus) {
            onWatchEvent?.call(const TxPoolWatchEvent(
              kind: TxPoolWatchKind.timeout,
              description: '60 秒内未收到交易池状态，可能转发失败或交易被静默丢弃',
              raw: 'timeout',
            ));
            debugPrint(
                '[ChainRpc.bgWatch] $txHashHex 60s timeout 未收到任何状态,可能 smoldot 转发失败或全节点静默 drop');
          } else {
            debugPrint(
                '[ChainRpc.bgWatch] $txHashHex 60s 后结束后台监听,交易仍交由 nonce 轮询确认');
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
  /// 仅 [_watchTxRejectInBackground] 使用:主流程已不再依赖归类,只关心 failure 一种。
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
  /// 返回原始 SCALE 编码字节。key 不存在时返回 null。
  Future<Uint8List?> fetchStorage(String storageKeyHex) async {
    // 中文注释：轻节点模式统一通过原生 storage 读取，逐步清理 Dart 层的裸 RPC。
    final valueHex =
        await SmoldotClientManager.instance.getStorageValueHex(storageKeyHex);
    if (valueHex == null) return null;
    return _hexDecode(
      valueHex.startsWith('0x') ? valueHex.substring(2) : valueHex,
    );
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

  /// 查询链上真实余额 = free + reserved,最新块(不传 block hash)。
  ///
  /// 中文注释:
  /// - 对齐 polkadot.js apps 的 total 余额口径;钱包详情页第 3 张卡片展示
  ///   的就是这个值,不能只取 free,否则锁仓 / 质押的 reserved 部分会漏算。
  /// - 走通用 `fetchStorageBatch` 取 `System.Account` 原始 bytes,在 Dart 侧
  ///   自行解码 AccountData 的 free + reserved,绕过原生 SystemAccountSnapshot
  ///   当前只暴露 freeFen 字段的限制。
  /// - 账户不存在或数据不完整均返回 0.0。
  Future<double> fetchTotalBalance(String pubkeyHex) async {
    // 1. 构造 System.Account storage key:prefix + blake2_128(accountId) + accountId
    final accountId = _hexDecode(
        pubkeyHex.startsWith('0x') ? pubkeyHex.substring(2) : pubkeyHex);
    final blake2 = Hasher.blake2b128.hash(accountId);
    final fullKey = Uint8List(
        _systemAccountPrefix.length + blake2.length + accountId.length);
    fullKey.setAll(0, _systemAccountPrefix);
    fullKey.setAll(_systemAccountPrefix.length, blake2);
    fullKey.setAll(_systemAccountPrefix.length + blake2.length, accountId);
    final keyHex = '0x${_hexEncode(fullKey)}';

    // 2. 批量接口复用,只查 1 个 key 也走同一路径。
    final batchResult = await fetchStorageBatch([keyHex]);
    final data = batchResult[keyHex];
    return _decodeTotalBalanceFromAccountData(data);
  }

  /// 从 System.Account 的 SCALE 编码数据中解码 free + reserved 总余额(yuan)。
  ///
  /// 中文注释:
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

  /// 批量查询多个账户的链上余额，返回 pubkeyHex → yuan 的映射。
  ///
  /// 一次 storage proof 请求查询所有账户，比逐个调用 [fetchBalance] 更高效。
  /// 账户不存在时对应值为 0.0。
  Future<Map<String, double>> fetchBalances(List<String> pubkeyHexList) async {
    if (pubkeyHexList.isEmpty) return {};

    // 1. 为每个账户构建 System.Account storage key
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

    // 2. 一次批量查询
    final batchResult = await fetchStorageBatch(storageKeys);

    // 3. 解码每个账户的余额
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

  /// 读取链上当前 SFID 主验签公钥（32 字节 AccountId）。
  ///
  /// 存储项：`SfidSystem::SfidMainAccount`，类型为 `Option<AccountId32>`。
  Future<String?> fetchCurrentSfidMainPubkeyHex() async {
    final cached = _cachedCurrentSfidMainPubkeyHex;
    if (cached != null && cached.isNotEmpty) {
      return cached;
    }

    final keyHex = '0x${_hexEncode(_sfidMainAccountKey)}';
    final result =
        await SmoldotClientManager.instance.getStorageValueHex(keyHex);
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

/// `submitExtrinsic` 内部状态归类:成功 / 失败 / 仍在等待。
enum _TxResult { success, failure, pending }
