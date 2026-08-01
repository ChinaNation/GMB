import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/security/local_cipher.dart';
import 'package:citizenapp/security/local_data_key.dart';

class _MemoryStore implements LocalKeyBlobStore {
  final Map<String, String> entries = <String, String>{};

  @override
  Future<String?> read(String key) async => entries[key];

  @override
  Future<void> write(String key, String value) async => entries[key] = value;

  @override
  Future<void> delete(String key) async => entries.remove(key);
}

void main() {
  const genesisHash =
      '0x1111111111111111111111111111111111111111111111111111111111111111';
  const cidNumber = 'GD-CTZN1-8F3A2B';
  const firstAccountId =
      '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
  const secondAccountId =
      '0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';
  final firstSecret = Uint8List.fromList(List<int>.generate(32, (i) => i));
  final secondSecret =
      Uint8List.fromList(List<int>.generate(32, (i) => 100 + i));
  const firstBinding = AccountDataBinding(
    genesisHash: genesisHash,
    cidNumber: cidNumber,
    bindingRevision: 1,
    accountId: firstAccountId,
  );
  const secondBinding = AccountDataBinding(
    genesisHash: genesisHash,
    cidNumber: cidNumber,
    bindingRevision: 2,
    accountId: secondAccountId,
  );

  group('当前钱包绑定元数据', () {
    late _MemoryStore store;
    late AccountDataBindingStore bindingStore;

    setUp(() {
      store = _MemoryStore();
      bindingStore = AccountDataBindingStore(store);
    });

    test('只保存公开绑定字段，不保存任何派生密钥', () async {
      await bindingStore.activate(firstBinding);
      final active = await bindingStore.readActiveBinding();
      expect(active?.genesisHash, genesisHash);
      expect(active?.cidNumber, cidNumber);
      expect(active?.bindingRevision, 1);
      expect(active?.accountId, firstAccountId);
      expect(store.entries.length, 1);
      expect(
          store.entries.values.single, isNot(contains(firstSecret.join(','))));
    });

    test('绑定版本禁止回退，同版本字段冲突失败关闭', () async {
      await bindingStore.activate(secondBinding);
      await expectLater(
        bindingStore.activate(firstBinding),
        throwsA(isA<AccountDataKeyException>()),
      );
      await expectLater(
        bindingStore.activate(const AccountDataBinding(
          genesisHash: genesisHash,
          cidNumber: cidNumber,
          bindingRevision: 2,
          accountId: firstAccountId,
        )),
        throwsA(isA<AccountDataKeyException>()),
      );
    });

    test('直接构造的无效链上绑定字段也失败关闭', () async {
      const invalidBindings = <AccountDataBinding>[
        AccountDataBinding(
          genesisHash: '0x01',
          cidNumber: cidNumber,
          bindingRevision: 1,
          accountId: firstAccountId,
        ),
        AccountDataBinding(
          genesisHash: genesisHash,
          cidNumber: '123456789012345678901234567890123',
          bindingRevision: 1,
          accountId: firstAccountId,
        ),
        AccountDataBinding(
          genesisHash: genesisHash,
          cidNumber: cidNumber,
          bindingRevision: 0,
          accountId: firstAccountId,
        ),
        AccountDataBinding(
          genesisHash: genesisHash,
          cidNumber: cidNumber,
          bindingRevision: 1,
          accountId: '0x01',
        ),
      ];
      for (final binding in invalidBindings) {
        await expectLater(
          bindingStore.activate(binding),
          throwsA(isA<AccountDataKeyException>()),
        );
        await expectLater(
          AccountDataKeyDeriver.derive(
            accountSecret: firstSecret,
            binding: binding,
            purpose: LocalKeyPurpose.chat,
          ),
          throwsA(isA<AccountDataKeyException>()),
        );
      }
      expect(store.entries, isEmpty);
    });

    test('换绑交接日志只保存相邻版本的公开绑定上下文并可清除', () async {
      await bindingStore.writePendingHandover(
        source: firstBinding,
        target: secondBinding,
      );
      final pending = await bindingStore.readPendingHandover();
      expect(pending?.source.accountId, firstAccountId);
      expect(pending?.target.accountId, secondAccountId);
      expect(pending?.target.bindingRevision, 2);
      expect(store.entries.keys, <String>[
        AccountDataBindingStore.pendingHandoverKey,
      ]);
      expect(
          store.entries.values.single, isNot(contains(firstSecret.join(','))));
      expect(
          store.entries.values.single, isNot(contains(secondSecret.join(','))));

      await bindingStore.clearPendingHandover();
      expect(await bindingStore.readPendingHandover(), isNull);
      expect(store.entries, isEmpty);
    });

    test('换绑交接拒绝跨 CID、跨创世、跳版本和同账户目标', () async {
      final invalidTargets = <AccountDataBinding>[
        const AccountDataBinding(
          genesisHash: genesisHash,
          cidNumber: 'GD-CTZN1-OTHER',
          bindingRevision: 2,
          accountId: secondAccountId,
        ),
        const AccountDataBinding(
          genesisHash:
              '0x2222222222222222222222222222222222222222222222222222222222222222',
          cidNumber: cidNumber,
          bindingRevision: 2,
          accountId: secondAccountId,
        ),
        const AccountDataBinding(
          genesisHash: genesisHash,
          cidNumber: cidNumber,
          bindingRevision: 3,
          accountId: secondAccountId,
        ),
        const AccountDataBinding(
          genesisHash: genesisHash,
          cidNumber: cidNumber,
          bindingRevision: 2,
          accountId: firstAccountId,
        ),
      ];
      for (final target in invalidTargets) {
        await expectLater(
          bindingStore.writePendingHandover(
            source: firstBinding,
            target: target,
          ),
          throwsA(isA<AccountDataKeyException>()),
        );
      }
      expect(store.entries, isEmpty);
    });

    test('已落盘交接记录被篡改成跳版本时读取也失败关闭', () async {
      await bindingStore.writePendingHandover(
        source: firstBinding,
        target: secondBinding,
      );
      final decoded = jsonDecode(
        store.entries[AccountDataBindingStore.pendingHandoverKey]!,
      ) as Map<String, dynamic>;
      (decoded['target'] as Map<String, dynamic>)['binding_revision'] = 3;
      store.entries[AccountDataBindingStore.pendingHandoverKey] =
          jsonEncode(decoded);

      await expectLater(
        bindingStore.readPendingHandover(),
        throwsA(isA<AccountDataKeyException>()),
      );
    });
  });

  group('当前钱包账户用途子钥', () {
    test('同一账户同一绑定跨设备派生结果一致', () async {
      final first = await AccountDataKeyDeriver.derive(
        accountSecret: firstSecret,
        binding: firstBinding,
        purpose: LocalKeyPurpose.chat,
      );
      final anotherDevice = await AccountDataKeyDeriver.derive(
        accountSecret: Uint8List.fromList(firstSecret),
        binding: firstBinding,
        purpose: LocalKeyPurpose.chat,
      );
      expect(anotherDevice, first);
    });

    test('全部用途域互相隔离', () async {
      final values = <String>{};
      for (final purpose in LocalKeyPurpose.values) {
        final key = await AccountDataKeyDeriver.derive(
          accountSecret: firstSecret,
          binding: firstBinding,
          purpose: purpose,
        );
        expect(key, hasLength(32));
        values.add(key.join(','));
      }
      expect(values.length, LocalKeyPurpose.values.length);
    });

    test('同一用途的 encryption 与 index 上下文互相隔离', () async {
      final encryptionKey = await AccountDataKeyDeriver.derive(
        accountSecret: firstSecret,
        binding: firstBinding,
        purpose: LocalKeyPurpose.contactsCloud,
        context: 'encryption',
      );
      final indexKey = await AccountDataKeyDeriver.derive(
        accountSecret: firstSecret,
        binding: firstBinding,
        purpose: LocalKeyPurpose.contactsCloud,
        context: 'index',
      );
      expect(indexKey, isNot(encryptionKey));
    });

    test('没有当前账户签名交接时，新钱包不能直接解密此前钱包历史私有密文', () async {
      final currentKey = await AccountDataKeyDeriver.derive(
        accountSecret: firstSecret,
        binding: firstBinding,
        purpose: LocalKeyPurpose.chat,
      );
      final oldCiphertext = await LocalCipher.encryptString(
        key: currentKey,
        plaintext: '此前钱包历史私有数据',
        aad: '${LocalKeyPurpose.chat.domain}|message-before-rebind',
      );
      final newKey = await AccountDataKeyDeriver.derive(
        accountSecret: secondSecret,
        binding: secondBinding,
        purpose: LocalKeyPurpose.chat,
      );
      expect(newKey, isNot(currentKey));
      await expectLater(
        LocalCipher.decryptString(
          key: newKey,
          blob: oldCiphertext,
          aad: '${LocalKeyPurpose.chat.domain}|message-before-rebind',
        ),
        throwsA(isA<LocalCipherException>()),
      );
    });
  });
}
