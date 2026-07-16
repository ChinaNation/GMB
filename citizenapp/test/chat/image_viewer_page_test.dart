import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/chat/viewer/image_viewer_page.dart';

// 用不存在的路径:widget 测试里解码真实图片会挂起 fake-async 时钟;errorBuilder
// 同步渲染占位,足以验证结构与保存流程。真实内联渲染以真机/集成为准。
void main() {
  testWidgets('ImageViewerPage 结构完整,保存按钮调用注入 saver 并提示成功', (tester) async {
    String? savedPath;
    String? savedName;
    await tester.pumpWidget(
      MaterialApp(
        home: ImageViewerPage(
          filePath: '/tmp/gmb-viewer-none.png',
          fileName: 'photo.png',
          onSaveToGallery: ({required filePath, required fileName}) async {
            savedPath = filePath;
            savedName = fileName;
            return true;
          },
        ),
      ),
    );
    await tester.pump();

    expect(find.byType(InteractiveViewer), findsOneWidget);
    expect(find.byType(Image), findsOneWidget);

    await tester.tap(find.byIcon(Icons.download_rounded));
    await tester.pump();

    expect(savedPath, '/tmp/gmb-viewer-none.png');
    expect(savedName, 'photo.png');
    expect(find.text('已保存到相册'), findsOneWidget);
  });

  testWidgets('ImageViewerPage 保存失败时提示失败', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        home: ImageViewerPage(
          filePath: '/tmp/gmb-viewer-none2.png',
          fileName: 'none.png',
          onSaveToGallery: ({required filePath, required fileName}) async =>
              false,
        ),
      ),
    );
    await tester.pump();
    await tester.tap(find.byIcon(Icons.download_rounded));
    await tester.pump();
    expect(find.text('保存失败'), findsOneWidget);
  });
}
