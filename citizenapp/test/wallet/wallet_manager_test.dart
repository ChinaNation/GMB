import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:sr25519/sr25519.dart' as sr;
import 'package:substrate_bip39/substrate_bip39.dart';
import 'package:citizenapp/isar/app_isar.dart';
import 'package:citizenapp/security/local_data_key.dart';
import 'package:citizenapp/wallet/core/secure_seed_store.dart';
import 'package:citizenapp/wallet/core/hardware_bound_seed_vault.dart';
import 'package:citizenapp/wallet/core/device_subkey.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

import '../support/fake_secure_seed_store.dart';
import '../support/isar_test_env.dart';

const _mnemonicA =
    'legal winner thank year wave sausage worth useful legal winner thank yellow';
// 另一条合法但派生不同公钥的助记词（独立导入与私钥存储校验用）。
const _mnemonicB =
    'abandon abandon abandon abandon abandon abandon abandon abandon '
    'abandon abandon abandon about';

String _coldSs58(int byte) =>
    Keyring().encodeAddress(List<int>.filled(32, byte), 2027);

String _hex(List<int> b) =>
    b.map((x) => x.toRadixString(16).padLeft(2, '0')).join();

/// 助记词 → 母种子（master mini-secret，32B）。
Future<Uint8List> _masterSeed(String mnemonic) async {
  final entropy = Mnemonic.fromSentence(mnemonic, Language.english).entropy;
  return Uint8List.fromList(await CryptoScheme.miniSecretFromEntropy(entropy));
}

/// 复现 WalletManager 的账户 child mini-secret 派生（金标同源）。
List<int> _childMiniSecret(List<int> seed, int index) {
  final junctions = SecretUri.fromStr('//$index').junctions;
  var rootSk = sr.MiniSecretKey.fromRawKey(seed).expandEd25519();
  late List<int> child;
  for (final j in junctions) {
    final cc = j.junctionId.sublist(0, 32);
    final derived = rootSk.hardDeriveMiniSecretKey(const <int>[], cc);
    child = derived.$1.encode();
    rootSk = derived.$1.expandEd25519();
  }
  return child;
}

/// 安全不变量断言：存的必须是账户0(`//0`) 的 child，绝不是母种子。
Future<void> _expectChildStoredNotSeed(
  String mnemonic,
  String storedHex,
) async {
  final master = await _masterSeed(mnemonic);
  final child = _childMiniSecret(master, 0);
  expect(storedHex, _hex(child), reason: '严档存的必须是账户0 //0 的 child');
  expect(storedHex, isNot(_hex(master)), reason: '绝不持久化母种子');
}

class _MemoryBlobStore implements VaultBlobStore {
  final Map<String, String> values = <String, String>{};

  @override
  Future<String?> read(String key) async => values[key];

  @override
  Future<void> write(String key, String value) async => values[key] = value;

  @override
  Future<void> delete(String key) async => values.remove(key);
}

class _RecordingDeviceSubkey extends DeviceSubkey {
  final List<int> deletedWalletIndexes = <int>[];
  final Set<int> failingWalletIndexes = <int>{};

  @override
  Future<void> delete(int walletIndex) async {
    deletedWalletIndexes.add(walletIndex);
    if (failingWalletIndexes.contains(walletIndex)) {
      throw StateError('设备子钥删除失败');
    }
  }
}

class _DeleteFailingSeedStore extends FakeSecureSeedStore {
  bool failAccountKeyDeletion = false;

