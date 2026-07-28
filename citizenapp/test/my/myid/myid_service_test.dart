import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:citizenapp/citizen/cid/cid_generator.dart';
import 'package:citizenapp/citizen/public/data/admin_division_store.dart';
import 'package:citizenapp/my/myid/citizen_identity_chain_reader.dart';
import 'package:citizenapp/my/myid/identity_account_resolver.dart';
import 'package:citizenapp/my/myid/identity_badge_snapshot_store.dart';
import 'package:citizenapp/my/myid/myid_service.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/rpc/citizen_identity_rpc.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// Alice 通用 SS58(校验和有效),仅用于让 `decodeAddress` 解出 32 字节账户;
/// 护照 App 真号是 prefix=2027,这里只需一个可解码地址驱动 storage key。
const _validAddress = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
const _validAccountId =
    '0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d';

void main() {
  MyIdService buildService({
    WalletProfile? wallet = const _AliceWallet(),
    Uint8List? voting,
    Uint8List? candidate,
    bool chainThrows = false,
    bool mismatchWallet = false,
    int cidStatus = 0,
    bool hasCid = false,
    DateTime? now,
  }) {
    return MyIdService(
      walletManager: _FakeWalletManager(wallet),
      chainRpc: _FakeChainRpc(
        voting: voting,
        candidate: candidate,
        throws: chainThrows,
        mismatchWallet: mismatchWallet,
        cidStatus: cidStatus,
        hasCid: hasCid,
      ),
      divisionStore: _FakeDivisionStore(),
      badgeSnapshotStore: _FakeBadgeStore(),
      nowProvider: () => now ?? DateTime.utc(2026, 6, 1),
    );
  }

  test('无默认热钱包时为访客并提示创建钱包', () async {
    final state = await buildService(wallet: null).getState();
    expect(state.tier, MyIdTier.visitor);
    expect(state.votingAccountId, isNull);
    expect(state.errorMessage, '请先创建钱包');
  });

  test('默认用户账户链上无投票身份时为访客轻节点', () async {
    final state = await buildService(voting: null).getState();
    expect(state.tier, MyIdTier.visitor);
    expect(state.votingAccountId, isNull);
    expect(state.status, isNull);
    expect(state.isAnonymousRegistered, isFalse);
    expect(state.cidNumber, isNull);
  });

  test('有 CID、绑定闭环但无投票身份时为匿名已注册(访客卡显 CID)', () async {
    final state = await buildService(voting: null, hasCid: true).getState();
    // 仍是访客档(不新增卡/色),但已占匿名 CID → isAnonymousRegistered。
    expect(state.tier, MyIdTier.visitor);
    expect(state.isAnonymousRegistered, isTrue);
    expect(state.cidNumber, 'GD-CTZN1-8F3A2B');
    expect(state.votingAccountId, _validAccountId);
    expect(state.status, isNull);
  });

  test('有投票身份、无候选身份时为投票公民并解出全部字段', () async {
    final state = await buildService(
      voting: _encodeVoting(
        from: 20260101,
        until: 20310101,
        status: 0,
        province: 'GD',
        city: '0755',
        town: '001',
      ),
    ).getState();

    expect(state.tier, MyIdTier.voting);
    expect(state.status, MyIdStatus.normal);
    expect(state.votingAccountId, _validAccountId);
    expect(state.cidNumber, 'GD-CTZN1-8F3A2B');
    expect(state.passportValidFrom, '2026-01-01');
    expect(state.passportValidUntil, '2031-01-01');
    // 居住选区经字典 join:省名 + 市名 + 镇名(fake 字典把 code 映射成 N(code))。
    expect(state.residenceDistrict, contains('N(0755)'));
    expect(state.residenceDistrict, contains('N(001)'));
    // 投票公民无候选专属字段。
    expect(state.familyName, isNull);
    expect(state.givenName, isNull);
    expect(state.birthDistrict, isNull);
  });

  test('同时有候选身份时为竞选公民并解出姓名/性别/出生地', () async {
    final state = await buildService(
      voting: _encodeVoting(
        from: 20260101,
        until: 20310101,
        status: 0,
        province: 'GD',
        city: '0755',
        town: '001',
      ),
      candidate: _encodeCandidate(
        province: 'GD',
        city: '0020',
        town: '005',
        familyName: '陈',
        givenName: '明',
        sex: 0,
      ),
    ).getState();

    expect(state.tier, MyIdTier.candidate);
    expect(state.familyName, '陈');
    expect(state.givenName, '明');
    expect(state.citizenSexLabel, '男');
    expect(state.birthDistrict, contains('N(0020)'));
    expect(state.citizenBirthDate, '2000-01-31');
  });

  test('护照未生效/已过期/已吊销状态派生正确', () async {
    Uint8List voting({required int status}) => _encodeVoting(
          from: 20260101,
          until: 20310101,
          status: status,
          province: 'GD',
          city: '0755',
          town: '001',
        );

    final notYet =
        await buildService(voting: voting(status: 0), now: DateTime.utc(2025))
            .getState();
    expect(notYet.status, MyIdStatus.notYetValid);

    final expired = await buildService(
      voting: voting(status: 0),
      now: DateTime.utc(2032),
    ).getState();
    expect(expired.status, MyIdStatus.expired);

    final revoked = await buildService(voting: voting(status: 1)).getState();
    expect(revoked.status, MyIdStatus.revoked);
  });

  test('链上读取失败时不静默降级访客,而是标记读取失败', () async {
    final state = await buildService(chainThrows: true).getState();
    expect(state.status, MyIdStatus.queryFailed);
    expect(state.errorMessage, '链上身份读取失败');
  });

  test('反向钱包错配或 CID 已吊销时不承认公民身份', () async {
    final voting = _encodeVoting(
      from: 20260101,
      until: 20310101,
      status: 0,
      province: 'GD',
      city: '0755',
      town: '001',
    );

    final mismatch =
        await buildService(voting: voting, mismatchWallet: true).getState();
    final revoked = await buildService(voting: voting, cidStatus: 1).getState();

    expect(mismatch.tier, MyIdTier.visitor);
    expect(revoked.tier, MyIdTier.visitor);
  });

  test('空居住镇码不会把公民误判为访客', () async {
    final state = await buildService(
      voting: _encodeVoting(
        from: 20260101,
        until: 20310101,
        status: 0,
        province: 'GD',
        city: '0755',
        town: '', // 空镇码
      ),
    ).getState();
    expect(state.tier, MyIdTier.voting);
    expect(state.cidNumber, 'GD-CTZN1-8F3A2B');
  });

  test('注册匿名 CID:用默认用户 accountId + UTC 年生成金标 CID 并提交自签占号', () async {
    final fakeRpc = _FakeIdentityRpc();
    final service = MyIdService(
      walletManager: _FakeWalletManager(const _AliceWallet()),
      chainRpc: _FakeChainRpc(),
      divisionStore: _FakeDivisionStore(),
      badgeSnapshotStore: _FakeBadgeStore(),
      identityRpc: fakeRpc,
      cidYearProvider: () => 2026,
    );

    final cid =
        await service.registerAnonymousCid(institution: kCidInstitutionCitizen);

    // 与 cid_generator 金标同源:accountId=_validAccountId, CTZN, 2026。
    final expected = generateCitizenCid(
      accountId: _validAccountId,
      institution: kCidInstitutionCitizen,
      year: 2026,
    );
    expect(cid, expected);
    expect(fakeRpc.occupiedCid, expected);
    expect(fakeRpc.occupiedAccountId, _validAccountId);
  });

  test('换绑:旧账户=默认用户、新账户=所选,参数逐一传对', () async {
    final newAccount = Account(
      masterId: _validAccountId,
      accountIndex: 5,
      accountId: '0x${'11' * 32}',
      ss58Address: 'new-ss58',
      accountName: '账户5',
    );
    final fakeRpc = _FakeIdentityRpc();
    final service = MyIdService(
      walletManager:
          _FakeWalletManager(const _AliceWallet(), accounts: [newAccount]),
      chainRpc: _FakeChainRpc(),
      divisionStore: _FakeDivisionStore(),
      badgeSnapshotStore: _FakeBadgeStore(),
      identityRpc: fakeRpc,
      identityResolver:
          _FakeIdentityResolver(_registeredIdentity(_validAccountId)),
    );

    await service.rebindCidTo(
      cidNumber: 'GD-CTZN1-8F3A2B',
      newAccountId: newAccount.accountId,
    );

    expect(fakeRpc.reboundCid, 'GD-CTZN1-8F3A2B');
    expect(fakeRpc.reboundOld, _validAccountId);
    expect(fakeRpc.reboundNew, newAccount.accountId);
  });

  test('换绑目标 == 当前身份账户时拒', () async {
    const self = Account(
      masterId: _validAccountId,
      accountIndex: 0,
      accountId: _validAccountId,
      ss58Address: _validAddress,
      accountName: '账户0',
    );
    final service = MyIdService(
      walletManager: _FakeWalletManager(const _AliceWallet(), accounts: [self]),
      chainRpc: _FakeChainRpc(),
      divisionStore: _FakeDivisionStore(),
      badgeSnapshotStore: _FakeBadgeStore(),
      identityRpc: _FakeIdentityRpc(),
      identityResolver:
          _FakeIdentityResolver(_registeredIdentity(_validAccountId)),
    );

    await expectLater(
      service.rebindCidTo(
        cidNumber: 'GD-CTZN1-8F3A2B',
        newAccountId: _validAccountId,
      ),
      throwsA(isA<WalletAuthException>()),
    );
  });

  test('listRebindTargets 排除默认用户账户', () async {
    const self = Account(
      masterId: _validAccountId,
      accountIndex: 0,
      accountId: _validAccountId,
      ss58Address: _validAddress,
      accountName: '账户0',
    );
    final other = Account(
      masterId: _validAccountId,
      accountIndex: 1,
      accountId: '0x${'22' * 32}',
      ss58Address: 'other-ss58',
      accountName: '账户1',
    );
    final service = MyIdService(
      walletManager:
          _FakeWalletManager(const _AliceWallet(), accounts: [self, other]),
      chainRpc: _FakeChainRpc(),
      divisionStore: _FakeDivisionStore(),
      badgeSnapshotStore: _FakeBadgeStore(),
      identityResolver:
          _FakeIdentityResolver(_registeredIdentity(_validAccountId)),
    );

    final targets = await service.listRebindTargets();
    expect(targets.map((account) => account.accountId).toList(),
        [other.accountId]);
  });

  test('listBindableAccounts 返回全部本地账户(含账户0)', () async {
    const acc0 = Account(
      masterId: _validAccountId,
      accountIndex: 0,
      accountId: _validAccountId,
      ss58Address: _validAddress,
      accountName: '账户0',
    );
    final acc5 = Account(
      masterId: _validAccountId,
      accountIndex: 5,
      accountId: '0x${'55' * 32}',
      ss58Address: 'ss5-addr',
      accountName: '账户5',
    );
    final service = MyIdService(
      walletManager:
          _FakeWalletManager(const _AliceWallet(), accounts: [acc0, acc5]),
      chainRpc: _FakeChainRpc(),
      divisionStore: _FakeDivisionStore(),
      badgeSnapshotStore: _FakeBadgeStore(),
    );

    final accounts = await service.listBindableAccounts();
    expect(accounts.map((a) => a.accountIndex).toList(), [0, 5]);
  });

  test('注册匿名 CID 可绑到所选子账户 //5(非账户0)', () async {
    final acc5 = Account(
      masterId: _validAccountId,
      accountIndex: 5,
      accountId: '0x${'55' * 32}',
      ss58Address: 'ss5-addr',
      accountName: '账户5',
    );
    final fakeRpc = _FakeIdentityRpc();
    final service = MyIdService(
      walletManager: _FakeWalletManager(const _AliceWallet(), accounts: [acc5]),
      chainRpc: _FakeChainRpc(),
      divisionStore: _FakeDivisionStore(),
      badgeSnapshotStore: _FakeBadgeStore(),
      identityRpc: fakeRpc,
      cidYearProvider: () => 2026,
    );

    final cid = await service.registerAnonymousCid(
      institution: kCidInstitutionCitizen,
      bindAccountId: acc5.accountId,
    );

    final expected = generateCitizenCid(
      accountId: acc5.accountId,
      institution: kCidInstitutionCitizen,
      year: 2026,
    );
    expect(cid, expected);
    expect(fakeRpc.occupiedCid, expected);
    // 绑到子账户 //5,而非默认账户0。
    expect(fakeRpc.occupiedAccountId, acc5.accountId);
  });
}

