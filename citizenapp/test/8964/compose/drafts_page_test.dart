import 'dart:async';

import 'package:citizenapp/8964/compose/drafts/compose_draft.dart';
import 'package:citizenapp/8964/compose/drafts/compose_draft_store.dart';
import 'package:citizenapp/8964/compose/drafts/drafts_page.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

class _PendingDraftStore implements SquareComposeDraftRepository {
  final Completer<List<SquareComposeDraft>> completer =
      Completer<List<SquareComposeDraft>>();

  @override
  Future<List<SquareComposeDraft>> list(String cidNumber) => completer.future;

  @override
  Future<void> delete(String cidNumber, String draftId) async {}

  @override
  Future<void> save(SquareComposeDraft draft) async {}
}

void main() {
  testWidgets('本地草稿未返回时直接显示草稿箱且不使用整页转圈', (tester) async {
    final store = _PendingDraftStore();
    await tester.pumpWidget(
      MaterialApp(
        home: DraftsPage(
          cidNumber: 'CN220-CTZN2-100000001-2026',
          store: store,
        ),
      ),
    );
    await tester.pump();

    expect(find.text('草稿箱'), findsOneWidget);
    expect(find.text('正在读取本地草稿'), findsOneWidget);
    expect(find.byKey(const ValueKey('drafts-load-progress')), findsOneWidget);
    expect(find.byType(CircularProgressIndicator), findsNothing);

    store.completer.complete(const <SquareComposeDraft>[]);
    await tester.pumpAndSettle();
    expect(find.text('还没有草稿'), findsOneWidget);
    expect(find.byKey(const ValueKey('drafts-load-progress')), findsNothing);
  });
}
