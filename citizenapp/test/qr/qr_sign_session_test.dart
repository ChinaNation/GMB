import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/qr/pages/qr_sign_session_page.dart';
import 'package:citizenapp/qr/qr_protocols.dart';
import 'package:citizenapp/signer/qr_signer.dart';

void main() {
  group('QrSignSessionPage', () {
    late QrSigner signer;
    late SignRequestEnvelope request;
    late String requestJson;

    setUp(() {
      signer = QrSigner();
      request = signer.buildRequest(
        requestId: 'tx-test-12345678901234',
        pubkey:
            '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
        payloadHex: '0x01020304',
        action: QrActions.transferWithRemark,
      );
      requestJson = signer.encodeRequest(request);
    });

    testWidgets('should display countdown and QR code', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: QrSignSessionPage(
            request: request,
            requestJson: requestJson,
            expectedPubkey: request.body.pubkeyHex,
          ),
        ),
      );

      expect(find.text('公民钱包签名'), findsOneWidget);
      expect(find.textContaining('签名请求有效期剩余'), findsOneWidget);
      expect(find.textContaining('请用离线设备扫描此二维码'), findsOneWidget);
      expect(find.text('取消'), findsOneWidget);
      expect(find.text('扫描响应'), findsOneWidget);
    });

    testWidgets('cancel should pop with null', (tester) async {
      SignResponseEnvelope? result;

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => FilledButton(
              onPressed: () async {
                result = await Navigator.push<SignResponseEnvelope>(
                  context,
                  MaterialPageRoute(
                    builder: (_) => QrSignSessionPage(
                      request: request,
                      requestJson: requestJson,
                      expectedPubkey: request.body.pubkeyHex,
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

      await tester.tap(find.text('取消'));
      await tester.pumpAndSettle();

      expect(result, isNull);
    });

    testWidgets('should show expired state when request expires',
        (tester) async {
      final expiredRequest = signer.buildRequest(
        requestId: 'tx-expired-12345678901',
        pubkey:
            '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
        payloadHex: '0x01020304',
        action: QrActions.transferWithRemark,
        nowEpochSeconds: DateTime.now().millisecondsSinceEpoch ~/ 1000 - 200,
      );
      final expiredJson = signer.encodeRequest(expiredRequest);

      await tester.pumpWidget(
        MaterialApp(
          home: QrSignSessionPage(
            request: expiredRequest,
            requestJson: expiredJson,
            expectedPubkey: expiredRequest.body.pubkeyHex,
          ),
        ),
      );

      expect(find.textContaining('签名请求已过期'), findsOneWidget);

      final scanButton = tester.widget<FilledButton>(
        find.widgetWithText(FilledButton, '扫描响应'),
      );
      expect(scanButton.onPressed, isNull);
    });
  });
}
