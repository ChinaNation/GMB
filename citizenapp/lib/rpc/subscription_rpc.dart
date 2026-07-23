import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/foundation.dart' show visibleForTesting;
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:polkadart/scale_codec.dart' show ByteOutput;

import 'chain_rpc.dart';
import 'signed_extrinsic_builder.dart';

/// 创作者链上付款档位中的周期价格。
class CreatorPeriodPriceInput {
  const CreatorPeriodPriceInput({
    required this.billingPeriod,
    required this.priceFen,
  });

  final String billingPeriod;
  final BigInt priceFen;
}

/// 创作者覆盖式提交的链上付款档位；名称、说明和权益不进入 SCALE。
class CreatorTierInput {
  const CreatorTierInput({required this.tierId, required this.pricesFen});

  final String tierId;
  final List<CreatorPeriodPriceInput> pricesFen;
}

/// finalized `Subscriptions` 中的平台或创作者付款计划。
class ChainSubscriptionPlan {
  const ChainSubscriptionPlan.platform(this.membershipLevel)
      : kind = 'platform',
        tierId = null,
        billingPeriod = null;

  const ChainSubscriptionPlan.creator(this.tierId, this.billingPeriod)
      : kind = 'creator',
        membershipLevel = null;

  final String kind;
  final String? membershipLevel;
  final String? tierId;
  final String? billingPeriod;
}

/// 链上订阅真态；所有时间字段均为 UTC Unix 毫秒时间戳。
class ChainSubscriptionState {
  const ChainSubscriptionState({
    required this.plan,
    required this.startedAt,
    required this.lastChargedAt,
    required this.lastChargedPriceFen,
    required this.paidUntil,
    required this.status,
    required this.authorizedPriceFen,
    required this.suspendReason,
  });

  final ChainSubscriptionPlan plan;
  final int startedAt;
  final int lastChargedAt;
  final BigInt lastChargedPriceFen;
  final int paidUntil;
  final String status;

  /// 订阅者已授权用于自动续费的价格；创作者改价后据此提示重新签名。
  final BigInt authorizedPriceFen;

  /// 挂起原因：`needReconsent` / `insufficientBalance` / `null`（非挂起态）。
  final String? suspendReason;

  /// Active 与已签名取消但仍在已付周期内的 Cancelled 都继续提供权益；
  /// suspended / creatorPaused 暂停期无权益。
  bool isEffectiveAt(int chainNowMs) =>
      (status == 'active' || status == 'cancelled') && chainNowMs < paidUntil;
}

/// 同一 finalized 区块上的订阅状态与共识时间戳，避免本机时钟参与权益判断。
class FinalizedSubscriptionSnapshot {
  const FinalizedSubscriptionSnapshot({
    required this.state,
    required this.chainNowMs,
    required this.blockHashHex,
  });

  final ChainSubscriptionState? state;
  final int chainNowMs;
  final String blockHashHex;
}

/// finalized `CreatorPlans` 中的单个链上付款档位。
class ChainCreatorTier {
  const ChainCreatorTier({required this.tierId, required this.pricesFen});

  final String tierId;
  final Map<String, BigInt> pricesFen;
}

/// 一次账户签名交易 finalized 后的完整本地证明；Cloudflare 重试只复用这些字节，不再签名。
typedef FinalizedSubscriptionTransaction = ({
  String txHash,
  int usedNonce,
  String blockHashHex,
  String signedExtrinsicHex,
});

/// SquarePost 订阅 SCALE 与标准热钱包 extrinsic 入口。
///
/// CitizenApp 只提交需要账户签名的订阅、取消、换档和创作者档位管理。首次扣款后的
/// 真实公历到期时间及后续自动扣款全部由 runtime 根据共识时间戳完成。
class SubscriptionRpc {
  SubscriptionRpc({ChainRpc? chainRpc}) : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;

  static const int _squarePostPalletIndex = 34;
  static const int _subscribeCallIndex = 1;
  static const int _cancelCallIndex = 2;
  static const int _setCreatorPlansCallIndex = 3;
  static const int _changePlanCallIndex = 4;

