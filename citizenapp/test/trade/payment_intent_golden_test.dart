import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/signer/signing.dart';
import 'package:citizenapp/transaction/offchain-transaction/models/payment_intent.dart';

/// 扫码支付 Step 2c-ii-a 新增:**跨端 golden vectors**。
///
/// 本测试与 `citizenchain/runtime/transaction/offchain-transaction/src/batch_item.rs::tests`
/// 中的 `golden_fixture1/2/3` 三组 fixture **逐字节同步**:相同输入 → 相同
/// SCALE 编码 → 相同 `signing_hash`。任一端实现漂移(字段顺序 / 字节序 /
/// 签名域前缀 / 哈希算法)都会立即断言失败,两端 CI 同时报红。
///
/// **锁定的不变量**:
/// - `NodePaymentIntent` 定长 204 字节 SCALE 布局
/// - 签名经统一原语 `signingMessage(OP_SIGN_L3_PAY=0x15, ...)`(ADR-026,
///   取代历史字符串域 `GMB_L3_PAY_V1`)
/// - `signingHash()` = `blake2b_256(GMB(3B) || 0x15 || scaleEncode())`

void main() {
  group('PaymentIntent golden vectors (cross-Rust)', () {
    test('fixture 1: simple same-bank payment', () {
      final intent = NodePaymentIntent(
        txId: _filledBytes(32, 0x00),
        payer: _filledBytes(32, 0x01),
        payerBank: _filledBytes(32, 0x02),
        recipient: _filledBytes(32, 0x03),
        recipientBank: _filledBytes(32, 0x02),
        amount: BigInt.from(10000),
        fee: BigInt.from(5),
        nonce: BigInt.from(1),
        expiresAt: 100,
      );
      _assertHexEq(
        'fixture1 encoded',
        intent.scaleEncode(),
        '0000000000000000000000000000000000000000000000000000000000000000'
            '0101010101010101010101010101010101010101010101010101010101010101'
            '0202020202020202020202020202020202020202020202020202020202020202'
            '0303030303030303030303030303030303030303030303030303030303030303'
            '0202020202020202020202020202020202020202020202020202020202020202'
            '10270000000000000000000000000000'
            '05000000000000000000000000000000'
            '0100000000000000'
            '64000000',
      );
      _assertHexEq(
        'fixture1 signing_hash',
        intent.signingHash(),
        '19c26c228363e18a119c0a11323bf54a21f9285ce205918f1311f9fa283b63e3',
      );
    });

    test('fixture 2: cross-bank with u128/u64/u32 max values', () {
      final intent = NodePaymentIntent(
        txId: _filledBytes(32, 0xFF),
        payer: _filledBytes(32, 0x11),
        payerBank: _filledBytes(32, 0xAA),
        recipient: _filledBytes(32, 0x22),
        recipientBank: _filledBytes(32, 0xBB),
        amount: _uMax(16), // u128::MAX
        fee: _uMax(16),
        nonce: _uMax(8), // u64::MAX
        expiresAt: 0xFFFFFFFF, // u32::MAX
      );
      _assertHexEq(
        'fixture2 encoded',
        intent.scaleEncode(),
        'ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff'
            '1111111111111111111111111111111111111111111111111111111111111111'
            'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'
            '2222222222222222222222222222222222222222222222222222222222222222'
            'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb'
            'ffffffffffffffffffffffffffffffff'
            'ffffffffffffffffffffffffffffffff'
            'ffffffffffffffff'
            'ffffffff',
      );
      _assertHexEq(
        'fixture2 signing_hash',
        intent.signingHash(),
        '5329809c9803906ae2141be93a3b1cd49bc89adb16a88ca9763fab864df30e90',
      );
    });

    test('fixture 3: zero amount / fee, incrementing tx_id bytes', () {
      final txBytes = Uint8List(32);
      for (var i = 0; i < 32; i++) {
        txBytes[i] = i; // 0x00..0x1F
      }
      final intent = NodePaymentIntent(
        txId: txBytes,
        payer: _filledBytes(32, 0x55),
        payerBank: _filledBytes(32, 0x77),
        recipient: _filledBytes(32, 0x66),
        recipientBank: _filledBytes(32, 0x77),
        amount: BigInt.zero,
        fee: BigInt.zero,
        nonce: BigInt.zero,
        expiresAt: 0,
      );
      _assertHexEq(
        'fixture3 encoded',
        intent.scaleEncode(),
        '000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f'
            '5555555555555555555555555555555555555555555555555555555555555555'
            '7777777777777777777777777777777777777777777777777777777777777777'
            '6666666666666666666666666666666666666666666666666666666666666666'
            '7777777777777777777777777777777777777777777777777777777777777777'
            '00000000000000000000000000000000'
            '00000000000000000000000000000000'
            '0000000000000000'
            '00000000',
      );
      _assertHexEq(
        'fixture3 signing_hash',
        intent.signingHash(),
        'c7fac179287401a2e0f3cb03f60dbf202d7ec48967d8407cd8f96daddcd287bf',
      );
    });

    test('signingHash 经统一原语 signingMessage(OP_SIGN_L3_PAY)', () {
      // 证明 signingHash() 与直调统一原语逐字节一致(域已从 GMB_L3_PAY_V1
      // 折成 GMB(3B)||0x15,ADR-026)。
      final intent = NodePaymentIntent(
        txId: _filledBytes(32, 0x00),
        payer: _filledBytes(32, 0x01),
        payerBank: _filledBytes(32, 0x02),
        recipient: _filledBytes(32, 0x03),
        recipientBank: _filledBytes(32, 0x02),
        amount: BigInt.from(10000),
        fee: BigInt.from(5),
        nonce: BigInt.from(1),
        expiresAt: 100,
      );
      final viaPrimitive = signingMessage(
        opTag: kOpSignL3Pay,
        scalePayload: intent.scaleEncode(),
      );
      expect(_hexLower(intent.signingHash()), _hexLower(viaPrimitive));
    });

    test('scaleEncode length is always 204 bytes', () {
      final intent = NodePaymentIntent(
        txId: _filledBytes(32, 0),
        payer: _filledBytes(32, 0),
        payerBank: _filledBytes(32, 0),
        recipient: _filledBytes(32, 0),
        recipientBank: _filledBytes(32, 0),
        amount: BigInt.zero,
        fee: BigInt.zero,
        nonce: BigInt.zero,
        expiresAt: 0,
      );
      expect(intent.scaleEncode().length, 204);
    });
  });
}

Uint8List _filledBytes(int len, int byte) {
  final out = Uint8List(len);
  for (var i = 0; i < len; i++) {
    out[i] = byte;
  }
  return out;
}

/// (1 << (bytes*8)) - 1,跨越 int64 溢出用 BigInt。
BigInt _uMax(int bytes) => (BigInt.one << (bytes * 8)) - BigInt.one;

String _hexLower(Uint8List bytes) {
  final buf = StringBuffer();
  for (final b in bytes) {
    buf.write(b.toRadixString(16).padLeft(2, '0'));
  }
  return buf.toString();
}

void _assertHexEq(String label, Uint8List actual, String expected) {
  final got = _hexLower(actual);
  expect(got, equals(expected), reason: '$label mismatch');
}
