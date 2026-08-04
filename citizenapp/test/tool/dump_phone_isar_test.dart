// 临时只读诊断:打开手机拉下来的 citizenapp.isar 副本,打印本机交易记录与游标。
// 不写手机、不清库、不改 app。用完删除本文件。
import 'dart:io';

import 'package:citizenapp/isar/app_isar.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:isar_community/isar.dart';

void main() {
  test('dump phone LocalTxEntity + cursor (read-only)', () async {
    const pulled =
        '/private/tmp/claude-501/-Users-rhett-GMB/fae50d6c-d1af-4117-acf5-6575a10e5185/scratchpad/citizenapp_live3.isar';
    final dir = Directory.systemTemp.createTempSync('citizenapp_dump_');
    File(pulled).copySync('${dir.path}/citizenapp.isar');

    WalletIsar.debugTestDirectoryOverride = dir.path;
    await WalletIsar.instance.ensureTestCoreInitialized();
    final isar = await WalletIsar.instance.db();

    final txs = await isar.localTxEntitys.where().findAll();
    // ignore: avoid_print
    print('===== LocalTxEntity 活记录 ${txs.length} 条 =====');
    for (final t in txs) {
      // ignore: avoid_print
      print('key=${t.recordKey}\n'
          '  status=${t.status} source=${t.source} txHash=${t.txHash}\n'
          '  blockHash=${t.blockHash} blockNumber=${t.blockNumber}');
    }
    final cursors = await isar.walletTxSyncCursorEntitys.where().findAll();
    // ignore: avoid_print
    print('===== 游标 ${cursors.length} 条 =====');
    for (final c in cursors) {
      // ignore: avoid_print
      print('accountId=${c.accountId} trackingStartBlock=${c.trackingStartBlock} '
          'lastSyncedBlock=${c.lastSyncedBlock}');
    }
  });
}
