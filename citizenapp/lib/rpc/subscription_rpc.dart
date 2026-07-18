import 'dart:typed_data';

import 'package:polkadart/scale_codec.dart' show ByteOutput;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import 'chain_rpc.dart';
import 'signed_extrinsic_builder.dart';

/// square-post 会员订阅上链 RPC：订阅 / 取消**平台会员**与**创作者会员**（热签标准
/// extrinsic）。
///
/// SCALE 布局逐字节对齐链端金标向量（`subscription_scale_vectors.json`，pallet=34）：
///   平台订阅   = [34][1][IssuerKey::Platform=00][SubscriptionPlan::Level=00+MembershipLevel]
///   平台取消   = [34][2][IssuerKey::Platform=00]
///   创作者订阅 = [34][1][IssuerKey::Creator=01+32B][SubscriptionPlan::CreatorPrice=01+u128LE]
///   创作者取消 = [34][2][IssuerKey::Creator=01+32B]
/// 订阅=授权按月自动扣款、取消=撤销授权，二者都必须用户签名（读硬件金库弹一次生物识别）；
/// 按月续扣由 keeper 依此授权 `charge_due` 拉取，不逐月再签。平台档价格全在链上
/// `PlatformPrice[level]` 单源存储，客户端不再随 call 传价（与创作者自定价档不同）。
class SubscriptionRpc {
  SubscriptionRpc({ChainRpc? chainRpc}) : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;

  static const int _squarePostPalletIndex = 34;
  static const int _subscribeCallIndex = 1;
  static const int _cancelCallIndex = 2;

  /// `IssuerKey::Platform` 变体标签（Platform=0x00 / Creator=0x01+32B）。
  static const int _issuerPlatformTag = 0;

  /// `IssuerKey::Creator` 变体标签（Platform=0x00 / Creator=0x01+32B）。
  static const int _issuerCreatorTag = 1;

  /// `SubscriptionPlan::Level` 变体标签（Level=0x00+MembershipLevel / CreatorPrice=0x01+u128LE）。
  static const int _planLevelTag = 0;

  /// `SubscriptionPlan::CreatorPrice` 变体标签（Level=0x00 / CreatorPrice=0x01+u128LE）。
  static const int _planCreatorPriceTag = 1;

  /// 平台会员档字符串 → `MembershipLevel` 单字节（Freedom=0/Democracy=1/Spark=2）。
  static int membershipLevelByte(String level) => switch (level) {
        'freedom' => 0,
        'democracy' => 1,
        'spark' => 2,
        _ => throw ArgumentError('未知平台会员档：$level'),
      };

  /// 订阅平台会员某档：`subscribe(Platform, Level(level))`。
  Future<({String txHash, int usedNonce, String blockHashHex})>
      subscribePlatform({
    required String fromAddress,
    required Uint8List signerPubkey,
    required String level,
    required Future<Uint8List> Function(Uint8List payload) sign,
    TxPoolWatchCallback? onWatchEvent,
  }) {
    final callData = buildSubscribePlatformCall(membershipLevelByte(level));
    return SignedExtrinsicBuilder(chainRpc: _rpc, logLabel: 'SubscriptionRpc')
        .signAndSubmitInBlock(
      callData: callData,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
      onWatchEvent: onWatchEvent,
    );
  }

  /// 取消平台会员：`cancel(Platform)`。
  Future<({String txHash, int usedNonce, String blockHashHex})> cancelPlatform({
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
    TxPoolWatchCallback? onWatchEvent,
  }) {
    final callData = buildCancelPlatformCall();
    return SignedExtrinsicBuilder(chainRpc: _rpc, logLabel: 'SubscriptionRpc')
        .signAndSubmitInBlock(
      callData: callData,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
      onWatchEvent: onWatchEvent,
    );
  }

  /// [34][1][00][00][levelByte]
  static Uint8List buildSubscribePlatformCall(int levelByte) {
    final output = ByteOutput();
    output.pushByte(_squarePostPalletIndex);
    output.pushByte(_subscribeCallIndex);
    output.pushByte(_issuerPlatformTag);
    output.pushByte(_planLevelTag);
    output.pushByte(levelByte);
    return output.toBytes();
  }

  /// [34][2][00]
  static Uint8List buildCancelPlatformCall() {
    final output = ByteOutput();
    output.pushByte(_squarePostPalletIndex);
    output.pushByte(_cancelCallIndex);
    output.pushByte(_issuerPlatformTag);
    return output.toBytes();
  }

  /// 订阅创作者会员：`subscribe(Creator(creator), CreatorPrice(priceFen))`。
  Future<({String txHash, int usedNonce, String blockHashHex})>
      subscribeCreator({
    required String fromAddress,
    required Uint8List signerPubkey,
    required String creatorAddress,
    required BigInt priceFen,
    required Future<Uint8List> Function(Uint8List payload) sign,
    TxPoolWatchCallback? onWatchEvent,
  }) {
    final creatorAccount = Keyring().decodeAddress(creatorAddress);
    final callData = buildSubscribeCreatorCall(creatorAccount, priceFen);
    return SignedExtrinsicBuilder(chainRpc: _rpc, logLabel: 'SubscriptionRpc')
        .signAndSubmitInBlock(
      callData: callData,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
      onWatchEvent: onWatchEvent,
    );
  }

  /// 取消创作者会员：`cancel(Creator(creator))`。
  Future<({String txHash, int usedNonce, String blockHashHex})> cancelCreator({
    required String fromAddress,
    required Uint8List signerPubkey,
    required String creatorAddress,
    required Future<Uint8List> Function(Uint8List payload) sign,
    TxPoolWatchCallback? onWatchEvent,
  }) {
    final creatorAccount = Keyring().decodeAddress(creatorAddress);
    final callData = buildCancelCreatorCall(creatorAccount);
    return SignedExtrinsicBuilder(chainRpc: _rpc, logLabel: 'SubscriptionRpc')
        .signAndSubmitInBlock(
      callData: callData,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
      onWatchEvent: onWatchEvent,
    );
  }

  /// [34][1][01][creator32B][01][priceFen u128LE]
  static Uint8List buildSubscribeCreatorCall(
      Uint8List creatorAccount, BigInt priceFen) {
    final output = ByteOutput();
    output.pushByte(_squarePostPalletIndex);
    output.pushByte(_subscribeCallIndex);
    output.pushByte(_issuerCreatorTag);
    output.write(creatorAccount);
    output.pushByte(_planCreatorPriceTag);
    output.write(_u128LittleEndian(priceFen));
    return output.toBytes();
  }

  /// [34][2][01][creator32B]
  static Uint8List buildCancelCreatorCall(Uint8List creatorAccount) {
    final output = ByteOutput();
    output.pushByte(_squarePostPalletIndex);
    output.pushByte(_cancelCallIndex);
    output.pushByte(_issuerCreatorTag);
    output.write(creatorAccount);
    return output.toBytes();
  }

  static Uint8List _u128LittleEndian(BigInt value) {
    if (value <= BigInt.zero) {
      throw ArgumentError('订阅金额必须大于 0');
    }
    final out = Uint8List(16);
    var remaining = value;
    for (var i = 0; i < out.length; i++) {
      out[i] = (remaining & BigInt.from(0xff)).toInt();
      remaining = remaining >> 8;
    }
    if (remaining != BigInt.zero) {
      throw ArgumentError('金额超出 u128 范围');
    }
    return out;
  }
}