  static const int _issuerPlatformTag = 0;
  static const int _issuerCreatorTag = 1;
  static const int _planPlatformTag = 0;
  static const int _planCreatorTag = 1;

  static int membershipLevelByte(String level) => switch (level) {
        'freedom' => 0,
        'democracy' => 1,
        'spark' => 2,
        _ => throw ArgumentError('未知平台会员档：$level'),
      };

  static int billingPeriodByte(String period) => switch (period) {
        'monthly' => 0,
        'quarterly' => 1,
        'yearly' => 2,
        _ => throw ArgumentError('未知订阅周期：$period'),
      };

  Future<FinalizedSubscriptionTransaction> subscribePlatform({
    required String fromSs58Address,
    required Uint8List signerPublicKey,
    required String level,
    required BigInt expectedPriceFen,
    required Future<Uint8List> Function(Uint8List payload) sign,
    TxPoolWatchCallback? onWatchEvent,
  }) =>
      _submitFinalized(
        callData: buildSubscribePlatformCall(
          membershipLevelByte(level),
          expectedPriceFen,
        ),
        fromSs58Address: fromSs58Address,
        signerPublicKey: signerPublicKey,
        sign: sign,
        onWatchEvent: onWatchEvent,
      );

  Future<FinalizedSubscriptionTransaction> subscribeCreator({
    required String fromSs58Address,
    required Uint8List signerPublicKey,
    required String creatorAccountId,
    required String tierId,
    required String billingPeriod,
    required BigInt expectedPriceFen,
    required Future<Uint8List> Function(Uint8List payload) sign,
    TxPoolWatchCallback? onWatchEvent,
  }) =>
      _submitFinalized(
        callData: buildSubscribeCreatorCall(
          accountIdBytes(creatorAccountId),
          tierId,
          billingPeriod,
          expectedPriceFen,
        ),
        fromSs58Address: fromSs58Address,
        signerPublicKey: signerPublicKey,
        sign: sign,
        onWatchEvent: onWatchEvent,
      );

  Future<FinalizedSubscriptionTransaction> cancelPlatform({
    required String fromSs58Address,
    required Uint8List signerPublicKey,
    required Future<Uint8List> Function(Uint8List payload) sign,
    TxPoolWatchCallback? onWatchEvent,
  }) =>
      _submitFinalized(
        callData: buildCancelPlatformCall(),
        fromSs58Address: fromSs58Address,
        signerPublicKey: signerPublicKey,
        sign: sign,
        onWatchEvent: onWatchEvent,
      );

  Future<FinalizedSubscriptionTransaction> cancelCreator({
    required String fromSs58Address,
    required Uint8List signerPublicKey,
    required String creatorAccountId,
    required Future<Uint8List> Function(Uint8List payload) sign,
    TxPoolWatchCallback? onWatchEvent,
  }) =>
      _submitFinalized(
        callData: buildCancelCreatorCall(
          accountIdBytes(creatorAccountId),
        ),
        fromSs58Address: fromSs58Address,
        signerPublicKey: signerPublicKey,
        sign: sign,
        onWatchEvent: onWatchEvent,
      );

  Future<FinalizedSubscriptionTransaction> changePlatformPlan({
    required String fromSs58Address,
    required Uint8List signerPublicKey,
    required String level,
    required BigInt expectedPriceFen,
    required Future<Uint8List> Function(Uint8List payload) sign,
    TxPoolWatchCallback? onWatchEvent,
  }) =>
      _submitFinalized(
        callData: buildChangePlatformPlanCall(
          membershipLevelByte(level),
          expectedPriceFen,
        ),
        fromSs58Address: fromSs58Address,
        signerPublicKey: signerPublicKey,
        sign: sign,
        onWatchEvent: onWatchEvent,
      );

  Future<FinalizedSubscriptionTransaction> changeCreatorPlan({
    required String fromSs58Address,
    required Uint8List signerPublicKey,
    required String creatorAccountId,
    required String tierId,
    required String billingPeriod,
    required BigInt expectedPriceFen,
    required Future<Uint8List> Function(Uint8List payload) sign,
    TxPoolWatchCallback? onWatchEvent,
  }) =>
      _submitFinalized(
        callData: buildChangeCreatorPlanCall(
          accountIdBytes(creatorAccountId),
          tierId,
          billingPeriod,
          expectedPriceFen,
        ),
        fromSs58Address: fromSs58Address,
        signerPublicKey: signerPublicKey,
        sign: sign,
        onWatchEvent: onWatchEvent,
      );

