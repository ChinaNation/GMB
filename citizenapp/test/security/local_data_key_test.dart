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
  const oldAccountId =
      '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
  const newAccountId =
      '0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';
  final oldSecret = Uint8List.fromList(List<int>.generate(32, (i) => i));
  final newSecret = Uint8List.fromList(List<int>.generate(32, (i) => 100 + i));

  late _MemoryStore store;
  late LocalDataKeyVault vault;

  setUp(() {
    store = _MemoryStore();
    vault = LocalDataKeyVault(store);
  });

  group('LDK 建立与幂等', () {
    test('首次生成后落地为密文，且不含 LDK 明文', () async {
      final ldk = await vault.ensureForAccount(
        accountId: oldAccountId,
        accountSecret: oldSecret,
      );
      expect(ldk.bytes.length, 32);
      final blob = store.entries[LocalDataKeyVault.storageKeyFor(oldAccountId)];
      expect(blob, isNotNull);
      expect(blob, isNot(contains(String.fromCharCodes(ldk.bytes))));
    });

    test('重复调用幂等：绝不换钥（换钥会作废已落盘密文）', () async {
      final first = await vault.ensureForAccount(
        accountId: oldAccountId,
        accountSecret: oldSecret,
      );
      final second = await vault.ensureForAccount(
        accountId: oldAccountId,
        accountSecret: oldSecret,
      );
      expect(second.bytes, first.bytes);
    });

    test('未建立过返回 null', () async {
      final got = await vault.readForAccount(
        accountId: oldAccountId,
        accountSecret: oldSecret,
      );
      expect(got, isNull);
    });

    test('错误账户密钥解包必须抛错，不得降级为 null', () async {
      await vault.ensureForAccount(
        accountId: oldAccountId,
        accountSecret: oldSecret,
      );
      await expectLater(
        vault.readForAccount(
          accountId: oldAccountId,
          accountSecret: newSecret,
        ),
        throwsA(isA<LocalCipherException>()),
      );
    });
  });

  group('用途子钥域隔离', () {
    test('五个用途派生出五把互不相同的子钥', () async {
      final ldk = await vault.ensureForAccount(
        accountId: oldAccountId,
        accountSecret: oldSecret,
      );
      final keys = <LocalKeyPurpose, Uint8List>{};
      for (final purpose in LocalKeyPurpose.values) {
        keys[purpose] = await ldk.subkey(purpose);
      }
      expect(keys.length, LocalKeyPurpose.values.length);
      for (final key in keys.values) {
        expect(key.length, 32);
      }
      final distinct = keys.values.map((k) => k.join(',')).toSet();
      expect(distinct.length, LocalKeyPurpose.values.length,
          reason: '任意两个用途的子钥都不得相同');
    });

    test('派生确定性：同 LDK 同用途逐字节稳定', () async {
      final ldk = LocalDataKey(
        Uint8List.fromList(List<int>.generate(32, (i) => i * 7 % 256)),
      );
      final a = await ldk.subkey(LocalKeyPurpose.chat);
      final b = await ldk.subkey(LocalKeyPurpose.chat);
      expect(a, b);
    });

    test('子钥可直接驱动 LocalCipher 往返', () async {
      final ldk = await vault.ensureForAccount(
        accountId: oldAccountId,
        accountSecret: oldSecret,
      );
      final key = await ldk.subkey(LocalKeyPurpose.chat);
      final blob = await LocalCipher.encryptString(
        key: key,
        plaintext: '聊天正文',
        aad: '${LocalKeyPurpose.chat.domain}|msg-1',
      );
      expect(
        await LocalCipher.decryptString(
          key: key,
          blob: blob,
          aad: '${LocalKeyPurpose.chat.domain}|msg-1',
        ),
        '聊天正文',
      );
    });

    test('跨用途子钥不可互解（域隔离真实生效）', () async {
      final ldk = await vault.ensureForAccount(
        accountId: oldAccountId,
        accountSecret: oldSecret,
      );
      final chatKey = await ldk.subkey(LocalKeyPurpose.chat);
      final mlsKey = await ldk.subkey(LocalKeyPurpose.mls);
      final blob = await LocalCipher.encryptString(
        key: chatKey,
        plaintext: '正文',
        aad: '${LocalKeyPurpose.chat.domain}|x',
      );
      await expectLater(
        LocalCipher.decryptBytes(
          key: mlsKey,
          blob: blob,
          aad: '${LocalKeyPurpose.chat.domain}|x',
        ),
        throwsA(isA<LocalCipherException>()),
      );
    });
  });

  group('CID 换绑：只重 wrap，数据不动', () {
    test('换绑后 LDK 与全部子钥逐字节不变', () async {
      final before = await vault.ensureForAccount(
        accountId: oldAccountId,
        accountSecret: oldSecret,
      );
      final subkeysBefore = <Uint8List>[
        for (final p in LocalKeyPurpose.values) await before.subkey(p),
      ];

      final after = await vault.rewrapForRebind(
        oldAccountId: oldAccountId,
        oldAccountSecret: oldSecret,
        newAccountId: newAccountId,
        newAccountSecret: newSecret,
      );

      expect(after.bytes, before.bytes, reason: 'LDK 终身不变');
      for (var i = 0; i < LocalKeyPurpose.values.length; i += 1) {
        expect(await after.subkey(LocalKeyPurpose.values[i]), subkeysBefore[i],
            reason: '换绑后子钥必须不变，否则已落盘密文全部作废');
      }
    });

    test('换绑前加密的密文，换绑后仍能解开', () async {
      final before = await vault.ensureForAccount(
        accountId: oldAccountId,
        accountSecret: oldSecret,
      );
      final blob = await LocalCipher.encryptString(
        key: await before.subkey(LocalKeyPurpose.chat),
        plaintext: '换绑前写下的聊天记录',
        aad: '${LocalKeyPurpose.chat.domain}|msg-9',
      );

      await vault.rewrapForRebind(
        oldAccountId: oldAccountId,
        oldAccountSecret: oldSecret,
        newAccountId: newAccountId,
        newAccountSecret: newSecret,
      );

      final reopened = await vault.readForAccount(
        accountId: newAccountId,
        accountSecret: newSecret,
      );
      expect(reopened, isNotNull);
      expect(
        await LocalCipher.decryptString(
          key: await reopened!.subkey(LocalKeyPurpose.chat),
          blob: blob,
          aad: '${LocalKeyPurpose.chat.domain}|msg-9',
        ),
        '换绑前写下的聊天记录',
      );
    });

    test('换绑后旧账户条目被清除，旧账户不再能解包', () async {
      await vault.ensureForAccount(
        accountId: oldAccountId,
        accountSecret: oldSecret,
      );
      await vault.rewrapForRebind(
        oldAccountId: oldAccountId,
        oldAccountSecret: oldSecret,
        newAccountId: newAccountId,
        newAccountSecret: newSecret,
      );
      expect(
        store.entries[LocalDataKeyVault.storageKeyFor(oldAccountId)],
        isNull,
      );
      expect(
        await vault.readForAccount(
          accountId: oldAccountId,
          accountSecret: oldSecret,
        ),
        isNull,
      );
    });

    test('writeForAccount：用已在手的 LDK 直接为新账户 wrap（省一次生物识别）', () async {
      final ldk = await vault.ensureForAccount(
        accountId: oldAccountId,
        accountSecret: oldSecret,
      );
      await vault.writeForAccount(
        accountId: newAccountId,
        accountSecret: newSecret,
        ldk: ldk,
      );
      final reopened = await vault.readForAccount(
        accountId: newAccountId,
        accountSecret: newSecret,
      );
      expect(reopened, isNotNull);
      expect(reopened!.bytes, ldk.bytes);
    });

    test('换绑前从未建立过 LDK：为新账户直接建一把', () async {
      final ldk = await vault.rewrapForRebind(
        oldAccountId: oldAccountId,
        oldAccountSecret: oldSecret,
        newAccountId: newAccountId,
        newAccountSecret: newSecret,
      );
      expect(ldk.bytes.length, 32);
      expect(
        store.entries[LocalDataKeyVault.storageKeyFor(newAccountId)],
        isNotNull,
      );
    });
  });

  test('deleteForAccount 清除条目', () async {
    await vault.ensureForAccount(
      accountId: oldAccountId,
      accountSecret: oldSecret,
    );
    await vault.deleteForAccount(oldAccountId);
    expect(store.entries, isEmpty);
  });
}
