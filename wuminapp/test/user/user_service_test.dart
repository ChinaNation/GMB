import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/user/user_service.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  setUp(() {
    SharedPreferences.setMockInitialValues(<String, Object>{});
  });

  group('UserProfileService', () {
    test('returns default nickname and disabled qr state before user setup',
        () async {
      final service = UserProfileService();

      final state = await service.getState();

      expect(state.nickname, UserProfileService.defaultNickname);
      expect(state.nicknameCustomized, isFalse);
      expect(service.isNicknameReady(state), isFalse);
    });

    test('updateNickname marks nickname as customized', () async {
      final service = UserProfileService();

      final state = await service.updateNickname('小节点');

      expect(state.nickname, '小节点');
      expect(state.nicknameCustomized, isTrue);
      expect(service.isNicknameReady(state), isTrue);
    });
  });

  group('UserContactService', () {
    test('imports contact from user qr payload and supports local rename',
        () async {
      final service = UserContactService();
      final payload = const UserQrPayload(
        nickname: '轻节点A',
        address:
            '0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef',
      ).toRawJson();

      final created = await service.addFromQrPayload(payload);
      expect(created.created, isTrue);
      expect(created.contact.displayNickname, '轻节点A');

      final contacts = await service.renameContact(
        created.contact.accountPubkeyHex,
        '本地备注',
      );

      expect(contacts.single.displayNickname, '本地备注');
      expect(contacts.single.sourceNickname, '轻节点A');
    });

    test('re-scan updates source nickname but keeps local alias', () async {
      final service = UserContactService();
      const account =
          '0xabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd';
      await service.addFromQrPayload(
        const UserQrPayload(
          nickname: '旧昵称',
          address: account,
        ).toRawJson(),
      );
      await service.renameContact(account, '本地昵称');

      final updated = await service.addFromQrPayload(
        const UserQrPayload(
          nickname: '新昵称',
          address: account,
        ).toRawJson(),
      );

      expect(updated.created, isFalse);
      expect(updated.contact.displayNickname, '本地昵称');
      expect(updated.contact.sourceNickname, '新昵称');
    });

    test('rejects adding self to contact book', () async {
      final service = UserContactService();
      final payload = const UserQrPayload(
        nickname: '自己',
        address:
            '0x1111111111111111111111111111111111111111111111111111111111111111',
      ).toRawJson();

      await expectLater(
        service.addFromQrPayload(
          payload,
          selfAccountPubkeyHex:
              '0x1111111111111111111111111111111111111111111111111111111111111111',
        ),
        throwsA(isA<FormatException>()),
      );
    });
  });
}
