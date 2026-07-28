import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/my/myid/citizen_identity_chain_reader.dart';
import 'package:citizenapp/my/myid/identity_account_cache.dart';
import 'package:citizenapp/my/myid/identity_account_resolver.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

const _account0 =
    '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
const _account5 =
    '0x5555555555555555555555555555555555555555555555555555555555555555';

const _wallet0 = WalletProfile(
  walletIndex: 1,
  walletName: '钱包1',
  walletIcon: 'wallet',
  balance: 0,
  ss58Address: 'ss58-0',
  accountId: _account0,
  alg: 'sr25519',
  ss58: 2027,
  createdAtMillis: 0,
  source: 'test',
  signMode: 'local',
);

ResolvedIdentity _identity(String accountId, {int index = 5}) =>
    ResolvedIdentity(
      accountId: accountId,
      ss58Address: 'ss58-$index',
      accountIndex: index,
      snapshot: CitizenIdentityChainSnapshot(
        cidNumber: 'CID-TEST',
        accountId: Uint8List(32),
        votingIdentity: null,
        candidateIdentity: null,
      ),
    );

void main() {
  IdentityAccountCache cache({
    ResolvedIdentity? result,
    bool throwIt = false,
    WalletProfile? wallet = _wallet0,
    void Function(int)? onResolve,
  }) {
    return IdentityAccountCache(
      resolver: _FakeResolver(result, throwIt: throwIt, onResolve: onResolve),
      walletManager: _FakeWalletManager(wallet),
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

  test('链读失败乐观回退账户0(未注册),且不缓存失败结果', () async {
    var calls = 0;
    final c = cache(throwIt: true, onResolve: (n) => calls = n);
    final r = await c.resolve();
    expect(r!.accountId, _account0);
    expect(r.accountIndex, 0);
    expect(r.isRegistered, isFalse);
    // 乐观回退不缓存 → 再取一次会再链读一次。
    await c.resolve();
    expect(calls, 2);
  });

  test('无热钱包 + 链读失败 → null', () async {
    final c = cache(throwIt: true, wallet: null);
    expect(await c.resolve(), isNull);
  });

  test('并发请求合并成一次链读', () async {
    var calls = 0;
    final c = cache(result: _identity(_account5), onResolve: (n) => calls = n);
    final results = await Future.wait([c.resolve(), c.resolve(), c.resolve()]);
    expect(results.map((r) => r!.accountId), everyElement(_account5));
    expect(calls, 1, reason: '并发应合并成一次链读');
  });

  test('accountId() 便捷入口', () async {
    final c = cache(result: _identity(_account5));
    expect(await c.accountId(), _account5);
  });

  test('allowChainRead=false 且无缓存:绝不链读,乐观回退账户0', () async {
    var calls = 0;
    final c = cache(result: _identity(_account5), onResolve: (n) => calls = n);
    final r = await c.resolve(allowChainRead: false);
    expect(r!.accountId, _account0);
    expect(calls, 0, reason: '广场浏览路径绝不启动 smoldot');
  });

  test('allowChainRead=false 命中已有缓存则返回身份账户', () async {
    final c = cache(result: _identity(_account5));
    await c.resolve(); // 先链读填缓存
    final r = await c.resolve(allowChainRead: false);
    expect(r!.accountId, _account5, reason: '命中缓存不受 allowChainRead 影响');
  });
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

class _FakeWalletManager extends WalletManager {
  _FakeWalletManager(this._wallet);
  final WalletProfile? _wallet;
  @override
  Future<WalletProfile?> getDefaultWallet() async => _wallet;
}
