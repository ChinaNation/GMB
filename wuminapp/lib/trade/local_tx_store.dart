import 'package:isar/isar.dart';
import 'package:wuminapp_mobile/Isar/wallet_isar.dart';

/// 本地交易记录存储服务。
class LocalTxStore {
  /// 写入一条交易记录。
  static Future<void> insert(LocalTxEntity entity) async {
    final isar = await WalletIsar.instance.db();
    await isar.writeTxn(() async {
      await isar.localTxEntitys.put(entity);
    });
  }

  /// 查询某个钱包的交易记录（按创建时间倒序）。
  static Future<List<LocalTxEntity>> queryByWallet(
    String walletAddress, {
    int limit = 20,
    int offset = 0,
  }) async {
    final isar = await WalletIsar.instance.db();
    return isar.localTxEntitys
        .where()
        .walletAddressEqualTo(walletAddress)
        .sortByCreatedAtMillisDesc()
        .offset(offset)
        .limit(limit)
        .findAll();
  }

  /// 查询某个钱包最近 N 条记录。
  static Future<List<LocalTxEntity>> queryRecent(
    String walletAddress, {
    int limit = 5,
  }) async {
    return queryByWallet(walletAddress, limit: limit);
  }

  /// 按 txId 更新状态。
  static Future<void> updateStatus(String txId, String status) async {
    final isar = await WalletIsar.instance.db();
    await isar.writeTxn(() async {
      final entity = await isar.localTxEntitys
          .where()
          .txIdEqualTo(txId)
          .findFirst();
      if (entity != null) {
        entity.status = status;
        entity.confirmedAtMillis = DateTime.now().millisecondsSinceEpoch;
        await isar.localTxEntitys.put(entity);
      }
    });
  }

  /// 按 txId 查询单条记录（防重复用）。
  static Future<LocalTxEntity?> queryByTxId(String txId) async {
    final isar = await WalletIsar.instance.db();
    return isar.localTxEntitys
        .where()
        .txIdEqualTo(txId)
        .findFirst();
  }

  /// 查询某个钱包的交易总数。
  static Future<int> countByWallet(String walletAddress) async {
    final isar = await WalletIsar.instance.db();
    return isar.localTxEntitys
        .where()
        .walletAddressEqualTo(walletAddress)
        .count();
  }
}
