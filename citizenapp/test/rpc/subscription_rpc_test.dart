import 'dart:typed_data';

import 'package:citizenapp/rpc/subscription_rpc.dart';
import 'package:flutter_test/flutter_test.dart';

String _hex(Uint8List bytes) =>
    bytes.map((byte) => byte.toRadixString(16).padLeft(2, '0')).join();

Uint8List _bytes(String hex) => Uint8List.fromList([
      for (var index = 0; index < hex.length; index += 2)
        int.parse(hex.substring(index, index + 2), radix: 16),
    ]);

void main() {
  final creator = Uint8List(32)..fillRange(0, 32, 2);
  const creatorAccountHex =
      '0202020202020202020202020202020202020202020202020202020202020202';

  group('SquarePost 订阅 SCALE', () {
    test('AccountId 文本只接受小写 0x + 64 位十六进制', () {
      const accountId = '0x$creatorAccountHex';
      expect(
        _hex(SubscriptionRpc.accountIdBytes(accountId)),
        creatorAccountHex,
      );
      expect(
        () => SubscriptionRpc.accountIdBytes(creatorAccountHex),
        throwsArgumentError,
      );
      expect(
        () => SubscriptionRpc.accountIdBytes(
          '0xABABABABABABABABABABABABABABABAB'
          'ABABABABABABABABABABABABABABABAB',
        ),
        throwsArgumentError,
      );
      expect(
        () => SubscriptionRpc.accountIdBytes(
            '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY'),
        throwsArgumentError,
      );
    });

    test('平台订阅携带当前签名价', () {
      final call = SubscriptionRpc.buildSubscribePlatformCall(
        SubscriptionRpc.membershipLevelByte('spark'),
        BigInt.from(5999900),
      );
      expect(
        _hex(call),
        '22010000021c8d5b00000000000000000000000000',
      );
    });

    test('创作者订阅只携带账户、tier_id、自然周期和当前签名价', () {
      final call = SubscriptionRpc.buildSubscribeCreatorCall(
        creator,
        'supporter',
        'monthly',
        BigInt.from(50),
      );
      expect(
        _hex(call),
        '220101${creatorAccountHex}0124737570706f7274657200'
        '32000000000000000000000000000000',
      );
    });

    test('取消只携带收款主体', () {
      expect(_hex(SubscriptionRpc.buildCancelPlatformCall()), '220200');
    });

    test('创作者套餐覆盖式编码', () {
      final call = SubscriptionRpc.buildSetCreatorPlansCall([
        CreatorTierInput(
          tierId: 'supporter',
          pricesFen: [
            CreatorPeriodPriceInput(
              billingPeriod: 'monthly',
              priceFen: BigInt.from(50),
            ),
          ],
        ),
      ]);
      expect(
        _hex(call),
        '22030424737570706f727465720400'
        '32000000000000000000000000000000',
      );
    });

    test('平台与创作者换档使用同一 call_index 并携带目标签名价', () {
      expect(
        _hex(SubscriptionRpc.buildChangePlatformPlanCall(1, BigInt.from(50))),
        '220400000132000000000000000000000000000000',
      );
      expect(
        _hex(SubscriptionRpc.buildChangeCreatorPlanCall(
          creator,
          'supporter',
          'yearly',
          BigInt.from(500),
        )),
        '220401${creatorAccountHex}0124737570706f7274657202'
        'f4010000000000000000000000000000',
      );
    });

    test('非法周期、空 tier_id 和非正价格被拒绝', () {
      expect(() => SubscriptionRpc.billingPeriodByte('daily'),
          throwsArgumentError);
      expect(
        () => SubscriptionRpc.buildSubscribeCreatorCall(
          creator,
          '',
          'monthly',
          BigInt.one,
        ),
        throwsArgumentError,
      );
      expect(
        () => SubscriptionRpc.buildSubscribePlatformCall(0, BigInt.zero),
        throwsArgumentError,
      );
    });
  });

  group('SquarePost finalized storage 解码', () {
    test('严格解码平台订阅真态与时间戳', () {
      // 与 runtime 金标 state_platform 逐字节一致（新布局：无 pending_plan，
      // 末尾含 authorized_price_fen + suspend_reason）。
      const stateHex =
          '00020068e5cf8b0100000068e5cf8b0100001c8d5b0000000000000000000000000000fc1a478c010000001c8d5b0000000000000000000000000000';
      final state = SubscriptionRpc.decodeSubscriptionState(_bytes(stateHex));
      expect(state.plan.kind, 'platform');
      expect(state.plan.membershipLevel, 'spark');
      expect(state.startedAt, 1700000000000);
      expect(state.lastChargedPriceFen, BigInt.from(5999900));
      expect(state.paidUntil, 1702000000000);
      expect(state.status, 'active');
      expect(state.authorizedPriceFen, BigInt.from(5999900));
      expect(state.suspendReason, isNull);
      expect(state.isEffectiveAt(1701000000000), isTrue);
    });

    test('解码挂起态（创作者改价待再签名）与创作者暂停态，暂停期无权益', () {
      // 由 active 向量改：status 字节 00→03、suspend_reason 00→0100（Some(NeedReconsent)）。
      const suspendedHex =
          '00020068e5cf8b0100000068e5cf8b0100001c8d5b0000000000000000000000000000fc1a478c010000031c8d5b000000000000000000000000000100';
      final suspended =
          SubscriptionRpc.decodeSubscriptionState(_bytes(suspendedHex));
      expect(suspended.status, 'suspended');
      expect(suspended.suspendReason, 'needReconsent');
      expect(suspended.isEffectiveAt(1701000000000), isFalse);

      // status 字节 00→04（CreatorPaused），suspend_reason 仍 00（None）。
      const creatorPausedHex =
          '00020068e5cf8b0100000068e5cf8b0100001c8d5b0000000000000000000000000000fc1a478c010000041c8d5b0000000000000000000000000000';
      final creatorPaused =
          SubscriptionRpc.decodeSubscriptionState(_bytes(creatorPausedHex));
      expect(creatorPaused.status, 'creatorPaused');
      expect(creatorPaused.suspendReason, isNull);
      expect(creatorPaused.isEffectiveAt(1701000000000), isFalse);
    });

    test('严格解码创作者链上档位', () {
      const price50 = '32000000000000000000000000000000';
      const price500 = 'f4010000000000000000000000000000';
      final tiers = SubscriptionRpc.decodeCreatorPlans(
        _bytes('0424737570706f727465720800${price50}02$price500'),
      );
      expect(tiers, hasLength(1));
      expect(tiers.single.tierId, 'supporter');
      expect(tiers.single.pricesFen, {
        'monthly': BigInt.from(50),
        'yearly': BigInt.from(500),
      });
    });

    test('非法枚举、截断和尾随字节必须报错', () {
      expect(
        () => SubscriptionRpc.decodeSubscriptionState(_bytes('0003')),
        throwsFormatException,
      );
      expect(
        () => SubscriptionRpc.decodeCreatorPlans(_bytes('0000')),
        throwsFormatException,
      );
    });

    test('Subscriptions 与 CreatorPlans storage key 使用不同真源项', () {
      final subscriptionKey =
          SubscriptionRpc.buildSubscriptionStorageKey(creator, null);
      final creatorPlansKey =
          SubscriptionRpc.buildCreatorPlansStorageKey(creator);
      expect(subscriptionKey.length, 81);
      expect(creatorPlansKey.length, 80);
      expect(_hex(subscriptionKey), isNot(_hex(creatorPlansKey)));
    });
  });
}
