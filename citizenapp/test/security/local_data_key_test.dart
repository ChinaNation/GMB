import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/security/local_cipher.dart';
import 'package:citizenapp/security/local_data_key.dart';

/// 内存版 blob 落地层，避免单测触碰真实 flutter_secure_storage。
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
  const cidNumber = 'GD-CTZN1-8F3A2B';
  const firstAccountId =
      '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
  const secondAccountId =
      '0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';
  final firstSecret = Uint8List.fromList(List<int>.generate(32, (i) => i));
  final secondSecret =
      Uint8List.fromList(List<int>.generate(32, (i) => 100 + i));
  final dataRoot =
      CidDataRoot(Uint8List.fromList(List<int>.generate(32, (i) => 200 - i)));

  late _MemoryStore store;
  late CidDataRootVault vault;
  late String dataRootHash;

  setUp(() async {
    store = _MemoryStore();
    vault = CidDataRootVault(store);
    dataRootHash = await CidDataRootVault.dataRootHash(dataRoot);
  });

  group('CID 数据根安装', () {
    test('当前账户独立安装并读回，不需要此前账户输入', () async {
      final installed = await vault.installForCurrentBinding(
        cidNumber: cidNumber,
        bindingRevision: 1,
        accountId: firstAccountId,
        accountSecret: firstSecret,
        dataRoot: dataRoot,
        expectedDataRootHash: dataRootHash,
      );
      expect(installed.bytes, dataRoot.bytes);
      final active = await vault.readActiveBinding();
      expect(active?.cidNumber, cidNumber);
      expect(active?.bindingRevision, 1);
      expect(active?.accountId, firstAccountId);
      expect(store.entries[active!.wrapperKey], isNotNull);
      expect(
        store.entries[active.wrapperKey],
        isNot(contains(String.fromCharCodes(dataRoot.bytes))),
      );
      final reopened = await vault.readForCurrentBinding(
        cidNumber: cidNumber,
        bindingRevision: 1,
        accountId: firstAccountId,
        accountSecret: firstSecret,
      );
      expect(reopened.bytes, dataRoot.bytes);
    });

    test('摘要不匹配时拒绝且不落任何激活状态', () async {
      await expectLater(
        vault.installForCurrentBinding(
          cidNumber: cidNumber,
          bindingRevision: 1,
          accountId: firstAccountId,
          accountSecret: firstSecret,
          dataRoot: dataRoot,
          expectedDataRootHash: '0' * 64,
        ),
        throwsA(isA<LocalCipherException>()),
      );
      expect(await vault.readActiveBinding(), isNull);
      expect(store.entries, isEmpty);
    });

    test('错误的新账户私钥不能解包', () async {
      await vault.installForCurrentBinding(
        cidNumber: cidNumber,
        bindingRevision: 1,
        accountId: firstAccountId,
        accountSecret: firstSecret,
        dataRoot: dataRoot,
        expectedDataRootHash: dataRootHash,
      );
      await expectLater(
        vault.readForCurrentBinding(
          cidNumber: cidNumber,
          bindingRevision: 1,
          accountId: firstAccountId,
          accountSecret: secondSecret,
        ),
        throwsA(isA<LocalCipherException>()),
      );
    });
  });

  group('用途子钥域隔离', () {
    test('全部用途派生出互不相同的稳定子钥', () async {
      final keys = <LocalKeyPurpose, Uint8List>{};
      for (final purpose in LocalKeyPurpose.values) {
        keys[purpose] = await dataRoot.subkey(purpose);
      }
      expect(keys.length, LocalKeyPurpose.values.length);
      for (final key in keys.values) {
        expect(key.length, 32);
      }
      final distinct = keys.values.map((k) => k.join(',')).toSet();
      expect(
        distinct.length,
        LocalKeyPurpose.values.length,
        reason: '任意两个用途的子钥都不得相同',
      );
    });

    test('数据根相同则换绑前后用途钥和密文均不变', () async {
      final key = await dataRoot.subkey(LocalKeyPurpose.chat);
      final blob = await LocalCipher.encryptString(
        key: key,
        plaintext: '换绑前聊天正文',
        aad: '${LocalKeyPurpose.chat.domain}|msg-1',
      );
      await vault.installForCurrentBinding(
        cidNumber: cidNumber,
        bindingRevision: 1,
        accountId: firstAccountId,
        accountSecret: firstSecret,
        dataRoot: dataRoot,
        expectedDataRootHash: dataRootHash,
      );
      final after = await vault.installForCurrentBinding(
        cidNumber: cidNumber,
        bindingRevision: 2,
        accountId: secondAccountId,
        accountSecret: secondSecret,
        dataRoot: dataRoot,
        expectedDataRootHash: dataRootHash,
      );
      expect(await after.subkey(LocalKeyPurpose.chat), key);
      expect(
        await LocalCipher.decryptString(
          key: await after.subkey(LocalKeyPurpose.chat),
          blob: blob,
          aad: '${LocalKeyPurpose.chat.domain}|msg-1',
        ),
        '换绑前聊天正文',
      );
    });
  });

  group('绑定版本接管', () {
    test('新账户验证上岗后清理低版本包装', () async {
      await vault.installForCurrentBinding(
        cidNumber: cidNumber,
        bindingRevision: 1,
        accountId: firstAccountId,
        accountSecret: firstSecret,
        dataRoot: dataRoot,
        expectedDataRootHash: dataRootHash,
      );
      final oldWrapper = (await vault.readActiveBinding())!.wrapperKey;
      await vault.installForCurrentBinding(
        cidNumber: cidNumber,
        bindingRevision: 2,
        accountId: secondAccountId,
        accountSecret: secondSecret,
        dataRoot: dataRoot,
        expectedDataRootHash: dataRootHash,
      );
      final active = await vault.readActiveBinding();
      expect(active?.bindingRevision, 2);
      expect(active?.accountId, secondAccountId);
      expect(store.entries[oldWrapper], isNull);
      expect(
        (await vault.readForCurrentBinding(
          cidNumber: cidNumber,
          bindingRevision: 2,
          accountId: secondAccountId,
          accountSecret: secondSecret,
        ))
            .bytes,
        dataRoot.bytes,
      );
    });

    test('本机绑定版本禁止回退', () async {
      await vault.installForCurrentBinding(
        cidNumber: cidNumber,
        bindingRevision: 2,
        accountId: secondAccountId,
        accountSecret: secondSecret,
        dataRoot: dataRoot,
        expectedDataRootHash: dataRootHash,
      );
      await expectLater(
        vault.installForCurrentBinding(
          cidNumber: cidNumber,
          bindingRevision: 1,
          accountId: firstAccountId,
          accountSecret: firstSecret,
          dataRoot: dataRoot,
          expectedDataRootHash: dataRootHash,
        ),
        throwsA(isA<LocalCipherException>()),
      );
    });

    test('同一 revision 不允许账户或数据根摘要冲突', () async {
      await vault.installForCurrentBinding(
        cidNumber: cidNumber,
        bindingRevision: 1,
        accountId: firstAccountId,
        accountSecret: firstSecret,
        dataRoot: dataRoot,
        expectedDataRootHash: dataRootHash,
      );
      await expectLater(
        vault.installForCurrentBinding(
          cidNumber: cidNumber,
          bindingRevision: 1,
          accountId: secondAccountId,
          accountSecret: secondSecret,
          dataRoot: dataRoot,
          expectedDataRootHash: dataRootHash,
        ),
        throwsA(isA<LocalCipherException>()),
      );
    });
  });

  test('clearActiveBinding 清除当前包装与激活标记', () async {
    await vault.installForCurrentBinding(
      cidNumber: cidNumber,
      bindingRevision: 1,
      accountId: firstAccountId,
      accountSecret: firstSecret,
      dataRoot: dataRoot,
      expectedDataRootHash: dataRootHash,
    );
    await vault.clearActiveBinding();
    expect(store.entries, isEmpty);
  });
}
