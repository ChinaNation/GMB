import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:substrate_bip39/substrate_bip39.dart';
import 'package:citizenapp/isar/app_isar.dart';
import 'package:citizenapp/security/local_cipher.dart';
import 'package:citizenapp/security/local_data_key.dart';
import 'package:citizenapp/wallet/core/secure_seed_store.dart';
import 'package:citizenapp/wallet/core/hardware_bound_seed_vault.dart';
import 'package:citizenapp/wallet/core/device_subkey.dart';
import 'package:citizenapp/wallet/core/device_data_key_vault.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:citizenapp/wallet/core/native_sr25519.dart';

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

const _genesisHash =
    '0x1111111111111111111111111111111111111111111111111111111111111111';

Future<void> _activateAccountDataBinding(
  WalletManager manager, {
  required String cidNumber,
  required int bindingRevision,
  required String accountId,
}) async {
  await manager.activateAccountDataBinding(
    genesisHash: _genesisHash,
    cidNumber: cidNumber,
    bindingRevision: bindingRevision,
    accountId: accountId,
  );
}

/// 助记词 → 母种子（master mini-secret，32B）。
Future<Uint8List> _masterSeed(String mnemonic) async {
  final entropy = Mnemonic.fromSentence(mnemonic, Language.english).entropy;
  return Uint8List.fromList(await CryptoScheme.miniSecretFromEntropy(entropy));
}

