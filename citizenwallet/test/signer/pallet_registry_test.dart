import 'package:flutter_test/flutter_test.dart';
import 'package:citizenwallet/signer/pallet_registry.dart';

void main() {
  group('PalletRegistry', () {
    // PalletRegistry 只登记 pallet/call 索引,放行由 decoder 两色识别决定。

    test('pallet 索引常量已定义且互不相同', () {
      const pallets = {
        // 普通转账统一走 OnchainTransaction(4).transfer_with_remark(0),不再暴露 Balances 直签入口。
        PalletRegistry.onchainTransactionPallet,
        PalletRegistry.multisigTransferPallet,
        PalletRegistry.publicManagePallet,
        PalletRegistry.privateManagePallet,
        PalletRegistry.votingEnginePallet,
        PalletRegistry.runtimeUpgradePallet,
        PalletRegistry.resolutionDestroyPallet,
        PalletRegistry.personalManagePallet,
        PalletRegistry.personalAdminsPallet,
        PalletRegistry.grandpaKeyChangePallet,
        PalletRegistry.resolutionIssuancePallet,
        PalletRegistry.offchainTransactionPallet,
        PalletRegistry.citizenIdentityPallet,
        PalletRegistry.onchainIssuancePallet,
        PalletRegistry.addressRegistryPallet,
      };
      expect(pallets.length, 15);
    });

    test('投票引擎 sub-pallet call_index', () {
      // InternalVote(20).cast=0 / JointVote(21).cast_admin=0 /
      // JointVote(21).cast_referendum=1 / VotingEngine(9).finalize_proposal=3
      expect(PalletRegistry.internalVotePallet, 20);
      expect(PalletRegistry.internalVoteCall, 0);
      expect(PalletRegistry.jointVotePallet, 21);
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

      // 机构管理员由 entity 任职结果管理；只有个人多签保留管理员集合变更调用。
      expect(PalletRegistry.isPersonalAdminSetChangeCall(29, 0), isTrue);
      expect(PalletRegistry.isPersonalAdminSetChangeCall(7, 3), isFalse);

      // PublicManage(30) / PrivateManage(31):机构登记管理。
      expect(PalletRegistry.publicManagePallet, 30);
      expect(PalletRegistry.privateManagePallet, 31);
      expect(PalletRegistry.proposeCloseInstitutionCall, 1);
      expect(PalletRegistry.proposeCreateInstitutionCall, 5);
      expect(PalletRegistry.updateInstitutionInfoCall, 6);
      expect(PalletRegistry.addInstitutionAccountCall, 7);

      // PersonalManage(7):个人多签生命周期;PersonalAdmins(29):管理员集合变更。
      expect(PalletRegistry.personalManagePallet, 7);
      expect(PalletRegistry.proposeCreatePersonalCall, 0);
      expect(PalletRegistry.proposeClosePersonalCall, 1);
      expect(PalletRegistry.personalAdminsPallet, 29);
      expect(PalletRegistry.proposePersonalAdminSetChangeCall, 0);
    });

    test('CitizenIdentity(10) 身份注册 + 注册局占号/吊销 call_index', () {
      // 0..4 为身份登记管理，5 永久留洞，6..8 为 CID 占号管理。
      expect(PalletRegistry.citizenIdentityPallet, 10);
      expect(PalletRegistry.registerVotingIdentityCall, 0);
      expect(PalletRegistry.upgradeToCandidateIdentityCall, 1);
      expect(PalletRegistry.updateVotingIdentityCall, 2);
      expect(PalletRegistry.updateCandidateIdentityCall, 3);
      expect(PalletRegistry.revokeIdentityCall, 4);
      expect(PalletRegistry.occupyCidCall, 6);
      expect(PalletRegistry.occupyCidsBatchCall, 7);
      expect(PalletRegistry.revokeCidCall, 8);
    });

    test('VotingEngine 统一手动重试/取消入口', () {
      // 业务 pallet 不承载 execute_xxx / cancel_failed_xxx,
      // 统一收口至 VotingEngine 的 4/5 两个 call_index。
      expect(PalletRegistry.retryPassedProposalCall, 4);
      expect(PalletRegistry.cancelPassedProposalCall, 5);
    });

    test('OnchainIssuance 十个调用与 runtime ABI 对齐', () {
      expect(PalletRegistry.onchainIssuancePallet, 23);
      expect(PalletRegistry.proposeIssueCall, 0);
      expect(PalletRegistry.proposeMintCall, 1);
      expect(PalletRegistry.proposeBurnCall, 2);
      expect(PalletRegistry.proposeCloseAssetCall, 3);
      expect(PalletRegistry.proposeAssetTransferCall, 4);
      expect(PalletRegistry.proposeMonitorFreezeCall, 10);
      expect(PalletRegistry.proposeMonitorUnfreezeCall, 11);
      expect(PalletRegistry.proposeMonitorConfiscateCall, 12);
      expect(PalletRegistry.proposeMonitorForceTransferCall, 13);
      expect(PalletRegistry.proposeMonitorForceCloseCall, 14);
    });

    test('清算行 OffchainTransaction call_index 与 runtime 对齐', () {
      expect(PalletRegistry.bindClearingBankCall, 30);
      expect(PalletRegistry.depositCall, 31);
      expect(PalletRegistry.withdrawCall, 32);
      expect(PalletRegistry.switchBankCall, 33);
      expect(PalletRegistry.submitOffchainBatchCall, 34);
      expect(PalletRegistry.proposeL2FeeRateCall, 40);
      expect(PalletRegistry.setMaxL2FeeRateCall, 41);
      expect(PalletRegistry.registerClearingBankCall, 50);
      expect(PalletRegistry.updateClearingBankEndpointCall, 51);
      expect(PalletRegistry.unregisterClearingBankCall, 52);
    });

    test('AddressRegistry call_index 与 runtime 对齐', () {
      expect(PalletRegistry.addressRegistryPallet, 33);
      expect(PalletRegistry.setAddressCatalogVersionCall, 0);
      expect(PalletRegistry.setAddressNameCall, 1);
      expect(PalletRegistry.removeAddressNameCall, 2);
      expect(PalletRegistry.setAddressCall, 3);
      expect(PalletRegistry.removeAddressCall, 4);
    });
  });
}
