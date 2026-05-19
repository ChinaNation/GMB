import 'package:isar/isar.dart';
import 'package:wuminapp_mobile/isar/wallet_isar.dart';

/// 本机钱包交易流水存储服务。
///
/// 中文注释：这里保存的是“钱包进入本机 App 之后”的余额变化流水。
/// 链上账户唯一性用 walletPubkeyHex，单条流水唯一性用 recordKey。
class LocalTxStore {
  static const String statusPending = 'pending';
  static const String statusInBlock = 'inBlock';
  static const String statusFinalized = 'finalized';

  static String normalizePubkey(String pubkeyHex) {
    return pubkeyHex.replaceFirst('0x', '').toLowerCase();
  }

  static String normalizeBlockHash(String blockHash) {
    return blockHash.startsWith('0x')
        ? blockHash.toLowerCase()
        : '0x${blockHash.toLowerCase()}';
  }

  static String pendingRecordKey(String walletPubkeyHex, String txHash) {
    return '${normalizePubkey(walletPubkeyHex)}:pending:${txHash.toLowerCase()}';
  }

  static String blockEventRecordKey(
    String walletPubkeyHex,
    String blockHash,
    int eventIndex,
  ) {
    return '${normalizePubkey(walletPubkeyHex)}:${normalizeBlockHash(blockHash)}:$eventIndex';
  }

  static String fenFromYuan(double amountYuan) {
    return BigInt.from((amountYuan * 100).round()).toString();
  }

  static double fenToYuan(String amountFen) {
    return BigInt.parse(amountFen).toDouble() / 100.0;
  }

  static String negateFen(String amountFen) {
    final value = BigInt.parse(amountFen);
    return (-value).toString();
  }

  /// 写入或替换一条交易流水。
  static Future<void> upsert(LocalTxEntity entity) async {
    entity.walletPubkeyHex = normalizePubkey(entity.walletPubkeyHex);
    await WalletIsar.instance.writeTxn((isar) async {
      await isar.localTxEntitys.put(entity);
    });
  }

  /// 查询某个钱包的交易流水（按本机记录时间倒序）。
  static Future<List<LocalTxEntity>> queryByWalletPubkey(
    String walletPubkeyHex, {
    int limit = 20,
    int offset = 0,
  }) async {
    final pubkey = normalizePubkey(walletPubkeyHex);
    return WalletIsar.instance.read((isar) {
      return isar.localTxEntitys
          .where()
          .walletPubkeyHexEqualTo(pubkey)
          .sortByCreatedAtMillisDesc()
          .offset(offset)
          .limit(limit)
          .findAll();
    });
  }

  /// 查询某个钱包最近 N 条记录。
  static Future<List<LocalTxEntity>> queryRecentByWalletPubkey(
    String walletPubkeyHex, {
    int limit = 5,
  }) async {
    return queryByWalletPubkey(walletPubkeyHex, limit: limit);
  }

  /// 按 recordKey 查询单条记录（防重复用）。
  static Future<LocalTxEntity?> queryByRecordKey(String recordKey) async {
    return WalletIsar.instance.read((isar) {
      return isar.localTxEntitys
          .where()
          .recordKeyEqualTo(recordKey)
          .findFirst();
    });
  }

