import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/main.dart';

void main() {
  testWidgets('app bootstraps', (tester) async {
    await tester.pumpWidget(const WuminApp());
    expect(find.text('广场'), findsWidgets);
    expect(find.text('广场页面（开发中）'), findsOneWidget);
  });
}
