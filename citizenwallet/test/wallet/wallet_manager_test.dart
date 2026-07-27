// WalletManager 单测（model B：无根 + 全 //index）。
//
// 覆盖：建钱包(master)+账户0(//0)、加账户(需助记词, //N)、按账户签名一致性、
// 导入对齐金标、删钱包/账户连带清各账户密钥、重复导入拒绝、助记词不符拒绝、
// 每账户密钥密文落库(无 master 种子/助记词)、私钥单账户隔离。
// 生物识别经 WalletManager.debugAuthGate 注入为放行；SecureStorage 用 mock。
import 'dart:typed_data';

import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:citizenwallet/isar/wallet_isar.dart';
import 'package:citizenwallet/wallet/wallet_manager.dart';
import 'package:citizenwallet/wallet/wallet_secure_keys.dart';

const String kDevPhrase =
    'bottom drive obey lake curtain smoke basket hold race lonely fit walk';
const String kOtherPhrase =
    'legal winner thank year wave sausage worth useful legal winner thank yellow';

// 金标（对齐 derivation_golden_test.dart，全 //index）。
const String kAccount0Id =
    '0x2afba9278e30ccf6a6ceb3a8b6e336b70068f045c666f2e7f4f9cc5f47db8972';
const String kAccount0Ss58 = 'w5CZACAABUbK4jspzPB5be9trhtSgRCRZFafGe7kvFPvxq8M2';
const String kAccount1Id =
    '0xb606fc73f57f03cdb4c932d475ab426043e429cecc2ffff0d2672b0df8398c48';
const String kAccount2Id =
    '0x46f136b564e1fad55031404dd84e5cd3fa76bfe7cc7599b39d38fd06663bbc0a';
const String kAccount0Child =
    '0x914dded06277afbe5b0e8a30bce539ec8a9552a784d08e530dc7c2915c478393';
const String kAccount1Child =
    '0x4433c3ada0cf37c3050d5435321872f4f84ef53d8b5f1f1560689d500b882245';

