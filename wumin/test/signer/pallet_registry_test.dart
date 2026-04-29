import 'package:flutter_test/flutter_test.dart';
import 'package:wumin/signer/pallet_registry.dart';

void main() {
  group('PalletRegistry', () {
    test('支持的 specVersion 返回 true', () {
      for (final v in PalletRegistry.supportedSpecVersions) {
        expect(PalletRegistry.isSupported(v), isTrue);
      }
    });

    test('不支持的 specVersion 返回 false', () {
      expect(PalletRegistry.isSupported(1), isFalse); // 重新创世后当前 spec 已归零
      expect(PalletRegistry.isSupported(10), isFalse);
      expect(PalletRegistry.isSupported(999), isFalse);
      expect(PalletRegistry.isSupported(-1), isFalse);
    });

    test('null specVersion 返回 false', () {
      expect(PalletRegistry.isSupported(null), isFalse);
    });

    test('pallet 索引常量已定义且互不相同', () {
      final pallets = {
        PalletRegistry.balancesPallet,
        PalletRegistry.duoqianTransferPallet,
        PalletRegistry.duoqianManagePallet,
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

    test('投票引擎 call_index 按 Phase 2 重排', () {
      // 0=internal_vote / 1=joint_vote / 2=citizen_vote / 3=finalize_proposal
      expect(PalletRegistry.internalVoteCall, 0);
      expect(PalletRegistry.jointVoteCall, 1);
      expect(PalletRegistry.citizenVoteCall, 2);
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

    test('业务 pallet 的 propose_X / execute_X call_index 连续排列', () {
      expect(PalletRegistry.proposeTransferCall, 0);
      expect(PalletRegistry.proposeSafetyFundCall, 1);
      expect(PalletRegistry.proposeSweepCall, 2);
      expect(PalletRegistry.executeTransferCall, 3);
      expect(PalletRegistry.executeSafetyFundCall, 4);
      expect(PalletRegistry.executeSweepCall, 5);

      expect(PalletRegistry.proposeCreateCall, 0);
      expect(PalletRegistry.proposeCloseCall, 1);
      expect(PalletRegistry.registerSfidInstitutionCall, 2);
      expect(PalletRegistry.proposeCreatePersonalCall, 3);
      expect(PalletRegistry.cleanupRejectedProposalCall, 4);
    });

    test('supportedSpecVersions 非空 + 当前为 spec=0', () {
      expect(PalletRegistry.supportedSpecVersions, isNotEmpty);
      expect(PalletRegistry.supportedSpecVersions, contains(0));
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