  @override
  Future<void> deleteAccountKey({
    required int walletIndex,
    required String accountId,
  }) async {
    if (failAccountKeyDeletion) {
      throw StateError('账户 child 删除失败');
    }
    await super.deleteAccountKey(
      walletIndex: walletIndex,
      accountId: accountId,
    );
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  useIsolatedIsar();

  late FakeSecureSeedStore fakeStore;
  late _MemoryBlobStore contactBlobStore;
  late _RecordingDeviceSubkey deviceSubkey;

  // 动钱动权验证已上移到 WalletManager 的硬件金库读 child；单测里把 local_auth
  // channel 打桩为「验证通过」，让纯 Dart 环境不因缺插件而抛。
  const localAuthChannel = MethodChannel('plugins.flutter.io/local_auth');

  setUp(() async {
    SharedPreferences.setMockInitialValues(<String, Object>{});
    fakeStore = FakeSecureSeedStore();
    WalletManager.debugSeedStore = fakeStore;
    contactBlobStore = _MemoryBlobStore();
    WalletManager.debugContactKeyStore = contactBlobStore;
    deviceSubkey = _RecordingDeviceSubkey();
    WalletManager.debugDeviceSubkey = deviceSubkey;
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(localAuthChannel, (call) async {
      switch (call.method) {
        case 'authenticate':
          return true;
        case 'getAvailableBiometrics':
          return <String>['fingerprint', 'face'];
        case 'isDeviceSupported':
        case 'deviceSupportsBiometrics':
        case 'canCheckBiometrics':
          return true;
        default:
          return null;
      }
    });
  });

  tearDown(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(localAuthChannel, null);
  });

