import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:citizenapp/my/user/user_service.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  setUp(() {
    SharedPreferences.setMockInitialValues(<String, Object>{});
  });

  group('UserProfileService', () {
    test('returns empty profile when nothing stored', () async {
      final service = UserProfileService();

      final state = await service.getState();

      expect(state.avatarPath, isNull);
      expect(state.backgroundPath, isNull);
    });

    test('persists avatar and background paths across reads', () async {
      final service = UserProfileService();

      await service.updateAvatarPath('/tmp/avatar.png');
      await service.updateBackgroundPath('/tmp/bg.png');

      final state = await UserProfileService().getState();

      expect(state.avatarPath, '/tmp/avatar.png');
      expect(state.backgroundPath, '/tmp/bg.png');
    });
  });

  group('UserContactService', () {
    test('imports contact via addContact and supports local rename', () async {
      final service = UserContactService();
      const address = 'w5Bc7ma8qUcECfQDJmRyQM2wGmga5XSYtz7DvEengQ86xBWrT';

      final created = await service.addContact(
        address: address,
        contactName: '轻节点A',
      );
      expect(created.created, isTrue);
      expect(created.contact.displayNickname, '轻节点A');

      final contacts = await service.renameContact(
        created.contact.address,
        '本地备注',
      );

      expect(contacts.single.displayNickname, '本地备注');
      expect(contacts.single.sourceNickname, '轻节点A');
    });

    test('re-adding updates source nickname but keeps local alias', () async {
      final service = UserContactService();
      const address = 'w5BdS7eTPBdtPHq22ViUGARtNnHUszX9A7f4369bufEtoejq6';
      await service.addContact(address: address, contactName: '旧昵称');
      await service.renameContact(address, '本地昵称');

      final updated = await service.addContact(
        address: address,
        contactName: '新昵称',
      );

      expect(updated.created, isFalse);
      expect(updated.contact.displayNickname, '本地昵称');
      expect(updated.contact.sourceNickname, '新昵称');
    });

    test('rejects adding self to contact book', () async {
      final service = UserContactService();
      const address = 'w5BekTimvtfYZvFpkDzy7ypqUntPgTbjRFCt9weR8vMgf7o8E';

      await expectLater(
        service.addContact(
          address: address,
          contactName: '自己',
          selfAddress: address,
        ),
        throwsA(isA<FormatException>()),
      );
    });
  });
}
