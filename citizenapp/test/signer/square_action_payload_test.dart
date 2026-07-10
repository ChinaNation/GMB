import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/signer/signing.dart';
import 'package:citizenapp/signer/square_action_payload.dart';

String _payloadHex({
  required String action,
  required String owner,
  required String challengeId,
  String? level,
  required int expiresAt,
}) {
  final bytes = <int>[
    ...scaleString(action),
    ...scaleString(owner),
    ...scaleString(challengeId),
    if (level != null) ...scaleString(level),
    ...u64Le(expiresAt),
  ];
  return Uint8List.fromList(bytes)
      .map((b) => b.toRadixString(16).padLeft(2, '0'))
      .join();
}

void main() {
  const owner = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';

  test('decodes cancel_membership (no context) byte-for-byte', () {
    final hex = _payloadHex(
      action: 'cancel_membership',
      owner: owner,
      challengeId: 'sqa_abc',
      expiresAt: 1700000000000,
    );
    final decoded = decodeSquareActionPayload(hex);
    expect(decoded, isNotNull);
    expect(decoded!.action, 'cancel_membership');
    expect(decoded.ownerAccount, owner);
    expect(decoded.challengeId, 'sqa_abc');
    expect(decoded.context, isNull);
    expect(decoded.expiresAt, 1700000000000);
    expect(decoded.displayTitle, '取消订阅');
  });

  test('decodes subscribe_membership with level context', () {
    final hex = _payloadHex(
      action: 'subscribe_membership',
      owner: owner,
      challengeId: 'sqa_xyz',
      level: 'voting',
      expiresAt: 1700000000000,
    );
    final decoded = decodeSquareActionPayload(hex);
    expect(decoded, isNotNull);
    expect(decoded!.action, 'subscribe_membership');
    expect(decoded.context, 'voting');
    expect(decoded.displayTitle, '订阅会员（voting）');
  });

  test('rejects unknown action → null (no blind sign)', () {
    final hex = _payloadHex(
      action: 'transfer_all_funds',
      owner: owner,
      challengeId: 'sqa_evil',
      expiresAt: 1,
    );
    expect(decodeSquareActionPayload(hex), isNull);
  });

  test('rejects trailing-byte / truncated payload → null', () {
    final hex = _payloadHex(
      action: 'cancel_membership',
      owner: owner,
      challengeId: 'sqa_abc',
      expiresAt: 1,
    );
    expect(decodeSquareActionPayload('${hex}ff'), isNull); // 多一字节
    expect(decodeSquareActionPayload(hex.substring(0, hex.length - 4)), isNull); // 缺尾
  });
}