  /// 创作者一次签名覆盖链上付款档位；Cloudflare 保存展示字段时不得再索要业务签名。
  Future<FinalizedSubscriptionTransaction> setCreatorPlans({
    required String fromSs58Address,
    required Uint8List signerPublicKey,
    required List<CreatorTierInput> tiers,
    required Future<Uint8List> Function(Uint8List payload) sign,
    TxPoolWatchCallback? onWatchEvent,
  }) =>
      _submitFinalized(
        callData: buildSetCreatorPlansCall(tiers),
        fromSs58Address: fromSs58Address,
        signerPublicKey: signerPublicKey,
        sign: sign,
        onWatchEvent: onWatchEvent,
      );

  /// 在同一个 finalized 区块读取订阅真态与 `Timestamp.Now`。
  Future<FinalizedSubscriptionSnapshot> fetchSubscriptionSnapshot({
    required String subscriberAccountId,
    String? creatorAccountId,
  }) async {
    final block = await _rpc.fetchFinalizedBlock();
    final blockHashHex = '0x${_hex(block.blockHash)}';
    final subscriptionKey = buildSubscriptionStorageKey(
      accountIdBytes(subscriberAccountId),
      creatorAccountId == null ? null : accountIdBytes(creatorAccountId),
    );
    final timestampKey = buildStorageValueKey('Timestamp', 'Now');
    final values = await Future.wait([
      _rpc.fetchStorageAtBlock('0x${_hex(subscriptionKey)}', blockHashHex),
      _rpc.fetchStorageAtBlock('0x${_hex(timestampKey)}', blockHashHex),
    ]);
    final timestamp = values[1];
    if (timestamp == null || timestamp.length != 8) {
      throw const FormatException('finalized Timestamp.Now 缺失或编码不合法');
    }
    return FinalizedSubscriptionSnapshot(
      state: values[0] == null ? null : decodeSubscriptionState(values[0]!),
      chainNowMs: _readUnsignedLittleEndian(timestamp, 0, 8).toInt(),
      blockHashHex: blockHashHex,
    );
  }

  /// 读取创作者 finalized 链上付款档位；名称等展示字段不在这里出现。
  Future<List<ChainCreatorTier>> fetchCreatorPlans(
      String creatorAccountId) async {
    final account = accountIdBytes(creatorAccountId);
    final key = buildCreatorPlansStorageKey(account);
    final data = await _rpc.fetchStorage('0x${_hex(key)}');
    return data == null ? const <ChainCreatorTier>[] : decodeCreatorPlans(data);
  }

  Future<FinalizedSubscriptionTransaction> _submitFinalized({
    required Uint8List callData,
    required String fromSs58Address,
    required Uint8List signerPublicKey,
    required Future<Uint8List> Function(Uint8List payload) sign,
    TxPoolWatchCallback? onWatchEvent,
  }) async {
    SignedExtrinsicTrace? signedTrace;
    final result = await SignedExtrinsicBuilder(
      chainRpc: _rpc,
      logLabel: 'SubscriptionRpc',
    ).signAndSubmitInBlock(
      callData: callData,
      fromSs58Address: fromSs58Address,
      signerPublicKey: signerPublicKey,
      sign: sign,
      onTrace: (trace) => signedTrace = trace,
      onWatchEvent: onWatchEvent,
      waitForFinalized: true,
    );
    final encoded = signedTrace?.encoded;
    if (encoded == null) {
      throw StateError('订阅交易已 finalized，但本地签名交易证明缺失');
    }
    return (
      txHash: result.txHash,
      usedNonce: result.usedNonce,
      blockHashHex: result.blockHashHex,
      signedExtrinsicHex: '0x${SignedExtrinsicBuilder.hexEncode(encoded)}',
    );
  }

