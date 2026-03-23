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
      expect(PalletRegistry.isSupported(0), isFalse);
      expect(PalletRegistry.isSupported(999), isFalse);
      expect(PalletRegistry.isSupported(-1), isFalse);
    });

    test('null specVersion 返回 false', () {
      expect(PalletRegistry.isSupported(null), isFalse);
    });

    test('pallet 索引常量已定义且互不相同', () {
      final pallets = {
        PalletRegistry.balancesPallet,
        PalletRegistry.duoqianTransferPowPallet,
        PalletRegistry.votingEngineSystemPallet,
      };
      // 三个 pallet 索引应互不相同
      expect(pallets.length, 3);
    });

    test('call 索引常量已定义', () {
      // Balances
      expect(PalletRegistry.transferKeepAliveCall, isNonNegative);
      // DuoqianTransferPow
      expect(PalletRegistry.proposeTransferCall, isNonNegative);
      expect(PalletRegistry.voteTransferCall, isNonNegative);
      // VotingEngineSystem
      expect(PalletRegistry.jointVoteCall, isNonNegative);
      expect(PalletRegistry.citizenVoteCall, isNonNegative);
    });

    test('supportedSpecVersions 非空', () {
      expect(PalletRegistry.supportedSpecVersions, isNotEmpty);
    });
  });
}