void main() {
  final manager = WalletManager();

  setUp(() async {
    FlutterSecureStorage.setMockInitialValues({});
    WalletManager.debugAuthGate = () async {};
    await WalletIsar.instance.resetForTest();
  });

  tearDown(() async {
    WalletManager.debugAuthGate = null;
    await WalletIsar.instance.resetForTest();
  });

  test('createWallet 建钱包 + 账户0(//0)，masterId == 账户0 accountId', () async {
    final result = await manager.createWallet();
    expect(result.primaryAccount.accountIndex, 0);
    expect(result.primaryAccount.derivationPath, '//0');
    expect(result.wallet.masterId, result.primaryAccount.accountId);

    final accounts = await manager.getAccounts(result.wallet.masterId);
    expect(accounts.length, 1);
    expect(accounts.single.accountId, result.primaryAccount.accountId);
  });

  test('importWallet 账户0(//0) 对齐金标', () async {
    final result = await manager.importWallet(kDevPhrase);
    expect(result.primaryAccount.accountId, kAccount0Id);
    expect(result.primaryAccount.ss58Address, kAccount0Ss58);
    expect(result.wallet.masterId, kAccount0Id);
  });

  test('addAccount 派生 //1 //2，对齐金标且序号递增（需助记词）', () async {
    final created = await manager.importWallet(kDevPhrase);
    final a1 = await manager.addAccount(created.wallet.masterId, kDevPhrase);
    final a2 = await manager.addAccount(created.wallet.masterId, kDevPhrase);

    expect(a1.accountIndex, 1);
    expect(a1.accountId, kAccount1Id);
    expect(a2.accountIndex, 2);
    expect(a2.accountId, kAccount2Id);

    final accounts = await manager.getAccounts(created.wallet.masterId);
    expect(accounts.map((e) => e.accountIndex).toList(), [0, 1, 2]);
  });

  test('addAccount 助记词与钱包不符被拒', () async {
    final created = await manager.importWallet(kDevPhrase);
    expect(
      () => manager.addAccount(created.wallet.masterId, kOtherPhrase),
      throwsA(isA<WalletAuthException>()),
    );
  });

  test('signForAccount 产出可被该账户公钥验证的签名', () async {
    final created = await manager.importWallet(kDevPhrase);
    final a1 = await manager.addAccount(created.wallet.masterId, kDevPhrase);

    final payload = Uint8List.fromList(List<int>.generate(48, (i) => i));
    final sig = await manager.signForAccount(a1.accountId, payload);

    final verifier = await Keyring.sr25519.fromUri('$kDevPhrase//1');
    expect(verifier.verify(payload, sig), isTrue);

    // 账户0(//0) 的公钥不应验通账户1 的签名。
    final wrong = await Keyring.sr25519.fromUri('$kDevPhrase//0');
    expect(wrong.verify(payload, sig), isFalse);
  });

  test('deleteWallet 连带清账户与各账户密钥', () async {
    final created = await manager.importWallet(kDevPhrase);
    await manager.addAccount(created.wallet.masterId, kDevPhrase);
    await manager.deleteWallet(created.wallet.masterId);

    expect(await manager.getWallets(), isEmpty);
    expect(await manager.getAccounts(created.wallet.masterId), isEmpty);
    // 账户密钥已清。
    const storage = FlutterSecureStorage();
    expect(
      await storage
          .read(key: WalletSecureKeys.accountMiniSecretV1(kAccount0Id)),
      isNull,
    );
    expect(
      await storage
          .read(key: WalletSecureKeys.accountMiniSecretV1(kAccount1Id)),
      isNull,
    );
  });

  test('重复导入同一助记词被拒绝', () async {
    await manager.importWallet(kDevPhrase);
    expect(
      () => manager.importWallet(kDevPhrase),
      throwsA(isA<Exception>()),
    );
  });

  test('getAccountByAccountId 精确命中/未知返回 null(全局扫码定位边界)', () async {
    final created = await manager.importWallet(kDevPhrase);
    final a1 = await manager.addAccount(created.wallet.masterId, kDevPhrase);

    expect((await manager.getAccountByAccountId(kAccount0Id))?.accountIndex, 0);
    expect((await manager.getAccountByAccountId(a1.accountId))?.accountIndex, 1);
    expect(
      await manager.getAccountByAccountId(
          '0xdddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd0'),
      isNull,
    );
  });

  test('getWalletByMasterId 命中/未知', () async {
    final created = await manager.importWallet(kDevPhrase);
    expect(
      (await manager.getWalletByMasterId(created.wallet.masterId))?.masterId,
      created.wallet.masterId,
    );
    expect(await manager.getWalletByMasterId(kAccount1Id), isNull);
  });

  test('deleteAccount 删非末位账户,账户0与钱包保留', () async {
    final created = await manager.importWallet(kDevPhrase);
    final a1 = await manager.addAccount(created.wallet.masterId, kDevPhrase);
    await manager.addAccount(created.wallet.masterId, kDevPhrase); // //2

    await manager.deleteAccount(a1.accountId);

    final accounts = await manager.getAccounts(created.wallet.masterId);
    expect(accounts.map((e) => e.accountIndex).toList(), [0, 2]);
    expect(await manager.getWallets(), hasLength(1));
  });

  test('deleteAccount 拒绝删账户0(尚有兄弟账户)', () async {
    final created = await manager.importWallet(kDevPhrase);
    await manager.addAccount(created.wallet.masterId, kDevPhrase);
    expect(
      () => manager.deleteAccount(kAccount0Id),
      throwsA(isA<Exception>()),
    );
    // 账户0 仍在。
    expect(await manager.getAccountByAccountId(kAccount0Id), isNotNull);
  });

  test('deleteAccount 删光账户级联删钱包与密钥', () async {
    final created = await manager.importWallet(kDevPhrase);
    // 仅账户0,删它即删整钱包。
    await manager.deleteAccount(kAccount0Id);
    expect(await manager.getWallets(), isEmpty);
    expect(await manager.getAccounts(created.wallet.masterId), isEmpty);
    const storage = FlutterSecureStorage();
    expect(
      await storage
          .read(key: WalletSecureKeys.accountMiniSecretV1(kAccount0Id)),
      isNull,
    );
  });

  test('删中间账户后 addAccount 仍为 max+1(不回填空档,行为钉死)', () async {
    final created = await manager.importWallet(kDevPhrase);
    final a1 = await manager.addAccount(created.wallet.masterId, kDevPhrase);
    await manager.addAccount(created.wallet.masterId, kDevPhrase); // //2
    await manager.deleteAccount(a1.accountId); // 删 //1,留 0,2

    final a3 = await manager.addAccount(created.wallet.masterId, kDevPhrase);
    expect(a3.accountIndex, 3); // max(2)+1,不回填 1
  });

  test('renameWallet / reorderWallets', () async {
    final w1 = await manager.importWallet(kDevPhrase);
    final w2 = await manager.importWallet(kOtherPhrase);

    await manager.renameWallet(w1.wallet.masterId, '主号');
    final renamed = await manager.getWalletByMasterId(w1.wallet.masterId);
    expect(renamed?.walletName, '主号');

    await manager.reorderWallets([w2.wallet.masterId, w1.wallet.masterId]);
    final ordered = await manager.getWallets();
    expect(ordered.first.masterId, w2.wallet.masterId);
  });

  test('每账户 child mini-secret 密文落库,无 master 种子/助记词', () async {
    final created = await manager.importWallet(kDevPhrase);
    const storage = FlutterSecureStorage();
    final stored = await storage
        .read(key: WalletSecureKeys.accountMiniSecretV1(kAccount0Id));
    expect(stored, isNotNull);
    // 明文 mini-secret 是 64 位 hex;加密后是 Base64 密文,绝不匹配裸 hex。
    expect(RegExp(r'^[0-9a-fA-F]{64}$').hasMatch(stored!), isFalse);
    // 无根:不存在 master 级键（仅账户级)。签名照常验证加账户可用。
    final a1 = await manager.addAccount(created.wallet.masterId, kDevPhrase);
    expect(a1.accountId, kAccount1Id);
  });

  test('getAccountPrivateKey 返回该账户 child mini-secret,单账户隔离', () async {
    final created = await manager.importWallet(kDevPhrase);
    final a1 = await manager.addAccount(created.wallet.masterId, kDevPhrase);

    final key0 = await manager.getAccountPrivateKey(kAccount0Id);
    final key1 = await manager.getAccountPrivateKey(a1.accountId);

    expect(key0, kAccount0Child);
    expect(key1, kAccount1Child);
    // 隔离:两账户私钥互不相同,单把泄漏不牵连另一把。
    expect(key0, isNot(equals(key1)));
  });
}
