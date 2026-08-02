import 'package:flutter/foundation.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/security/local_data_key.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

import 'identity_account_resolver.dart';

/// 身份账户缓存 —— 给聊天/广场/会话等**高频调用方**提供「身份账户」的低成本入口。
///
/// 身份账户解析要链读(遍历本地账户 `readByAccountId`),不能每次调用都链读。本类
/// 缓存 [IdentityAccountResolver] 的结果,按钱包身份版本失效(占号/换绑/切钱包后
/// 自动重算);并发请求合并成一次链读。链读失败或禁止链读且缓存未命中时返回 null，
/// 签名、鉴权和付款调用方必须失败关闭，绝不虚构账户0是当前身份账户。
class IdentityAccountCache {
  IdentityAccountCache({
    IdentityAccountResolver? resolver,
    ChainRpc? chainRpc,
  })  : _resolver = resolver ?? IdentityAccountResolver(chainRpc: chainRpc),
        _chainRpc = chainRpc ?? ChainRpc();

  /// 全 App 共享单例(生产);高频调用方直接用 [instance],无需逐个注入。
  static IdentityAccountCache _instance = IdentityAccountCache();
  static IdentityAccountCache get instance => _instance;

  /// 测试覆盖点(类似 `WalletManager.debugSeedStore`):测试 setUp 设 fake、
  /// tearDown 调 [resetDebugInstance] 复位,避免每个调用方逐处注入。
  @visibleForTesting
  static set debugInstance(IdentityAccountCache cache) => _instance = cache;

  @visibleForTesting
  static void resetDebugInstance() => _instance = IdentityAccountCache();

  final IdentityAccountResolver _resolver;
  final ChainRpc _chainRpc;

  ResolvedIdentity? _cached;
  int _cachedRevision = -1;
  Future<ResolvedIdentity?>? _inflight;

  /// 当前身份账户；链读失败上抛，禁止链读且缓存未命中返回 null。
  ///
  /// [allowChainRead] 为 false 时**绝不链读**(广场浏览等不启动 smoldot 的路径):
  /// 命中缓存则返回，否则返回 null。与 `square_identity_state`
  /// 的 `readLiveChain=false` 契约一致——不启动 smoldot，也不虚构身份。
  Future<ResolvedIdentity?> resolve({bool allowChainRead = true}) async {
    final revision = WalletManager.walletsRevision.value;
    final cached = _cached;
    if (cached != null && _cachedRevision == revision) {
      return cached;
    }
    if (!allowChainRead) {
      return null;
    }
    // 合并并发请求:冷启动广场/聊天/我的同时拉身份,只做一次链读。
    final inflight = _inflight;
    if (inflight != null) {
      return inflight;
    }
    final future = _resolveFresh(revision);
    _inflight = future;
    try {
      return await future;
    } finally {
      _inflight = null;
    }
  }

  /// 身份账户 accountId(便捷);无热钱包返回 null。
  Future<String?> accountId({bool allowChainRead = true}) async =>
      (await resolve(allowChainRead: allowChainRead))?.accountId;

  /// 实际业务发现设备子钥缺失时读取 finalized 当前绑定上下文。
  ///
  /// 这里只读取公开链上信息，不读取钱包账户 child；页面门禁不得用它制造额外状态。
  Future<AccountDataBinding?> binding({bool allowChainRead = true}) async {
    final resolved = await resolve(allowChainRead: allowChainRead);
    final snapshot = resolved?.snapshot;
    if (resolved == null || snapshot == null) return null;
    final genesisHash = await _chainRpc.fetchGenesisHash();
    if (genesisHash.length != 32) {
      throw StateError('创世哈希无效，无法初始化当前账户设备子钥');
    }
    return AccountDataBinding(
      genesisHash: '0x${_lowerHex(genesisHash)}',
      cidNumber: snapshot.cidNumber,
      bindingRevision: snapshot.bindingRevision,
      accountId: resolved.accountId,
    );
  }

  Future<ResolvedIdentity?> _resolveFresh(int revision) async {
    final resolved = await _resolver.resolve();
    _cached = resolved;
    _cachedRevision = revision;
    return resolved;
  }

  /// 强制失效(显式刷新用;`walletsRevision` 变化已自动失效,一般无需手调)。
  void invalidate() {
    _cached = null;
    _cachedRevision = -1;
  }

  static String _lowerHex(List<int> bytes) =>
      bytes.map((value) => value.toRadixString(16).padLeft(2, '0')).join();
}
