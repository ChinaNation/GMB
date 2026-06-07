import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/my/myid/myid_page.dart';
import 'package:wuminapp_mobile/my/myid/myid_service.dart';

void main() {
  testWidgets('电子护照页展示身份ID、投票账户和正常状态', (tester) async {
    final service = _FakeMyIdService(
      const MyIdState(
        bindStatus: MyIdBindStatus.bound,
        walletAddress: '5F-test-address',
        walletPubkeyHex: 'abcd',
        walletIndex: 1,
        sfidNumber: 'LN001-GCB05-944805165-2026',
        identityStatus: 'NORMAL',
        validFrom: '2026-05-24',
        validUntil: '2036-05-23',
      ),
    );

    await tester.pumpWidget(
      MaterialApp(
        home: MyIdPage(myIdService: service),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('身份ID'), findsOneWidget);
    expect(find.text('LN001-GCB05-944805165-2026'), findsOneWidget);
    expect(find.text('投票账户'), findsOneWidget);
    expect(find.text('绑定账户'), findsNothing);
    expect(find.text('5F-test-address'), findsOneWidget);
    expect(find.text('状态：正常'), findsOneWidget);
    expect(find.text('有效期：2026年05月24日-2036年05月23日'), findsOneWidget);
  });

  testWidgets('电子护照待绑定时按钮左右显示更换钱包和扫码签名', (tester) async {
    final service = _FakeMyIdService(
      const MyIdState(
        bindStatus: MyIdBindStatus.pending,
        walletAddress: '5F-test-address',
        walletPubkeyHex: 'abcd',
        walletIndex: 1,
      ),
    );

    await tester.pumpWidget(
      MaterialApp(
        home: MyIdPage(myIdService: service),
      ),
    );
    await tester.pumpAndSettle();
    await tester.drag(find.byType(ListView), const Offset(0, -500));
    await tester.pumpAndSettle();

    expect(find.text('更换钱包'), findsOneWidget);
    expect(find.text('扫码签名'), findsOneWidget);
  });
}

class _FakeMyIdService extends MyIdService {
  _FakeMyIdService(this.state);

  final MyIdState state;

  @override
  Future<MyIdState> getState() async => state;

  @override
  Future<MyIdState> syncFromBackend() async => state;
}
