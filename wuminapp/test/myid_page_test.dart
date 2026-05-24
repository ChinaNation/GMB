import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/my/myid/myid_page.dart';
import 'package:wuminapp_mobile/my/myid/myid_service.dart';

void main() {
  testWidgets('电子护照页展示身份ID、投票账户和正常状态', (tester) async {
    final service = _FakeMyIdService(
      const MyIdState(
        status: MyIdStatus.bound,
        walletAddress: '5F-test-address',
        walletPubkeyHex: 'abcd',
        sfidCode: '1234567890',
        identityStatus: 'NORMAL',
      ),
    );

    await tester.pumpWidget(
      MaterialApp(
        home: MyIdPage(myIdService: service),
      ),
    );
    await tester.pump();

    expect(find.text('身份ID'), findsOneWidget);
    expect(find.text('1234567890'), findsOneWidget);
    expect(find.text('投票账户'), findsOneWidget);
    expect(find.text('绑定账户'), findsNothing);
    expect(find.text('5F-test-address'), findsOneWidget);
    expect(find.text('状态：正常'), findsOneWidget);
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
