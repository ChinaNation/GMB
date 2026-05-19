import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/governance/shared/proposal/proposal_detail_local_store.dart';
import 'package:wuminapp_mobile/isar/wallet_isar.dart';
import 'package:wuminapp_mobile/transaction/shared/account_balance_snapshot_store.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  setUpAll(() async {
    await WalletIsar.instance.ensureTestCoreInitialized();
  });

  setUp(() async {
    await WalletIsar.instance.resetForTest();
  });

  test('提案详情快照可持久化管理员投票和业务详情', () async {
    final snapshot = ProposalDetailSnapshot(
      proposalId: 77,
      typeKey: 'transfer',
      updatedAtMillis: DateTime.now().millisecondsSinceEpoch,
      status: 0,
      yesCount: 2,
      noCount: 1,
      threshold: 6,
      admins: const ['aa', 'bb'],
      adminVotes: const {'aa': true, 'bb': null},
      pendingPubkeys: const ['bb'],
      detail: const {
        'kind': 'transfer',
        'amount_fen': '12300',
      },
    );

    await ProposalDetailLocalStore.instance.put(snapshot);

    final loaded = await ProposalDetailLocalStore.instance.read('transfer', 77);

    expect(loaded, isNotNull);
    expect(loaded!.proposalId, 77);
    expect(loaded.adminVotes['aa'], isTrue);
    expect(loaded.adminVotes['bb'], isNull);
    expect(loaded.detail['amount_fen'], '12300');
    expect(loaded.isFresh(ProposalDetailLocalStore.activeTtl), isTrue);
  });

  test('账户余额快照只作为展示缓存读取', () async {
    await AccountBalanceSnapshotStore.instance.put(
      accountHex: '0xABCDEF',
      balanceYuan: 12.34,
    );

    final loaded = await AccountBalanceSnapshotStore.instance.readFresh(
      'abcdef',
    );

    expect(loaded, isNotNull);
    expect(loaded!.accountHex, 'abcdef');
    expect(loaded.balanceYuan, 12.34);
  });
}
