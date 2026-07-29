import 'package:citizenapp/my/myid/identity_account_cache.dart';
import 'package:citizenapp/my/myid/identity_account_resolver.dart';
import 'package:citizenapp/my/myid/identity_badge_snapshot_store.dart';
import 'package:citizenapp/my/myid/myid_service.dart';
import 'package:citizenapp/my/user/user.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/rpc/smoldot_client.dart';
import 'package:citizenapp/ui/identity_badge.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../8964/profile/fake_profile.dart';

/// 身份账户缓存 fake：resolve/accountId 返回 null，让调用方回退 wallet.accountId
/// （身份=账户0 常态），行为与迁移前一致；避免 instance 触发真链读/真 Isar。
class _NullIdentityCache extends IdentityAccountCache {
  @override
  Future<ResolvedIdentity?> resolve({bool allowChainRead = true}) async => null;
  @override
  Future<String?> accountId({bool allowChainRead = true}) async => null;
}

class _FakeWalletManager extends WalletManager {
  _FakeWalletManager(this.wallet);

  final WalletProfile wallet;

  @override
  Future<WalletProfile?> getDefaultWallet() async => wallet;
}

class _CountingMyIdService extends MyIdService {
  int liveReadCount = 0;

  @override
  Future<MyIdState> getState() async {
    liveReadCount += 1;
    return const MyIdState(
      tier: MyIdTier.visitor,
      status: MyIdStatus.queryFailed,
    );
  }
}

class _FakeSquareApi extends SquareApiClient {
  int membershipCalls = 0;

  @override
  Future<SquareMembershipState> fetchMembership(SquareSession session) async {
    membershipCalls += 1;
    return const SquareMembershipState(active: false, paidUntil: 0);
  }
}

void main() {
  setUp(() {
    IdentityAccountCache.debugInstance = _NullIdentityCache();
  });

  tearDown(IdentityAccountCache.resetDebugInstance);

  testWidgets('我的页面只读徽章快照且不启动轻节点', (tester) async {
    SharedPreferences.setMockInitialValues({});
    final preferences = await SharedPreferences.getInstance();
    final snapshotStore = IdentityBadgeSnapshotStore(
      preferences: preferences,
    );
    const wallet = WalletProfile(
      walletIndex: 1,
      walletName: '测试钱包',
      walletIcon: '',
      balance: 0,
      ss58Address: 'wallet_profile_test',
      accountId:
          '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
      alg: 'sr25519',
      ss58: 2027,
      createdAtMillis: 1,
      source: 'test',
      signMode: 'local',
    );
    await snapshotStore.write(
      accountId: wallet.accountId,
      identityLevel: 'candidate',
    );

    var startCount = 0;
    final smoldot = SmoldotClientManager.forTesting(
      initialize: () async => startCount += 1,
    );
    final myIdService = _CountingMyIdService();

    await tester.pumpWidget(
      MaterialApp(
        home: MyTab(
          walletManager: _FakeWalletManager(wallet),
          myIdService: myIdService,
          badgeSnapshotStore: snapshotStore,
          smoldotClientManager: smoldot,
        ),
      ),
    );
    await tester.pump(const Duration(milliseconds: 300));

    expect(startCount, 0);
    expect(myIdService.liveReadCount, 0);
    expect(find.byType(IdentityBadge), findsOneWidget);
    expect(find.text('钱包'), findsOneWidget);
    expect(find.text('管理账户'), findsOneWidget);
    expect(find.text('身份'), findsOneWidget);
    expect(find.text('注册与查看'), findsOneWidget);
    expect(find.text('个人服务'), findsOneWidget);
    expect(find.text('会员｜订阅'), findsOneWidget);
    expect(find.text('创作者'), findsOneWidget);
    expect(find.text('通讯录'), findsOneWidget);
    final creatorTop = tester.getTopLeft(find.text('创作者')).dy;
    final contactsTop = tester.getTopLeft(find.text('通讯录')).dy;
    final membershipTop = tester.getTopLeft(find.text('会员｜订阅')).dy;
    expect(creatorTop, lessThan(contactsTop));
    expect(contactsTop, lessThan(membershipTop));

    await tester.pumpWidget(const SizedBox.shrink());
    await smoldot.dispose();
  });

  testWidgets('我的页面展示公开昵称且钱包改名广播不重复刷新资料', (tester) async {
    SharedPreferences.setMockInitialValues({});
    const wallet = WalletProfile(
      walletIndex: 1,
      walletName: '不得公开的钱包名',
      walletIcon: '',
      balance: 0,
      ss58Address: 'wallet_profile_test',
      accountId:
          '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
      alg: 'sr25519',
      ss58: 2027,
      createdAtMillis: 1,
      source: 'test',
      signMode: 'local',
    );
    final profileApi = FakeProfileApi(sampleProfile(displayName: '公开昵称'));
    final profileCache = FakeProfileCache(sampleProfile(displayName: '缓存昵称'));
    final squareApi = _FakeSquareApi();
    final smoldot = SmoldotClientManager.forTesting(initialize: () async {});

    await tester.pumpWidget(
      MaterialApp(
        home: MyTab(
          walletManager: _FakeWalletManager(wallet),
          myIdService: _CountingMyIdService(),
          smoldotClientManager: smoldot,
          profileApi: profileApi,
          profileCache: profileCache,
          sessionProvider: FakeSessionProvider(fakeSession()),
          squareApi: squareApi,
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('公开昵称'), findsOneWidget);
    expect(find.text('不得公开的钱包名'), findsNothing);
    expect(profileApi.calls, 1);
    expect(squareApi.membershipCalls, 1);

    // 钱包名变更也会产生 revision 广播，但身份账户不变时不得重拉公开资料。
    WalletManager.walletsRevision.value += 1;
    await tester.pumpAndSettle();
    expect(profileApi.calls, 1);

    await tester.pumpWidget(const SizedBox.shrink());
    await smoldot.dispose();
  });
}
