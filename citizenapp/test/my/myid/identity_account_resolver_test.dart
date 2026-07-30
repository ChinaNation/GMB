import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/my/myid/citizen_identity_chain_reader.dart';
import 'package:citizenapp/my/myid/identity_account_resolver.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';
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

Account _acc(int index, String accountId) => Account(
      masterId: _account0,
      accountIndex: index,
      accountId: accountId,
      ss58Address: 'ss58-$index',
      accountName: '账户$index',
    );

CitizenIdentityChainSnapshot _anonSnapshot() => CitizenIdentityChainSnapshot(
      cidNumber: 'CID-TEST-0001',
      accountId: Uint8List(32),
      bindingRevision: 1,
      votingIdentity: null,
      candidateIdentity: null,
    );

void main() {
  IdentityAccountResolver resolver({
    WalletProfile? wallet = _wallet0,
    List<Account> accounts = const [],
    Map<String, CitizenIdentityChainSnapshot> chain = const {},
    String? throwFor,
  }) {
    return IdentityAccountResolver(
      walletManager: _FakeWalletManager(wallet, accounts),
      chainReader: _FakeReader(chain, throwFor: throwFor),
    );
  }

  test('账户0 绑 CID → 身份账户 = 账户0(只 1 次链读即命中)', () async {
    final r = await resolver(
      accounts: [_acc(0, _account0), _acc(5, _account5)],
      chain: {_account0: _anonSnapshot()},
    ).resolve();
    expect(r, isNotNull);
    expect(r!.accountId, _account0);
    expect(r.accountIndex, 0);
    expect(r.isRegistered, isTrue);
  });

  test('账户0 未绑、子账户 //5 绑 CID → 身份账户 = //5', () async {
    final r = await resolver(
      accounts: [_acc(0, _account0), _acc(5, _account5)],
      chain: {_account5: _anonSnapshot()},
    ).resolve();
    expect(r!.accountId, _account5);
    expect(r.accountIndex, 5);
    expect(r.isRegistered, isTrue);
  });

  test('全部账户都无 CID → 回退账户0、未注册', () async {
    final r = await resolver(
      accounts: [_acc(0, _account0), _acc(5, _account5)],
      chain: const {},
    ).resolve();
    expect(r!.accountId, _account0);
    expect(r.accountIndex, 0);
    expect(r.isRegistered, isFalse);
    expect(r.snapshot, isNull);
  });

  test('无热钱包 → null', () async {
    final r = await resolver(wallet: null).resolve();
    expect(r, isNull);
  });

  test('链读异常上抛,绝不吞成访客/未注册', () async {
    await expectLater(
      resolver(
        accounts: [_acc(0, _account0)],
        throwFor: _account0,
      ).resolve(),
      throwsA(isA<StateError>()),
    );
  });

  test('按 CID 读取绑定锚定同一 finalized 区块并校验双向闭环', () async {
    const cidNumber = 'CN220-CTZN2-100000001-2026';
    final accountId = Uint8List.fromList(List<int>.filled(32, 0xaa));
    final cidScale = CitizenIdentityChainReader.encodeBoundedBytes(
      cidNumber.codeUnits,
    );
    String key(String storageName, Uint8List data) =>
        CitizenIdentityChainReader.hexEncode(
          CitizenIdentityChainReader.storageMapKey(
            'CitizenIdentity',
            storageName,
            data,
          ),
        );
    final chainRpc = _BindingChainRpc(<String, Uint8List>{
      key('AccountIdByCid', cidScale): accountId,
      key('CidRegistry', cidScale): Uint8List.fromList([
        ..._bounded('FEDERAL_REGISTRY-CID'),
        ...List<int>.filled(32, 7),
        ..._bounded('GD'),
        ..._bounded('0755'),
        0,
        1,
        0,
        0,
        0,
        0,
      ]),
      key('BindingRevisionByCid', cidScale):
          Uint8List.fromList([2, 0, 0, 0, 0, 0, 0, 0]),
      key('CidByAccountId', accountId):
          CitizenIdentityChainReader.encodeBoundedBytes(cidNumber.codeUnits),
    });

    final result = await CitizenIdentityChainReader(chainRpc: chainRpc)
        .readBindingByCidNumber(cidNumber);

    expect(result, isNotNull);
    expect(result!.accountIdText, _accountIdText(accountId));
    expect(result.bindingRevision, 2);
    expect(
      chainRpc.blockHashes.toSet(),
      {'0x${List<String>.filled(32, '00').join()}'},
    );
  });

  test('按 CID 读取时反向账户映射不一致必须失败关闭', () async {
    const cidNumber = 'CN220-CTZN2-100000001-2026';
    final accountId = Uint8List.fromList(List<int>.filled(32, 0xbb));
    final cidScale = CitizenIdentityChainReader.encodeBoundedBytes(
      cidNumber.codeUnits,
    );
    String key(String storageName, Uint8List data) =>
        CitizenIdentityChainReader.hexEncode(
          CitizenIdentityChainReader.storageMapKey(
            'CitizenIdentity',
            storageName,
            data,
          ),
        );
    final chainRpc = _BindingChainRpc(<String, Uint8List>{
      key('AccountIdByCid', cidScale): accountId,
      key('CidRegistry', cidScale): Uint8List.fromList([
        ..._bounded('FEDERAL_REGISTRY-CID'),
        ...List<int>.filled(32, 7),
        ..._bounded('GD'),
        ..._bounded('0755'),
        0,
        1,
        0,
        0,
        0,
        0,
      ]),
      key('BindingRevisionByCid', cidScale):
          Uint8List.fromList([1, 0, 0, 0, 0, 0, 0, 0]),
      key('CidByAccountId', accountId):
          CitizenIdentityChainReader.encodeBoundedBytes('OTHER-CID'.codeUnits),
    });

    final result = await CitizenIdentityChainReader(chainRpc: chainRpc)
        .readBindingByCidNumber(cidNumber);

    expect(result, isNull);
  });
}

List<int> _bounded(String value) =>
    CitizenIdentityChainReader.encodeBoundedBytes(value.codeUnits);

String _accountIdText(Uint8List bytes) =>
    CitizenIdentityChainReader.hexEncode(bytes);

class _FakeWalletManager extends WalletManager {
  _FakeWalletManager(this._wallet, this._accounts);
  final WalletProfile? _wallet;
  final List<Account> _accounts;
  @override
  Future<WalletProfile?> getDefaultWallet() async => _wallet;
  @override
  Future<List<Account>> getAccounts(String masterId) async => _accounts;
}

class _FakeReader extends CitizenIdentityChainReader {
  _FakeReader(this._chain, {this.throwFor});
  final Map<String, CitizenIdentityChainSnapshot> _chain;
  final String? throwFor;
  @override
  Future<CitizenIdentityChainSnapshot?> readByAccountId(
      String accountId) async {
    if (accountId == throwFor) throw StateError('chain down');
    return _chain[accountId];
  }
}

class _BindingChainRpc extends ChainRpc {
  _BindingChainRpc(this.storage);

  final Map<String, Uint8List> storage;
  final List<String> blockHashes = <String>[];

  @override
  Future<({Uint8List blockHash, int blockNumber})>
      fetchFinalizedBlock() async => (blockHash: Uint8List(32), blockNumber: 7);

  @override
  Future<Uint8List?> fetchStorageAtBlock(
    String storageKeyHex,
    String blockHashHex,
  ) async {
    blockHashes.add(blockHashHex);
    return storage[storageKeyHex];
  }
}
