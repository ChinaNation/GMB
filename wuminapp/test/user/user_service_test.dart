import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/user/user_service.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  setUp(() {
    SharedPreferences.setMockInitialValues(<String, Object>{});
  });

  group('UserProfileService', () {
    test('returns default nickname when no communication wallet is set',
        () async {
      final service = UserProfileService();

      final state = await service.getState();

      expect(state.nickname, UserProfileService.defaultNickname);
      expect(state.communicationWalletName, isNull);
    });

    test('setCommunicationWallet stores wallet and exposes nickname', () async {
      final service = UserProfileService();

      final state = await service.setCommunicationWallet(
        walletIndex: 0,
        address: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
        walletName: '小节点',
      );

      expect(state.nickname, '小节点');
      expect(state.communicationWalletIndex, 0);
      expect(state.communicationAddress,
          '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY');
    });

    test('updateCommunicationWalletName updates nickname', () async {
      final service = UserProfileService();
      await service.setCommunicationWallet(
        walletIndex: 0,
        address: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
        walletName: '旧名称',
      );

      final state = await service.updateCommunicationWalletName('新名称');

      expect(state.nickname, '新名称');
    });
  });

  group('UserContactService', () {
    test('imports contact via addContact and supports local rename', () async {
      final service = UserContactService();
      const address =
          '0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef';

      final created = await service.addContact(
        address: address,
        name: '轻节点A',
      );
      expect(created.created, isTrue);
      expect(created.contact.displayNickname, '轻节点A');

      final contacts = await service.renameContact(
        created.contact.accountPubkeyHex,
        '本地备注',
      );

      expect(contacts.single.displayNickname, '本地备注');
      expect(contacts.single.sourceNickname, '轻节点A');
    });

    test('re-adding updates source nickname but keeps local alias', () async {
      final service = UserContactService();
      const account =
          '0xabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd';
      await service.addContact(address: account, name: '旧昵称');
      await service.renameContact(account, '本地昵称');

      final updated = await service.addContact(
        address: account,
        name: '新昵称',
      );

      expect(updated.created, isFalse);
      expect(updated.contact.displayNickname, '本地昵称');
      expect(updated.contact.sourceNickname, '新昵称');
    });

    test('rejects adding self to contact book', () async {
      final service = UserContactService();
      const address =
          '0x1111111111111111111111111111111111111111111111111111111111111111';

      await expectLater(
        service.addContact(
          address: address,
          name: '自己',
          selfAddress: address,
        ),
        throwsA(isA<FormatException>()),
      );
    });
  });
}
