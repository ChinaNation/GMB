import 'package:flutter_test/flutter_test.dart';
import 'package:citizenwallet/main.dart';

void main() {
  testWidgets('App builds without error', (WidgetTester tester) async {
    await tester.pumpWidget(const CitizenWalletApp());
    await tester.pump();
    // App 入口包含 _AppLockGate，测试环境下显示加载指示器即可视为正常构建
    expect(find.byType(CitizenWalletApp), findsOneWidget);
  });
}