  static Uint8List buildSubscribePlatformCall(
    int levelByte,
    BigInt expectedPriceFen,
  ) {
    final output = _call(_subscribeCallIndex)
      ..pushByte(_issuerPlatformTag)
      ..pushByte(_planPlatformTag)
      ..pushByte(levelByte)
      ..write(_u128LittleEndian(expectedPriceFen));
    return output.toBytes();
  }

  static Uint8List buildSubscribeCreatorCall(
    Uint8List creatorAccountId,
    String tierId,
    String billingPeriod,
    BigInt expectedPriceFen,
  ) {
    final output = _call(_subscribeCallIndex);
    _writeCreatorIssuerAndPlan(
      output,
      creatorAccountId,
      tierId,
      billingPeriod,
    );
    output.write(_u128LittleEndian(expectedPriceFen));
    return output.toBytes();
  }

  static Uint8List buildCancelPlatformCall() =>
      (_call(_cancelCallIndex)..pushByte(_issuerPlatformTag)).toBytes();

  static Uint8List buildCancelCreatorCall(Uint8List creatorAccountId) =>
      (_call(_cancelCallIndex)
            ..pushByte(_issuerCreatorTag)
            ..write(_account32(creatorAccountId)))
          .toBytes();

  static Uint8List buildChangePlatformPlanCall(
    int levelByte,
    BigInt expectedPriceFen,
  ) =>
      (_call(_changePlanCallIndex)
            ..pushByte(_issuerPlatformTag)
            ..pushByte(_planPlatformTag)
            ..pushByte(levelByte)
            ..write(_u128LittleEndian(expectedPriceFen)))
          .toBytes();

  static Uint8List buildChangeCreatorPlanCall(
    Uint8List creatorAccountId,
    String tierId,
    String billingPeriod,
    BigInt expectedPriceFen,
  ) {
    final output = _call(_changePlanCallIndex);
    _writeCreatorIssuerAndPlan(
      output,
      creatorAccountId,
      tierId,
      billingPeriod,
    );
    output.write(_u128LittleEndian(expectedPriceFen));
    return output.toBytes();
  }

  static Uint8List buildSetCreatorPlansCall(List<CreatorTierInput> tiers) {
    final output = _call(_setCreatorPlansCallIndex);
    _writeCompactLength(output, tiers.length);
    for (final tier in tiers) {
      _writeBytes(output, Uint8List.fromList(tier.tierId.codeUnits));
      _writeCompactLength(output, tier.pricesFen.length);
      for (final price in tier.pricesFen) {
        output.pushByte(billingPeriodByte(price.billingPeriod));
        output.write(_u128LittleEndian(price.priceFen));
      }
    }
    return output.toBytes();
  }

  static ByteOutput _call(int callIndex) => ByteOutput()
    ..pushByte(_squarePostPalletIndex)
    ..pushByte(callIndex);

  static void _writeCreatorIssuerAndPlan(
    ByteOutput output,
    Uint8List creatorAccountId,
    String tierId,
    String billingPeriod,
  ) {
    output
      ..pushByte(_issuerCreatorTag)
      ..write(_account32(creatorAccountId))
      ..pushByte(_planCreatorTag);
    _writeBytes(output, Uint8List.fromList(tierId.codeUnits));
    output.pushByte(billingPeriodByte(billingPeriod));
  }

  static Uint8List _account32(Uint8List value) {
    if (value.length != 32) throw ArgumentError('AccountId 必须为 32 字节');
    return value;
  }

  /// 链账户只接受 ADR-040 的规范文本，不在授权或 storage key 路径兼容 SS58。
  @visibleForTesting
  static Uint8List accountIdBytes(String accountId) {
    if (!RegExp(r'^0x[0-9a-f]{64}$').hasMatch(accountId)) {
      throw ArgumentError('account_id 必须为小写 0x + 64 位十六进制');
    }
    return Uint8List.fromList([
      for (var index = 2; index < accountId.length; index += 2)
        int.parse(accountId.substring(index, index + 2), radix: 16),
    ]);
  }

  static void _writeBytes(ByteOutput output, Uint8List value) {
    if (value.isEmpty ||
        value.length > 32 ||
        value.any((byte) => byte > 0x7f)) {
      throw ArgumentError('tier_id 必须为 1-32 字节 ASCII');
    }
    _writeCompactLength(output, value.length);
    output.write(value);
  }