  group('WalletManager — 热钱包创建/导入/删除（ROOTLESS）', () {
    test('通讯录密钥只读写新域并主动删除旧命名残留', () async {
      final manager = WalletManager();
      final created = await manager.importWallet(_mnemonicA);
      final accountId = created.accountId;
      final legacyKey = 'wallet_contacts_key_v1_$accountId';
      const cidNumber = 'GD-CTZN1-TEST';
      const currentKey = 'citizenapp_cid_contacts_key_GD-CTZN1-TEST';
      contactBlobStore.values[legacyKey] = '旧派生密钥';
      final root = CidDataRoot(Uint8List(32));
      await manager.installCidDataRootForCurrentBinding(
        cidNumber: cidNumber,
        bindingRevision: 1,
        accountId: accountId,
        dataRoot: root,
        dataRootHash: await CidDataRootVault.dataRootHash(root),
      );

      final material =
          await manager.ensureContactKeyMaterialForAccountId(accountId);

      expect(material.encryptionKey, hasLength(32));
      expect(material.indexKey, hasLength(32));
      expect(contactBlobStore.values, isNot(contains(legacyKey)));
      expect(contactBlobStore.values, contains(currentKey));
    });

    test('create/import/delete 只存账户0 child，不存种子/助记词', () async {
      final manager = WalletManager();

      final created = await manager.createWallet();
      expect(created.profile.walletIndex, 1);
      expect(created.profile.alg, 'sr25519');
      expect(created.profile.ss58, 2027);
      expect(created.profile.signMode, 'local');
      expect(created.mnemonic.trim().split(RegExp(r'\s+')).length, 12);
      // 严档只落账户0 child（64 hex）；无母种子、无助记词档。
      final createdKey = fakeStore.accountKeys[created.profile.accountId];
      expect(createdKey, isNotNull);
      expect(createdKey!.length, 64);
      await _expectChildStoredNotSeed(created.mnemonic, createdKey);
      // 助记词绝不出现在任何持久化条目里（只一次性返回供备份）。
      expect(fakeStore.accountKeys.values, isNot(contains(created.mnemonic)));
      expect(fakeStore.accountKeys.length, 1);
      // 账户0 作为锚点账户同步落库,出现在 getAccounts。
      final createdAccounts =
          await manager.getAccounts(created.profile.accountId);
      expect(createdAccounts.map((a) => a.accountIndex).toList(), [0]);

      // 一台设备仅一个热钱包:删掉当前热钱包后才能导入下一只。
      await manager.deleteWallet(created.profile.walletIndex);
      expect(await manager.getWallets(), isEmpty);
      expect(fakeStore.accountKeys, isEmpty);
      expect(deviceSubkey.deletedWalletIndexes, [created.profile.walletIndex]);

      final imported = await manager.importWallet(_mnemonicA);
      expect(imported.walletIndex, 1);
      expect(imported.signMode, 'local');
      final importedKey = fakeStore.accountKeys[imported.accountId];
      expect(importedKey, isNotNull);
      await _expectChildStoredNotSeed(_mnemonicA, importedKey!);

      await manager.deleteWallet(imported.walletIndex);
      expect(fakeStore.accountKeys.containsKey(imported.accountId), isFalse);
      expect(await manager.getWallet(), isNull);
      expect(await manager.getWallets(), isEmpty);
      expect(fakeStore.accountKeys, isEmpty);
      expect(
        deviceSubkey.deletedWalletIndexes,
        [created.profile.walletIndex, imported.walletIndex],
      );
    });

    test('删除非末账户只清账户秘密，删除整钱包才清共享设备子钥', () async {
      final manager = WalletManager();
      final wallet = await manager.importWallet(_mnemonicA);
      final account1 = await manager.addNextAccount(
        wallet.accountId,
        _mnemonicA,
      );

      final root = CidDataRoot(Uint8List(32));
      await manager.installCidDataRootForCurrentBinding(
        cidNumber: 'GD-CTZN1-TEST',
        bindingRevision: 1,
        accountId: account1.accountId,
        dataRoot: root,
        dataRootHash: await CidDataRootVault.dataRootHash(root),
      );
      const contactKey = 'contact_book_by_cid:GD-CTZN1-TEST';
      await WalletIsar.instance.writeTxn((isar) async {
        await isar.appKvEntitys.put(
          AppKvEntity()
            ..key = contactKey
            ..stringValue = 'CID 通讯录密文',
        );
      });
      await manager.deleteAccount(account1.accountId);

      expect(deviceSubkey.deletedWalletIndexes, isEmpty);
      expect(fakeStore.accountKeys.containsKey(account1.accountId), isFalse);
      expect(
        await WalletIsar.instance.read(
          (isar) => isar.appKvEntitys.getByKey(contactKey),
        ),
        isNotNull,
        reason: '删除换绑后的旧账户不得删除永久 CID 的本机数据',
      );

      await manager.deleteWallet(wallet.walletIndex);
      expect(deviceSubkey.deletedWalletIndexes, [wallet.walletIndex]);
      expect(contactBlobStore.values, isEmpty);
      expect(
        await WalletIsar.instance.read(
          (isar) => isar.appKvEntitys.getByKey(contactKey),
        ),
        isNull,
        reason: '明确删除当前热钱包时执行本机 CID 隐私数据擦除',
      );
    });

    test('clearWallet 清全部账户秘密并只删除热钱包设备子钥', () async {
      final manager = WalletManager();
      final hot = await manager.importWallet(_mnemonicA);
      final cold = await manager.importColdWallet(ss58Address: _coldSs58(0x44));
      final root = CidDataRoot(Uint8List(32));
      await manager.installCidDataRootForCurrentBinding(
        cidNumber: 'GD-CTZN1-TEST',
        bindingRevision: 1,
        accountId: hot.accountId,
        dataRoot: root,
        dataRootHash: await CidDataRootVault.dataRootHash(root),
      );

      await manager.clearWallet();

      expect(await manager.getWallets(), isEmpty);
      expect(fakeStore.accountKeys, isEmpty);
      expect(contactBlobStore.values, isEmpty);
      expect(deviceSubkey.deletedWalletIndexes, [hot.walletIndex]);
      expect(
          deviceSubkey.deletedWalletIndexes, isNot(contains(cold.walletIndex)));
    });

    test('账户 child 清理失败仍继续删除钱包 KEK 与设备子钥', () async {
      final failingStore = _DeleteFailingSeedStore();
      WalletManager.debugSeedStore = failingStore;
      final manager = WalletManager();
      final wallet = await manager.importWallet(_mnemonicA);
      failingStore.failAccountKeyDeletion = true;
      deviceSubkey.failingWalletIndexes.add(wallet.walletIndex);

      await expectLater(
        manager.deleteWallet(wallet.walletIndex),
        throwsA(
          isA<WalletLocalCleanupException>().having(
            (error) => error.failures.join('\n'),
            'failures',
            allOf(
              contains('账户 child 删除失败'),
              contains('设备子钥删除失败'),
            ),
          ),
        ),
      );

      expect(await manager.getWallets(), isEmpty);
      expect(failingStore.deletedWalletKeyIndexes, [wallet.walletIndex]);
      expect(deviceSubkey.deletedWalletIndexes, [wallet.walletIndex]);
    });

    test('importWallet 拒绝非法助记词', () async {
      final manager = WalletManager();
      expect(
        () => manager.importWallet('hello world'),
        throwsA(
          isA<Exception>().having(
            (e) => e.toString(),
            'message',
            contains('助记词无效'),
          ),
        ),
      );
    });

    test('getDefaultWallet 忽略冷钱包（WalletGate 门禁判定依据）', () async {
      final manager = WalletManager();

      expect(await manager.getDefaultWallet(), isNull);

      await manager.importColdWallet(
        ss58Address: _coldSs58(0x22),
      );
      expect(await manager.getDefaultWallet(), isNull);

      final imported = await manager.importWallet(_mnemonicA);
      final def = await manager.getDefaultWallet();
      expect(def, isNotNull);
      expect(def!.walletIndex, imported.walletIndex);
      expect(def.isHotWallet, isTrue);
    });

    test('D3：无锁屏设备拒绝创建热钱包（fail-closed）', () async {
      fakeStore.noDeviceLock = true;
      final manager = WalletManager();
      await expectLater(
        manager.createWallet(),
        throwsA(isA<WalletAuthException>()),
      );
      // 未落库、未写密钥。
      expect(await manager.getWallets(), isEmpty);
      expect(fakeStore.accountKeys, isEmpty);
    });
  });

