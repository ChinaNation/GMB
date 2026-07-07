import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/my/myid/myid_page.dart';
import 'package:citizenapp/my/myid/myid_service.dart';

void main() {
  testWidgets('电子护照页只展示链上唯一身份核心字段', (tester) async {
    final service = _FakeMyIdService(
      const MyIdState(
        identityStatus: MyIdIdentityStatus.normal,
        identityWalletAccount: '5F-test-address',
        identityCidNumber: 'LN001-NRC0G-944805165-2026',
        passportValidFrom: '2026-05-24',
        passportValidUntil: '2036-05-23',
      ),
    );

    await tester.pumpWidget(
      MaterialApp(
        home: MyIdPage(myIdService: service),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('投票账户'), findsOneWidget);
    expect(find.text('5F-test-address'), findsOneWidget);
    expect(find.text('身份 CID 号'), findsOneWidget);
    expect(find.text('LN001-NRC0G-944805165-2026'), findsOneWidget);
    expect(find.text('状态'), findsOneWidget);
    expect(find.text('正常'), findsWidgets);
    expect(find.text('有效期'), findsOneWidget);
    expect(find.text('2026年05月24日-2036年05月23日'), findsOneWidget);

    expect(find.text('护照号'), findsNothing);
    expect(find.text('选择钱包'), findsNothing);
    expect(find.text('更换钱包'), findsNothing);
    expect(find.text('扫码签名'), findsNothing);
    expect(find.text('钱包地址二维码'), findsNothing);
  });

  testWidgets('未发现链上身份时电子护照页不显示登记操作', (tester) async {
    final service = _FakeMyIdService(
      const MyIdState(identityStatus: MyIdIdentityStatus.notOnchain),
    );

    await tester.pumpWidget(
      MaterialApp(
        home: MyIdPage(myIdService: service),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('未上链'), findsWidgets);
    expect(find.text('选择钱包'), findsNothing);
    expect(find.text('扫码签名'), findsNothing);
  });
}

class _FakeMyIdService extends MyIdService {
  _FakeMyIdService(this.state);

  final MyIdState state;

  @override
  Future<MyIdState> getState() async => state;
}
