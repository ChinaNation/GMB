// WalletManager HD 两级模型单测（S1.1）。
//
// 覆盖：建钱包(master)+账户0、加账户(//N)、按账户签名一致性、导入对齐金标、
// 删钱包连带清账户+密钥、重复导入拒绝。
// 生物识别经 WalletManager.debugAuthGate 注入为放行；SecureStorage 用 mock。
import 'dart:typed_data';

import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:citizenwallet/isar/wallet_isar.dart';
import 'package:citizenwallet/wallet/wallet_manager.dart';

const String kDevPhrase =
    'bottom drive obey lake curtain smoke basket hold race lonely fit walk';

// 金标（对齐 derivation_golden_test.dart）。
const String kAccount0Id =
    '0x46ebddef8cd9bb167dc30878d7113b7e168e6f0646beffd77d69d39bad76b47a';
const String kAccount0Ss58 = 'w5DBnqoUytARopdnyWhmBq7ZPr74cJJewugoafJJynKLrirdE';
const String kAccount1Id =
    '0xb606fc73f57f03cdb4c932d475ab426043e429cecc2ffff0d2672b0df8398c48';
const String kAccount2Id =
    '0x46f136b564e1fad55031404dd84e5cd3fa76bfe7cc7599b39d38fd06663bbc0a';

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

  test('createWallet 建钱包 + 账户0，masterId == 账户0 accountId', () async {
    final result = await manager.createWallet();
    expect(result.primaryAccount.accountIndex, 0);
    expect(result.wallet.masterId, result.primaryAccount.accountId);

    final accounts = await manager.getAccounts(result.wallet.masterId);
    expect(accounts.length, 1);
    expect(accounts.single.accountId, result.primaryAccount.accountId);
  });

  test('importWallet 账户0 对齐金标', () async {
    final result = await manager.importWallet(kDevPhrase);
    expect(result.primaryAccount.accountId, kAccount0Id);
    expect(result.primaryAccount.ss58Address, kAccount0Ss58);
    expect(result.wallet.masterId, kAccount0Id);
  });

  test('addAccount 派生 //1 //2，对齐金标且序号递增', () async {
    final created = await manager.importWallet(kDevPhrase);
    final a1 = await manager.addAccount(created.wallet.masterId);
    final a2 = await manager.addAccount(created.wallet.masterId);

    expect(a1.accountIndex, 1);
    expect(a1.accountId, kAccount1Id);
    expect(a2.accountIndex, 2);
    expect(a2.accountId, kAccount2Id);

    final accounts = await manager.getAccounts(created.wallet.masterId);
    expect(accounts.map((e) => e.accountIndex).toList(), [0, 1, 2]);
  });

  test('signForAccount 产出可被该账户公钥验证的签名', () async {
    final created = await manager.importWallet(kDevPhrase);
    final a1 = await manager.addAccount(created.wallet.masterId);

    final payload = Uint8List.fromList(List<int>.generate(48, (i) => i));
    final sig = await manager.signForAccount(a1.accountId, payload);

    final verifier = await Keyring.sr25519.fromUri('$kDevPhrase//1');
    expect(verifier.verify(payload, sig), isTrue);

    // 账户0 的公钥不应验通账户1 的签名。
    final wrong = await Keyring.sr25519.fromUri(kDevPhrase);
    expect(wrong.verify(payload, sig), isFalse);
  });

  test('deleteWallet 连带清账户与密钥', () async {
    final created = await manager.importWallet(kDevPhrase);
    await manager.addAccount(created.wallet.masterId);
    await manager.deleteWallet(created.wallet.masterId);

    expect(await manager.getWallets(), isEmpty);
    expect(await manager.getAccounts(created.wallet.masterId), isEmpty);
    expect(await manager.getMasterMnemonic(created.wallet.masterId), isNull);
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
    final a1 = await manager.addAccount(created.wallet.masterId);

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
    final a1 = await manager.addAccount(created.wallet.masterId);
    await manager.addAccount(created.wallet.masterId); // //2

    await manager.deleteAccount(a1.accountId);

    final accounts = await manager.getAccounts(created.wallet.masterId);
    expect(accounts.map((e) => e.accountIndex).toList(), [0, 2]);
    expect(await manager.getWallets(), hasLength(1));
  });

  test('deleteAccount 拒绝删账户0(尚有兄弟账户)', () async {
    final created = await manager.importWallet(kDevPhrase);
    await manager.addAccount(created.wallet.masterId);
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
    expect(await manager.getMasterMnemonic(created.wallet.masterId), isNull);
  });

  test('删中间账户后 addAccount 仍为 max+1(不回填空档,行为钉死)', () async {
    final created = await manager.importWallet(kDevPhrase);
    final a1 = await manager.addAccount(created.wallet.masterId); // //1
    await manager.addAccount(created.wallet.masterId); // //2
    await manager.deleteAccount(a1.accountId); // 删 //1,留 0,2

    final a3 = await manager.addAccount(created.wallet.masterId);
    expect(a3.accountIndex, 3); // max(2)+1,不回填 1
  });

  test('renameWallet / reorderWallets', () async {
    final w1 = await manager.importWallet(kDevPhrase);
    // 第二个钱包(不同助记词)。
    const otherPhrase =
        'legal winner thank year wave sausage worth useful legal winner thank yellow';
    final w2 = await manager.importWallet(otherPhrase);

    await manager.renameWallet(w1.wallet.walletIndex, '主号');
    final renamed = await manager.getWalletByMasterId(w1.wallet.masterId);
    expect(renamed?.walletName, '主号');

    await manager
        .reorderWallets([w2.wallet.walletIndex, w1.wallet.walletIndex]);
    final ordered = await manager.getWallets();
    expect(ordered.first.masterId, w2.wallet.masterId);
  });
}
