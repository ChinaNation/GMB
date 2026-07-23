import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/signer/signing.dart';
import 'package:citizenapp/signer/square_action_payload.dart';

String _payloadHex({
  required String action,
  required String accountId,
  required String challengeId,
  String? level,
  required int expiresAt,
}) {
  final bytes = <int>[
    ...scaleString(action),
    ...scaleString(accountId),
    ...scaleString(challengeId),
    if (level != null) ...scaleString(level),
    ...u64Le(expiresAt),
  ];
  return Uint8List.fromList(bytes)
      .map((b) => b.toRadixString(16).padLeft(2, '0'))
      .join();
}

void main() {
  const accountId =
      '0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d';

  test('decodes cancel_membership (no context) byte-for-byte', () {
    final hex = _payloadHex(
      action: 'cancel_membership',
      accountId: accountId,
      challengeId: 'sqa_abc',
      expiresAt: 1700000000000,
    );
    final decoded = decodeSquareActionPayload(hex);
    expect(decoded, isNotNull);
    expect(decoded!.action, 'cancel_membership');
    expect(decoded.accountId, accountId);
    expect(decoded.challengeId, 'sqa_abc');
    expect(decoded.context, isNull);
    expect(decoded.expiresAt, 1700000000000);
    expect(decoded.actionTypeLabel, '取消订阅');
    expect(
      decoded.reviewFields!.map((field) => field.label),
      containsAll(['操作类型', '账户', '挑战编号', '过期时间']),
    );
  });

  test('decodes subscribe_membership with level context', () {
    final hex = _payloadHex(
      action: 'subscribe_membership',
      accountId: accountId,
      challengeId: 'sqa_xyz',
      level: 'voting',
      expiresAt: 1700000000000,
    );
    final decoded = decodeSquareActionPayload(hex);
    expect(decoded, isNotNull);
    expect(decoded!.action, 'subscribe_membership');
    expect(decoded.context, 'voting');
    expect(decoded.actionTypeLabel, '订阅会员');
    expect(
      decoded.reviewFields!.map((field) => '${field.label}:${field.value}'),
      contains('会员等级:voting'),
    );
  });

  test('rejects unknown action → null (no blind sign)', () {
    final hex = _payloadHex(
      action: 'transfer_all_funds',
      accountId: accountId,
      challengeId: 'sqa_evil',
      expiresAt: 1,
    );
    expect(decodeSquareActionPayload(hex), isNull);
  });

  test('rejects trailing-byte / truncated payload → null', () {
    final hex = _payloadHex(
      action: 'cancel_membership',
      accountId: accountId,
      challengeId: 'sqa_abc',
      expiresAt: 1,
    );
    expect(decodeSquareActionPayload('${hex}ff'), isNull); // 多一字节
    expect(decodeSquareActionPayload(hex.substring(0, hex.length - 4)),
        isNull); // 缺尾
  });
}
