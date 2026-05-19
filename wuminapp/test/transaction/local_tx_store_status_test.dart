import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/isar/wallet_isar.dart';
import 'package:wuminapp_mobile/transaction/shared/local_tx_store.dart';

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
    final pending = LocalTxEntity()
      ..recordKey = LocalTxStore.pendingRecordKey(fromPubkey, '0xabc')
      ..walletAddress = fromAddress
      ..walletPubkeyHex = fromPubkey
      ..type = 'transfer'
      ..amountDeltaFen = '-101'
      ..transferAmountFen = '100'
      ..feeFen = '1'
      ..counterpartyAddress = toAddress
      ..fromAddress = fromAddress
      ..toAddress = toAddress
      ..status = LocalTxStore.statusPending
      ..source = 'local_submit'
      ..txHash = '0xabc'
      ..createdAtMillis = 1;

    await LocalTxStore.upsert(pending);
    await LocalTxStore.markLocalSubmitInBlock(
      walletPubkeyHex: fromPubkey,
      txHash: '0xabc',
      blockHash: '0x11',
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
}
