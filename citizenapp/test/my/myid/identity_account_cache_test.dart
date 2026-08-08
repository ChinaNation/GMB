import 'dart:async';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/my/myid/citizen_identity_chain_reader.dart';
import 'package:citizenapp/my/myid/identity_account_cache.dart';
import 'package:citizenapp/my/myid/identity_account_resolver.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

const _account5 =
    '0x5555555555555555555555555555555555555555555555555555555555555555';

ResolvedIdentity _identity(String accountId, {int index = 5}) =>
    ResolvedIdentity(
      accountId: accountId,
      ss58Address: 'ss58-$index',
      accountIndex: index,
      snapshot: CitizenIdentityChainSnapshot(
        cidNumber: 'CID-TEST',
        accountId: Uint8List(32),
        bindingRevision: 1,
        votingIdentity: null,
        candidateIdentity: null,
      ),
    );

void main() {
  IdentityAccountCache cache({
    ResolvedIdentity? result,
    bool throwIt = false,
    void Function(int)? onResolve,
    ChainRpc? chainRpc,
  }) {
    return IdentityAccountCache(
      resolver: _FakeResolver(result, throwIt: throwIt, onResolve: onResolve),
      chainRpc: chainRpc,
    );
  }

  test('命中缓存:同一 walletsRevision 只链读一次', () async {
    var calls = 0;
    final c = cache(result: _identity(_account5), onResolve: (n) => calls = n);
    final a = await c.resolve();
    final b = await c.resolve();
    expect(a!.accountId, _account5);
    expect(b!.accountId, _account5);
    expect(calls, 1, reason: '第二次应命中缓存,不再链读');
  });

  test('walletsRevision 变化后失效重算', () async {
    var calls = 0;
    final c = cache(result: _identity(_account5), onResolve: (n) => calls = n);
    await c.resolve();
    WalletManager.walletsRevision.value++;
    await c.resolve();
    expect(calls, 2, reason: '身份绑定变化后应重新链读');
  });

  test('链读失败直接上抛，绝不虚构账户0授权', () async {
    var calls = 0;
    final c = cache(throwIt: true, onResolve: (n) => calls = n);
    await expectLater(c.resolve(), throwsA(isA<StateError>()));
    expect(calls, 1);
  });

  test('并发请求合并成一次链读', () async {
    var calls = 0;
    final c = cache(result: _identity(_account5), onResolve: (n) => calls = n);
    final results = await Future.wait([c.resolve(), c.resolve(), c.resolve()]);
    expect(results.map((r) => r!.accountId), everyElement(_account5));
    expect(calls, 1, reason: '并发应合并成一次链读');
  });

  test('失效后的新版本不复用旧未注册请求，旧结果也不能覆盖新缓存', () async {
    final resolver = _ControlledResolver();
    final c = IdentityAccountCache(resolver: resolver);

    final stale = c.resolve();
    expect(resolver.calls, 1);

    c.invalidate();
    WalletManager.walletsRevision.value++;
    final fresh = c.resolve();
    expect(resolver.calls, 2, reason: '新身份版本必须启动独立链读');

    final registered = _identity(_account5);
    resolver.completers[1].complete(registered);
    expect(await fresh, same(registered));

    resolver.completers[0].complete(
      const ResolvedIdentity(
        accountId: _account5,
        ss58Address: 'stale-ss58',
        accountIndex: 5,
        snapshot: null,
      ),
    );
    expect((await stale)?.isRegistered, isFalse);
    expect(await c.resolve(), same(registered));
    expect(resolver.calls, 2, reason: '旧请求结束后不得覆盖新版本缓存');
  });

  test('accountId() 便捷入口', () async {
    final c = cache(result: _identity(_account5));
    expect(await c.accountId(), _account5);
  });

  test('binding() 只组合公开 finalized 身份与创世，不读取钱包私钥', () async {
    final c = cache(
      result: _identity(_account5),
      chainRpc: _FakeChainRpc(),
    );
    final binding = await c.binding();
    expect(binding, isNotNull);
    expect(binding!.accountId, _account5);
    expect(binding.cidNumber, 'CID-TEST');
    expect(binding.bindingRevision, 1);
    expect(binding.genesisHash, '0x${'ab' * 32}');
  });

  test('allowChainRead=false 且无缓存：绝不链读并返回 null', () async {
    var calls = 0;
    final c = cache(result: _identity(_account5), onResolve: (n) => calls = n);
    final r = await c.resolve(allowChainRead: false);
    expect(r, isNull);
    expect(calls, 0, reason: '广场浏览路径绝不启动 smoldot');
  });

  test('allowChainRead=false 命中已有缓存则返回身份账户', () async {
    final c = cache(result: _identity(_account5));
    await c.resolve(); // 先链读填缓存
    final r = await c.resolve(allowChainRead: false);
    expect(r!.accountId, _account5, reason: '命中缓存不受 allowChainRead 影响');
  });
}

class _FakeChainRpc extends ChainRpc {
  @override
  Future<Uint8List> fetchGenesisHash() async => Uint8List.fromList(
        List<int>.filled(32, 0xab),
      );
}

class _FakeResolver extends IdentityAccountResolver {
  _FakeResolver(this._result, {this.throwIt = false, this.onResolve});
  final ResolvedIdentity? _result;
  final bool throwIt;
  final void Function(int)? onResolve;
  int _calls = 0;
  @override
  Future<ResolvedIdentity?> resolve() async {
    _calls++;
    onResolve?.call(_calls);
    if (throwIt) throw StateError('chain down');
    return _result;
  }
}

class _ControlledResolver extends IdentityAccountResolver {
  final List<Completer<ResolvedIdentity?>> completers =
      <Completer<ResolvedIdentity?>>[];

  int get calls => completers.length;

  @override
  Future<ResolvedIdentity?> resolve() {
    final completer = Completer<ResolvedIdentity?>();
    completers.add(completer);
    return completer.future;
  }
}