// ── SCALE 编码夹具(镜像 citizen-identity pallet 的 VotingIdentity/CandidateIdentity 布局) ──

List<int> _compact(int n) {
  if (n < 64) return [n << 2];
  if (n < 16384) {
    final x = (n << 2) | 1;
    return [x & 0xff, (x >> 8) & 0xff];
  }
  throw ArgumentError('测试夹具只覆盖短向量');
}

List<int> _vec(String s) {
  final bytes = utf8.encode(s);
  return [..._compact(bytes.length), ...bytes];
}

List<int> _u32(int v) =>
    [v & 0xff, (v >> 8) & 0xff, (v >> 16) & 0xff, (v >> 24) & 0xff];

Uint8List _encodeVoting({
  required int from,
  required int until,
  required int status,
  required String province,
  required String city,
  required String town,
  int updatedAt = 1,
}) {
  return Uint8List.fromList([
    ..._u32(from),
    ..._u32(until),
    status,
    ..._vec(province),
    ..._vec(city),
    ..._vec(town),
    ..._u32(updatedAt),
  ]);
}

Uint8List _encodeCandidate({
  required String province,
  required String city,
  required String town,
  required String familyName,
  required String givenName,
  required int sex,
  int birthDate = 20000131,
  int updatedAt = 1,
}) {
  return Uint8List.fromList([
    ..._vec(province),
    ..._vec(city),
    ..._vec(town),
    ..._vec(familyName),
    ..._vec(givenName),
    sex,
    ..._u32(birthDate),
    ..._u32(updatedAt),
  ]);
}