  static void _writeCompactLength(ByteOutput output, int length) {
    if (length < 0 || length >= 64) {
      throw ArgumentError('当前 SCALE 紧凑长度只接受 0-63');
    }
    output.pushByte(length << 2);
  }

  static Uint8List _u128LittleEndian(BigInt value) {
    if (value <= BigInt.zero) throw ArgumentError('订阅金额必须大于 0');
    final out = Uint8List(16);
    var remaining = value;
    for (var index = 0; index < out.length; index++) {
      out[index] = (remaining & BigInt.from(0xff)).toInt();
      remaining >>= 8;
    }
    if (remaining != BigInt.zero) throw ArgumentError('金额超出 u128 范围');
    return out;
  }

  /// `Subscriptions[(subscriber, issuer)]` 的 Blake2_128Concat 单键布局。
  @visibleForTesting
  static Uint8List buildSubscriptionStorageKey(
    Uint8List subscriberAccountId,
    Uint8List? creatorAccountId,
  ) {
    final raw = BytesBuilder(copy: false)
      ..add(_account32(subscriberAccountId))
      ..addByte(
          creatorAccountId == null ? _issuerPlatformTag : _issuerCreatorTag);
    if (creatorAccountId != null) raw.add(_account32(creatorAccountId));
    return _storageMapKey('SquarePost', 'Subscriptions', raw.takeBytes());
  }

  /// `CreatorPlans[creator]` 的 Blake2_128Concat 单键布局。
  @visibleForTesting
  static Uint8List buildCreatorPlansStorageKey(Uint8List creatorAccountId) =>
      _storageMapKey(
          'SquarePost', 'CreatorPlans', _account32(creatorAccountId));

  @visibleForTesting
  static Uint8List buildStorageValueKey(String pallet, String storage) =>
      Uint8List.fromList([
        ...Hasher.twoxx128.hashString(pallet),
        ...Hasher.twoxx128.hashString(storage),
      ]);

  static Uint8List _storageMapKey(
    String pallet,
    String storage,
    Uint8List keyData,
  ) =>
      Uint8List.fromList([
        ...buildStorageValueKey(pallet, storage),
        ...Hasher.blake2b128.hash(keyData),
        ...keyData,
      ]);

  /// 严格解码 `SubscriptionState`；截断、非法枚举和尾随字节直接报错，禁止降级成无订阅。
  @visibleForTesting
  static ChainSubscriptionState decodeSubscriptionState(Uint8List data) {
    final reader = _ScaleReader(data);
    final plan = _readPlan(reader);
    final startedAt = reader.u64();
    final lastChargedAt = reader.u64();
    final lastChargedPriceFen = reader.u128();
    final paidUntil = reader.u64();
    final status = switch (reader.byte()) {
      0 => 'active',
      1 => 'cancelled',
      2 => 'terminated',
      3 => 'suspended',
      4 => 'creatorPaused',
      _ => throw const FormatException('subscription_status 枚举不合法'),
    };
    final authorizedPriceFen = reader.u128();
    final suspendTag = reader.byte();
    final String? suspendReason;
    if (suspendTag == 0) {
      suspendReason = null;
    } else if (suspendTag == 1) {
      suspendReason = switch (reader.byte()) {
        0 => 'needReconsent',
        1 => 'insufficientBalance',
        _ => throw const FormatException('suspend_reason 枚举不合法'),
      };
    } else {
      throw const FormatException('suspend_reason Option 枚举不合法');
    }
    reader.requireEnd();
    if (paidUntil <= lastChargedAt) {
      throw const FormatException('paid_until 必须晚于最近扣款时间');
    }
    return ChainSubscriptionState(
      plan: plan,
      startedAt: startedAt,
      lastChargedAt: lastChargedAt,
      lastChargedPriceFen: lastChargedPriceFen,
      paidUntil: paidUntil,
      status: status,
      authorizedPriceFen: authorizedPriceFen,
      suspendReason: suspendReason,
    );
  }

