import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/qr/pages/qr_sign_session_page.dart';
import 'package:wuminapp_mobile/signer/qr_signer.dart';

void main() {
  group('QrSignSessionPage', () {
    late QrSigner signer;
    late QrSignRequest request;
    late String requestJson;

    setUp(() {
      signer = QrSigner();
      request = signer.buildRequest(
        scope: QrSignScope.onchainTx,
        requestId: 'tx-test-1234',
        account: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
        pubkey:
            '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
        payloadHex: '0x01020304',
      );
      requestJson = signer.encodeRequest(request);
    });

    testWidgets('should display countdown and QR code', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: QrSignSessionPage(
            request: request,
            requestJson: requestJson,
          ),
        ),
      );

      // 标题
      expect(find.text('冷钱包签名'), findsOneWidget);
      // 倒计时状态栏（应包含 "s" 后缀）
      expect(find.textContaining('签名请求有效期剩余'), findsOneWidget);
      // 提示文字
      expect(find.textContaining('请用离线设备扫描此二维码'), findsOneWidget);
      // 按钮
      expect(find.text('取消'), findsOneWidget);
      expect(find.text('扫描回执'), findsOneWidget);
    });

    testWidgets('cancel should pop with null', (tester) async {
      QrSignResponse? result = const QrSignResponse(
        proto: 'sentinel',
        requestId: 'sentinel',
        pubkey: 'sentinel',
        sigAlg: 'sentinel',
        signature: 'sentinel',
        signedAt: 0,
      );

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => FilledButton(
              onPressed: () async {
                result = await Navigator.push<QrSignResponse>(
                  context,
                  MaterialPageRoute(
                    builder: (_) => QrSignSessionPage(
                      request: request,
                      requestJson: requestJson,
                    ),
                  ),
                );
              },
              child: const Text('open'),
            ),
          ),
        ),
      );

      await tester.tap(find.text('open'));
      await tester.pumpAndSettle();

      // 点击取消
      await tester.tap(find.text('取消'));
      await tester.pumpAndSettle();

      expect(result, isNull);
    });

    testWidgets('should show expired state when request expires',
        (tester) async {
      // 构建一个已过期的请求
      final expiredRequest = signer.buildRequest(
        scope: QrSignScope.onchainTx,
        requestId: 'tx-expired-1',
        account: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
        pubkey:
            '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
        payloadHex: '0x01020304',
        nowEpochSeconds:
            DateTime.now().millisecondsSinceEpoch ~/ 1000 - 200,
      );
      final expiredJson = signer.encodeRequest(expiredRequest);

      await tester.pumpWidget(
        MaterialApp(
          home: QrSignSessionPage(
            request: expiredRequest,
            requestJson: expiredJson,
          ),
        ),
      );

      // 应显示过期提示
      expect(find.textContaining('签名请求已过期'), findsOneWidget);

      // "扫描回执" 按钮应被禁用
      final scanButton = tester.widget<FilledButton>(
        find.widgetWithText(FilledButton, '扫描回执'),
      );
      expect(scanButton.onPressed, isNull);
    });
  });
}