/// 复现 WalletManager 的账户 child mini-secret 派生（金标同源）。
List<int> _childMiniSecret(List<int> seed, int index) {
  // 与生产同一条原生路径(NativeSr25519),测试不另立第二套实现。
  final junctions = SecretUri.fromStr('//$index').junctions;
  var current = List<int>.from(seed);
  late List<int> child;
  for (final j in junctions) {
    child = NativeSr25519.deriveHard(current, j.junctionId.sublist(0, 32));
    current = List<int>.from(child);
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
  Future<String> publicKeyHex(int walletIndex) async =>
      '04${List<String>.filled(64, '11').join()}';

  @override
  Future<void> delete(int walletIndex) async {
    deletedWalletIndexes.add(walletIndex);
    if (failingWalletIndexes.contains(walletIndex)) {
      throw StateError('设备子钥删除失败');
    }
  }
}

/// 纯内存设备数据钥金库。测试验证钱包层状态机，不依赖原生 Keystore/SE 通道。
class _MemoryDeviceDataKeyVault extends DeviceDataKeyVault {
  final Map<int, Map<String, Uint8List>> values =
      <int, Map<String, Uint8List>>{};
  final List<int> deletedWalletIndexes = <int>[];
  final Set<int> failingWalletIndexes = <int>{};
  int sealCount = 0;
  int openCount = 0;
  int? failSealAt;

  String _aadKey(Uint8List aad) => _hex(aad);

  @override
  Future<String> seal({
    required int walletIndex,
    required Uint8List plaintext,
    required Uint8List aad,
  }) async {
    sealCount++;
    if (sealCount == failSealAt) {
      throw const DeviceDataKeyVaultException('测试设备数据钥封装失败');
    }
    values.putIfAbsent(walletIndex, () => <String, Uint8List>{})[_aadKey(aad)] =
        Uint8List.fromList(plaintext);
    return 'sealed:$walletIndex:${_aadKey(aad)}';
  }

  @override
  Future<Uint8List> open({
    required int walletIndex,
    required String blob,
    required Uint8List aad,
  }) async {
    openCount++;
    final value = values[walletIndex]?[_aadKey(aad)];
    if (value == null) {
      throw const DeviceDataKeyVaultException('测试设备数据钥不存在');
    }
    return Uint8List.fromList(value);
  }

  @override
  Future<void> delete(int walletIndex) async {
    deletedWalletIndexes.add(walletIndex);
    if (failingWalletIndexes.contains(walletIndex)) {
      throw const DeviceDataKeyVaultException('测试设备数据钥删除失败');
    }
    values.remove(walletIndex);
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
  late _MemoryDeviceDataKeyVault deviceDataKeyVault;

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
    deviceDataKeyVault = _MemoryDeviceDataKeyVault();
    WalletManager.debugDeviceDataKeyVault = deviceDataKeyVault;
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
    test('通讯录已有用途钥静默读取；实际缺钥时只鉴权一次生成', () async {
      final manager = WalletManager();
      final created = await manager.importWallet(_mnemonicA);
      final accountId = created.accountId;
      const cidNumber = 'GD-CTZN1-TEST';
      await _activateAccountDataBinding(
        manager,
        cidNumber: cidNumber,
        bindingRevision: 1,
        accountId: accountId,
      );

      fakeStore.readCount = 0;
      final material =
          await manager.ensureContactKeyMaterialForAccountId(accountId);
      expect(material.encryptionKey, hasLength(32));
      expect(material.indexKey, hasLength(32));
      expect(material.encryptionKey, isNot(material.indexKey));
      expect(fakeStore.readCount, 1, reason: '真实缺钥只允许读取一次账户 child');

      final second =
          await manager.ensureContactKeyMaterialForAccountId(accountId);
      expect(second.encryptionKey, material.encryptionKey);
      expect(second.indexKey, material.indexKey);
      expect(fakeStore.readCount, 1, reason: '已有用途钥必须直接静默使用');
      expect(deviceDataKeyVault.sealCount, 7);
      expect(deviceDataKeyVault.openCount, 4);
    });

    test('设备数据钥实际丢失时鉴权一次重建，后续静默读取', () async {
      final manager = WalletManager();
      final created = await manager.importWallet(_mnemonicA);
      final accountId = created.accountId;
      await _activateAccountDataBinding(
        manager,
        cidNumber: 'GD-CTZN1-TEST',
        bindingRevision: 1,
        accountId: accountId,
      );

      final first =
          await manager.ensureContactKeyMaterialForAccountId(accountId);
      first.dispose();
      fakeStore.readCount = 0;
      // 模拟 Keystore / Secure Enclave 中的设备数据钥真实丢失，
      // 但公开绑定和密文仍在；这才是允许读取账户 child 的鉴权场景。
      deviceDataKeyVault.values.clear();

      final rebuilt =
          await manager.ensureContactKeyMaterialForAccountId(accountId);
      expect(rebuilt.encryptionKey, hasLength(32));
      expect(rebuilt.indexKey, hasLength(32));
      expect(fakeStore.readCount, 1, reason: '硬件数据钥真实丢失只允许鉴权一次');

      final silent =
          await manager.ensureContactKeyMaterialForAccountId(accountId);
      expect(silent.encryptionKey, rebuilt.encryptionKey);
      expect(silent.indexKey, rebuilt.indexKey);
      expect(fakeStore.readCount, 1, reason: '重建后必须恢复设备金库静默读取');
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
      expect(
        deviceDataKeyVault.deletedWalletIndexes,
        [created.profile.walletIndex],
      );

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
      expect(
        deviceDataKeyVault.deletedWalletIndexes,
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
      expect(
        await manager.walletIndexForAccountId(account1.accountId),
        wallet.walletIndex,
        reason: '设备子钥必须按当前 account_id 定位其实际所属热钱包',
      );

      await _activateAccountDataBinding(
        manager,
        cidNumber: 'GD-CTZN1-TEST',
        bindingRevision: 1,
        accountId: account1.accountId,
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
      expect(deviceDataKeyVault.deletedWalletIndexes, isEmpty);
      expect(fakeStore.accountKeys.containsKey(account1.accountId), isFalse);
      expect(
        await WalletIsar.instance.read(
          (isar) => isar.appKvEntitys.getByKey(contactKey),
        ),
        isNotNull,
        reason: '删除换绑后的此前账户不得删除永久 CID 的本机数据',
      );

      await manager.deleteWallet(wallet.walletIndex);
      expect(deviceSubkey.deletedWalletIndexes, [wallet.walletIndex]);
      expect(deviceDataKeyVault.deletedWalletIndexes, [wallet.walletIndex]);
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
      await _activateAccountDataBinding(
        manager,
        cidNumber: 'GD-CTZN1-TEST',
        bindingRevision: 1,
        accountId: hot.accountId,
      );

      await manager.clearWallet();

      expect(await manager.getWallets(), isEmpty);
      expect(fakeStore.accountKeys, isEmpty);
      expect(contactBlobStore.values, isEmpty);
      expect(deviceSubkey.deletedWalletIndexes, [hot.walletIndex]);
      expect(deviceDataKeyVault.deletedWalletIndexes, [hot.walletIndex]);
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
      deviceDataKeyVault.failingWalletIndexes.add(wallet.walletIndex);

      await expectLater(
        manager.deleteWallet(wallet.walletIndex),
        throwsA(
          isA<WalletLocalCleanupException>().having(
            (error) => error.failures.join('\n'),
            'failures',
            allOf(
              contains('账户 child 删除失败'),
              contains('设备子钥删除失败'),
              contains('设备数据钥删除失败'),
            ),
          ),
        ),
      );

      expect(await manager.getWallets(), isEmpty);
      expect(failingStore.deletedWalletKeyIndexes, [wallet.walletIndex]);
      expect(deviceSubkey.deletedWalletIndexes, [wallet.walletIndex]);
      expect(deviceDataKeyVault.deletedWalletIndexes, [wallet.walletIndex]);
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

    test('换绑到不同钱包后新账户不能直接解密此前账户历史私有密文', () async {
      const cidNumber = 'CN220-CTZN2-198805200-2026';
      final aad = '${LocalKeyPurpose.chat.domain}|message-before-rebind';
      final manager = WalletManager();
      final walletA = await manager.importWallet(_mnemonicA);
      await _activateAccountDataBinding(
        manager,
        cidNumber: cidNumber,
        bindingRevision: 1,
        accountId: walletA.accountId,
      );
      final bindingA =
          await manager.accountDataBindingForAccountId(walletA.accountId);
      await manager.ensureDeviceDataKeysForBinding(bindingA);
      final keyA = await manager.readDataKeyForCurrentBinding(
        walletA.accountId,
        LocalKeyPurpose.chat,
      );
      final oldCiphertext = await LocalCipher.encryptString(
        key: keyA,
        plaintext: 'A 钱包时期的 CID 私有数据',
        aad: aad,
      );

      // 删除 A 整只热钱包后只剩此前密文；系统没有可供 B 领取的额外数据密钥。
      await manager.deleteWallet(walletA.walletIndex);
      expect(fakeStore.accountKeys, isEmpty);
      expect(
        contactBlobStore.values[AccountDataBindingStore.activeBindingKey],
        isNull,
      );

      final walletB = await manager.importWallet(_mnemonicB);
      expect(walletB.accountId, isNot(walletA.accountId));
      await _activateAccountDataBinding(
        manager,
        cidNumber: cidNumber,
        bindingRevision: 2,
        accountId: walletB.accountId,
      );
      final bindingB =
          await manager.accountDataBindingForAccountId(walletB.accountId);
      await manager.ensureDeviceDataKeysForBinding(bindingB);
      final keyB = await manager.readDataKeyForCurrentBinding(
        walletB.accountId,
        LocalKeyPurpose.chat,
      );
      expect(keyB, isNot(keyA));
      await expectLater(
        LocalCipher.decryptString(
          key: keyB,
          blob: oldCiphertext,
          aad: aad,
        ),
        throwsA(isA<LocalCipherException>()),
      );
      expect(fakeStore.accountKeys.containsKey(walletA.accountId), isFalse);
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

  group('实际缺钥一次生成：页面门禁不参与', () {
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

    test('真实通讯录数据缺钥只生成本地数据钥，登记后端失败也不受影响', () async {
      WalletManager.subkeyRegistrar = failingRegistrar;
      final manager = WalletManager();
      final profile = await manager.importWallet(_mnemonicA);
      await _activateAccountDataBinding(
        manager,
        cidNumber: 'CN220-CTZN2-198805200-2026',
        bindingRevision: 1,
        accountId: profile.accountId,
      );
      fakeStore.readCount = 0;

      final material =
          await manager.ensureContactKeyMaterialForAccountId(profile.accountId);

      expect(material.encryptionKey, hasLength(32));
      expect(material.indexKey, hasLength(32));
      expect(fakeStore.readCount, 1);
      expect(deviceDataKeyVault.sealCount, 7);
    });

    test('Worker 确认未登记后才按 CID 当前账户签 P-256 绑定证明', () async {
      String? seenCidNumber;
      int? seenBindingRevision;
      String? seenAccountId;
      final manager = WalletManager();
      final created = await manager.createWallet();
      // 建钱包、页面门禁和数据钥生成都不调用 registrar；只有 Worker 确认设备未登记
      // 后才进入远端登记入口。
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
      await _activateAccountDataBinding(
        manager,
        cidNumber: 'CN220-CTZN2-198805200-2026',
        bindingRevision: 1,
        accountId: created.profile.accountId,
      );
      final binding = await manager.accountDataBindingForAccountId(
        created.profile.accountId,
      );
      fakeStore.readCount = 0;
      expect(fakeStore.readCount, 0);
      await manager.registerDeviceSubkeyForBinding(binding);
      expect(seenCidNumber, 'CN220-CTZN2-198805200-2026');
      expect(seenBindingRevision, 1);
      expect(seenAccountId, created.profile.accountId);
      expect(fakeStore.readCount, 1);
    });

    test('本地数据钥并发生成全局去重，只读取一次 child，且绝不登记 P-256', () async {
      var registrations = 0;
      final manager = WalletManager();
      final created = await manager.createWallet();
      WalletManager.subkeyRegistrar = ({
        required int walletIndex,
        required String cidNumber,
        required int bindingRevision,
        required String accountId,
        required Future<String> Function(Uint8List bindingMessage) signBinding,
      }) async {
        registrations++;
        await signBinding(Uint8List(32));
      };
      await _activateAccountDataBinding(
        manager,
        cidNumber: 'CN220-CTZN2-198805200-2026',
        bindingRevision: 1,
        accountId: created.profile.accountId,
      );
      final binding = await manager.accountDataBindingForAccountId(
        created.profile.accountId,
      );
      fakeStore.readCount = 0;

      await Future.wait(<Future<void>>[
        WalletManager().ensureDeviceDataKeysForBinding(binding),
        WalletManager().ensureDeviceDataKeysForBinding(binding),
        manager.ensureDeviceDataKeysForBinding(binding),
      ]);
      expect(registrations, 0);
      expect(fakeStore.readCount, 1);
      expect(deviceDataKeyVault.sealCount, 7);

      await manager.ensureDeviceDataKeysForBinding(binding);
      expect(registrations, 0);
      expect(fakeStore.readCount, 1, reason: '已有数据钥的相同账户不得再次读取 child');
    });

    test('P-256 登记拥有独立全局并发去重，不生成本地设备数据钥', () async {
      var registrations = 0;
      final manager = WalletManager();
      final created = await manager.createWallet();
      WalletManager.subkeyRegistrar = ({
        required int walletIndex,
        required String cidNumber,
        required int bindingRevision,
        required String accountId,
        required Future<String> Function(Uint8List bindingMessage) signBinding,
      }) async {
        registrations++;
        await signBinding(Uint8List(32));
      };
      await _activateAccountDataBinding(
        manager,
        cidNumber: 'CN220-CTZN2-198805200-2026',
        bindingRevision: 1,
        accountId: created.profile.accountId,
      );
      final binding = await manager.accountDataBindingForAccountId(
        created.profile.accountId,
      );
      fakeStore.readCount = 0;

      await Future.wait(<Future<void>>[
        WalletManager().registerDeviceSubkeyForBinding(binding),
        WalletManager().registerDeviceSubkeyForBinding(binding),
        manager.registerDeviceSubkeyForBinding(binding),
      ]);

      expect(registrations, 1);
      expect(fakeStore.readCount, 1);
      expect(deviceDataKeyVault.sealCount, 0);
    });

    test('本地数据钥只补缺少的用途，不覆盖已经存在的用途密文', () async {
      final manager = WalletManager();
      final created = await manager.createWallet();
      await _activateAccountDataBinding(
        manager,
        cidNumber: 'CN220-CTZN2-198805200-2026',
        bindingRevision: 1,
        accountId: created.profile.accountId,
      );
      final binding = await manager.accountDataBindingForAccountId(
        created.profile.accountId,
      );
      await manager.ensureDeviceDataKeysForBinding(binding);
      final dataBlobNames = contactBlobStore.values.keys
          .where((key) => key.startsWith('citizenapp_device_data_key_'))
          .toList(growable: false);
      expect(dataBlobNames, hasLength(7));
      final retained = <String, String>{
        for (final name in dataBlobNames.skip(1))
          name: contactBlobStore.values[name]!,
      };
      await contactBlobStore.delete(dataBlobNames.first);
      fakeStore.readCount = 0;

      await manager.ensureDeviceDataKeysForBinding(binding);

      expect(fakeStore.readCount, 1);
      expect(deviceDataKeyVault.sealCount, 8, reason: '只允许补封装缺少的一把用途钥');
      for (final entry in retained.entries) {
        expect(contactBlobStore.values[entry.key], entry.value);
      }
    });

    test('本地数据钥生成失败只回滚本次数据 blob，不删除 P-256 登记标记', () async {
      var registrations = 0;
      final manager = WalletManager();
      final created = await manager.createWallet();
      WalletManager.subkeyRegistrar = ({
        required int walletIndex,
        required String cidNumber,
        required int bindingRevision,
        required String accountId,
        required Future<String> Function(Uint8List bindingMessage) signBinding,
      }) async {
        registrations++;
        await signBinding(Uint8List(32));
      };
      await _activateAccountDataBinding(
        manager,
        cidNumber: 'CN220-CTZN2-198805200-2026',
        bindingRevision: 1,
        accountId: created.profile.accountId,
      );
      final binding = await manager.accountDataBindingForAccountId(
        created.profile.accountId,
      );
      await manager.registerDeviceSubkeyForBinding(binding);
      final markerNames = contactBlobStore.values.keys
          .where((key) => key.startsWith('citizenapp_cid_subkey_bound_'))
          .toList(growable: false);
      expect(markerNames, hasLength(1));
      deviceDataKeyVault.failSealAt = 3;

      await expectLater(
        manager.ensureDeviceDataKeysForBinding(binding),
        throwsA(isA<DeviceDataKeyVaultException>()),
      );

      expect(registrations, 1);
      expect(contactBlobStore.values[markerNames.single], '1');
      expect(
        contactBlobStore.values.keys.where(
          (key) => key.startsWith('citizenapp_device_data_key_'),
        ),
        isEmpty,
      );
    });

    test('相同 account_id 不得用新 revision 伪装换绑，两类入口拒绝前均不读 child', () async {
      final manager = WalletManager();
      final created = await manager.createWallet();
      await _activateAccountDataBinding(
        manager,
        cidNumber: 'CN220-CTZN2-198805200-2026',
        bindingRevision: 1,
        accountId: created.profile.accountId,
      );
      fakeStore.readCount = 0;
      final invalid = AccountDataBinding(
        genesisHash: _genesisHash,
        cidNumber: 'CN220-CTZN2-198805200-2026',
        bindingRevision: 2,
        accountId: created.profile.accountId,
      );

      await expectLater(
        manager.ensureDeviceDataKeysForBinding(invalid),
        throwsA(isA<WalletAuthException>()),
      );
      await expectLater(
        manager.registerDeviceSubkeyForBinding(invalid),
        throwsA(isA<WalletAuthException>()),
      );
      expect(fakeStore.readCount, 0);
    });

    test('P-256 登记失败不删除已生成的数据钥，重试也不重新封装数据钥', () async {
      var registrations = 0;
      final manager = WalletManager();
      final created = await manager.createWallet();
      WalletManager.subkeyRegistrar = ({
        required int walletIndex,
        required String cidNumber,
        required int bindingRevision,
        required String accountId,
        required Future<String> Function(Uint8List bindingMessage) signBinding,
      }) async {
        registrations++;
        if (registrations == 1) throw Exception('后端登记失败');
      };
      await _activateAccountDataBinding(
        manager,
        cidNumber: 'CN220-CTZN2-198805200-2026',
        bindingRevision: 1,
        accountId: created.profile.accountId,
      );
      final binding = await manager.accountDataBindingForAccountId(
        created.profile.accountId,
      );
      await manager.ensureDeviceDataKeysForBinding(binding);
      final dataBlobs = Map<String, String>.fromEntries(
        contactBlobStore.values.entries.where(
          (entry) => entry.key.startsWith('citizenapp_device_data_key_'),
        ),
      );
      expect(dataBlobs, hasLength(7));
      expect(deviceDataKeyVault.sealCount, 7);
      fakeStore.readCount = 0;

      await expectLater(
        manager.registerDeviceSubkeyForBinding(binding),
        throwsA(isA<Exception>()),
      );
      expect(
        Map<String, String>.fromEntries(
          contactBlobStore.values.entries.where(
            (entry) => entry.key.startsWith('citizenapp_device_data_key_'),
          ),
        ),
        dataBlobs,
      );
      await manager.registerDeviceSubkeyForBinding(binding);
      expect(registrations, 2);
      expect(fakeStore.readCount, 2, reason: '两次远端登记尝试各鉴权一次');
      expect(deviceDataKeyVault.sealCount, 7, reason: '登记失败与重试不得碰数据钥');
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
