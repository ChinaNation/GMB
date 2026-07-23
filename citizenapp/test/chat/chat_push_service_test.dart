import 'package:citizenapp/chat/chat_push_service.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  test('只接受无内容聊天唤醒载荷', () {
    expect(
      ChatPushService.wakeSenderFromData(const {
        'kind': 'chat_wake',
        'sender_account_id':
            '0x1111111111111111111111111111111111111111111111111111111111111111',
      }),
      '0x1111111111111111111111111111111111111111111111111111111111111111',
    );
    expect(
      ChatPushService.wakeSenderFromData(const {
        'kind': 'chat_wake',
        'sender_account_id':
            '0x1111111111111111111111111111111111111111111111111111111111111111',
        'message': '不得进入推送',
      }),
      isNull,
    );
    expect(
      ChatPushService.wakeSenderFromData(const {
        'kind': 'chat_message',
        'sender_account_id':
            '0x1111111111111111111111111111111111111111111111111111111111111111',
      }),
      isNull,
    );
  });

  test('后台连续唤醒会去重保存全部发送方', () async {
    SharedPreferences.setMockInitialValues({});
    await ChatPushService.storeWakeSender(
        '0x1111111111111111111111111111111111111111111111111111111111111111');
    await ChatPushService.storeWakeSender(
        '0x2222222222222222222222222222222222222222222222222222222222222222');
    await ChatPushService.storeWakeSender(
        '0x1111111111111111111111111111111111111111111111111111111111111111');

    final service = ChatPushService();
    expect(
      await service.takePendingWakeSenders(),
      [
        '0x1111111111111111111111111111111111111111111111111111111111111111',
        '0x2222222222222222222222222222222222222222222222222222222222222222'
      ],
    );
    expect(await service.takePendingWakeSenders(), isEmpty);
    await service.dispose();
  });
}
