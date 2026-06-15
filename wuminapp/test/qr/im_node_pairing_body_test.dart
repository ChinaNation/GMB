import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/qr/bodies/im_node_pairing_body.dart';
import 'package:wuminapp_mobile/qr/envelope.dart';
import 'package:wuminapp_mobile/qr/qr_protocols.dart';
import 'package:wuminapp_mobile/qr/qr_router.dart';

void main() {
  test('通信节点二维码按 WUMIN_QR_V1 解析并路由', () {
    const body = ImNodePairingBody(
      nodePeerId: '12D3KooWNode',
      nodeMultiaddr: '/ip4/192.168.1.8/tcp/30333/wss/p2p/12D3KooWNode',
      endpointKind: 'ip4',
    );
    final raw = const QrEnvelope<ImNodePairingBody>(
      kind: QrKind.imNodePairing,
      id: null,
      issuedAt: null,
      expiresAt: null,
      body: body,
    ).toRawJson();

    final envelope = QrEnvelope.parse(raw);
    final route = QrRouter().route(raw);

    expect(envelope.kind, QrKind.imNodePairing);
    expect(envelope.body, isA<ImNodePairingBody>());
    expect(route.type, QrRouteType.imNodePairing);
    expect((envelope.body as ImNodePairingBody).nodePeerId, '12D3KooWNode');
  });

  test('通信节点二维码支持 ip6 端点', () {
    const body = ImNodePairingBody(
      nodePeerId: '12D3KooWNode',
      nodeMultiaddr: '/ip6/::1/tcp/30333/wss/p2p/12D3KooWNode',
      endpointKind: 'ip6',
    );

    expect(body.toJson()['endpoint_kind'], 'ip6');
    expect(() => body.validate(), returnsNormally);
  });

  test('通信节点二维码拒绝 PeerId 与 multiaddr 不一致', () {
    const body = ImNodePairingBody(
      nodePeerId: '12D3KooWNodeA',
      nodeMultiaddr: '/ip4/192.168.1.8/tcp/30333/wss/p2p/12D3KooWNodeB',
      endpointKind: 'ip4',
    );

    expect(() => body.validate(), throwsFormatException);
  });
}
