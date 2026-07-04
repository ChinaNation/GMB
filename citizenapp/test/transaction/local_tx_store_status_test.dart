import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/isar/wallet_isar.dart';
import 'package:citizenapp/transaction/shared/local_tx_store.dart';

void main() {
  const fromPubkey =
      'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
  const toPubkey =
      'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';
  const fromAddress = 'from-wallet';
  const toAddress = 'to-wallet';

  setUpAll(() async {
    await WalletIsar.instance.ensureTestCoreInitialized();
  });

  tearDown(() async {
    await WalletIsar.instance.resetForTest();
  });

  test('本机转出记录按 pending -> inBlock -> finalized 升级且不重复', () async {
    await LocalTxStore.upsertLocalSubmitTransfer(
      walletAddress: fromAddress,
      walletPubkeyHex: fromPubkey,
      txHash: '0xabc',
      amountDeltaFen: '-101',
      transferAmountFen: '100',
      feeFen: '1',
      counterpartyAddress: toAddress,
      fromAddress: fromAddress,
      toAddress: toAddress,
      usedNonce: 7,
      createdAtMillis: 1,
    );
    await LocalTxStore.markLocalSubmitInBlock(
      walletPubkeyHex: fromPubkey,
      txHash: '0xabc',
      blockHash: '0x22',
    );

    var records = await LocalTxStore.queryByWalletPubkey(fromPubkey);
    expect(records, hasLength(1));
    expect(records.single.status, LocalTxStore.statusInBlock);
    expect(records.single.recordKey, contains(':pending:'));

    final eventKey = LocalTxStore.blockEventRecordKey(fromPubkey, '0x22', 3);
    await LocalTxStore.upsertBlockTransferEvent(
      walletAddress: fromAddress,
      walletPubkeyHex: fromPubkey,
      recordKey: eventKey,
      status: LocalTxStore.statusFinalized,
      amountDeltaFen: '-100',
      transferAmountFen: '100',
      fromAddress: fromAddress,
      toAddress: toAddress,
      counterpartyAddress: toAddress,
      blockNumber: 9,
      blockHash: '0x22',
      eventIndex: 3,
    );

    records = await LocalTxStore.queryByWalletPubkey(fromPubkey);
    expect(records, hasLength(1));
    expect(records.single.recordKey, eventKey);
    expect(records.single.status, LocalTxStore.statusFinalized);
    expect(records.single.amountDeltaFen, '-101');
    expect(records.single.txHash, '0xabc');
    expect(records.single.confirmedAtMillis, isNotNull);
  });

  test('区块事件先到时，本机提交记录按同区块同转账合并为一条', () async {
    final eventKey = LocalTxStore.blockEventRecordKey(fromPubkey, '0x44', 2);
    await LocalTxStore.upsertBlockTransferEvent(
      walletAddress: fromAddress,
      walletPubkeyHex: fromPubkey,
      recordKey: eventKey,
      status: LocalTxStore.statusInBlock,
      amountDeltaFen: '-210',
      transferAmountFen: '210',
      fromAddress: fromAddress,
      toAddress: toAddress,
      counterpartyAddress: toAddress,
      blockNumber: 11,
      blockHash: '0x44',
      eventIndex: 2,
    );

    await LocalTxStore.upsertLocalSubmitTransfer(
      walletAddress: fromAddress,
      walletPubkeyHex: fromPubkey,
      txHash: '0xdef',
      amountDeltaFen: '-220',
      transferAmountFen: '210',
      feeFen: '10',
      counterpartyAddress: toAddress,
      fromAddress: fromAddress,
      toAddress: toAddress,
      usedNonce: 8,
      createdAtMillis: 2,
      blockHash: '0x44',
    );

    final records = await LocalTxStore.queryByWalletPubkey(fromPubkey);
    expect(records, hasLength(1));
    expect(records.single.recordKey, eventKey);
    expect(records.single.status, LocalTxStore.statusInBlock);
    expect(records.single.amountDeltaFen, '-220');
    expect(records.single.feeFen, '10');
    expect(records.single.txHash, '0xdef');
  });

  test('收款钱包先写入 inBlock 收入记录，finalized 再升级同一条记录', () async {
    final eventKey = LocalTxStore.blockEventRecordKey(toPubkey, '0x33', 5);
    await LocalTxStore.upsertBlockTransferEvent(
      walletAddress: toAddress,
      walletPubkeyHex: toPubkey,
      recordKey: eventKey,
      status: LocalTxStore.statusInBlock,
      amountDeltaFen: '100',
      transferAmountFen: '100',
      fromAddress: fromAddress,
      toAddress: toAddress,
      counterpartyAddress: fromAddress,
      blockNumber: 10,
      blockHash: '0x33',
      eventIndex: 5,
    );

    var records = await LocalTxStore.queryByWalletPubkey(toPubkey);
    expect(records, hasLength(1));
    expect(records.single.status, LocalTxStore.statusInBlock);
    expect(records.single.amountDeltaFen, '100');
    expect(records.single.confirmedAtMillis, isNull);

    await LocalTxStore.upsertBlockTransferEvent(
      walletAddress: toAddress,
      walletPubkeyHex: toPubkey,
      recordKey: eventKey,
      status: LocalTxStore.statusFinalized,
      amountDeltaFen: '100',
      transferAmountFen: '100',
      fromAddress: fromAddress,
      toAddress: toAddress,
      counterpartyAddress: fromAddress,
      blockNumber: 10,
      blockHash: '0x33',
      eventIndex: 5,
    );

    records = await LocalTxStore.queryByWalletPubkey(toPubkey);
    expect(records, hasLength(1));
    expect(records.single.status, LocalTxStore.statusFinalized);
    expect(records.single.confirmedAtMillis, isNotNull);
  });

  test('同一区块同一收入事件重复处理时只升级状态不新增记录', () async {
    final firstKey = LocalTxStore.blockEventRecordKey(toPubkey, '0x55', 5);
    await LocalTxStore.upsertBlockTransferEvent(
      walletAddress: toAddress,
      walletPubkeyHex: toPubkey,
      recordKey: firstKey,
      status: LocalTxStore.statusInBlock,
      amountDeltaFen: '210',
      transferAmountFen: '210',
      fromAddress: fromAddress,
      toAddress: toAddress,
      counterpartyAddress: fromAddress,
      blockNumber: 12,
      blockHash: '0x55',
      eventIndex: 5,
    );

    final secondKey = LocalTxStore.blockEventRecordKey(toPubkey, '0x55', 6);
    await LocalTxStore.upsertBlockTransferEvent(
      walletAddress: toAddress,
      walletPubkeyHex: toPubkey,
      recordKey: secondKey,
      status: LocalTxStore.statusFinalized,
      amountDeltaFen: '210',
      transferAmountFen: '210',
      fromAddress: fromAddress,
      toAddress: toAddress,
      counterpartyAddress: fromAddress,
      blockNumber: 12,
      blockHash: '0x55',
      eventIndex: 6,
    );

    final records = await LocalTxStore.queryByWalletPubkey(toPubkey);
    expect(records, hasLength(1));
    expect(records.single.recordKey, firstKey);
    expect(records.single.status, LocalTxStore.statusFinalized);
    expect(records.single.amountDeltaFen, '210');
  });

  test('本机提交备注在区块事件合并后保留', () async {
    await LocalTxStore.upsertLocalSubmitTransfer(
      walletAddress: fromAddress,
      walletPubkeyHex: fromPubkey,
      txHash: '0xremark',
      amountDeltaFen: '-110',
      transferAmountFen: '100',
      feeFen: '10',
      counterpartyAddress: toAddress,
      fromAddress: fromAddress,
      toAddress: toAddress,
      usedNonce: 9,
      createdAtMillis: 3,
      remark: '中华联邦创世',
    );

    final eventKey = LocalTxStore.blockEventRecordKey(fromPubkey, '0x66', 4);
    await LocalTxStore.upsertBlockTransferEvent(
      walletAddress: fromAddress,
      walletPubkeyHex: fromPubkey,
      recordKey: eventKey,
      status: LocalTxStore.statusFinalized,
      amountDeltaFen: '-100',
      transferAmountFen: '100',
      fromAddress: fromAddress,
      toAddress: toAddress,
      counterpartyAddress: toAddress,
      blockNumber: 13,
      blockHash: '0x66',
      eventIndex: 4,
    );

    final records = await LocalTxStore.queryByWalletPubkey(fromPubkey);
    expect(records, hasLength(1));
    expect(records.single.recordKey, eventKey);
    expect(records.single.status, LocalTxStore.statusFinalized);
    expect(records.single.remark, '中华联邦创世');
  });
}
