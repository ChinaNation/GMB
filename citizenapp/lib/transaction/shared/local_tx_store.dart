import 'package:isar_community/isar.dart';
import 'package:citizenapp/isar/app_isar.dart';

/// 本机钱包交易流水存储服务。
///
/// 这里保存的是“钱包进入本机 App 之后”的余额变化流水。
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

  /// 写入本机发起的普通转账记录。
  ///
  /// 交易池和区块事件可能先于页面本地写入返回。这里先查是否
  /// 已有同钱包、同发送方、同接收方、同本金的区块事件记录；若有，直接
  /// 合并手续费、txHash 和 nonce，避免“本金事件 + 本机扣费记录”显示两条。
  static Future<void> upsertLocalSubmitTransfer({
    required String walletAddress,
    required String walletPubkeyHex,
    required String txHash,
    required String amountDeltaFen,
    required String transferAmountFen,
    required String feeFen,
    required String counterpartyAddress,
    required String fromAddress,
    required String toAddress,
    required int usedNonce,
    required int createdAtMillis,
    String? remark,
    String? blockHash,
  }) async {
    final pubkey = normalizePubkey(walletPubkeyHex);
    final normalizedTxHash = txHash.toLowerCase();
    final pendingKey = pendingRecordKey(pubkey, normalizedTxHash);
    final normalizedBlockHash = blockHash == null || blockHash.isEmpty
        ? null
        : normalizeBlockHash(blockHash);
    await WalletIsar.instance.writeTxn((isar) async {
      final existingPending = await isar.localTxEntitys
          .where()
          .recordKeyEqualTo(pendingKey)
          .findFirst();
      if (existingPending != null) {
        existingPending
          ..walletAddress = walletAddress
          ..walletPubkeyHex = pubkey
          ..type = 'transfer'
          ..amountDeltaFen = amountDeltaFen
          ..transferAmountFen = transferAmountFen
          ..feeFen = feeFen
          ..counterpartyAddress = counterpartyAddress
          ..fromAddress = fromAddress
          ..toAddress = toAddress
          ..remark = _mergeRemark(remark, existingPending.remark)
          ..status = _mergeStatus(existingPending.status, statusPending)
          ..source = 'local_submit'
          ..txHash = normalizedTxHash
          ..usedNonce = usedNonce
          ..createdAtMillis = existingPending.createdAtMillis
          ..failureReason = null;
        await isar.localTxEntitys.put(existingPending);
        return;
      }

      final existingEvent = normalizedBlockHash == null
          ? null
          : await _findSemanticBlockTransferInTxn(
              isar,
              walletPubkeyHex: pubkey,
              blockNumber: null,
              blockHash: normalizedBlockHash,
              fromAddress: fromAddress,
              toAddress: toAddress,
              transferAmountFen: transferAmountFen,
              extrinsicIndex: null,
              eventIndex: null,
            );
      final entity = existingEvent ?? LocalTxEntity();
      entity
        ..recordKey = existingEvent?.recordKey ?? pendingKey
        ..walletAddress = walletAddress
        ..walletPubkeyHex = pubkey
        ..type = 'transfer'
        ..amountDeltaFen = amountDeltaFen
        ..transferAmountFen = transferAmountFen
        ..feeFen = feeFen
        ..counterpartyAddress = counterpartyAddress
        ..fromAddress = fromAddress
        ..toAddress = toAddress
        ..remark = _mergeRemark(remark, existingEvent?.remark)
        ..status = _mergeStatus(existingEvent?.status, statusPending)
        ..source = 'local_submit'
        ..txHash = normalizedTxHash
        ..usedNonce = usedNonce
        ..createdAtMillis = existingEvent?.createdAtMillis ?? createdAtMillis
        ..failureReason = null;
      await isar.localTxEntitys.put(entity);
    });
  }

  /// 写入链上区块转账事件；如能匹配本机发起记录，则更新原记录。
  ///
  /// (ADR-017 全端 finalized 单一口径)：本方法由只扫 finalized 链的
  /// ChainTxMonitor 调用，写入/升级的流水状态恒为 finalized(已确认)。收入
  /// (别人转入)没有本机 pending，只在对应区块 finalized 后用同一个区块事件
  /// 唯一键写入，避免“余额到账但无收入记录”。inBlock 进度态由交易提交
  /// watch 单独产生(见 [markLocalSubmitInBlock])，不在本路径。
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
    String? remark,
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
          ..remark = _mergeRemark(remark, existing.remark)
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

      final semanticExisting = await _findSemanticBlockTransferInTxn(
        isar,
        walletPubkeyHex: pubkey,
        blockNumber: blockNumber,
        blockHash: normalizedBlockHash,
        fromAddress: fromAddress,
        toAddress: toAddress,
        transferAmountFen: transferAmountFen,
        extrinsicIndex: extrinsicIndex,
        eventIndex: eventIndex,
      );
      if (semanticExisting != null) {
        semanticExisting
          ..recordKey = semanticExisting.recordKey.contains(':pending:')
              ? recordKey
              : semanticExisting.recordKey
          ..walletAddress = walletAddress
          ..walletPubkeyHex = pubkey
          ..amountDeltaFen = semanticExisting.feeFen != null
              ? semanticExisting.amountDeltaFen
              : amountDeltaFen
          ..transferAmountFen =
              semanticExisting.transferAmountFen ?? transferAmountFen
          ..fromAddress = semanticExisting.fromAddress ?? fromAddress
          ..toAddress = semanticExisting.toAddress ?? toAddress
          ..counterpartyAddress =
              semanticExisting.counterpartyAddress ?? counterpartyAddress
          ..remark = _mergeRemark(remark, semanticExisting.remark)
          ..status = _mergeStatus(semanticExisting.status, status)
          ..blockNumber = blockNumber
          ..blockHash = normalizedBlockHash
          ..eventIndex = semanticExisting.eventIndex ?? eventIndex
          ..extrinsicIndex = semanticExisting.extrinsicIndex ?? extrinsicIndex
          ..confirmedAtMillis = status == statusFinalized
              ? (confirmedAtMillis ?? now)
              : semanticExisting.confirmedAtMillis
          ..failureReason = null;
        await isar.localTxEntitys.put(semanticExisting);
        return;
      }

      // 本机发起转账会先写 pending，交易池 inBlock 回调可能先把它
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
        ..remark = _mergeRemark(remark, localSubmit?.remark)
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
  /// 这里不把它直接改成 finalized；最终确认仍由 finalized 区块事件
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

  static String? _mergeRemark(String? incoming, String? existing) {
    final normalized = incoming == null || incoming.isEmpty ? null : incoming;
    return normalized ?? existing;
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

  static Future<LocalTxEntity?> _findSemanticBlockTransferInTxn(
    Isar isar, {
    required String walletPubkeyHex,
    required int? blockNumber,
    required String? blockHash,
    required String fromAddress,
    required String toAddress,
    required String transferAmountFen,
    required int? extrinsicIndex,
    required int? eventIndex,
  }) async {
    final records = await isar.localTxEntitys
        .filter()
        .walletPubkeyHexEqualTo(walletPubkeyHex)
        .typeEqualTo('transfer')
        .findAll();
    for (final record in records) {
      if (record.fromAddress != fromAddress ||
          record.toAddress != toAddress ||
          record.transferAmountFen != transferAmountFen) {
        continue;
      }
      if (blockHash != null && record.blockHash != null) {
        if (normalizeBlockHash(record.blockHash!) != blockHash) continue;
      }
      if (blockNumber != null &&
          record.blockNumber != null &&
          record.blockNumber != blockNumber) {
        continue;
      }
      if (extrinsicIndex != null &&
          record.extrinsicIndex != null &&
          record.extrinsicIndex != extrinsicIndex) {
        continue;
      }
      if (record.status == statusPending && record.source != 'local_submit') {
        continue;
      }
      return record;
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
