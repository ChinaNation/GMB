import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/transaction/shared/local_tx_store.dart';

import '../support/isar_test_env.dart';

void main() {
  useIsolatedIsar();

  const fromAccountId =
      '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
  const toAccountId =
      '0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';
  const fromSs58Address = 'from-wallet';
  const toSs58Address = 'to-wallet';

  test('本机转出记录按 pending -> inBlock -> finalized 升级且不重复', () async {
    await LocalTxStore.upsertLocalSubmitTransfer(
      ss58Address: fromSs58Address,
      accountId: fromAccountId,
      txHash: '0xabc',
      amountDeltaFen: '-101',
      transferAmountFen: '100',
      feeFen: '1',
      counterpartySs58Address: toSs58Address,
      fromSs58Address: fromSs58Address,
      toSs58Address: toSs58Address,
      usedNonce: 7,
      createdAtMillis: 1,
    );
    await LocalTxStore.markLocalSubmitInBlock(
      accountId: fromAccountId,
      txHash: '0xabc',
      blockHash: '0x22',
    );

    var records = await LocalTxStore.queryByAccountId(fromAccountId);
    expect(records, hasLength(1));
    expect(records.single.status, LocalTxStore.statusInBlock);
    expect(records.single.recordKey, contains(':pending:'));

    final eventKey = LocalTxStore.blockEventRecordKey(fromAccountId, '0x22', 3);
    await LocalTxStore.upsertBlockTransferEvent(
      ss58Address: fromSs58Address,
      accountId: fromAccountId,
      recordKey: eventKey,
      status: LocalTxStore.statusFinalized,
      amountDeltaFen: '-100',
      transferAmountFen: '100',
      fromSs58Address: fromSs58Address,
      toSs58Address: toSs58Address,
      counterpartySs58Address: toSs58Address,
      blockNumber: 9,
      blockHash: '0x22',
      eventIndex: 3,
    );

    records = await LocalTxStore.queryByAccountId(fromAccountId);
    expect(records, hasLength(1));
    expect(records.single.recordKey, eventKey);
    expect(records.single.status, LocalTxStore.statusFinalized);
    expect(records.single.amountDeltaFen, '-101');
    expect(records.single.txHash, '0xabc');
    expect(records.single.confirmedAtMillis, isNotNull);
  });

  test('区块事件先到时，本机提交记录按同区块同转账合并为一条', () async {
    final eventKey = LocalTxStore.blockEventRecordKey(fromAccountId, '0x44', 2);
    await LocalTxStore.upsertBlockTransferEvent(
      ss58Address: fromSs58Address,
      accountId: fromAccountId,
      recordKey: eventKey,
      status: LocalTxStore.statusInBlock,
      amountDeltaFen: '-210',
      transferAmountFen: '210',
      fromSs58Address: fromSs58Address,
      toSs58Address: toSs58Address,
      counterpartySs58Address: toSs58Address,
      blockNumber: 11,
      blockHash: '0x44',
      eventIndex: 2,
    );

    await LocalTxStore.upsertLocalSubmitTransfer(
      ss58Address: fromSs58Address,
      accountId: fromAccountId,
      txHash: '0xdef',
      amountDeltaFen: '-220',
      transferAmountFen: '210',
      feeFen: '10',
      counterpartySs58Address: toSs58Address,
      fromSs58Address: fromSs58Address,
      toSs58Address: toSs58Address,
      usedNonce: 8,
      createdAtMillis: 2,
      blockHash: '0x44',
    );

    final records = await LocalTxStore.queryByAccountId(fromAccountId);
    expect(records, hasLength(1));
    expect(records.single.recordKey, eventKey);
    expect(records.single.status, LocalTxStore.statusInBlock);
    expect(records.single.amountDeltaFen, '-220');
    expect(records.single.feeFen, '10');
    expect(records.single.txHash, '0xdef');
  });

  test('收款钱包先写入 inBlock 收入记录，finalized 再升级同一条记录', () async {
    final eventKey = LocalTxStore.blockEventRecordKey(toAccountId, '0x33', 5);
    await LocalTxStore.upsertBlockTransferEvent(
      ss58Address: toSs58Address,
      accountId: toAccountId,
      recordKey: eventKey,
      status: LocalTxStore.statusInBlock,
      amountDeltaFen: '100',
      transferAmountFen: '100',
      fromSs58Address: fromSs58Address,
      toSs58Address: toSs58Address,
      counterpartySs58Address: fromSs58Address,
      blockNumber: 10,
      blockHash: '0x33',
      eventIndex: 5,
    );

    var records = await LocalTxStore.queryByAccountId(toAccountId);
    expect(records, hasLength(1));
    expect(records.single.status, LocalTxStore.statusInBlock);
    expect(records.single.amountDeltaFen, '100');
    expect(records.single.confirmedAtMillis, isNull);

    await LocalTxStore.upsertBlockTransferEvent(
      ss58Address: toSs58Address,
      accountId: toAccountId,
      recordKey: eventKey,
      status: LocalTxStore.statusFinalized,
      amountDeltaFen: '100',
      transferAmountFen: '100',
      fromSs58Address: fromSs58Address,
      toSs58Address: toSs58Address,
      counterpartySs58Address: fromSs58Address,
      blockNumber: 10,
      blockHash: '0x33',
      eventIndex: 5,
    );

    records = await LocalTxStore.queryByAccountId(toAccountId);
    expect(records, hasLength(1));
    expect(records.single.status, LocalTxStore.statusFinalized);
    expect(records.single.confirmedAtMillis, isNotNull);
  });

  test('同一区块同一收入事件重复处理时只升级状态不新增记录', () async {
    final firstKey = LocalTxStore.blockEventRecordKey(toAccountId, '0x55', 5);
    await LocalTxStore.upsertBlockTransferEvent(
      ss58Address: toSs58Address,
      accountId: toAccountId,
      recordKey: firstKey,
      status: LocalTxStore.statusInBlock,
      amountDeltaFen: '210',
      transferAmountFen: '210',
      fromSs58Address: fromSs58Address,
      toSs58Address: toSs58Address,
      counterpartySs58Address: fromSs58Address,
      blockNumber: 12,
      blockHash: '0x55',
      eventIndex: 5,
    );

    final secondKey = LocalTxStore.blockEventRecordKey(toAccountId, '0x55', 6);
    await LocalTxStore.upsertBlockTransferEvent(
      ss58Address: toSs58Address,
      accountId: toAccountId,
      recordKey: secondKey,
      status: LocalTxStore.statusFinalized,
      amountDeltaFen: '210',
      transferAmountFen: '210',
      fromSs58Address: fromSs58Address,
      toSs58Address: toSs58Address,
      counterpartySs58Address: fromSs58Address,
      blockNumber: 12,
      blockHash: '0x55',
      eventIndex: 6,
    );

    final records = await LocalTxStore.queryByAccountId(toAccountId);
    expect(records, hasLength(1));
    expect(records.single.recordKey, firstKey);
    expect(records.single.status, LocalTxStore.statusFinalized);
    expect(records.single.amountDeltaFen, '210');
  });

  test('本机提交备注在区块事件合并后保留', () async {
    await LocalTxStore.upsertLocalSubmitTransfer(
      ss58Address: fromSs58Address,
      accountId: fromAccountId,
      txHash: '0xremark',
      amountDeltaFen: '-110',
      transferAmountFen: '100',
      feeFen: '10',
      counterpartySs58Address: toSs58Address,
      fromSs58Address: fromSs58Address,
      toSs58Address: toSs58Address,
      usedNonce: 9,
      createdAtMillis: 3,
      remark: '中华联邦创世',
    );

    final eventKey = LocalTxStore.blockEventRecordKey(fromAccountId, '0x66', 4);
    await LocalTxStore.upsertBlockTransferEvent(
      ss58Address: fromSs58Address,
      accountId: fromAccountId,
      recordKey: eventKey,
      status: LocalTxStore.statusFinalized,
      amountDeltaFen: '-100',
      transferAmountFen: '100',
      fromSs58Address: fromSs58Address,
      toSs58Address: toSs58Address,
      counterpartySs58Address: toSs58Address,
      blockNumber: 13,
      blockHash: '0x66',
      eventIndex: 4,
    );

    final records = await LocalTxStore.queryByAccountId(fromAccountId);
    expect(records, hasLength(1));
    expect(records.single.recordKey, eventKey);
    expect(records.single.status, LocalTxStore.statusFinalized);
    expect(records.single.remark, '中华联邦创世');
  });
}
