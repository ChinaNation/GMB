import 'package:citizenapp/isar/app_isar.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:isar_community/isar.dart';

import '../support/isar_test_env.dart';

void main() {
  useIsolatedIsar();

  test('数据库重开会删除旧钱包名同步队列并保留无关 KV', () async {
    final isar = await WalletIsar.instance.db();
    await isar.writeTxn(() async {
      await isar.appKvEntitys.putAll([
        AppKvEntity()
          ..key = 'wallet_name_pending:0x01'
          ..stringValue = '旧待同步名',
        AppKvEntity()
          ..key = 'wallet_name_synced_at:0x01'
          ..intValue = 123,
        AppKvEntity()
          ..key = 'unrelated:key'
          ..stringValue = 'keep',
      ]);
    });

    // 模拟升级后的下一次进程打开；目标态清理必须同时覆盖已有数据库。
    await isar.close();
    final reopened = await WalletIsar.instance.db();
    final rows = await reopened.appKvEntitys.where().findAll();
    final keys = rows.map((row) => row.key).toSet();

    expect(keys, contains('unrelated:key'));
    expect(keys, isNot(contains('wallet_name_pending:0x01')));
    expect(keys, isNot(contains('wallet_name_synced_at:0x01')));
  });

  test('钱包改名只更新本机钱包标签且不创建公开昵称同步状态', () async {
    final isar = await WalletIsar.instance.db();
    await isar.writeTxn(() async {
      await isar.walletProfileEntitys.put(
        WalletProfileEntity()
          ..walletIndex = 1
          ..walletName = '钱包1'
          ..walletIcon = 'wallet'
          ..balance = 0
          ..accountId =
              '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'
          ..masterId =
              '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'
          ..ss58Address = 'citizen_test_wallet'
          ..alg = 'sr25519'
          ..ss58 = 2027
          ..createdAtMillis = 1
          ..source = 'test'
          ..signMode = 'local'
          ..sortOrder = 0,
      );
    });

    await WalletManager().renameWallet(1, '仅本机标签');

    final wallet = await isar.walletProfileEntitys
        .filter()
        .walletIndexEqualTo(1)
        .findFirst();
    final kvRows = await isar.appKvEntitys.where().findAll();
    expect(wallet?.walletName, '仅本机标签');
    expect(
      kvRows.where(
        (row) =>
            row.key.startsWith('wallet_name_pending:') ||
            row.key.startsWith('wallet_name_synced_at:'),
      ),
      isEmpty,
    );
  });
}
