import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart/polkadart.dart' show RuntimeMetadata;

import 'package:citizenapp/rpc/chain_rpc.dart';

/// 用真实 SCALE 编码的 V14 metadata 覆盖 [ChainRpc.fetchPalletConstant] /
/// [ChainRpc.fetchPalletConstantU128] / [ChainRpc.fetchMinSelfPayBalanceFen] 的
/// **真实解码路径**(`RuntimeMetadata.fromHex` → `constant.type.decode(ByteInput(bytes))`)。
///
/// 之前这条路径只被 fake 跳过(其它测试文件里的 `_FakeChainRpc` 直接覆写
/// `fetchMinSelfPayBalanceFen` 返回固定值),真实解码从未被验证过。
///
/// [_fixtureMetadataHex] 由一次性 Rust 生成器手搭而成:用与本仓库锁定同版本的
/// `frame-metadata=23.0.1` / `scale-info=2.11.6` / `parity-scale-codec=3.7.5`
/// 构造一段真实的 `RuntimeMetadataV14`(非伪造/简化格式),含两个 pallet:
/// - `OnchainTransaction.OnchainMinFee`(u128 = 10)—— 对应链上 `ONCHAIN_MIN_FEE`;
/// - `Balances.ExistentialDeposit`(u128 = 111)—— 对应链上 `ACCOUNT_EXISTENTIAL_DEPOSIT`;
/// - `Balances.SomeFlag`(bool = true)—— 用于覆盖"常量存在但非整数类型"的拒绝分支。
/// 生成器本身是一次性脚本,不进入仓库;只有生成结果(hex)固化进本测试。
const _fixtureMetadataHex =
    '0x6d6574610e0c00000005070004000005000008000004000008484f6e636861696e5472616e73616374696f6e00000004344f6e636861696e4d696e46656500400a0000000000000000000000000000000000002042616c616e63657300000008484578697374656e7469616c4465706f73697400406f0000000000000000000000000000000020536f6d65466c616704040100000108040008';

/// 让 `fetchMetadata()` 直接返回上述真实 metadata,不触碰 smoldot 原生桥
/// (与本仓库既有测试同一模式:只桩原生边界,解码路径全走真实实现)。
class _FixtureChainRpc extends ChainRpc {
  @override
  Future<RuntimeMetadata> fetchMetadata() async =>
      RuntimeMetadata.fromHex(_fixtureMetadataHex);
}

void main() {
  late _FixtureChainRpc rpc;

  setUp(() => rpc = _FixtureChainRpc());

  group('fetchPalletConstant / fetchPalletConstantU128', () {
    test('解出 OnchainTransaction.OnchainMinFee = 10', () async {
      final value = await rpc.fetchPalletConstantU128(
          'OnchainTransaction', 'OnchainMinFee');
      expect(value, BigInt.from(10));
    });

    test('解出 Balances.ExistentialDeposit = 111', () async {
      final value =
          await rpc.fetchPalletConstantU128('Balances', 'ExistentialDeposit');
      expect(value, BigInt.from(111));
    });

    test('pallet 或常量名不存在 → StateError,不静默兜默认值', () async {
      await expectLater(
        rpc.fetchPalletConstant('Balances', 'NotARealConstant'),
        throwsA(isA<StateError>()),
      );
      await expectLater(
        rpc.fetchPalletConstant('NotARealPallet', 'ExistentialDeposit'),
        throwsA(isA<StateError>()),
      );
    });

    test('常量存在但非整数类型 → fetchPalletConstantU128 拒绝', () async {
      await expectLater(
        rpc.fetchPalletConstantU128('Balances', 'SomeFlag'),
        throwsA(isA<StateError>()),
      );
    });
  });

  test('fetchMinSelfPayBalanceFen = OnchainMinFee + ExistentialDeposit = 121',
      () async {
    final total = await rpc.fetchMinSelfPayBalanceFen();
    expect(total, BigInt.from(121));
  });

  group('交易池观察状态的终局语义', () {
    test('只有 invalid 与 usurped 是确定失败', () {
      for (final kind in <TxPoolWatchKind>[
        TxPoolWatchKind.invalid,
        TxPoolWatchKind.usurped,
      ]) {
        expect(
          TxPoolWatchEvent(
            kind: kind,
            description: kind.name,
            raw: kind.name,
          ).isFailure,
          isTrue,
        );
      }
    });

    test('dropped 等停止观察状态必须交给 finalized 业务对账', () {
      for (final kind in <TxPoolWatchKind>[
        TxPoolWatchKind.future,
        TxPoolWatchKind.dropped,
        TxPoolWatchKind.retracted,
        TxPoolWatchKind.finalityTimeout,
        TxPoolWatchKind.timeout,
        TxPoolWatchKind.error,
      ]) {
        expect(
          TxPoolWatchEvent(
            kind: kind,
            description: kind.name,
            raw: kind.name,
          ).isFailure,
          isFalse,
        );
      }
    });
  });
}
