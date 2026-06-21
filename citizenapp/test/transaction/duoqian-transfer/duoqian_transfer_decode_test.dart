import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:citizenapp/transaction/duoqian-transfer/duoqian_transfer_service.dart';

/// 批量解码路径（_decodeProposalData）布局回归。
///
/// 2026-06-11 事故:最小长度守卫残留旧 48 字节主体(要求 ≥136),链上真实
/// propose_transfer payload 为 132 字节(机构已是 32 字节 AccountId32),
/// 短备注(<16 字节)提案被静默判为不可解码——广场点击进不去详情、
/// 机构详情列表不显示。本测试用链上抓取的真实字节固化布局。
void main() {
  Uint8List hexToBytes(String hex) {
    final clean = hex.startsWith('0x') ? hex.substring(2) : hex;
    return Uint8List.fromList(List<int>.generate(
      clean.length ~/ 2,
      (i) => int.parse(clean.substring(i * 2, i * 2 + 2), radix: 16),
      growable: false,
    ));
  }

  List<int> compactU32(int value) {
    if (value < 64) return [value << 2];
    final v = (value << 2) | 1;
    return [v & 0xff, (v >> 8) & 0xff];
  }

  final service = DuoqianTransferService();

  // 链上 VotingEngine::ProposalData[0] 实抓字节(2026-06-11,创世 0x49d465…):
  // Compact(132) + "dq-xfer" + institution(32) + beneficiary(32)
  // + amount u128 LE(16) + remark Vec(Compact(12)+「转账测试」) + proposer(32)
  const onchainProposalDataHex = '0x110264712d786665723993'
      '6ebd8564c61f315662ff859d8fb5470ac3f1b4bfbf86746aff391d14db3d'
      '683bc22649494a1f018aba3effe507a0d25acab9b1b835e7407343f45981'
      '0c6380969800000000000000000000000000'
      '30e8bdace8b4a6e6b58be8af95'
      '6832b6f09d231e1ddf34be1a81f68eb958da88be370f999e2d98f3407cb72d05';

  test('链上真实 propose_transfer payload(短备注 132 字节)必须可解码', () {
    final decoded = service.debugDecodeProposalData(
      0,
      hexToBytes(onchainProposalDataHex),
    );

    expect(decoded, isNotNull,
        reason: '旧 48 字节主体长度守卫会把短备注提案静默丢弃(广场点不进/机构列表不显示)');
    expect(
      decoded!.institutionBytes,
      hexToBytes(
          '39936ebd8564c61f315662ff859d8fb5470ac3f1b4bfbf86746aff391d14db3d'),
    );
    expect(decoded.amountFen, BigInt.from(10000000)); // 100,000.00 元
    expect(decoded.remark, '转账测试');
    expect(
      decoded.beneficiary,
      Keyring().encodeAddress(
        hexToBytes(
            '683bc22649494a1f018aba3effe507a0d25acab9b1b835e7407343f459810c63'),
        2027,
      ),
    );
  });

  test('空备注 payload(下限 120 字节)必须可解码', () {
    final body = <int>[
      ...'dq-xfer'.codeUnits,
      ...List<int>.filled(32, 0x11), // institution
      ...List<int>.filled(32, 0x22), // beneficiary
      ...List<int>.filled(16, 0), // amount = 0
      0x00, // remark Compact(0)
      ...List<int>.filled(32, 0x33), // proposer
    ];
    final raw = Uint8List.fromList([...compactU32(body.length), ...body]);

    final decoded = service.debugDecodeProposalData(1, raw);

    expect(decoded, isNotNull);
    expect(decoded!.remark, '');
  });

  test('截断 payload(不足下限)返回 null', () {
    final body = <int>[
      ...'dq-xfer'.codeUnits,
      ...List<int>.filled(32, 0x11),
      ...List<int>.filled(32, 0x22),
      ...List<int>.filled(16, 0),
      0x00,
      ...List<int>.filled(31, 0x33), // proposer 缺 1 字节
    ];
    final raw = Uint8List.fromList([...compactU32(body.length), ...body]);

    expect(service.debugDecodeProposalData(2, raw), isNull);
  });

  test('MODULE_TAG 不符返回 null', () {
    final body = <int>[
      ...'dq-xxxx'.codeUnits,
      ...List<int>.filled(113, 0x11),
    ];
    final raw = Uint8List.fromList([...compactU32(body.length), ...body]);

    expect(service.debugDecodeProposalData(3, raw), isNull);
  });
}
