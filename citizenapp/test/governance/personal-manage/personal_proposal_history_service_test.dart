// 个人多签提案历史服务单元测试。
//
// 仅覆盖 Isar 持久层路径(`recordOrUpdate` 写 / `fetchAll` 读 / 状态字段映射 /
// snapshot JSON 序列化反序列化)。链上拉取依赖 smoldot,在测试环境下走容错回退,
// 等价于"链上失败 → 仅返回 Isar"路径。

import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/citizen/shared/proposal/proposal_query_service.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/transaction/personal-manage/personal_proposal_history_service.dart';
import '../../support/isar_test_env.dart';

/// 离线 ChainRpc:本单测只验证 Isar 持久层,注入它切断真链依赖——链读一律空、
/// 可达探针为 false(等价「链不可达 → 仅返回本机 Isar」)。避免本机是否连着真链
/// 造成 flaky;配合服务端「链不可达不删幽灵」的容错回退,写入的历史必被完整读回。
class _OfflineChainRpc extends ChainRpc {
  @override
  Future<bool> isFinalizedChainReachable() async => false;

  @override
  Future<Uint8List?> fetchStorage(String storageKeyHex) async => null;
}

PersonalProposalHistoryService _service() => PersonalProposalHistoryService(
      chainRpc: _OfflineChainRpc(),
      proposalService: ProposalQueryService(chainRpc: _OfflineChainRpc()),
    );

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  useIsolatedIsar();

  const personalAccount = 'aabbccddeeff00112233445566778899'
      '00112233445566778899aabbccddeeff';

  test('recordOrUpdate inserts new entity then readAllFromIsar 返回单条', () async {
    final service = _service();
    await service.recordOrUpdate(
      personalAccountHex: personalAccount,
      proposalId: 42,
      action: PersonalProposalAction.create,
      status: PersonalProposalStatus.voting,
      yesVotes: 1,
      noVotes: 0,
      snapshot: {'name': 'TestPersonal', 'amount_fen': '1000'},
    );

    final list = await service.fetchAll(personalAccount);
    expect(list.length, 1);
    expect(list[0].proposalId, 42);
    expect(list[0].action, PersonalProposalAction.create);
    expect(list[0].status, PersonalProposalStatus.voting);
    expect(list[0].yesVotes, 1);
    expect(list[0].noVotes, 0);
    expect(list[0].isActive, isTrue);
    expect(list[0].finalStatusAtMillis, isNull);
    expect(list[0].snapshot?['name'], 'TestPersonal');
    expect(list[0].snapshot?['amount_fen'], '1000');
  });

  test(
      'recordOrUpdate 同 proposalId upsert,createdAt 保留首次值,'
      'snapshot 沿用旧值若新调用未传', () async {
    final service = _service();
    await service.recordOrUpdate(
      personalAccountHex: personalAccount,
      proposalId: 7,
      action: PersonalProposalAction.create,
      status: PersonalProposalStatus.voting,
      yesVotes: 0,
      noVotes: 0,
      snapshot: {'name': 'X'},
    );
    final firstList = await service.fetchAll(personalAccount);
    final firstCreatedAt = firstList[0].createdAtMillis;

    await Future<void>.delayed(const Duration(milliseconds: 5));

    // 二次写入只更新投票计数和状态,不传 snapshot
    await service.recordOrUpdate(
      personalAccountHex: personalAccount,
      proposalId: 7,
      action: PersonalProposalAction.create,
      status: PersonalProposalStatus.executed,
      yesVotes: 3,
      noVotes: 0,
    );
    final list = await service.fetchAll(personalAccount);
    expect(list.length, 1);
    expect(list[0].status, PersonalProposalStatus.executed);
    expect(list[0].yesVotes, 3);
    expect(list[0].finalStatusAtMillis, isNotNull);
    expect(list[0].createdAtMillis, firstCreatedAt,
        reason: '二次 upsert 必须保留首次 createdAt');
    expect(list[0].snapshot?['name'], 'X', reason: 'snapshot 旧值应保留');
  });

  test('voting → executed 转换会写入 finalStatusAtMillis', () async {
    final service = _service();
    await service.recordOrUpdate(
      personalAccountHex: personalAccount,
      proposalId: 1,
      action: PersonalProposalAction.create,
      status: PersonalProposalStatus.voting,
      yesVotes: 0,
      noVotes: 0,
    );
    var list = await service.fetchAll(personalAccount);
    expect(list[0].finalStatusAtMillis, isNull);
    expect(list[0].isFinal, isFalse);

    await service.recordOrUpdate(
      personalAccountHex: personalAccount,
      proposalId: 1,
      action: PersonalProposalAction.create,
      status: PersonalProposalStatus.rejected,
      yesVotes: 0,
      noVotes: 3,
    );
    list = await service.fetchAll(personalAccount);
    expect(list[0].finalStatusAtMillis, isNotNull);
    expect(list[0].isFinal, isTrue);
  });

  test('多个 proposal 按 createdAt desc 排序', () async {
    final service = _service();
    await service.recordOrUpdate(
      personalAccountHex: personalAccount,
      proposalId: 100,
      action: PersonalProposalAction.create,
      status: PersonalProposalStatus.voting,
      yesVotes: 0,
      noVotes: 0,
    );
    await Future<void>.delayed(const Duration(milliseconds: 5));
    await service.recordOrUpdate(
      personalAccountHex: personalAccount,
      proposalId: 101,
      action: PersonalProposalAction.transfer,
      status: PersonalProposalStatus.voting,
      yesVotes: 1,
      noVotes: 0,
    );
    await Future<void>.delayed(const Duration(milliseconds: 5));
    await service.recordOrUpdate(
      personalAccountHex: personalAccount,
      proposalId: 102,
      action: PersonalProposalAction.close,
      status: PersonalProposalStatus.executed,
      yesVotes: 3,
      noVotes: 0,
    );

    final list = await service.fetchAll(personalAccount);
    expect(list.map((v) => v.proposalId).toList(), [102, 101, 100]);
  });

  test('mapChainStatus 链上 u8 → 字符串映射穷尽', () {
    expect(mapChainStatus(0), PersonalProposalStatus.voting);
    expect(mapChainStatus(1), PersonalProposalStatus.passed);
    expect(mapChainStatus(2), PersonalProposalStatus.rejected);
    expect(mapChainStatus(3), PersonalProposalStatus.executed);
    expect(mapChainStatus(4), PersonalProposalStatus.executionFailed);
    expect(mapChainStatus(null), PersonalProposalStatus.voting);
    expect(mapChainStatus(99), PersonalProposalStatus.voting);
  });
}