// ── Fakes ──

class _AliceWallet implements WalletProfile {
  const _AliceWallet();
  @override
  String get accountId => _validAccountId;
  @override
  String get ss58Address => _validAddress;
  @override
  bool get isHotWallet => true;
  @override
  bool get isColdWallet => false;
  @override
  dynamic noSuchMethod(Invocation invocation) => throw UnimplementedError();
}

class _FakeWalletManager extends WalletManager {
  _FakeWalletManager(this._wallet, {this.accounts = const <Account>[]});
  final WalletProfile? _wallet;
  final List<Account> accounts;
  @override
  Future<WalletProfile?> getDefaultWallet() async => _wallet;
  @override
  Future<List<Account>> getAccounts(String masterId) async => accounts;
  @override
  Future<Account?> getAccountByAccountId(String accountId) async {
    for (final account in accounts) {
      if (account.accountId == accountId) return account;
    }
    return null;
  }
}

/// 预设身份账户解析结果的假 resolver(换绑/列举目标测试用,绕开真链读序列)。
class _FakeIdentityResolver extends IdentityAccountResolver {
  _FakeIdentityResolver(this._resolved);
  final ResolvedIdentity? _resolved;
  @override
  Future<ResolvedIdentity?> resolve() async => _resolved;
}

/// 造一个「已注册(匿名)身份账户」解析结果:accountId 绑了 CID(snapshot 非空)。
ResolvedIdentity _registeredIdentity(String accountId) => ResolvedIdentity(
      accountId: accountId,
      ss58Address: _validAddress,
      accountIndex: 0,
      snapshot: CitizenIdentityChainSnapshot(
        cidNumber: 'GD-CTZN1-8F3A2B',
        accountId: Uint8List(32),
        votingIdentity: null,
        candidateIdentity: null,
      ),
    );

