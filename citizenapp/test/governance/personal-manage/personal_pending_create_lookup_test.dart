// 个人多签待激活创建提案反查单元测试。
//
// 验证:
// - Isar 无 entity → null
// - 仅有非 voting 状态 entity → null(只匹配 voting + create 双条件)
// - 有 voting 状态 entity → 返回其 proposalId
// - 同地址多 entity 时按状态过滤选 create+voting

import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/isar/app_isar.dart';
import 'package:citizenapp/transaction/personal-manage/personal_pending_create_lookup.dart';
import 'package:citizenapp/transaction/personal-manage/personal_proposal_history_service.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  setUpAll(() async {
    await WalletIsar.instance.ensureTestCoreInitialized();
  });

  setUp(() async {
    await WalletIsar.instance.resetForTest();
  });

  const personalAccount = '11223344556677889900aabbccddeeff'
      'ffeeddccbbaa00998877665544332211';

  test('无 entity → 返回 null', () async {
    final lookup = PersonalPendingCreateLookup();
    expect(await lookup.findActiveCreate(personalAccount), isNull);
  });

  test('仅有 executed/rejected entity → 不命中,返回 null', () async {
    final service = PersonalProposalHistoryService();
    await service.recordOrUpdate(
      personalAccountHex: personalAccount,
      proposalId: 5,
      action: PersonalProposalAction.create,
      status: PersonalProposalStatus.executed,
      yesVotes: 3,
      noVotes: 0,
    );
    await service.recordOrUpdate(
      personalAccountHex: personalAccount,
      proposalId: 6,
      action: PersonalProposalAction.create,
      status: PersonalProposalStatus.rejected,
      yesVotes: 0,
      noVotes: 3,
    );

    final lookup = PersonalPendingCreateLookup();
    expect(await lookup.findActiveCreate(personalAccount), isNull);
  });

  test('voting 状态的 create entity 命中,返回 proposalId', () async {
    final service = PersonalProposalHistoryService();
    await service.recordOrUpdate(
      personalAccountHex: personalAccount,
      proposalId: 99,
      action: PersonalProposalAction.create,
      status: PersonalProposalStatus.voting,
      yesVotes: 1,
      noVotes: 0,
    );

    final lookup = PersonalPendingCreateLookup();
    expect(await lookup.findActiveCreate(personalAccount), 99);
  });

  test('voting 但 action != create(如 transfer)不命中', () async {
    final service = PersonalProposalHistoryService();
    await service.recordOrUpdate(
      personalAccountHex: personalAccount,
      proposalId: 200,
      action: PersonalProposalAction.transfer,
      status: PersonalProposalStatus.voting,
      yesVotes: 0,
      noVotes: 0,
    );

    final lookup = PersonalPendingCreateLookup();
    expect(await lookup.findActiveCreate(personalAccount), isNull);
  });

  test('其他多签账户的 entity 不命中(filter 按地址过滤)', () async {
    final service = PersonalProposalHistoryService();
    const otherAddress =
        'deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef';
    await service.recordOrUpdate(
      personalAccountHex: otherAddress,
      proposalId: 77,
      action: PersonalProposalAction.create,
      status: PersonalProposalStatus.voting,
      yesVotes: 0,
      noVotes: 0,
    );

    final lookup = PersonalPendingCreateLookup();
    expect(await lookup.findActiveCreate(personalAccount), isNull);
    expect(await lookup.findActiveCreate(otherAddress), 77);
  });
}