  /// 写入链上区块转账事件；如能匹配本机发起记录，则更新原记录。
  ///
  /// 中文注释：钱包流水状态分三段：
  /// pending=已提交、inBlock=已出块、finalized=已确认。
  /// 收入没有本机 pending，因此 inBlock 阶段就要写入本地；finalized 阶段
  /// 再用同一个区块事件唯一键升级状态，避免“余额到账但无收入记录”。
  static Future<void> upsertBlockTransferEvent({
    required String walletAddress,
    required String walletPubkeyHex,
    required String recordKey,
    required String status,
    required String amountDeltaFen,
    required String transferAmountFen,
    required String fromAddress,
    required String toAddress,
    required String counterpartyAddress,
    required int blockNumber,
    required String blockHash,
    required int eventIndex,
    int? extrinsicIndex,
    int? confirmedAtMillis,
  }) async {
    final pubkey = normalizePubkey(walletPubkeyHex);
    final normalizedBlockHash = normalizeBlockHash(blockHash);
    final now = DateTime.now().millisecondsSinceEpoch;
    await WalletIsar.instance.writeTxn((isar) async {
      final existing = await isar.localTxEntitys
          .where()
          .recordKeyEqualTo(recordKey)
          .findFirst();
      if (existing != null) {
        existing
          ..status = _mergeStatus(existing.status, status)
          ..walletAddress = walletAddress
          ..walletPubkeyHex = pubkey
          ..transferAmountFen = existing.transferAmountFen ?? transferAmountFen
          ..fromAddress = existing.fromAddress ?? fromAddress
          ..toAddress = existing.toAddress ?? toAddress
          ..counterpartyAddress =
              existing.counterpartyAddress ?? counterpartyAddress
          ..blockNumber = blockNumber
          ..blockHash = normalizedBlockHash
          ..eventIndex = eventIndex
          ..extrinsicIndex = extrinsicIndex ?? existing.extrinsicIndex
          ..confirmedAtMillis = status == statusFinalized
              ? (confirmedAtMillis ?? now)
              : existing.confirmedAtMillis
          ..failureReason = null;
        await isar.localTxEntitys.put(existing);
        return;
      }

      // 中文注释：本机发起转账会先写 pending，交易池 inBlock 回调可能先把它
      // 标成 inBlock。链上 Transfer 事件回来后，用同钱包、同收款人、同本金
      // 匹配并改成区块事件唯一键，避免列表里出现重复流水。
      final localSubmit = await _findMatchingLocalSubmitTransferInTxn(
        isar,
        walletPubkeyHex: pubkey,
        fromAddress: fromAddress,
        toAddress: toAddress,
        transferAmountFen: transferAmountFen,
      );
      final entity = localSubmit ?? LocalTxEntity();
      entity
        ..recordKey = recordKey
        ..walletAddress = walletAddress
        ..walletPubkeyHex = pubkey
        ..type = 'transfer'
        ..amountDeltaFen = localSubmit?.amountDeltaFen ?? amountDeltaFen
        ..transferAmountFen = transferAmountFen
        ..counterpartyAddress = counterpartyAddress
        ..fromAddress = fromAddress
        ..toAddress = toAddress
        ..status = _mergeStatus(localSubmit?.status, status)
        ..source = localSubmit?.source ?? 'chain_event'
        ..blockNumber = blockNumber
        ..blockHash = normalizedBlockHash
        ..eventIndex = eventIndex
        ..extrinsicIndex = extrinsicIndex
        ..createdAtMillis = localSubmit?.createdAtMillis ?? now
        ..confirmedAtMillis =
            status == statusFinalized ? (confirmedAtMillis ?? now) : null
        ..failureReason = null;
      await isar.localTxEntitys.put(entity);
    });
  }

  /// 交易池回调显示交易已进入区块时，先把本机 pending 记录升级为 inBlock。
  ///
  /// 中文注释：这里不把它直接改成 finalized；最终确认仍由 finalized 区块事件
  /// 写回，保留回滚边界。
  static Future<void> markLocalSubmitInBlock({
    required String walletPubkeyHex,
    required String txHash,
    String? blockHash,
  }) async {
    final recordKey = pendingRecordKey(walletPubkeyHex, txHash);
    await WalletIsar.instance.writeTxn((isar) async {
      final entity = await isar.localTxEntitys
          .where()
          .recordKeyEqualTo(recordKey)
          .findFirst();
      if (entity == null || entity.status == statusFinalized) return;
      entity.status = statusInBlock;
      if (blockHash != null && blockHash.isNotEmpty) {
        entity.blockHash = normalizeBlockHash(blockHash);
      }
      entity.failureReason = null;
      await isar.localTxEntitys.put(entity);
    });
  }

  static String _mergeStatus(String? current, String incoming) {
    final currentRank = _statusRank(current);
    final incomingRank = _statusRank(incoming);
    return incomingRank >= currentRank ? incoming : (current ?? incoming);
  }

  static int _statusRank(String? status) {
    switch (status) {
      case statusFinalized:
        return 3;
      case statusInBlock:
        return 2;
      case statusPending:
        return 1;
      default:
        return 0;
    }
  }

