import 'dart:typed_data';

import 'package:citizenapp/rpc/subscription_rpc.dart';
import 'package:flutter_test/flutter_test.dart';

String _hex(Uint8List bytes) =>
    bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();

void main() {
  // 与链端金标向量 subscription_scale_vectors.json 对齐（creator=0x02*32, price=599900）：
  //   Creator issuer = 01 + 02*32；CreatorPrice = 01 + u128LE(599900)=5c2709...；pallet=34=0x22。
  final account = Uint8List(32)..fillRange(0, 32, 2);
  const acctHex =
      '0202020202020202020202020202020202020202020202020202020202020202';
  const priceLe = '5c270900000000000000000000000000'; // u128LE(599900)

  test('subscribe(Creator, CreatorPrice) 字节对齐金标向量', () {
    final call =
        SubscriptionRpc.buildSubscribeCreatorCall(account, BigInt.from(599900));
    // [34][1][01][acct32][01][priceLE]
    expect(_hex(call), '220101${acctHex}01$priceLe');
  });

  test('cancel(Creator) 字节对齐', () {
    final call = SubscriptionRpc.buildCancelCreatorCall(account);
    // [34][2][01][acct32]
    expect(_hex(call), '220201$acctHex');
  });

  test('订阅金额必须为正', () {
    expect(
      () => SubscriptionRpc.buildSubscribeCreatorCall(account, BigInt.zero),
      throwsArgumentError,
    );
  });

  // 平台轨金标向量（subscription_scale_vectors.json）：
  //   subscribe = [34][1][Platform=00][Level=00][MembershipLevel]；cancel = [34][2][00]。
  test('subscribe(Platform, Level(spark)) 字节对齐金标向量', () {
    final call = SubscriptionRpc.buildSubscribePlatformCall(
      SubscriptionRpc.membershipLevelByte('spark'),
    );
    // calldata_subscribe_platform_spark = 2201000002
    expect(_hex(call), '2201000002');
  });

  test('subscribe(Platform) 三档字节', () {
    expect(
      _hex(SubscriptionRpc.buildSubscribePlatformCall(
          SubscriptionRpc.membershipLevelByte('freedom'))),
      '2201000000',
    );
    expect(
      _hex(SubscriptionRpc.buildSubscribePlatformCall(
          SubscriptionRpc.membershipLevelByte('democracy'))),
      '2201000001',
    );
  });

  test('cancel(Platform) 字节对齐', () {
    // call_cancel_platform=0200 → 带 pallet 前缀 = 220200
    expect(_hex(SubscriptionRpc.buildCancelPlatformCall()), '220200');
  });

  test('未知平台会员档抛 ArgumentError', () {
    expect(
      () => SubscriptionRpc.membershipLevelByte('gold'),
      throwsArgumentError,
    );
  });
}
