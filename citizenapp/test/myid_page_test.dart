import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/my/myid/myid_page.dart';
import 'package:citizenapp/my/myid/myid_service.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

const MyIdState _votingState = MyIdState(
  tier: MyIdTier.voting,
  status: MyIdStatus.normal,
  votingAccountId: 'w5BekTimvtfYZvFpkDzy7ypqUntPgTbjRFCt9weR8vMgf7o8E',
  cidNumber: 'CID-2026-0715',
  residenceDistrict: '中枢省 · 固市 · 和平镇',
  passportValidFrom: '2026-07-15',
  passportValidUntil: '2036-07-14',
);

const MyIdState _candidateState = MyIdState(
  tier: MyIdTier.candidate,
  status: MyIdStatus.normal,
  votingAccountId: 'w5BekTimvtfYZvFpkDzy7ypqUntPgTbjRFCt9weR8vMgf7o8E',
  cidNumber: 'CID-2026-0715',
  residenceDistrict: '中枢省 · 固市 · 和平镇',
  passportValidFrom: '2026-07-15',
  passportValidUntil: '2036-07-14',
  familyName: '张',
  givenName: '三',
  citizenSexLabel: '男',
  birthDistrict: '中枢省 · 固市 · 和平镇',
  citizenBirthDate: '1992-05-18',
);

