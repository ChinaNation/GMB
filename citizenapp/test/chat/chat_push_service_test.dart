import 'package:citizenapp/chat/chat_push_service.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  test('只接受无内容聊天唤醒载荷', () {
    expect(
      ChatPushService.wakeSenderFromData(const {
        'kind': 'chat_wake',
        'sender_account': 'alice-wallet',
      }),
      'alice-wallet',
    );
    expect(
      ChatPushService.wakeSenderFromData(const {
        'kind': 'chat_wake',
        'sender_account': 'alice-wallet',
        'message': '不得进入推送',
      }),
      isNull,
    );
    expect(
      ChatPushService.wakeSenderFromData(const {
        'kind': 'chat_message',
        'sender_account': 'alice-wallet',
      }),
      isNull,
    );
  });

  test('后台连续唤醒会去重保存全部发送方', () async {
    SharedPreferences.setMockInitialValues({});
    await ChatPushService.storeWakeSender('alice-wallet');
    await ChatPushService.storeWakeSender('bob-wallet');
    await ChatPushService.storeWakeSender('alice-wallet');

    final service = ChatPushService();
    expect(
      await service.takePendingWakeSenders(),
      ['alice-wallet', 'bob-wallet'],
    );
    expect(await service.takePendingWakeSenders(), isEmpty);
    await service.dispose();
  });
}
