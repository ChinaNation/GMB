import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/qr/bodies/im_node_pairing_body.dart';
import 'package:wuminapp_mobile/qr/envelope.dart';
import 'package:wuminapp_mobile/qr/qr_protocols.dart';
import 'package:wuminapp_mobile/qr/qr_router.dart';

void main() {
  test('通信节点二维码按 WUMIN_QR_V1 解析并路由', () {
    const body = ImNodePairingBody(
      nodePeerId: '12D3KooWNode',
      rpcUrl: 'http://192.168.1.8:9944/',
      nodeMultiaddr: '/ip4/192.168.1.8/tcp/30333/wss/p2p/12D3KooWNode',
      endpointKind: 'ip4',
      pairingNonce: 'nonce-1',
      createdAtMillis: 1800000,
      expiresAtMillis: 2400000,
    );
    final raw = const QrEnvelope<ImNodePairingBody>(
      kind: QrKind.imNodePairing,
      id: 'im-node-nonce-1',
      issuedAt: 1800,
      expiresAt: 2400,
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
      rpcUrl: 'http://[::1]:9944/',
      nodeMultiaddr: '/ip6/::1/tcp/30333/wss/p2p/12D3KooWNode',
      endpointKind: 'ip6',
      pairingNonce: 'nonce-1',
      createdAtMillis: 1800000,
      expiresAtMillis: 2400000,
    );

    expect(body.toJson()['endpoint_kind'], 'ip6');
    expect(() => body.validate(), returnsNormally);
  });

  test('通信节点二维码拒绝 PeerId 与 multiaddr 不一致', () {
    const body = ImNodePairingBody(
      nodePeerId: '12D3KooWNodeA',
      rpcUrl: 'http://192.168.1.8:9944/',
      nodeMultiaddr: '/ip4/192.168.1.8/tcp/30333/wss/p2p/12D3KooWNodeB',
      endpointKind: 'ip4',
      pairingNonce: 'nonce-1',
      createdAtMillis: 1800000,
      expiresAtMillis: 2400000,
    );

    expect(() => body.validate(), throwsFormatException);
  });
}
