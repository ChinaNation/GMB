import 'package:citizenapp/my/myid/identity_badge_snapshot_store.dart';
import 'package:citizenapp/my/myid/myid_service.dart';
import 'package:citizenapp/my/user/user.dart';
import 'package:citizenapp/rpc/smoldot_client.dart';
import 'package:citizenapp/ui/identity_badge.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';

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

void main() {
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
      address: 'wallet_profile_test',
      pubkeyHex:
          'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
      alg: 'sr25519',
      ss58: 2027,
      createdAtMillis: 1,
      source: 'test',
      signMode: 'local',
    );
    await snapshotStore.write(
      walletAccount: wallet.address,
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

    await tester.pumpWidget(const SizedBox.shrink());
    await smoldot.dispose();
  });
}
