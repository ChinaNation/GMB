import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/8964/chain/square_chain_service.dart';
import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/pages/square_home_page.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/8964/services/square_identity_state.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:shared_preferences/shared_preferences.dart';

class _FakeWalletManager extends WalletManager {
  _FakeWalletManager(this.wallet);

  final WalletProfile? wallet;

  @override
  Future<WalletProfile?> getWallet() async => wallet;

  @override
  Future<WalletProfile?> getDefaultWallet() async => wallet;
}

/// 可切换默认钱包的 fake：模拟「我的钱包」拖拽置顶另一个热钱包。
class _SwitchableWalletManager extends WalletManager {
  _SwitchableWalletManager(this.wallet);

  WalletProfile? wallet;

  @override
  Future<WalletProfile?> getWallet() async => wallet;

  @override
  Future<WalletProfile?> getDefaultWallet() async => wallet;
}

WalletProfile _hotWallet({required int index, required String address}) {
  return WalletProfile(
    walletIndex: index,
    walletName: '钱包$index',
    walletIcon: '',
    balance: 0,
    address: address,
    pubkeyHex: 'a' * 64,
    alg: 'sr25519',
    ss58: 2027,
    createdAtMillis: index,
    source: 'test',
    signMode: 'local',
  );
}

class _FakeSquareChainService extends SquareChainService {
  _FakeSquareChainService(this.cidNumber);

  final String? cidNumber;
  int fetchIdentityCount = 0;

  @override
  Future<String?> fetchNormalCitizenCidNumber(String ownerAccount) async {
    return cidNumber;
  }

  @override
  Future<({String? cidNumber, String identityLevel})> fetchIdentity(
    String ownerAccount,
  ) async {
    fetchIdentityCount += 1;
    return (
      cidNumber: cidNumber,
      identityLevel: cidNumber == null ? 'visitor' : 'voting',
    );
  }
}

class _FakeFeedSource implements SquareFeedSource {
  const _FakeFeedSource();

  @override
  Future<List<SquarePost>> fetchFeed({
    required SquareFeedKind feedKind,
    int limit = 20,
    SquareSession? session,
  }) async {
    return const <SquarePost>[];
  }
}

Widget _wrap(Widget child) {
  return MaterialApp(
    theme: AppTheme.lightTheme,
    home: Scaffold(body: child),
  );
}

void main() {
  setUp(() {
    SharedPreferences.setMockInitialValues({});
  });

  testWidgets('广场首页默认进入推荐流并可切换分类', (tester) async {
    final identityService = SquareIdentityService(
      walletManager: _FakeWalletManager(null),
    );

    await tester.pumpWidget(
      _wrap(SquareHomePage(
        identityService: identityService,
        feedSource: const _FakeFeedSource(),
        membershipLoader: () async => const SquareMembershipState(
          active: true,
          expiresAt: 9999999999999,
          membershipLevel: 'freedom',
        ),
      )),
    );
    await tester.pumpAndSettle();

    expect(find.text('广场'), findsOneWidget);
    expect(find.text('推荐'), findsWidgets);
    expect(find.text('暂无推荐动态'), findsOneWidget);

    await tester.tap(find.text('关注'));
    await tester.pumpAndSettle();
    expect(find.text('暂无关注动态'), findsOneWidget);

    await tester.tap(find.text('竞选'));
    await tester.pumpAndSettle();
    expect(find.text('暂无竞选动态'), findsOneWidget);
  });

  testWidgets('无订阅钱包禁止打开任何发布页', (tester) async {
    final chainService = _FakeSquareChainService(null);
    final identityService = SquareIdentityService(
      walletManager: _FakeWalletManager(
        const WalletProfile(
          walletIndex: 1,
          walletName: '测试钱包',
          walletIcon: '',
          balance: 0,
          address: 'gmb_test_owner_account',
          pubkeyHex:
              'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
          alg: 'sr25519',
          ss58: 2027,
          createdAtMillis: 1,
          source: 'test',
          signMode: 'local',
        ),
      ),
      chainService: chainService,
    );

    await tester.pumpWidget(
      _wrap(SquareHomePage(
        identityService: identityService,
        feedSource: const _FakeFeedSource(),
        membershipLoader: () async => const SquareMembershipState(
          active: false,
          expiresAt: 0,
        ),
      )),
    );
    await tester.pumpAndSettle();

    // 广场首页只读本地徽章快照，不读取链。
    expect(chainService.fetchIdentityCount, 0);

    await tester.tap(find.byTooltip('发布动态'));
    await tester.pumpAndSettle();

    // 无订阅时服务入口立即阻断，不打开类型选择或编辑器，也不触发链身份查询。
    expect(find.text('需要有效会员才能发布广场内容'), findsOneWidget);
    expect(find.text('发动态'), findsNothing);
    expect(find.text('发文章'), findsNothing);
    expect(chainService.fetchIdentityCount, 0);
  });

  testWidgets('walletsRevision 自增(切换默认用户)后广场身份即时重载', (tester) async {
    // 地址 ≤14 字符时 accountLabel 原样显示,便于用 Tooltip 断言身份切换。
    final walletManager = _SwitchableWalletManager(
      _hotWallet(index: 1, address: 'addr_user_a'),
    );
    final identityService = SquareIdentityService(
      walletManager: walletManager,
      chainService: _FakeSquareChainService(null),
    );

    await tester.pumpWidget(
      _wrap(SquareHomePage(
        identityService: identityService,
        feedSource: const _FakeFeedSource(),
      )),
    );
    await tester.pumpAndSettle();
    expect(find.byTooltip('addr_user_a'), findsOneWidget);

    // 模拟在「我的钱包」拖拽置顶另一个热钱包:默认钱包变化 + 版本号广播。
    walletManager.wallet = _hotWallet(index: 2, address: 'addr_user_b');
    WalletManager.walletsRevision.value++;
    await tester.pumpAndSettle();

    expect(find.byTooltip('addr_user_b'), findsOneWidget);
    expect(find.byTooltip('addr_user_a'), findsNothing);
  });
}
