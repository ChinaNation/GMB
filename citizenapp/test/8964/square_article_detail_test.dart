import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/pages/square_article_detail_page.dart';

import 'profile/profile_test_doubles.dart';

void main() {
  testWidgets('renders the article title and body', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        home: SquareArticleDetailPage(
          post: samplePost(
            contentFormat: SquarePostContentFormat.article,
            title: '标题X',
            text: '正文Y',
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('标题X'), findsOneWidget);
    expect(find.text('正文Y'), findsOneWidget);
  });
}