  group('设备子钥懒绑定：建钱包不注册子钥', () {
    tearDown(() => WalletManager.subkeyRegistrar = null);

    /// 一旦被调用即抛，用来证明建钱包/导入根本不会走到子钥注册。
    Future<void> failingRegistrar({
      required int walletIndex,
      required String cidNumber,
      required int bindingRevision,
      required String accountId,
      required Future<String> Function(Uint8List bindingMessage) signBinding,
    }) async {
      throw Exception('建钱包阶段不应注册设备子钥');
    }

    test('createWallet 不注册子钥：即使 registrar 必抛也照常建成', () async {
      // 子钥只服务广场 / 聊天 / 通讯录等需 CID 的场景，而建钱包这一刻账户还没有 CID
      // （后端 device/register 要求已绑 CID）。只用钱包和交易的用户根本不需要子钥，
      // 更不该因为后端不可用就建不出钱包。
      WalletManager.subkeyRegistrar = failingRegistrar;
      final manager = WalletManager();
      final created = await manager.createWallet();
      expect((await manager.getWallets()).length, 1);
      expect(fakeStore.accountKeys[created.profile.accountId], isNotNull);
    });

    test('importWallet 不注册子钥：即使 registrar 必抛也照常导入', () async {
      WalletManager.subkeyRegistrar = failingRegistrar;
      final manager = WalletManager();
      final profile = await manager.importWallet(_mnemonicA);
      expect((await manager.getWallets()).length, 1);
      expect(fakeStore.accountKeys[profile.accountId], isNotNull);
    });

    test('bindDeviceSubkeyToCurrentBinding 才是唯一绑定入口，按 CID 当前账户签证明', () async {
      String? seenCidNumber;
      int? seenBindingRevision;
      String? seenAccountId;
      final manager = WalletManager();
      final created = await manager.createWallet();
      // 建钱包阶段一次都不该调 registrar；绑定只在进入需 CID 页面时由门禁触发。
      WalletManager.subkeyRegistrar = ({
        required int walletIndex,
        required String cidNumber,
        required int bindingRevision,
        required String accountId,
        required Future<String> Function(Uint8List bindingMessage) signBinding,
      }) async {
        seenCidNumber = cidNumber;
        seenBindingRevision = bindingRevision;
        seenAccountId = accountId;
        final signature = await signBinding(Uint8List(32));
        expect(signature.startsWith('0x'), isTrue);
      };
      await manager.bindDeviceSubkeyToCurrentBinding(
        cidNumber: 'CN220-CTZN2-198805200-2026',
        bindingRevision: 1,
        accountId: created.profile.accountId,
      );
      expect(seenCidNumber, 'CN220-CTZN2-198805200-2026');
      expect(seenBindingRevision, 1);
      expect(seenAccountId, created.profile.accountId);
    });
  });