/// 记录占号 / 换绑调用参数的假 RPC(不上链),验证 service 编排把账户与 CID 传对。
class _FakeIdentityRpc extends CitizenIdentityRpc {
  _FakeIdentityRpc();

  String? occupiedCid;
  String? occupiedAccountId;
  String? reboundCid;
  String? reboundOld;
  String? reboundNew;

  @override
  Future<({String txHash, int usedNonce, String blockHashHex})> selfOccupyCid({
    required String cidNumber,
    required String accountId,
    required String fromSs58Address,
  }) async {
    occupiedCid = cidNumber;
    occupiedAccountId = accountId;
    return (txHash: '0xtx', usedNonce: 0, blockHashHex: '0xblk');
  }

  @override
  Future<({String txHash, int usedNonce, String blockHashHex})>
      selfRebindCidAccount({
    required String cidNumber,
    required String newAccountId,
    required String oldAccountId,
    required String newFromSs58Address,
  }) async {
    reboundCid = cidNumber;
    reboundNew = newAccountId;
    reboundOld = oldAccountId;
    return (txHash: '0xtx', usedNonce: 0, blockHashHex: '0xblk');
  }
}

class _FakeChainRpc extends ChainRpc {
  _FakeChainRpc({
    this.voting,
    this.candidate,
    this.throws = false,
    this.mismatchWallet = false,
    this.cidStatus = 0,
    this.hasCid = false,
  });
  final Uint8List? voting;
  final Uint8List? candidate;
  final bool throws;
  static const String cidNumber = 'GD-CTZN1-8F3A2B';
  final bool mismatchWallet;
  final int cidStatus;

