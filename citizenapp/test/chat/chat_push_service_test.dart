import 'package:citizenapp/chat/chat_push_service.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';

/// 唤醒载荷按发件人身份主键 CID 号标识（Worker R5 口径）；下游 peer_ready / 补发
/// 一律按 CID 寻址，钱包账户 account_id 不进推送。
const _senderCidNumber = 'CN220-CTZN2-100000001-2026';
const _otherCidNumber = 'CN220-CTZN2-100000002-2026';

void main() {
  test('只接受无内容聊天唤醒载荷', () {
    expect(
      ChatPushService.wakeSenderFromData(const {
        'kind': 'chat_wake',
        'sender_cid_number': _senderCidNumber,
      }),
      _senderCidNumber,
    );
    expect(
      ChatPushService.wakeSenderFromData(const {
        'kind': 'chat_wake',
        'sender_cid_number': _senderCidNumber,
        'message': '不得进入推送',
      }),
      isNull,
    );
    expect(
      ChatPushService.wakeSenderFromData(const {
        'kind': 'chat_message',
        'sender_cid_number': _senderCidNumber,
      }),
      isNull,
    );
  });

  test('后台连续唤醒会去重保存全部发送方', () async {
    SharedPreferences.setMockInitialValues({});
    await ChatPushService.storeWakeSender(_senderCidNumber);
    await ChatPushService.storeWakeSender(_otherCidNumber);
    await ChatPushService.storeWakeSender(_senderCidNumber);

    final service = ChatPushService();
    expect(
      await service.takePendingWakeSenders(),
      [_senderCidNumber, _otherCidNumber],
    );
    expect(await service.takePendingWakeSenders(), isEmpty);
    await service.dispose();
  });
}
