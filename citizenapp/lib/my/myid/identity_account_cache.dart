import 'package:flutter/foundation.dart';

import 'package:citizenapp/wallet/core/wallet_manager.dart';

import 'identity_account_resolver.dart';

/// 身份账户缓存 —— 给聊天/广场/会话等**高频调用方**提供「身份账户」的低成本入口。
///
/// 身份账户解析要链读(遍历本地账户 `readByAccountId`),不能每次调用都链读。本类
/// 缓存 [IdentityAccountResolver] 的结果,按 [WalletManager.walletsRevision] 失效
/// (占号/换绑/切钱包后自动重算);并发请求合并成一次链读。
///
/// **链读失败时乐观回退账户0**(常态身份就是账户0,保证聊天/广场可用;换绑后
/// `walletsRevision` 会触发重算纠正)——与身份页 [MyIdService.getState] 的严格
/// fail-closed 分工明确:那条服务身份展示、失败即提示重试;这条服务「用哪个账户
/// 发消息/建会话/签名」、失败尽力可用。见 memory `citizenapp-cid-identity-master-key`。
class IdentityAccountCache {
  IdentityAccountCache({
    IdentityAccountResolver? resolver,
    WalletManager? walletManager,
  })  : _resolver = resolver ?? IdentityAccountResolver(),
        _walletManager = walletManager ?? WalletManager();

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
  final WalletManager _walletManager;

  ResolvedIdentity? _cached;
  int _cachedRevision = -1;
  Future<ResolvedIdentity?>? _inflight;

  /// 当前身份账户(命中缓存直接返回;链读失败乐观回退账户0;无热钱包 null)。
  ///
  /// [allowChainRead] 为 false 时**绝不链读**(广场浏览等不启动 smoldot 的路径):
  /// 命中缓存则返回,否则乐观回退账户0(不缓存)。与 `square_identity_state`
  /// 的 `readLiveChain=false` 契约一致——绝不因身份解析把 smoldot 顶起来。
  Future<ResolvedIdentity?> resolve({bool allowChainRead = true}) async {
    final revision = WalletManager.walletsRevision.value;
    final cached = _cached;
    if (cached != null && _cachedRevision == revision) {
      return cached;
    }
    if (!allowChainRead) {
      // 不链读:乐观回退账户0(不缓存,下次允许链读时再算)。
      return _fallbackToAccount0();
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

  Future<ResolvedIdentity?> _resolveFresh(int revision) async {
    try {
      final resolved = await _resolver.resolve();
      // 只缓存成功结果;乐观回退不缓存,下次重试。
      _cached = resolved;
      _cachedRevision = revision;
      return resolved;
    } on Object {
      return _fallbackToAccount0();
    }
  }

  Future<ResolvedIdentity?> _fallbackToAccount0() async {
    final wallet = await _walletManager.getDefaultWallet();
    if (wallet == null) return null;
    return ResolvedIdentity(
      accountId: wallet.accountId,
      ss58Address: wallet.ss58Address,
      accountIndex: 0,
      snapshot: null,
    );
  }

  /// 强制失效(显式刷新用;`walletsRevision` 变化已自动失效,一般无需手调)。
  void invalidate() {
    _cached = null;
    _cachedRevision = -1;
  }
}
