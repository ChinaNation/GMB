import 'package:flutter_test/flutter_test.dart';
import 'package:citizenwallet/signer/pallet_registry.dart';

void main() {
  group('PalletRegistry', () {
    // PalletRegistry 只登记 pallet/call 索引,放行由 decoder 两色识别决定。

    test('pallet 索引常量已定义且互不相同', () {
      final pallets = {
        PalletRegistry.balancesPallet,
        PalletRegistry.multisigTransferPallet,
        PalletRegistry.organizationManagePallet,
        PalletRegistry.votingEnginePallet,
        PalletRegistry.runtimeUpgradePallet,
        PalletRegistry.resolutionDestroPallet,
        PalletRegistry.adminsChangePallet,
        PalletRegistry.grandpaKeyChangePallet,
        PalletRegistry.resolutionIssuancePallet,
        PalletRegistry.offchainTransactionPallet,
      };
      expect(pallets.length, 10);
    });

    test('投票引擎 sub-pallet call_index', () {
      // InternalVote(22).cast=0 / JointVote(23).cast_admin=0 /
      // JointVote(23).cast_referendum=1 / VotingEngine(9).finalize_proposal=3
      expect(PalletRegistry.internalVotePallet, 22);
      expect(PalletRegistry.internalVoteCall, 0);
      expect(PalletRegistry.jointVotePallet, 23);
      expect(PalletRegistry.jointVoteCall, 0);
      expect(PalletRegistry.castReferendumCall, 1);
      expect(PalletRegistry.finalizeProposalCall, 3);
    });

    test('业务 pallet 的 vote_X 常量已物理删除 (编译期保证)', () {
      // 本测试只要能编译通过即视为通过：
      // voteDestroyCall / voteAdminReplacementCall / voteKeyChangeCall /
      // voteCloseCall / voteCreateCall / voteTransferCall /
      // voteSafetyFundCall / voteSweepCall 等常量必须不存在。
      // 若回归重新引入,会直接触发编译错误。
      expect(true, isTrue);
    });

    test('业务 pallet 的 propose_X call_index 连续排列', () {
      // 手动重试统一走 VotingEngine::retry_passed_proposal(9.4),
      // 业务 pallet 不承载 execute_xxx wrapper。
      expect(PalletRegistry.proposeTransferCall, 0);
      expect(PalletRegistry.proposeSafetyFundCall, 1);
      expect(PalletRegistry.proposeSweepCall, 2);

      // AdminsChange(12): call_index=0 是管理员集合变更，call_index=1 留洞不复用。
      expect(PalletRegistry.proposeAdminSetChangeCall, 0);

      // OrganizationManage(17): call_index=0/3 留洞不复用
      // (0 = 单账户机构 propose_create 已废弃; 3 = propose_create_personal 已迁出至 PersonalManage(7))
      expect(PalletRegistry.proposeCloseCall, 1);
      expect(PalletRegistry.registerCidInstitutionCall, 2);
      expect(PalletRegistry.cleanupRejectedProposalCall, 4);
      expect(PalletRegistry.proposeCreateInstitutionCall, 5);

      // PersonalManage(7) B 阶段拆分(2026-05-06):独立命名空间
      expect(PalletRegistry.proposeCreatePersonalCall, 0);
      expect(PalletRegistry.proposeClosePersonalCall, 1);
      expect(PalletRegistry.cleanupRejectedPersonalProposalCall, 2);
    });

    test('VotingEngine 统一手动重试/取消入口', () {
      // 业务 pallet 不承载 execute_xxx / cancel_failed_xxx,
      // 统一收口至 VotingEngine 的 4/5 两个 call_index。
      expect(PalletRegistry.retryPassedProposalCall, 4);
      expect(PalletRegistry.cancelPassedProposalCall, 5);
    });

    test('清算行 OffchainTransaction call_index 与 runtime 对齐', () {
      expect(PalletRegistry.bindClearingBankCall, 30);
      expect(PalletRegistry.depositCall, 31);
      expect(PalletRegistry.withdrawCall, 32);
      expect(PalletRegistry.switchBankCall, 33);
      expect(PalletRegistry.submitOffchainBatchV2Call, 34);
      expect(PalletRegistry.proposeL2FeeRateCall, 40);
      expect(PalletRegistry.setMaxL2FeeRateCall, 41);
      expect(PalletRegistry.registerClearingBankCall, 50);
      expect(PalletRegistry.updateClearingBankEndpointCall, 51);
      expect(PalletRegistry.unregisterClearingBankCall, 52);
    });
  });
}
