import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/qr/pages/qr_sign_session_page.dart';
import 'package:wuminapp_mobile/signer/qr_signer.dart';

void main() {
  group('QrSignSessionPage', () {
    late QrSigner signer;
    late QrSignRequest request;
    late String requestJson;

    final display = {
      'action': 'transfer',
      'summary': '转账 1.00 GMB',
      'fields': {'amount_yuan': '1.00'},
    };

    setUp(() {
      signer = QrSigner();
      request = signer.buildRequest(
        requestId: 'tx-test-1234',
        account: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
        pubkey:
            '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
        payloadHex: '0x01020304',
        display: display,
      );
      requestJson = signer.encodeRequest(request);
    });

    testWidgets('should display countdown and QR code', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: QrSignSessionPage(
            request: request,
            requestJson: requestJson,
            expectedPubkey: request.pubkey,
          ),
        ),
      );

      expect(find.text('冷钱包签名'), findsOneWidget);
      expect(find.textContaining('签名请求有效期剩余'), findsOneWidget);
      expect(find.textContaining('请用离线设备扫描此二维码'), findsOneWidget);
      expect(find.text('取消'), findsOneWidget);
      expect(find.text('扫描回执'), findsOneWidget);
    });

    testWidgets('cancel should pop with null', (tester) async {
      QrSignResponse? result = QrSignResponse(
        proto: 'sentinel',
        requestId: 'sentinel',
        pubkey: 'sentinel',
        sigAlg: 'sentinel',
        signature: 'sentinel',
        payloadHash: 'sentinel',
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
                      expectedPubkey: request.pubkey,
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
        requestId: 'tx-expired-1',
        account: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
        pubkey:
            '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
        payloadHex: '0x01020304',
        display: display,
        nowEpochSeconds: DateTime.now().millisecondsSinceEpoch ~/ 1000 - 200,
      );
      final expiredJson = signer.encodeRequest(expiredRequest);

      await tester.pumpWidget(
        MaterialApp(
          home: QrSignSessionPage(
            request: expiredRequest,
            requestJson: expiredJson,
            expectedPubkey: expiredRequest.pubkey,
          ),
        ),
      );

      expect(find.textContaining('签名请求已过期'), findsOneWidget);

      final scanButton = tester.widget<FilledButton>(
        find.widgetWithText(FilledButton, '扫描回执'),
      );
      expect(scanButton.onPressed, isNull);
    });
  });
}
