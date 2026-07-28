// CitizenIdentity(pallet 10)自助占号 / 换绑 call data SCALE 布局测试。
//
// 逐字节钉死 pallet/call 前缀、CidNumberBound(BoundedVec<u8>)与 SignatureOf
// (BoundedVec<u8>)编码,以及旧账户换绑授权摘要 signing_message(0x11, ...),
// 防与链端 `self_occupy_cid` / `self_rebind_cid_account` 漂移。

import 'dart:typed_data';

import 'package:citizenapp/rpc/citizen_identity_rpc.dart';
import 'package:citizenapp/signer/signing.dart'
    show kOpSignCidRebind, signingMessage;
import 'package:flutter_test/flutter_test.dart';

void main() {
  // 采用链端 CTZN 金标号做样本(26 字节 ASCII)。
  const cid = 'CN951-CTZN1-539598435-2026';
  final cidBytes = Uint8List.fromList(cid.codeUnits);
  // 26 < 64 ⇒ 单字节 SCALE compact = len << 2 = 104。
  const compactCid = 26 << 2; // 0x68

  group('self_occupy_cid call data', () {
    test('布局 [10,5, compact(len), ...cid.utf8]', () {
      final call = CitizenIdentityRpc.buildSelfOccupyCidCall(cid);
      expect(call[0], 10);
      expect(call[1], 5);
      expect(call[2], compactCid);
      expect(call.sublist(3), cidBytes);
      expect(call.length, 3 + 26);
    });

    test('空 / 超 32 字节 cid 被拒', () {
      expect(() => CitizenIdentityRpc.buildSelfOccupyCidCall(''),
          throwsArgumentError);
      expect(
        () => CitizenIdentityRpc.buildSelfOccupyCidCall('X' * 33),
        throwsArgumentError,
      );
    });
  });

  group('self_rebind_cid_account call data', () {
    final sig = Uint8List(64)..fillRange(0, 64, 0xAB);

    test('布局 [10,9, BoundedVec(cid), BoundedVec(sig=64B)]', () {
      final call = CitizenIdentityRpc.buildSelfRebindCidAccountCall(cid, sig);
      expect(call.sublist(0, 2), <int>[10, 9]);
      expect(call[2], compactCid);
      expect(call.sublist(3, 3 + 26), cidBytes);
      // sig 段:BoundedVec<u8> ⇒ compact(64) 两字节 [0x01,0x01] ++ 64 字节。
      const sigStart = 3 + 26;
      expect(call.sublist(sigStart, sigStart + 2), <int>[0x01, 0x01]);
      expect(call.sublist(sigStart + 2), sig);
      expect(call.length, sigStart + 2 + 64); // 95
    });

    test('非 64 字节签名被拒', () {
      expect(
        () => CitizenIdentityRpc.buildSelfRebindCidAccountCall(
            cid, Uint8List(63)),
        throwsArgumentError,
      );
    });
  });

  group('rebind 旧账户授权摘要', () {
    const newAccount =
        '0x1111111111111111111111111111111111111111111111111111111111111111';
    final newAccountBytes = Uint8List.fromList(
      List<int>.filled(32, 0x11),
    );

    test('op_tag = 0x11 且摘要 = signing_message(0x11, compact(cid)++cid++new32)', () {
      final digest = CitizenIdentityRpc.buildRebindSigningDigest(
        cidNumber: cid,
        newAccountId: newAccount,
      );
      expect(kOpSignCidRebind, 0x11);
      expect(digest.length, 32);

      final payload = <int>[
        compactCid,
        ...cidBytes,
        ...newAccountBytes,
      ];
      final expected =
          signingMessage(opTag: kOpSignCidRebind, scalePayload: payload);
      expect(digest, expected);
    });

    test('非法 newAccountId 文本被拒', () {
      expect(
        () => CitizenIdentityRpc.buildRebindSigningDigest(
          cidNumber: cid,
          newAccountId: 'not-hex',
        ),
        throwsArgumentError,
      );
    });
  });
}