  /// 严格解码 `CreatorTiers`；价格、周期和 tier_id 全部来自 finalized 链上真态。
  @visibleForTesting
  static List<ChainCreatorTier> decodeCreatorPlans(Uint8List data) {
    final reader = _ScaleReader(data);
    final count = reader.compactLength(max: 10, allowZero: true);
    final tiers = <ChainCreatorTier>[];
    final tierIds = <String>{};
    for (var index = 0; index < count; index++) {
      final tierId = reader.ascii(maxLength: 32);
      if (!tierIds.add(tierId)) {
        throw const FormatException('链上 creator tier_id 重复');
      }
      final priceCount = reader.compactLength(max: 3);
      final prices = <String, BigInt>{};
      for (var priceIndex = 0; priceIndex < priceCount; priceIndex++) {
        final period = switch (reader.byte()) {
          0 => 'monthly',
          1 => 'quarterly',
          2 => 'yearly',
          _ => throw const FormatException('billing_period 枚举不合法'),
        };
        final price = reader.u128();
        if (price <= BigInt.zero || prices.containsKey(period)) {
          throw const FormatException('链上创作者周期价格不合法');
        }
        prices[period] = price;
      }
      tiers.add(ChainCreatorTier(tierId: tierId, pricesFen: prices));
    }
    reader.requireEnd();
    return List.unmodifiable(tiers);
  }

  static ChainSubscriptionPlan _readPlan(_ScaleReader reader) {
    final tag = reader.byte();
    if (tag == _planPlatformTag) {
      final level = switch (reader.byte()) {
        0 => 'freedom',
        1 => 'democracy',
        2 => 'spark',
        _ => throw const FormatException('membership_level 枚举不合法'),
      };
      return ChainSubscriptionPlan.platform(level);
    }
    if (tag == _planCreatorTag) {
      final tierId = reader.ascii(maxLength: 32);
      final period = switch (reader.byte()) {
        0 => 'monthly',
        1 => 'quarterly',
        2 => 'yearly',
        _ => throw const FormatException('billing_period 枚举不合法'),
      };
      return ChainSubscriptionPlan.creator(tierId, period);
    }
    throw const FormatException('subscription plan 枚举不合法');
  }

  static BigInt _readUnsignedLittleEndian(
      Uint8List data, int offset, int length) {
    if (offset < 0 || length < 0 || offset + length > data.length) {
      throw const FormatException('无符号整数 SCALE 数据截断');
    }
    var value = BigInt.zero;
    for (var index = length - 1; index >= 0; index--) {
      value = (value << 8) | BigInt.from(data[offset + index]);
    }
    return value;
  }

  static String _hex(Uint8List bytes) =>
      bytes.map((byte) => byte.toRadixString(16).padLeft(2, '0')).join();
}

class _ScaleReader {
  _ScaleReader(this.data);

  final Uint8List data;
  int offset = 0;

  int byte() {
    if (offset >= data.length) throw const FormatException('SCALE 数据截断');
    return data[offset++];
  }

  int compactLength({required int max, bool allowZero = false}) {
    final first = byte();
    if ((first & 0x03) != 0) {
      throw const FormatException('紧凑长度编码超出当前协议边界');
    }
    final length = first >> 2;
    if ((!allowZero && length == 0) || length > max) {
      throw const FormatException('紧凑长度不合法');
    }
    return length;
  }

  String ascii({required int maxLength}) {
    final length = compactLength(max: maxLength);
    if (offset + length > data.length) {
      throw const FormatException('SCALE 字节串截断');
    }
    final bytes = data.sublist(offset, offset + length);
    offset += length;
    if (bytes.any((value) => value > 0x7f)) {
      throw const FormatException('tier_id 必须为 ASCII');
    }
    return utf8.decode(bytes, allowMalformed: false);
  }

  int u64() =>
      SubscriptionRpc._readUnsignedLittleEndian(data, _take(8), 8).toInt();

  BigInt u128() =>
      SubscriptionRpc._readUnsignedLittleEndian(data, _take(16), 16);

  int _take(int length) {
    final start = offset;
    if (start + length > data.length) {
      throw const FormatException('SCALE 整数截断');
    }
    offset += length;
    return start;
  }

  void requireEnd() {
    if (offset != data.length) throw const FormatException('SCALE 存在尾随字节');
  }
}
