import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/my/myid/widgets/rebind_account_sheet.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart' show Account;

Account _account({
  int index = 5,
  String name = '账户5',
  String accountId =
      '0x1111111111111111111111111111111111111111111111111111111111111111',
}) {
  return Account(
    masterId:
        '0x0000000000000000000000000000000000000000000000000000000000000001',
    accountIndex: index,
    accountId: accountId,
    ss58Address: 'w5BekTimvtfYZvFpkDzy7ypqUntPgTbjRFCt9weR8vMgf7o8E',
    accountName: name,
  );
}

void main() {
  testWidgets('空 targets 显示「暂无可换绑」提示', (tester) async {
    await tester.pumpWidget(
      const MaterialApp(
        home: Scaffold(body: RebindAccountSheet(targets: [])),
      ),
    );
    expect(find.textContaining('暂无可换绑'), findsOneWidget);
  });

  testWidgets('点选账户回传其 accountId', (tester) async {
    String? picked = 'UNSET';
    final account = _account();
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Builder(
            builder: (context) => Center(
              child: ElevatedButton(
                onPressed: () async => picked = await showRebindAccountSheet(
                  context,
                  targets: [account],
                ),
                child: const Text('open'),
              ),
            ),
          ),
        ),
      ),
    );
    await tester.tap(find.text('open'));
    await tester.pumpAndSettle();

    expect(find.text('账户5'), findsOneWidget);
    expect(find.text('#5'), findsOneWidget);

    await tester.tap(find.text('账户5'));
    await tester.pumpAndSettle();
    expect(picked, account.accountId);
  });
}
