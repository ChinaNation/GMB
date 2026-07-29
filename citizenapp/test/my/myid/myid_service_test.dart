import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:shared_preferences/shared_preferences.dart';
import 'package:citizenapp/citizen/cid/cid_generator.dart';
import 'package:citizenapp/citizen/public/data/admin_division_store.dart';
import 'package:citizenapp/my/myid/citizen_identity_chain_reader.dart';
import 'package:citizenapp/my/myid/identity_account_resolver.dart';
import 'package:citizenapp/my/myid/identity_badge_snapshot_store.dart';
import 'package:citizenapp/my/myid/identity_rebind_revoker.dart';
import 'package:citizenapp/my/myid/identity_synced_account_store.dart';
import 'package:citizenapp/my/myid/myid_service.dart';
import 'package:citizenapp/my/user/contact_service.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/rpc/citizen_identity_rpc.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// Alice 通用 SS58(校验和有效),仅用于让 `decodeAddress` 解出 32 字节账户;
/// 护照 App 真号是 prefix=2027,这里只需一个可解码地址驱动 storage key。
const _validAddress = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
const _validAccountId =
    '0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  setUp(() {
    // 换绑续跑意图存 SharedPreferences;每个用例从空白起,互不串扰。
    SharedPreferences.setMockInitialValues(<String, Object>{});
  });

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

  group('子钥懒绑定 ensureDeviceSubkeyBound', () {
    setUp(() => SharedPreferences.setMockInitialValues(<String, Object>{}));

    test('未注册 CID 时拒绝绑定:后端不收未绑 CID 账户的子钥', () async {
      final wallet = _FakeWalletManager(const _AliceWallet());
      final service = MyIdService(
        walletManager: wallet,
        identityResolver: _FakeIdentityResolver(_unregisteredIdentity(_validAccountId)),
        chainRpc: _FakeChainRpc(),
        divisionStore: _FakeDivisionStore(),
        badgeSnapshotStore: _FakeBadgeStore(),
      );
      await expectLater(
        service.ensureDeviceSubkeyBound(),
        throwsA(isA<WalletAuthException>()),
      );
      expect(wallet.subkeyRebindCalls, 0);
    });

    test('首次进入需 CID 页面时绑定一次并推进标记;再次进入不重复弹窗', () async {
      final wallet = _FakeWalletManager(const _AliceWallet());
      final service = MyIdService(
        walletManager: wallet,
        identityResolver: _FakeIdentityResolver(_registeredIdentity(_validAccountId)),
        chainRpc: _FakeChainRpc(),
        divisionStore: _FakeDivisionStore(),
        badgeSnapshotStore: _FakeBadgeStore(),
      );
      await service.ensureDeviceSubkeyBound();
      expect(wallet.subkeyRebindCalls, 1);
      expect(await IdentitySyncedAccountStore().read(), _validAccountId);

      // 标记已与身份账户一致 → 直接短路,不再绑、不再弹生物识别。
      await service.ensureDeviceSubkeyBound();
      expect(wallet.subkeyRebindCalls, 1);
    });

    test('绑定失败不推进标记,下次仍会重试', () async {
      final wallet =
          _FakeWalletManager(const _AliceWallet(), failFirstSubkeyRebind: true);
      final service = MyIdService(
        walletManager: wallet,
        identityResolver: _FakeIdentityResolver(_registeredIdentity(_validAccountId)),
        chainRpc: _FakeChainRpc(),
        divisionStore: _FakeDivisionStore(),
        badgeSnapshotStore: _FakeBadgeStore(),
      );
      await expectLater(service.ensureDeviceSubkeyBound(), throwsA(isA<Object>()));
      expect(await IdentitySyncedAccountStore().read(), isNull);

      await service.ensureDeviceSubkeyBound();
      expect(wallet.subkeyRebindCalls, 2);
      expect(await IdentitySyncedAccountStore().read(), _validAccountId);
    });
  });

  group('注册前余额闸 fetchRegistrationAffordability', () {
    test('门槛取自链上常量,余额旁路缓存读取', () async {
      final rpc = _FakeChainRpc()
        ..minSelfPayFen = BigInt.from(121)
        ..balanceYuan = 1.21;
      final service = MyIdService(
        walletManager: _FakeWalletManager(const _AliceWallet()),
        chainRpc: rpc,
        divisionStore: _FakeDivisionStore(),
        badgeSnapshotStore: _FakeBadgeStore(),
      );
      final result =
          await service.fetchRegistrationAffordability(_validAccountId);
      expect(result.requiredFen, BigInt.from(121));
      expect(result.balanceFen, BigInt.from(121));
    });

    test('链读失败必须上抛,绝不静默当成余额充足或不足', () async {
      final rpc = _FakeChainRpc()..balanceThrows = true;
      final service = MyIdService(
        walletManager: _FakeWalletManager(const _AliceWallet()),
        chainRpc: rpc,
        divisionStore: _FakeDivisionStore(),
        badgeSnapshotStore: _FakeBadgeStore(),
      );
      await expectLater(
        service.fetchRegistrationAffordability(_validAccountId),
        throwsA(isA<StateError>()),
      );
    });
  });

  test('无默认热钱包时为访客并提示创建钱包', () async {
    final state = await buildService(wallet: null).getState();
    expect(state.tier, MyIdTier.visitor);
    expect(state.votingAccountId, isNull);
    expect(state.errorMessage, '请先创建钱包');
  });

  test('账户0 链上无投票身份时为访客轻节点', () async {
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

  test('注册匿名 CID:用账户0 accountId + UTC 年生成金标 CID 并提交自签占号', () async {
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

  test('换绑:旧账户=账户0、新账户=所选,参数逐一传对', () async {
    final newAccount = Account(
      masterId: _validAccountId,
      accountIndex: 5,
      accountId: '0x${'11' * 32}',
      ss58Address: 'new-ss58',
      accountName: '账户5',
    );
    final fakeRpc = _FakeIdentityRpc();
    final fakeWallet =
        _FakeWalletManager(const _AliceWallet(), accounts: [newAccount]);
    final fakeContact = _FakeContactService();
    final fakeRevoker = _FakeRebindRevoker();
    // 两个 fake 共用一条轨迹,用于断言本地重建各步的先后顺序。
    final trace = <String>[];
    fakeWallet.trace = trace;
    fakeContact.trace = trace;
    final service = MyIdService(
      walletManager: fakeWallet,
      chainRpc: _FakeChainRpc(),
      divisionStore: _FakeDivisionStore(),
      badgeSnapshotStore: _FakeBadgeStore(),
      identityRpc: fakeRpc,
      identityResolver:
          _FakeIdentityResolver(_registeredIdentity(_validAccountId)),
      contactService: fakeContact,
      rebindRevoker: fakeRevoker,
    );

    await service.rebindCidTo(
      cidNumber: 'GD-CTZN1-8F3A2B',
      newAccountId: newAccount.accountId,
    );

    // 链上换绑参数传对。
    expect(fakeRpc.reboundCid, 'GD-CTZN1-8F3A2B');
    expect(fakeRpc.reboundOld, _validAccountId);
    expect(fakeRpc.reboundNew, newAccount.accountId);
    // Finalized 后本地重建:设备子钥重绑 + 通讯录迁移都指向新账户。
    expect(fakeWallet.subkeyRebindTargets, [newAccount.accountId]);
    expect(fakeContact.migrations.single.oldAccountId, _validAccountId);
    expect(fakeContact.migrations.single.newAccountId, newAccount.accountId);
    // 本地静止态数据密钥(LDK)必须重 wrap 到新账户。
    // 漏这一步 → 新账户读不到 LDK wrap → 会新生成一把 → 聊天/MLS/附件/通讯录
    // 已落盘密文全部不可解密(死契约 [[cid-rebind-subkeys-must-auto-migrate]])。
    expect(fakeWallet.ldkRewraps.single.oldAccountId, _validAccountId);
    expect(fakeWallet.ldkRewraps.single.newAccountId, newAccount.accountId);
    // 顺序钉死:通讯录迁移要读旧账户本地密文,而重 wrap 会删旧账户 LDK wrap,
    // 倒过来迁移就读不出旧数据了。
    expect(trace, <String>['contact_migrate', 'ldk_rewrap']);
    // 吊销旧账户云端隐私/鉴权数据(换绑止损)。
    expect(fakeRevoker.revoked, [_validAccountId]);
    // 重建完成 → 「已同步账户」标记推进到新账户。
    expect(await IdentitySyncedAccountStore().read(), newAccount.accountId);
  });

  test('换绑本地重建中断:标记不推进,对账自动补齐(死契约)', () async {
    final newAccount = Account(
      masterId: _validAccountId,
      accountIndex: 5,
      accountId: '0x${'11' * 32}',
      ss58Address: 'new-ss58',
      accountName: '账户5',
    );
    final fakeWallet = _FakeWalletManager(
      const _AliceWallet(),
      accounts: [newAccount],
      failFirstSubkeyRebind: true,
    );
    final fakeContact = _FakeContactService();
    final fakeRevoker = _FakeRebindRevoker();
    // 链上先 = 旧账户;稳态基线:已同步标记 = 旧账户。
    final resolver = _MutableResolver(_validAccountId);
    await IdentitySyncedAccountStore().write(_validAccountId);
    final service = MyIdService(
      walletManager: fakeWallet,
      chainRpc: _FakeChainRpc(),
      divisionStore: _FakeDivisionStore(),
      badgeSnapshotStore: _FakeBadgeStore(),
      identityRpc: _FakeIdentityRpc(),
      identityResolver: resolver,
      contactService: fakeContact,
      rebindRevoker: fakeRevoker,
    );

    // 换绑链上成功,但设备子钥重绑首次失败 → 本地重建中断上抛。
    await expectLater(
      service.rebindCidTo(
        cidNumber: 'GD-CTZN1-8F3A2B',
        newAccountId: newAccount.accountId,
      ),
      throwsA(isA<StateError>()),
    );
    // 通讯录未迁移;标记未推进(仍旧账户)——链上真值与标记的差异留待对账,无一次性意图丢失窗口。
    expect(fakeContact.migrations, isEmpty);
    expect(await IdentitySyncedAccountStore().read(), _validAccountId);

    // 链上已换绑到新账户;下次对账:链上真值(new) != 标记(old) → 补齐迁移。
    resolver.setAccountId(newAccount.accountId);
    await service.reconcileIdentityRebuild();
    expect(fakeWallet.subkeyRebindTargets, [newAccount.accountId]);
    expect(fakeContact.migrations.single.newAccountId, newAccount.accountId);
    expect(fakeRevoker.revoked, [_validAccountId]);
    expect(await IdentitySyncedAccountStore().read(), newAccount.accountId);
  });

  test('吊销网络失败不丢 outbox:同步标记已推进仍由后续对账独立重试', () async {
    final newAccount = Account(
      masterId: _validAccountId,
      accountIndex: 5,
      accountId: '0x${'11' * 32}',
      ss58Address: 'new-ss58',
      accountName: '账户5',
    );
    final resolver = _MutableResolver(_validAccountId);
    final revoker = _FakeRebindRevoker(failFirstRevoke: true);
    await IdentitySyncedAccountStore().write(_validAccountId);
    final service = MyIdService(
      walletManager:
          _FakeWalletManager(const _AliceWallet(), accounts: [newAccount]),
      chainRpc: _FakeChainRpc(),
      divisionStore: _FakeDivisionStore(),
      badgeSnapshotStore: _FakeBadgeStore(),
      identityRpc: _FakeIdentityRpc(),
      identityResolver: resolver,
      contactService: _FakeContactService(),
      rebindRevoker: revoker,
    );

    await service.rebindCidTo(
      cidNumber: 'GD-CTZN1-8F3A2B',
      newAccountId: newAccount.accountId,
    );
    // 链上和功能凭证迁移已经成功；安全清理首次网络失败时记录仍在，不能假装完成。
    expect(await IdentitySyncedAccountStore().read(), newAccount.accountId);
    expect(revoker.pending?.oldAccountId, _validAccountId);
    expect(revoker.revoked, isEmpty);

    resolver.setAccountId(newAccount.accountId);
    await service.reconcileIdentityRebuild();
    expect(revoker.revoked, [_validAccountId]);
    expect(revoker.pending, isNull);
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

  test('listRebindTargets 排除当前身份账户', () async {
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
  _FakeWalletManager(
    this._wallet, {
    this.accounts = const <Account>[],
    this.failFirstSubkeyRebind = false,
  });
  final WalletProfile? _wallet;
  final List<Account> accounts;

  /// true 时 [bindDeviceSubkeyToAccountId] 首次抛错、之后成功,用于验证换绑本地
  /// 重建中断后意图留存 + 续跑补齐(死契约 [[cid-rebind-subkeys-must-auto-migrate]])。
  final bool failFirstSubkeyRebind;
  int subkeyRebindCalls = 0;
  final List<String> subkeyRebindTargets = <String>[];

  /// 换绑本地重建的调用顺序轨迹。与 [_FakeContactService.trace] 指向同一个 list,
  /// 用来钉死「通讯录迁移 → LDK 重 wrap」的先后关系。
  List<String> trace = <String>[];

  /// LDK 重 wrap 的实参记录。
  final List<({String oldAccountId, String newAccountId})> ldkRewraps =
      <({String oldAccountId, String newAccountId})>[];

  @override
  Future<void> rewrapLocalDataKeyForRebind({
    required String oldAccountId,
    required String newAccountId,
  }) async {
    trace.add('ldk_rewrap');
    ldkRewraps.add((oldAccountId: oldAccountId, newAccountId: newAccountId));
  }

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

  @override
  Future<void> bindDeviceSubkeyToAccountId(String identityAccountId) async {
    subkeyRebindCalls++;
    if (failFirstSubkeyRebind && subkeyRebindCalls == 1) {
      throw StateError('设备子钥重绑首次失败(模拟网络抖动)');
    }
    subkeyRebindTargets.add(identityAccountId);
  }
}

/// 记录换绑通讯录迁移调用的假通讯录服务(不触碰 Isar / secure storage / 网络)。
class _FakeContactService extends UserContactService {
  _FakeContactService() : super(autoSync: false);
  final List<({String oldAccountId, String newAccountId})> migrations =
      <({String oldAccountId, String newAccountId})>[];

  /// 与 [_FakeWalletManager.trace] 共用同一个 list,记录调用顺序。
  List<String> trace = <String>[];

  @override
  Future<void> migrateContactsToNewIdentity(
    String oldAccountId,
    String newAccountId,
  ) async {
    trace.add('contact_migrate');
    migrations.add((oldAccountId: oldAccountId, newAccountId: newAccountId));
  }
}

/// 预设身份账户解析结果的假 resolver(换绑/列举目标测试用,绕开真链读序列)。
class _FakeIdentityResolver extends IdentityAccountResolver {
  _FakeIdentityResolver(this._resolved);
  final ResolvedIdentity? _resolved;
  @override
  Future<ResolvedIdentity?> resolve() async => _resolved;
}

/// 可变身份账户的假 resolver(对账测试:换绑前后链上身份账户切换)。
class _MutableResolver extends IdentityAccountResolver {
  _MutableResolver(this._accountId);
  String _accountId;
  void setAccountId(String accountId) => _accountId = accountId;
  @override
  Future<ResolvedIdentity?> resolve() async => _registeredIdentity(_accountId);
}

/// 记录旧账户云端吊销调用的假 revoker(不触网 / 不触碰设备子钥)。
class _FakeRebindRevoker extends IdentityRebindRevoker {
  _FakeRebindRevoker({this.failFirstRevoke = false});

  final bool failFirstRevoke;
  int revokeCalls = 0;
  final List<String> revoked = <String>[];
  PendingRebindCleanup? pending;

  @override
  Future<void> stagePendingCleanup({
    required String cidNumber,
    required String oldAccountId,
    required String newAccountId,
    required String oldAccountSignature,
  }) async {
    pending = PendingRebindCleanup(
      cidNumber: cidNumber,
      oldAccountId: oldAccountId,
      newAccountId: newAccountId,
      oldAccountSignature: oldAccountSignature,
    );
  }

  @override
  Future<void> ensureCanStartRebind({
    required String cidNumber,
    required String oldAccountId,
    required String newAccountId,
  }) async {}

  @override
  Future<bool> retryPendingCleanup({
    required String cidNumber,
    required String currentAccountId,
  }) async {
    final record = pending;
    if (record == null || record.newAccountId != currentAccountId) return false;
    revokeCalls += 1;
    if (failFirstRevoke && revokeCalls == 1) {
      throw StateError('模拟吊销网络失败');
    }
    revoked.add(record.oldAccountId);
    pending = null;
    return true;
  }
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

/// 未注册身份(有热钱包、无 CID):懒绑定必须在这种态下拒绝绑定子钥。
ResolvedIdentity _unregisteredIdentity(String accountId) => ResolvedIdentity(
      accountId: accountId,
      ss58Address: _validAddress,
      accountIndex: 0,
      snapshot: null,
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
    Future<void> Function(String oldAccountSignature)? onOldAuthorizationReady,
  }) async {
    reboundCid = cidNumber;
    reboundNew = newAccountId;
    reboundOld = oldAccountId;
    await onOldAuthorizationReady?.call('0x${'ab' * 64}');
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

  /// 链上下发的自付门槛(分)与账户余额(元),供注册前余额闸用例驱动三分支。
  BigInt minSelfPayFen = BigInt.from(121);
  double balanceYuan = 0.0;
  bool balanceThrows = false;

  @override
  Future<BigInt> fetchMinSelfPayBalanceFen() async {
    if (balanceThrows) throw StateError('metadata 未就绪');
    return minSelfPayFen;
  }

  @override
  Future<double> fetchFinalizedBalance(String publicKey,
      {bool forceFresh = false}) async {
    if (balanceThrows) throw StateError('smoldot 未就绪');
    // 余额闸必须旁路块内缓存,否则刚充完钱的用户会被拿旧值再踢回充值页。
    expect(forceFresh, isTrue);
    return balanceYuan;
  }

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
