import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/my/myid/myid_page.dart';
import 'package:citizenapp/my/myid/myid_service.dart';

void main() {
  Future<void> pumpPage(WidgetTester tester, MyIdState state) async {
    await tester.pumpWidget(
      MaterialApp(home: MyIdPage(myIdService: _FakeMyIdService(state))),
    );
    await tester.pumpAndSettle();
  }

  testWidgets('访客档显示完全匿名,不显示公民字段', (tester) async {
    await pumpPage(tester, const MyIdState(tier: MyIdTier.visitor));

    expect(find.text('访客'), findsWidgets);
    expect(find.text('完全匿名'), findsOneWidget);
    expect(find.text('无公民身份'), findsOneWidget);
    expect(find.text('公民身份 CID 号'), findsNothing);
    expect(find.text('竞选公开信息'), findsNothing);
  });

  testWidgets('投票公民档显示 CID/居住选区/有效期', (tester) async {
    await pumpPage(
      tester,
      const MyIdState(
        tier: MyIdTier.voting,
        status: MyIdStatus.normal,
        votingAccount: '5FtestVotingAcc',
        cidNumber: 'GD-CTZN1-8F3A2B',
        residenceDistrict: '广东 · 深圳 · 南山',
        passportValidFrom: '2026-01-01',
        passportValidUntil: '2031-01-01',
      ),
    );

    expect(find.text('投票公民'), findsOneWidget);
    expect(find.text('投票账户'), findsOneWidget);
    expect(find.text('5FtestVotingAcc'), findsOneWidget);
    expect(find.text('公民身份 CID 号'), findsOneWidget);
    expect(find.text('GD-CTZN1-8F3A2B'), findsOneWidget);
    expect(find.text('居住选区'), findsOneWidget);
    expect(find.text('广东 · 深圳 · 南山'), findsOneWidget);
    expect(find.text('投票身份有效期'), findsOneWidget);
    expect(find.text('2026年01月01日 至 2031年01月01日'), findsOneWidget);
    expect(find.text('正常'), findsWidgets);
    // 投票公民不展示竞选专属分区。
    expect(find.text('竞选公开信息'), findsNothing);
    expect(find.text('完全匿名'), findsNothing);
  });

  testWidgets('竞选公民档多展示姓名/性别/出生地', (tester) async {
    await pumpPage(
      tester,
      const MyIdState(
        tier: MyIdTier.candidate,
        status: MyIdStatus.normal,
        votingAccount: '5FtestVotingAcc',
        cidNumber: 'GD-CTZN1-8F3A2B',
        residenceDistrict: '广东 · 深圳 · 南山',
        passportValidFrom: '2026-01-01',
        passportValidUntil: '2031-01-01',
        citizenFullName: '陈明',
        citizenSexLabel: '男',
        birthDistrict: '广东 · 广州 · 天河',
      ),
    );

    expect(find.text('竞选公民'), findsOneWidget);
    expect(find.text('竞选公开信息'), findsOneWidget);
    expect(find.text('姓名'), findsOneWidget);
    expect(find.text('陈明'), findsOneWidget);
    expect(find.text('性别'), findsOneWidget);
    expect(find.text('男'), findsOneWidget);
    expect(find.text('出生地'), findsOneWidget);
    expect(find.text('广东 · 广州 · 天河'), findsOneWidget);
  });

  testWidgets('链读失败档显示读取失败与重试', (tester) async {
    await pumpPage(
      tester,
      const MyIdState(
        tier: MyIdTier.visitor,
        status: MyIdStatus.queryFailed,
        errorMessage: '链上身份读取失败',
      ),
    );

    expect(find.text('读取失败'), findsWidgets);
    expect(find.text('链上身份读取失败'), findsOneWidget);
    expect(find.text('重试'), findsOneWidget);
    expect(find.text('完全匿名'), findsNothing);
  });

  testWidgets('不出现已下线的登记/换钱包/扫码入口', (tester) async {
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