void main() {
  Finder card(MyIdTier tier) =>
      find.byKey(ValueKey<String>('passport-card-${tier.name}'));

  double cardTop(WidgetTester tester, MyIdTier tier) =>
      tester.getTopLeft(card(tier)).dy;

  Future<void> pumpPage(
    WidgetTester tester,
    MyIdState state, {
    Size? surfaceSize,
  }) async {
    if (surfaceSize != null) {
      tester.view.physicalSize = surfaceSize;
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
    }
    await tester.pumpWidget(
      MaterialApp(home: MyIdPage(myIdService: _FakeMyIdService(state))),
    );
    await tester.pumpAndSettle();
  }

  testWidgets('三张身份卡始终存在且旧访客文案彻底删除', (tester) async {
    await pumpPage(tester, const MyIdState(tier: MyIdTier.visitor));

    expect(find.text('访客轻节点'), findsOneWidget);
    expect(find.text('公民身份 · 投票'), findsOneWidget);
    expect(find.text('公民身份 · 竞选'), findsOneWidget);
    // 旧文案零残留
    expect(find.text('匿名访客'), findsNothing);
    expect(find.text('公民 · 投票身份'), findsNothing);
    expect(find.text('公民 · 竞选身份'), findsNothing);
    expect(find.text('没有公民身份信息'), findsNothing);
    // 访客卡改用“匿名”小标签替代整段空态
    expect(find.byKey(const ValueKey<String>('passport-anonymous-tag')),
        findsOneWidget);
    expect(find.text('匿名'), findsOneWidget);
  });

  testWidgets('访客当前卡排第一且公民卡只显示字段名称', (tester) async {
    await pumpPage(tester, const MyIdState(tier: MyIdTier.visitor));

    expect(cardTop(tester, MyIdTier.visitor),
        lessThan(cardTop(tester, MyIdTier.voting)));
    expect(cardTop(tester, MyIdTier.voting),
        lessThan(cardTop(tester, MyIdTier.candidate)));
    expect(find.text('当前身份'), findsOneWidget);
    expect(find.byKey(const ValueKey<String>('current-identity-visitor')),
        findsOneWidget);
    expect(find.text('投票账户'), findsNWidgets(2));
    expect(find.text('公民姓名'), findsOneWidget);
    expect(find.text('—'), findsNothing);
  });

  testWidgets('投票身份卡置顶并且只有该卡显示真实值', (tester) async {
    await pumpPage(tester, _votingState);

    expect(cardTop(tester, MyIdTier.voting),
        lessThan(cardTop(tester, MyIdTier.visitor)));
    expect(cardTop(tester, MyIdTier.visitor),
        lessThan(cardTop(tester, MyIdTier.candidate)));
    expect(find.byKey(const ValueKey<String>('current-identity-voting')),
        findsOneWidget);
    expect(find.text('当前身份'), findsOneWidget);
    expect(find.text('w5BekTim…vMgf7o8E'), findsOneWidget);
    expect(find.text('CID-2026-0715'), findsOneWidget);
    expect(find.text('中枢省 · 固市 · 和平镇'), findsOneWidget);
    expect(find.text('正常'), findsOneWidget);
    expect(find.text('2026年07月15日 至 2036年07月14日'), findsOneWidget);

    final candidateCard = card(MyIdTier.candidate);
    expect(
      find.descendant(of: candidateCard, matching: find.text('公民姓名')),
      findsOneWidget,
    );
    expect(
      find.descendant(of: candidateCard, matching: find.text('CID-2026-0715')),
      findsNothing,
    );
  });

  testWidgets('竞选身份卡置顶并显示九项真实字段，投票卡不重复数据', (tester) async {
    await pumpPage(tester, _candidateState);

    expect(cardTop(tester, MyIdTier.candidate),
        lessThan(cardTop(tester, MyIdTier.visitor)));
    expect(cardTop(tester, MyIdTier.visitor),
        lessThan(cardTop(tester, MyIdTier.voting)));
    expect(find.byKey(const ValueKey<String>('current-identity-candidate')),
        findsOneWidget);
    expect(find.text('当前身份'), findsOneWidget);
    expect(find.text('w5BekTim…vMgf7o8E'), findsOneWidget);
    expect(find.text('CID-2026-0715'), findsOneWidget);
    expect(find.text('张三'), findsOneWidget);
    expect(find.text('男'), findsOneWidget);
    expect(find.text('1992年05月18日'), findsOneWidget);

    final votingCard = card(MyIdTier.voting);
    expect(
      find.descendant(of: votingCard, matching: find.text('投票账户')),
      findsOneWidget,
    );
    expect(
      find.descendant(of: votingCard, matching: find.text('CID-2026-0715')),
      findsNothing,
    );
    expect(
      find.descendant(of: votingCard, matching: find.text('w5BekTim…vMgf7o8E')),
      findsNothing,
    );
  });

  testWidgets('链读失败不降级访客，三卡都没有当前身份和真实值', (tester) async {
    await pumpPage(
      tester,
      const MyIdState(
        tier: MyIdTier.visitor,
        status: MyIdStatus.queryFailed,
        errorMessage: '链上身份读取失败',
      ),
    );

    expect(find.text('链上身份读取失败'), findsOneWidget);
    expect(find.text('重试'), findsOneWidget);
    expect(find.text('当前身份'), findsNothing);
    expect(find.text('访客轻节点'), findsOneWidget);
    expect(find.text('公民身份 · 投票'), findsOneWidget);
    expect(find.text('公民身份 · 竞选'), findsOneWidget);
    expect(find.text('—'), findsNothing);
  });

  testWidgets('没有默认热钱包时仍是访客当前身份并显示引导', (tester) async {
    await pumpPage(
      tester,
      const MyIdState(
        tier: MyIdTier.visitor,
        errorMessage: '请先创建钱包',
      ),
    );

    expect(find.text('请先创建钱包'), findsOneWidget);
    expect(find.byKey(const ValueKey<String>('current-identity-visitor')),
        findsOneWidget);
    // 空态文案已删，访客卡改以“匿名”小标签呈现
    expect(find.text('没有公民身份信息'), findsNothing);
    expect(find.byKey(const ValueKey<String>('passport-anonymous-tag')),
        findsOneWidget);
  });

  testWidgets('过期和吊销只改变当前卡状态，不改变身份排序', (tester) async {
    await pumpPage(
      tester,
      const MyIdState(
        tier: MyIdTier.voting,
        status: MyIdStatus.expired,
        votingAccountId: 'w5BekTimvtfYZvFpkDzy7ypqUntPgTbjRFCt9weR8vMgf7o8E',
        cidNumber: 'CID-2026-0715',
        residenceDistrict: '中枢省 · 固市 · 和平镇',
        passportValidFrom: '2020-01-01',
        passportValidUntil: '2025-01-01',
      ),
    );

    expect(cardTop(tester, MyIdTier.voting),
        lessThan(cardTop(tester, MyIdTier.visitor)));
    expect(find.text('已过期'), findsOneWidget);
  });

  testWidgets('窄屏和长字段不会产生布局溢出', (tester) async {
    await pumpPage(
      tester,
      const MyIdState(
        tier: MyIdTier.candidate,
        status: MyIdStatus.normal,
        votingAccountId: 'w5BekTimvtfYZvFpkDzy7ypqUntPgTbjRFCt9weR8vMgf7o8E',
        cidNumber: 'CID-VERY-LONG-2026-0715-EXAMPLE',
        residenceDistrict: '中枢省 · 很长的城市名称 · 很长的乡镇名称',
        passportValidFrom: '2026-07-15',
        passportValidUntil: '2036-07-14',
        familyName: '这是一个用于验证窄屏自动换行的较长公民',
        givenName: '姓名',
        citizenSexLabel: '男',
        birthDistrict: '中枢省 · 很长的出生城市名称 · 很长的出生乡镇名称',
        citizenBirthDate: '1992-05-18',
      ),
      surfaceSize: const Size(320, 1600),
    );

    expect(tester.takeException(), isNull);
    expect(find.text('公民身份 · 竞选'), findsOneWidget);
  });

  testWidgets('默认用户身份变化后重新排序且只保留一个当前标记', (tester) async {
    final service = _MutableMyIdService(
      const MyIdState(tier: MyIdTier.visitor),
    );
    await tester.pumpWidget(MaterialApp(home: MyIdPage(myIdService: service)));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey<String>('current-identity-visitor')),
        findsOneWidget);

    service.state = _candidateState;
    WalletManager.walletsRevision.value++;
    await tester.pumpAndSettle();

    expect(find.text('当前身份'), findsOneWidget);
    expect(find.byKey(const ValueKey<String>('current-identity-candidate')),
        findsOneWidget);
    expect(cardTop(tester, MyIdTier.candidate),
        lessThan(cardTop(tester, MyIdTier.visitor)));
  });

  testWidgets('不出现已下线的登记、换钱包和扫码入口', (tester) async {
    await pumpPage(tester, const MyIdState(tier: MyIdTier.visitor));
    expect(find.text('护照号'), findsNothing);
    expect(find.text('选择钱包'), findsNothing);
    expect(find.text('更换钱包'), findsNothing);
    expect(find.text('扫码签名'), findsNothing);
  });
}

class _FakeMyIdService extends MyIdService {
  _FakeMyIdService(this.state);

  final MyIdState state;

  @override
  Future<MyIdState> getState() async => state;
}

class _MutableMyIdService extends MyIdService {
  _MutableMyIdService(this.state);

  MyIdState state;

  @override
  Future<MyIdState> getState() async => state;
}