  /// 是否存在 `CidByAccountId`。有 voting 必有 cid;匿名态显式置 true 表示
  /// 「有 CID 且绑定闭环、但无 voting」,用来驱动 reader 的匿名已注册分支。
  final bool hasCid;
  int _readIndex = 0;

  @override
  Future<({Uint8List blockHash, int blockNumber})>
      fetchFinalizedBlock() async => (blockHash: Uint8List(32), blockNumber: 1);

  @override
  Future<Uint8List?> fetchStorageAtBlock(
    String storageKeyHex,
    String blockHashHex,
  ) async {
    if (throws) throw StateError('smoldot 未就绪');
    final current = _readIndex++;
    if (current == 0) {
      // 有投票身份必有 CID;匿名态(hasCid=true)也有 CID 但后续 voting 读为 null。
      final present = voting != null || hasCid;
      return present ? Uint8List.fromList(_vec(cidNumber)) : null;
    }
    if (current == 1) {
      final accountId = Uint8List.fromList(
        Keyring().decodeAddress(_validAddress),
      );
      if (mismatchWallet) accountId[0] ^= 0xff;
      return accountId;
    }
    if (current == 2) {
      return Uint8List.fromList([
        ..._vec('FEDERAL_REGISTRY-CID'),
        ...List<int>.filled(32, 7),
        ..._vec('GD'),
        ..._vec('0755'),
        cidStatus,
        ..._u32(1),
        0,
      ]);
    }
    if (current == 3) return voting;
    if (current == 4) return candidate;
    throw StateError('读取次数超出身份闭环');
  }
}

class _FakeDivisionStore implements AdminDivisionStore {
  @override
  Future<String> divisionName(
          String level, String scopeKey, String code) async =>
      'N($code)';
  @override
  dynamic noSuchMethod(Invocation invocation) => throw UnimplementedError();
}

class _FakeBadgeStore extends IdentityBadgeSnapshotStore {
  @override
  Future<void> write({
    required String accountId,
    required String identityLevel,
  }) async {}
  @override
  Future<IdentityBadgeSnapshot?> read(String accountId) async => null;
}
