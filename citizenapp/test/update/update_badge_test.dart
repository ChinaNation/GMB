import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/update/update_badge.dart';

void main() {
  testWidgets('有更新时显示红点', (tester) async {
    await tester.pumpWidget(
      const MaterialApp(
        home: Scaffold(
          body: UpdateDotBadge(
            show: true,
            dotKey: Key('update-dot'),
            child: Icon(Icons.settings),
          ),
        ),
      ),
    );

    expect(find.byKey(const Key('update-dot')), findsOneWidget);
  });

  testWidgets('没有更新时不显示红点', (tester) async {
    await tester.pumpWidget(
      const MaterialApp(
        home: Scaffold(
          body: UpdateDotBadge(
            show: false,
            dotKey: Key('update-dot'),
            child: Icon(Icons.settings),
          ),
        ),
      ),
    );

    expect(find.byKey(const Key('update-dot')), findsNothing);
  });
}
