import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/im/im_node_settings_page.dart';
import 'package:citizenapp/im/im_runtime.dart';
import 'package:citizenapp/qr/bodies/im_node_pairing_body.dart';
import 'package:citizenapp/qr/envelope.dart';
import 'package:citizenapp/qr/qr_protocols.dart';

void main() {
  testWidgets('设置通信节点页面显示未设置状态', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        home: ImNodeSettingsPage(
          runtime: _FakeImRuntime(ImPairedNodeConfig.empty),
        ),
      ),
    );
    await tester.pump();

    expect(find.text('设置通信节点'), findsOneWidget);
    expect(find.text('尚未设置通信节点'), findsOneWidget);
    expect(find.text('扫描通信节点'), findsWidgets);
  });

  testWidgets('设置通信节点页面显示已保存节点概要', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        home: ImNodeSettingsPage(
          runtime: _FakeImRuntime(_pairedConfig()),
        ),
      ),
    );
    await tester.pump();

    expect(find.text('已设置通信节点'), findsOneWidget);
    expect(find.text('RPC'), findsNothing);
    expect(find.text('12D3KooWNode'), findsOneWidget);
  });

  testWidgets('扫码通信节点二维码后保存配对', (tester) async {
    final runtime = _FakeImRuntime(ImPairedNodeConfig.empty);
    await tester.pumpWidget(
      MaterialApp(
        home: ImNodeSettingsPage(
          runtime: runtime,
          scanner: (_) async => _rawPairingQr(),
        ),
      ),
    );
    await tester.pump();
    await tester.tap(find.widgetWithText(FilledButton, '扫描通信节点'));
    await tester.pumpAndSettle();

    expect(runtime.lastPairedBody?.nodePeerId, '12D3KooWNode');
    expect(find.text('已设置通信节点'), findsOneWidget);
    expect(find.text('RPC'), findsNothing);
  });
}

class _FakeImRuntime extends ImRuntime {
  _FakeImRuntime(this._config);

  ImPairedNodeConfig _config;
  ImNodePairingBody? lastPairedBody;

  @override
  Future<ImPairedNodeConfig> readPairedNodeConfig() async {
    return _config;
  }

  @override
  Future<ImPairedNodeConfig> pairCommunicationNode(
    ImNodePairingBody body,
  ) async {
    lastPairedBody = body;
    _config = ImPairedNodeConfig(
      peerId: body.nodePeerId,
      multiaddr: body.nodeMultiaddr,
      pairedAtMillis: 1900000,
    );
    return _config;
  }
}

ImPairedNodeConfig _pairedConfig() {
  return const ImPairedNodeConfig(
    peerId: '12D3KooWNode',
    multiaddr: '/ip4/192.168.1.8/tcp/30333/wss/p2p/12D3KooWNode',
    pairedAtMillis: 1900000,
  );
}

String _rawPairingQr() {
  const body = ImNodePairingBody(
    nodePeerId: '12D3KooWNode',
    nodeMultiaddr: '/ip4/192.168.1.8/tcp/30333/wss/p2p/12D3KooWNode',
    endpointKind: 'ip4',
  );
  return const QrEnvelope<ImNodePairingBody>(
    kind: QrKind.imNodePairing,
    id: null,
    issuedAt: null,
    expiresAt: null,
    body: body,
  ).toRawJson();
}
