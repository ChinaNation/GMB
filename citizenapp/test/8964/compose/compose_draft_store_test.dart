import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/compose/drafts/compose_draft.dart';
import 'package:citizenapp/8964/compose/drafts/compose_draft_store.dart';
import 'package:citizenapp/8964/models/square_models.dart';

import '../../support/isar_test_env.dart';

SquareComposeDraft _draft(String id, int updatedAt,
        {String accountId = 'accountId'}) =>
    SquareComposeDraft(
      draftId: id,
      accountId: accountId,
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
    await store.save(_draft('x', 9999,
        accountId:
            '0x9999999999999999999999999999999999999999999999999999999999999999'));

    final drafts = await store.list('accountId');
    expect(drafts.map((d) => d.draftId).toList(), ['b', 'c', 'a']);
    expect(drafts.every((d) => d.accountId == 'accountId'), isTrue);
  });

  test('同 draftId 再存为覆盖，不新增', () async {
    await store.save(_draft('s', 1000,
        accountId:
            '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'));
    await store.save(_draft('s', 5000,
        accountId:
            '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'));
    final drafts = await store.list(
        '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa');
    expect(drafts.length, 1);
    expect(drafts.single.updatedAtMillis, 5000);
  });
}
