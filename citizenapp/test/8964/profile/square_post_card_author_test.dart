import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/profile/models/profile_presentation.dart';
import 'package:citizenapp/8964/widgets/square_post_card.dart';

import 'fake_profile.dart';

void main() {
  testWidgets('tapping the author region fires onAuthorTap', (tester) async {
    var authorTapped = false;
    var cardTapped = false;

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: SquarePostCard(
            post: samplePost(text: '正文', displayName: '张三'),
            onTap: () => cardTapped = true,
            onAuthorTap: () => authorTapped = true,
          ),
        ),
      ),
    );

    await tester.tap(find.text('张三'));
    await tester.pumpAndSettle();

    expect(authorTapped, isTrue);
    expect(cardTapped, isFalse);
  });

  testWidgets('missing author profile uses the stable local name and image',
      (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: SquarePostCard(
            post: samplePost(displayName: ''),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(
      find.text(ProfilePresentation.forAccount(kOwner).fallbackName),
      findsOneWidget,
    );
    expect(find.byType(Image), findsWidgets);
  });
}
