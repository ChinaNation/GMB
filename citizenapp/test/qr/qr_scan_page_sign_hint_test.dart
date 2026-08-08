import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/qr/bodies/sign_request_body.dart';
import 'package:citizenapp/qr/pages/qr_scan_page.dart';
import 'package:citizenapp/qr/qr_protocols.dart';

/// 扫码填地址时扫到签名请求:必须给明确去向,不得用「无法识别」含糊过去。
///
/// 签名请求(广场动作 / 公民身份 / 注册局占号换绑)统一在「聊天 → 扫一扫」处理;
/// 交易页与多签各页的地址框扫码都走 [QrScanMode.transfer],共用这一条提示。
void main() {
  // 与 qr_router_test 的 k=1 样本同形,保证是扫码页真正会收到的字节形态。
  final signRequestCode = jsonEncode({
    'p': QrProtocol.qrV1,
    'k': QrKind.signRequest.code,
    'i': 'ch-0123456789abcdef',
    'e': 1090,
    'b': SignRequestBody.fromHex(
      action: QrActions.squareAccountAction,
      signerPublicKeyHex:
          '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
      payloadHex: '0x6369647c736967',
    ).toJson(),
  });

  testWidgets('transfer 模式扫到签名请求:指路聊天扫一扫,不回传结果', (tester) async {
    QrScanTransferResult? popped;
    final navigatorKey = GlobalKey<NavigatorState>();
    await tester.pumpWidget(MaterialApp(
      navigatorKey: navigatorKey,
      home: const SizedBox.shrink(),
    ));

    unawaitedPush(navigatorKey, signRequestCode).then((value) {
      popped = value;
    });
    await tester.pumpAndSettle();

    expect(find.text('这是签名请求'), findsOneWidget);
    expect(find.text('此处只扫收款地址。请到「聊天 → 扫一扫」。'), findsOneWidget);
    // 未识别文案不得同时出现:两条提示并存等于没给去向。
    expect(find.text('无法识别二维码'), findsNothing);

    await tester.tap(find.text('确定'));
    await tester.pumpAndSettle();

    // 扫码页停留在原地继续扫,不带结果返回调用方。
    expect(find.byType(QrScanPage), findsOneWidget);
    expect(popped, isNull);
  });
}

Future<QrScanTransferResult?> unawaitedPush(
  GlobalKey<NavigatorState> navigatorKey,
  String initialCode,
) {
  return navigatorKey.currentState!.push<QrScanTransferResult>(
    MaterialPageRoute(
      builder: (_) => QrScanPage(
        mode: QrScanMode.transfer,
        initialCode: initialCode,
      ),
    ),
  );
}
