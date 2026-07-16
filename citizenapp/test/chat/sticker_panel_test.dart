import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/chat/compose/sticker_panel.dart';
import 'package:citizenapp/chat/stickers/sticker_pack.dart';

void main() {
  testWidgets('点选贴纸回调 (packId, stickerId)', (tester) async {
    String? pickedPack;
    String? pickedId;
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: StickerPanel(
            onPick: (packId, stickerId) {
              pickedPack = packId;
              pickedId = stickerId;
            },
          ),
        ),
      ),
    );
    await tester.pump(); // 不 settle:Image.asset 异步解码不阻塞交互

    // 首个分类(表情)默认展示,点第一个贴纸 grinning_face。
    await tester.tap(find.byKey(const ValueKey('sticker-grinning_face')));
    await tester.pump();

    expect(pickedPack, StickerPack.packId);
    expect(pickedId, 'grinning_face');
  });

  testWidgets('渲染四个分类 Tab', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(body: StickerPanel(onPick: (_, __) {})),
      ),
    );
    await tester.pump();

    expect(find.text('表情'), findsOneWidget);
    expect(find.text('手势'), findsOneWidget);
    expect(find.text('爱心'), findsOneWidget);
    expect(find.text('庆祝'), findsOneWidget);
  });
}