  group('WalletManager — 统一签名', () {
    final payload = Uint8List.fromList(List<int>.generate(32, (i) => i));

    test('统一签名：每次都读一次 child（无会话缓存）', () async {
      final manager = WalletManager();
      await manager.importWallet(_mnemonicA);
      fakeStore.readCount = 0;

      final sig = await manager.signWithWallet(1, payload);
      await manager.signWithWallet(1, payload);

      expect(sig.length, 64);
      // 两次签名 = 两次读 child（两次验证），不复用、无会话密钥。
      expect(fakeStore.readCount, 2);
    });

    test('verifyWalletAccess 读一次 child 触发一次验证（切换身份用）', () async {
      final manager = WalletManager();
      await manager.importWallet(_mnemonicA);
      fakeStore.readCount = 0;

      await manager.verifyWalletAccess(1);

      expect(fakeStore.readCount, 1);
    });

    test('AuthCancelled 上抛，绝不吞没', () async {
      final manager = WalletManager();
      final imported = await manager.importWallet(_mnemonicA);
      fakeStore.putCount = 0;
      fakeStore.cancelReads.add(imported.accountId);

      await expectLater(
        manager.signWithWallet(1, payload),
        throwsA(isA<AuthCancelled>()),
      );
      // 无根 = 无自愈重写。
      expect(fakeStore.putCount, 0);
    });
  });

  group('WalletManager — 设备私钥失效 fail-closed', () {
    final payload = Uint8List.fromList(List<int>.generate(32, (_) => 7));

    test('KEK 失效 → 只报告设备安全存储异常，绝不自动重写 child', () async {
      final manager = WalletManager();
      final imported = await manager.importWallet(_mnemonicA);
      fakeStore.putCount = 0;
      fakeStore.invalidatedAccountIds.add(imported.accountId);

      await expectLater(
        manager.signWithWallet(1, payload),
        throwsA(
          isA<WalletAuthException>().having(
            (e) => e.message,
            'message',
            contains('设备安全存储'),
          ),
        ),
      );
      // App 不持久化母种子 / 助记词，查看或签名流程绝不重派生、重写。
      expect(fakeStore.putCount, 0);
    });

    test('child 条目缺失 → 只报告设备安全存储中没有账户私钥', () async {
      final manager = WalletManager();
      final imported = await manager.importWallet(_mnemonicA);
      fakeStore.accountKeys.remove(imported.accountId);

      await expectLater(
        manager.signWithWallet(1, payload),
        throwsA(
          isA<WalletAuthException>().having(
            (e) => e.message,
            'message',
            contains('没有该账户私钥'),
          ),
        ),
      );
    });

    test('另一钱包导入后也只从严档读取账户 child，不保存助记词', () async {
      final manager = WalletManager();
      final imported = await manager.importWallet(_mnemonicB);
      final key = fakeStore.accountKeys[imported.accountId];
      expect(key, isNotNull);
      await _expectChildStoredNotSeed(_mnemonicB, key!);
    });
  });

  group('WalletManager — 冷钱包', () {
    test('importColdWallet 只存公开账户资料，child 金库无条目', () async {
      final manager = WalletManager();
      final cold = await manager.importColdWallet(ss58Address: _coldSs58(0x11));
      expect(cold.signMode, 'external');
      expect(fakeStore.accountKeys.containsKey(cold.accountId), isFalse);
    });

    test('deleteWallet 冷钱包不影响 child 金库', () async {
      final manager = WalletManager();
      final cold = await manager.importColdWallet(ss58Address: _coldSs58(0x11));
      await manager.deleteWallet(cold.walletIndex);
      final wallets = await manager.getWallets();
      expect(wallets.where((w) => w.walletIndex == cold.walletIndex), isEmpty);
      expect(deviceSubkey.deletedWalletIndexes, isEmpty);
    });
  });
}
