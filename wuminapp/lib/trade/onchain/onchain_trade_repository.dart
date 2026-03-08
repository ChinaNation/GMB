import 'package:isar/isar.dart';
import 'package:wuminapp_mobile/trade/onchain/onchain_trade_models.dart';
import 'package:wuminapp_mobile/Isar/wallet_isar.dart';

abstract class OnchainTradeRepository {
  Future<void> save(OnchainTxRecord record);

  Future<void> upsert(OnchainTxRecord record);

  Future<List<OnchainTxRecord>> listRecent();
}

class LocalOnchainTradeRepository implements OnchainTradeRepository {
  @override
  Future<void> save(OnchainTxRecord record) async {
    await upsert(record);
  }

  @override
  Future<void> upsert(OnchainTxRecord record) async {
    final isar = await WalletIsar.instance.db();
    await isar.writeTxn(() async {
      await isar.txRecordEntitys.put(_toEntity(record));
    });
  }

  @override
  Future<List<OnchainTxRecord>> listRecent() async {
    final isar = await WalletIsar.instance.db();
    final rows = await isar.txRecordEntitys
        .where()
        .anyId()
        .sortByCreatedAtMillisDesc()
        .findAll();
    return rows.map(_toModel).toList(growable: false);
  }

  TxRecordEntity _toEntity(OnchainTxRecord model) {
    return TxRecordEntity()
      ..txHash = model.txHash
      ..fromAddress = model.fromAddress
      ..toAddress = model.toAddress
      ..amount = model.amount
      ..symbol = model.symbol
      ..createdAtMillis = model.createdAt.millisecondsSinceEpoch
      ..status = onchainTxStatusToString(model.status)
      ..failureReason = model.failureReason;
  }

  OnchainTxRecord _toModel(TxRecordEntity entity) {
    return OnchainTxRecord(
      txHash: entity.txHash,
      fromAddress: entity.fromAddress,
      toAddress: entity.toAddress,
      amount: entity.amount,
      symbol: entity.symbol,
      createdAt: DateTime.fromMillisecondsSinceEpoch(entity.createdAtMillis),
      status: onchainTxStatusFromString(entity.status),
      failureReason: entity.failureReason,
    );
  }
}

class InMemoryOnchainTradeRepository implements OnchainTradeRepository {
  final List<OnchainTxRecord> _records = <OnchainTxRecord>[];

  @override
  Future<void> save(OnchainTxRecord record) async {
    await upsert(record);
  }

  @override
  Future<void> upsert(OnchainTxRecord record) async {
    final index = _records.indexWhere((it) => it.txHash == record.txHash);
    if (index >= 0) {
      _records[index] = record;
      return;
    }
    _records.insert(0, record);
  }

  @override
  Future<List<OnchainTxRecord>> listRecent() async {
    return List<OnchainTxRecord>.unmodifiable(_records);
  }
}
