import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/8964/chain/square_chain_service.dart';
import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/pages/square_home_page.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/8964/services/square_identity_state.dart';
import 'package:citizenapp/8964/storage/square_draft_store.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

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

  @override
  Future<String?> fetchNormalCitizenCidNumber(String ownerAccount) async {
    return cidNumber;
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

class _EmptyDraftStore implements SquareDraftRepository {
  const _EmptyDraftStore();

  @override
  Future<void> delete(String ownerAccount) async {}

  @override
  Future<SquarePublishDraft?> read(String ownerAccount) async => null;

  @override
  Future<void> save(SquarePublishDraft draft) async {}
}

Widget _wrap(Widget child) {
  return MaterialApp(
    theme: AppTheme.lightTheme,
    home: child,
  );
}

void main() {
  testWidgets('广场首页默认进入推荐流并可切换分类', (tester) async {
    final identityService = SquareIdentityService(
      walletManager: _FakeWalletManager(null),
    );

    await tester.pumpWidget(
      _wrap(SquareHomePage(
        identityService: identityService,
        feedSource: const _FakeFeedSource(),
        draftStore: const _EmptyDraftStore(),
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

  testWidgets('未认证钱包打开发布页时竞选发布入口禁用', (tester) async {
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
      chainService: _FakeSquareChainService(null),
    );

    await tester.pumpWidget(
      _wrap(SquareHomePage(
        identityService: identityService,
        feedSource: const _FakeFeedSource(),
        draftStore: const _EmptyDraftStore(),
      )),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('发布动态'));
    await tester.pumpAndSettle();

    // 发布入口先选类型：进「发动态」到达动态发布页。
    await tester.tap(find.text('发动态'));
    await tester.pumpAndSettle();

    expect(find.text('发布动态'), findsOneWidget);
    expect(find.text('测试钱包'), findsOneWidget);
    expect(find.text('当前钱包未认证，不能发布竞选内容。'), findsOneWidget);
    expect(find.widgetWithText(FilledButton, '签名发布'), findsOneWidget);
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
        draftStore: const _EmptyDraftStore(),
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
