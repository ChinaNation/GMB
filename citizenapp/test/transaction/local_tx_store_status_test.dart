import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/isar/app_isar.dart';
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

  test('本机转出记录全程一条 :tx: 键，pending->inBlock->finalized 就地翻、不另建',
      () async {
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

    var records = await LocalTxStore.queryByAccountId(fromAccountId);
    expect(records, hasLength(1));
    expect(records.single.status, LocalTxStore.statusPending);
    expect(records.single.recordKey, contains(':tx:'));
    final txKey = records.single.recordKey;

    await LocalTxStore.markLocalSubmitInBlock(
      accountId: fromAccountId,
      txHash: '0xabc',
      blockHash: '0x22',
    );
    records = await LocalTxStore.queryByAccountId(fromAccountId);
    expect(records, hasLength(1));
    expect(records.single.status, LocalTxStore.statusInBlock);
    expect(records.single.recordKey, txKey);

    // 最终性由 txHash 精确认：就地翻 finalized，键不变、记录数不变、绝不另建。
    await LocalTxStore.markLocalSubmitFinalized(
      accountId: fromAccountId,
      txHash: '0xabc',
      blockHash: '0x22',
      blockNumber: 9,
      extrinsicIndex: 3,
    );

    records = await LocalTxStore.queryByAccountId(fromAccountId);
    expect(records, hasLength(1));
    expect(records.single.recordKey, txKey);
    expect(records.single.status, LocalTxStore.statusFinalized);
    expect(records.single.amountDeltaFen, '-101');
    expect(records.single.txHash, '0xabc');
    expect(records.single.blockNumber, 9);
    expect(records.single.extrinsicIndex, 3);
    expect(records.single.confirmedAtMillis, isNotNull);
  });

  test('dropped 保持待确认（不失败），txHash 认到后就地翻已确认、仍只有一条',
      () async {
    await LocalTxStore.upsertLocalSubmitTransfer(
      ss58Address: fromSs58Address,
      accountId: fromAccountId,
      txHash: '0xdef',
      amountDeltaFen: '-51',
      transferAmountFen: '50',
      feeFen: '1',
      counterpartySs58Address: toSs58Address,
      fromSs58Address: fromSs58Address,
      toSs58Address: toSs58Address,
      usedNonce: 8,
      createdAtMillis: 2,
    );

    // dropped 走 markLocalSubmitPending（不判失败），保持待确认。
    await LocalTxStore.markLocalSubmitPending(
      accountId: fromAccountId,
      txHash: '0xdef',
    );
    var records = await LocalTxStore.queryByAccountId(fromAccountId);
    expect(records, hasLength(1));
    expect(records.single.status, LocalTxStore.statusPending);
    final txKey = records.single.recordKey;
    expect(txKey, contains(':tx:'));

    // 该 txHash 在最终块被认到 → 就地翻已确认。
    await LocalTxStore.markLocalSubmitFinalized(
      accountId: fromAccountId,
      txHash: '0xdef',
      blockHash: '0x33',
      blockNumber: 12,
      extrinsicIndex: 1,
    );
    records = await LocalTxStore.queryByAccountId(fromAccountId);
    expect(records, hasLength(1));
    expect(records.single.recordKey, txKey);
    expect(records.single.status, LocalTxStore.statusFinalized);
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

  test('非最终区块回滚后恢复待确认，最终性确认记录不允许回滚', () async {
    await LocalTxStore.upsertLocalSubmitTransfer(
      ss58Address: fromSs58Address,
      accountId: fromAccountId,
      txHash: '0xretracted',
      amountDeltaFen: '-110',
      transferAmountFen: '100',
      feeFen: '10',
      counterpartySs58Address: toSs58Address,
      fromSs58Address: fromSs58Address,
      toSs58Address: toSs58Address,
      usedNonce: 10,
      createdAtMillis: 4,
    );
    await LocalTxStore.markLocalSubmitInBlock(
      accountId: fromAccountId,
      txHash: '0xretracted',
      blockHash: '0x77',
    );
    await LocalTxStore.markLocalSubmitPending(
      accountId: fromAccountId,
      txHash: '0xretracted',
    );

    final records = await LocalTxStore.queryByAccountId(fromAccountId);
    var record = records.singleWhere((item) => item.txHash == '0xretracted');
    expect(record.status, LocalTxStore.statusPending);
    expect(record.blockHash, isNull);

    final eventKey = LocalTxStore.blockEventRecordKey(
      fromAccountId,
      '0x99',
      9,
    );
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
      blockNumber: 15,
      blockHash: '0x99',
      eventIndex: 9,
    );
    await LocalTxStore.markLocalSubmitPending(
      accountId: fromAccountId,
      txHash: '0xretracted',
    );

    final finalizedRecords = await LocalTxStore.queryByAccountId(fromAccountId);
    record = finalizedRecords.singleWhere(
      (item) => item.txHash == '0xretracted',
    );
    expect(record.status, LocalTxStore.statusFinalized);
    expect(record.confirmedAtMillis, isNotNull);
  });

  test('已签名提交交易被明确拒绝后写入失败状态和原因', () async {
    await LocalTxStore.upsertLocalSubmitTransfer(
      ss58Address: fromSs58Address,
      accountId: fromAccountId,
      txHash: '0xfailed',
      amountDeltaFen: '-110',
      transferAmountFen: '100',
      feeFen: '10',
      counterpartySs58Address: toSs58Address,
      fromSs58Address: fromSs58Address,
      toSs58Address: toSs58Address,
      usedNonce: 11,
      createdAtMillis: 5,
    );
    await LocalTxStore.markLocalSubmitFailed(
      accountId: fromAccountId,
      txHash: '0xfailed',
      failureReason: '交易无效',
    );

    var records = await LocalTxStore.queryByAccountId(fromAccountId);
    var record = records.singleWhere((item) => item.txHash == '0xfailed');
    expect(record.status, LocalTxStore.statusFailed);
    expect(record.failureReason, '交易无效');

    // 明确失败是终态，迟到的非最终入块进度不能把它改回待确认。
    await LocalTxStore.markLocalSubmitInBlock(
      accountId: fromAccountId,
      txHash: '0xfailed',
      blockHash: '0x88',
    );
    records = await LocalTxStore.queryByAccountId(fromAccountId);
    record = records.singleWhere((item) => item.txHash == '0xfailed');
    expect(record.status, LocalTxStore.statusFailed);
    expect(record.failureReason, '交易无效');
  });

  test('purgeLegacyPendingRecords 只删旧 :pending: 孤儿，保留 :tx: 与 :blockHash:',
      () async {
    // 旧格式孤儿（:pending: 键）——直接注入，模拟迁移前遗留。
    await LocalTxStore.upsert(LocalTxEntity()
      ..recordKey = '$fromAccountId:pending:0xold'
      ..ss58Address = fromSs58Address
      ..accountId = fromAccountId
      ..type = 'transfer'
      ..amountDeltaFen = '-10'
      ..transferAmountFen = '10'
      ..fromSs58Address = fromSs58Address
      ..toSs58Address = toSs58Address
      ..source = 'local_submit'
      ..status = LocalTxStore.statusPending
      ..txHash = '0xold'
      ..createdAtMillis = 1);

    // 新格式本机提交（:tx: 键）。
    await LocalTxStore.upsertLocalSubmitTransfer(
      ss58Address: fromSs58Address,
      accountId: fromAccountId,
      txHash: '0xnew',
      amountDeltaFen: '-11',
      transferAmountFen: '10',
      feeFen: '1',
      counterpartySs58Address: toSs58Address,
      fromSs58Address: fromSs58Address,
      toSs58Address: toSs58Address,
      usedNonce: 1,
      createdAtMillis: 2,
    );

    // 区块事件（:blockHash: 键，收入侧）。
    final eventKey = LocalTxStore.blockEventRecordKey(toAccountId, '0x77', 5);
    await LocalTxStore.upsertBlockTransferEvent(
      ss58Address: toSs58Address,
      accountId: toAccountId,
      recordKey: eventKey,
      status: LocalTxStore.statusFinalized,
      amountDeltaFen: '20',
      transferAmountFen: '20',
      fromSs58Address: fromSs58Address,
      toSs58Address: toSs58Address,
      counterpartySs58Address: fromSs58Address,
      blockNumber: 3,
      blockHash: '0x77',
      eventIndex: 5,
    );

    final removed = await LocalTxStore.purgeLegacyPendingRecords();
    expect(removed, 1); // 只删了那条 :pending: 孤儿

    final fromRecords = await LocalTxStore.queryByAccountId(fromAccountId);
    expect(fromRecords, hasLength(1));
    expect(fromRecords.single.recordKey, contains(':tx:'));
    expect(fromRecords.single.txHash, '0xnew');

    final toRecords = await LocalTxStore.queryByAccountId(toAccountId);
    expect(toRecords, hasLength(1));
    expect(toRecords.single.recordKey, eventKey);

    // 幂等：清完再调 0 命中。
    expect(await LocalTxStore.purgeLegacyPendingRecords(), 0);
  });
}
