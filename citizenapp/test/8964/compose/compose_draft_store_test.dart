import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/compose/drafts/compose_draft.dart';
import 'package:citizenapp/8964/compose/drafts/compose_draft_store.dart';
import 'package:citizenapp/8964/models/square_models.dart';

import '../../support/isar_test_env.dart';

SquareComposeDraft _draft(String id, int updatedAt,
        {String cidNumber = 'CN001-CTZN-100000001-2026'}) =>
    SquareComposeDraft(
      draftId: id,
      cidNumber: cidNumber,
      contentFormat: SquarePostContentFormat.normal,
      postCategory: SquarePostCategory.normal,
      text: '内容 $id',
      media: const <SquareLocalMediaDraft>[],
      updatedAtMillis: updatedAt,
    );

void main() {
  useIsolatedIsar();
  TestWidgetsFlutterBinding.ensureInitialized();

  final store = SquareComposeDraftStore.instance;

  test('多草稿按 updated_at 新→旧列出，仅本人可见', () async {
    await store.save(_draft('a', 1000));
    await store.save(_draft('b', 3000));
    await store.save(_draft('c', 2000));
    await store.save(_draft('x', 9999, cidNumber: 'CN001-CTZN-999999999-2026'));

    final drafts = await store.list('CN001-CTZN-100000001-2026');
    expect(drafts.map((d) => d.draftId).toList(), ['b', 'c', 'a']);
    expect(
      drafts.every((d) => d.cidNumber == 'CN001-CTZN-100000001-2026'),
      isTrue,
    );
  });

  test('同 draftId 再存为覆盖，不新增', () async {
    await store.save(
      _draft('s', 1000, cidNumber: 'CN001-CTZN-200000001-2026'),
    );
    await store.save(
      _draft('s', 5000, cidNumber: 'CN001-CTZN-200000001-2026'),
    );
    final drafts = await store.list('CN001-CTZN-200000001-2026');
    expect(drafts.length, 1);
    expect(drafts.single.updatedAtMillis, 5000);
  });
}
