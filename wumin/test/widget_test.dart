import 'package:flutter_test/flutter_test.dart';
import 'package:wumin/main.dart';

void main() {
  testWidgets('App builds without error', (WidgetTester tester) async {
    await tester.pumpWidget(const WuminApp());
    expect(find.text('Wumin'), findsOneWidget);
  });
}
