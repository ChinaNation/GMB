import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:wuminapp_mobile/qr/bodies/user_contact_body.dart';
import 'package:wuminapp_mobile/qr/envelope.dart';
import 'package:wuminapp_mobile/qr/qr_protocols.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';
import 'package:wuminapp_mobile/wallet/ui/cards/wallet_qr_dialog.dart';

/// 中文注释:WalletQrDialog 弹窗验证(v3:副标题已删 / 下载改文字按钮)。
///
/// 验证点:
/// - 弹窗能正常 show,渲染 QrImageView、地址、钱包名、关闭按钮
/// - QR 载荷构造规则与 user_contact 匹配(对 QrEnvelope 做同构构造比对,
///   QrImageView.data 是私有字段,测不到具体 payload,用 envelope 对照即可)
/// - 地址行右侧有 Icons.copy 图标按钮,点击不崩溃(Clipboard 在 test 环境
///   由 services binding 提供,tap 能跑通即可)
/// - 关闭按钮右侧有"下载"文字按钮(TextButton + Text),点击进入保存流程;
///   单测环境 SaverGallery MethodChannel 没有 native 实现,最终会走 catch
///   分支或返回失败态 SnackBar,测试只需能完成点击 + pump 不抛异常。
void main() {
  const wallet = WalletProfile(
    walletIndex: 0,
    walletName: '我的钱包',
    walletIcon: 'wallet',
    balance: 0.0,
    address: '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty',
    pubkeyHex: '0x00',
    alg: 'sr25519',
    ss58: 2027,
    createdAtMillis: 0,
    source: 'test',
    signMode: 'local',
  );

  Future<void> openDialog(WidgetTester tester) async {
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Builder(
            builder: (context) => ElevatedButton(
              onPressed: () => WalletQrDialog.show(
                context,
                wallet: wallet,
                name: '我的钱包',
              ),
              child: const Text('open'),
            ),
          ),
        ),
      ),
    );
    await tester.tap(find.text('open'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
  }

  testWidgets('dialog renders QR, wallet name, address, close/copy/download',
      (tester) async {
    await openDialog(tester);

    // QrImageView 应被渲染
    expect(find.byType(QrImageView), findsOneWidget);
    // 钱包名(对话框标题)也应该显示
    expect(find.text('我的钱包'), findsWidgets);
    // 底部完整地址可见
    expect(find.text(wallet.address), findsOneWidget);
    // 关闭按钮
    expect(find.text('关闭'), findsOneWidget);
    // 地址右侧复制图标仍可见(只是 size 改小)
    expect(find.byIcon(Icons.copy), findsOneWidget);
    // v3:下载改成文字按钮,和关闭对称
    expect(find.widgetWithText(TextButton, '下载'), findsOneWidget);
  });

  testWidgets('tapping copy icon does not throw', (tester) async {
    await openDialog(tester);
    // 中文注释:Clipboard.setData 在 test 环境默认会被 services binding 静默
    // 接管,这里只验证 tap 调用链不会抛异常;不强求 SnackBar 必然渲染,
    // 因为 ScaffoldMessenger 属于 Dialog 外层,SnackBar pump 时序不稳。
    await tester.tap(find.byIcon(Icons.copy));
    await tester.pump();
    // 仍能看到弹窗(未崩溃 / 未关闭)
    expect(find.byType(QrImageView), findsOneWidget);
  });

  testWidgets('tapping download button enters saving flow without throwing',
      (tester) async {
    await openDialog(tester);
    // 中文注释:SaverGallery 的 MethodChannel 在 test 环境没有 native handler,
    // `saveImage` 内部 try-catch 会把异常包成 SaveResult(false, ...) 返回;
    // 即使某一步抛了,外层 _saveQrToGallery 也有 try-catch 兜底。
    //
    // 不用 pumpAndSettle:保存中 `_isSaving=true` 会持续渲染 CircularProgress-
    // Indicator,动画永远不 settle。改用固定次数 pump 推进异步 micro-task 链。
    await tester.tap(find.widgetWithText(TextButton, '下载'));
    await tester.pump(); // setState(_isSaving = true)
    // pump 若干帧给 boundary.toImage / invokeMethod 的 async chain 走完。
    for (var i = 0; i < 10; i++) {
      await tester.pump(const Duration(milliseconds: 100));
    }
    // 二维码仍在(弹窗没被关闭),说明流程没有把上下文炸掉。
    expect(find.byType(QrImageView), findsOneWidget);
  });

  test('user_contact envelope raw json contains WUMIN_QR_V1 and kind',
      () {
    final raw = QrEnvelope<UserContactBody>(
      kind: QrKind.userContact,
      id: null,
      issuedAt: null,
      expiresAt: null,
      body: UserContactBody(address: wallet.address, name: '我的钱包'),
    ).toRawJson();
    expect(raw.contains(QrProtocols.v1), isTrue,
        reason: 'payload should include WUMIN_QR_V1 proto');
    expect(raw.contains('user_contact'), isTrue,
        reason: 'payload should include kind user_contact');
    expect(raw.contains(wallet.address), isTrue);
  });
}