  static Future<LocalTxEntity?> _findMatchingLocalSubmitTransferInTxn(
    Isar isar, {
    required String walletPubkeyHex,
    required String fromAddress,
    required String toAddress,
    required String transferAmountFen,
  }) async {
    final pending = await isar.localTxEntitys
        .filter()
        .walletPubkeyHexEqualTo(walletPubkeyHex)
        .typeEqualTo('transfer')
        .findAll();
    for (final record in pending) {
      if (record.fromAddress == fromAddress &&
          record.toAddress == toAddress &&
          record.transferAmountFen == transferAmountFen &&
          record.source == 'local_submit' &&
          (record.status == statusPending || record.status == statusInBlock)) {
        return record;
      }
    }
    return null;
  }

  /// 删除某个钱包本机记录周期内的所有交易流水和同步游标。
  static Future<void> deleteWalletLocalHistory(String walletPubkeyHex) async {
    final pubkey = normalizePubkey(walletPubkeyHex);
    await WalletIsar.instance.writeTxn((isar) async {
      await isar.localTxEntitys
          .filter()
          .walletPubkeyHexEqualTo(pubkey)
          .deleteAll();
      await isar.walletTxSyncCursorEntitys
          .filter()
          .walletPubkeyHexEqualTo(pubkey)
          .deleteAll();
    });
  }

  /// 清空所有钱包交易流水和同步游标。
  static Future<void> clearAllWalletLocalHistory() async {
    await WalletIsar.instance.writeTxn((isar) async {
      await isar.localTxEntitys.clear();
      await isar.walletTxSyncCursorEntitys.clear();
    });
  }

  /// 确保钱包交易同步游标存在。
  static Future<WalletTxSyncCursorEntity> ensureCursor({
    required String walletAddress,
    required String walletPubkeyHex,
    required int trackingStartBlock,
    required int lastSyncedBlock,
  }) async {
    final pubkey = normalizePubkey(walletPubkeyHex);
    final now = DateTime.now().millisecondsSinceEpoch;
    return WalletIsar.instance.writeTxn((isar) async {
      final existing = await isar.walletTxSyncCursorEntitys
          .filter()
          .walletPubkeyHexEqualTo(pubkey)
          .findFirst();
      if (existing != null) {
        existing
          ..walletAddress = walletAddress
          ..updatedAtMillis = now;
        await isar.walletTxSyncCursorEntitys.put(existing);
        return existing;
      }
      final created = WalletTxSyncCursorEntity()
        ..walletAddress = walletAddress
        ..walletPubkeyHex = pubkey
        ..trackingStartBlock = trackingStartBlock
        ..lastSyncedBlock = lastSyncedBlock
        ..createdAtMillis = now
        ..updatedAtMillis = now;
      await isar.walletTxSyncCursorEntitys.put(created);
      return created;
    });
  }

  /// 读取当前监控钱包的同步游标；缺失的钱包会以指定区块作为本机起点。
  static Future<List<WalletTxSyncCursorEntity>> ensureCursorsForWallets({
    required Map<String, String> walletAddressByPubkey,
    required int startBlock,
  }) async {
    final result = <WalletTxSyncCursorEntity>[];
    for (final entry in walletAddressByPubkey.entries) {
      final cursor = await ensureCursor(
        walletAddress: entry.value,
        walletPubkeyHex: entry.key,
        trackingStartBlock: startBlock,
        lastSyncedBlock: startBlock,
      );
      result.add(cursor);
    }
    return result;
  }

  /// 标记钱包已经同步到某个 finalized 区块。
  static Future<void> markCursorSynced({
    required String walletPubkeyHex,
    required int blockNumber,
  }) async {
    final pubkey = normalizePubkey(walletPubkeyHex);
    await WalletIsar.instance.writeTxn((isar) async {
      final cursor = await isar.walletTxSyncCursorEntitys
          .filter()
          .walletPubkeyHexEqualTo(pubkey)
          .findFirst();
      if (cursor == null || cursor.lastSyncedBlock >= blockNumber) {
        return;
      }
      cursor
        ..lastSyncedBlock = blockNumber
        ..updatedAtMillis = DateTime.now().millisecondsSinceEpoch;
      await isar.walletTxSyncCursorEntitys.put(cursor);
    });
  }

  /// 查询某个钱包的交易总数。
  static Future<int> countByWalletPubkey(String walletPubkeyHex) async {
    final pubkey = normalizePubkey(walletPubkeyHex);
    return WalletIsar.instance.read((isar) {
      return isar.localTxEntitys.where().walletPubkeyHexEqualTo(pubkey).count();
    });
  }
}
