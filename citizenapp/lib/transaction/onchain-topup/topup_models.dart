// 稳定币充值购买公民币 · 数据模型(与 Worker /v1/square/topup/* 对齐)。

/// 一条「币 + 链」入金轨(由 Worker config 下发,App 不写死合约)。
class TopupRail {
  const TopupRail({
    required this.token,
    required this.chainId,
    required this.tokenContract,
    required this.tokenDecimals,
    required this.label,
  });

  final String token; // 'USDC' | 'USDT'
  final int chainId;
  final String tokenContract;
  final int tokenDecimals;
  final String label;

  factory TopupRail.fromJson(Map<String, dynamic> json) {
    return TopupRail(
      token: json['token']?.toString() ?? '',
      chainId: _asInt(json['chain_id']),
      tokenContract: json['token_contract']?.toString() ?? '',
      tokenDecimals: _asInt(json['token_decimals']),
      label: json['label']?.toString() ?? '',
    );
  }

  /// WalletConnect 链标识(eip155:{chainId})。
  String get caip2 => 'eip155:$chainId';
}

/// 充值套餐:pay_amount=应付稳定币最小单位(字符串,防溢出);coin_fen=应发公民币分额。
class TopupPackage {
  const TopupPackage({
    required this.packageId,
    required this.payDisplay,
    required this.payAmount,
    required this.coinDisplay,
    required this.coinFen,
  });

  final String packageId;
  final String payDisplay;
  final String payAmount;
  final String coinDisplay;
  final String coinFen;

  BigInt get payAmountValue => BigInt.tryParse(payAmount) ?? BigInt.zero;

  factory TopupPackage.fromJson(Map<String, dynamic> json) {
    return TopupPackage(
      packageId: json['package_id']?.toString() ?? '',
      payDisplay: json['pay_display']?.toString() ?? '',
      payAmount: json['pay_amount']?.toString() ?? '0',
      coinDisplay: json['coin_display']?.toString() ?? '',
      coinFen: json['coin_fen']?.toString() ?? '0',
    );
  }
}

/// GET /topup/config 响应。
class TopupConfig {
  const TopupConfig({
    required this.network,
    required this.recvAddress,
    required this.rails,
    required this.packages,
  });

  final String network;
  final String recvAddress;
  final List<TopupRail> rails;
  final List<TopupPackage> packages;

  factory TopupConfig.fromJson(Map<String, dynamic> json) {
    final rails = json['rails'];
    final packages = json['packages'];
    return TopupConfig(
      network: json['network']?.toString() ?? '',
      recvAddress: json['recv_address']?.toString() ?? '',
      rails: rails is List
          ? rails
              .whereType<Map<String, dynamic>>()
              .map(TopupRail.fromJson)
              .toList(growable: false)
          : const <TopupRail>[],
      packages: packages is List
          ? packages
              .whereType<Map<String, dynamic>>()
              .map(TopupPackage.fromJson)
              .toList(growable: false)
          : const <TopupPackage>[],
    );
  }
}

/// 订单状态:pending=待支付 / paid=已支付 / exception=异常;
/// confirming/notFound 是轮询过渡响应,不是台账业务态。
enum TopupOrderStatus {
  confirming,
  pending,
  paid,
  exception,
  notFound,
  unknown
}

TopupOrderStatus topupOrderStatusFrom(String? raw) {
  switch (raw) {
    case 'confirming':
      return TopupOrderStatus.confirming;
    case 'pending':
      return TopupOrderStatus.pending;
    case 'paid':
      return TopupOrderStatus.paid;
    case 'exception':
      return TopupOrderStatus.exception;
    case 'not_found':
      return TopupOrderStatus.notFound;
    default:
      return TopupOrderStatus.unknown;
  }
}

/// POST /topup/submit 结果。
class TopupSubmitResult {
  const TopupSubmitResult({required this.status, this.orderId});

  final TopupOrderStatus status;
  final String? orderId;

  factory TopupSubmitResult.fromJson(Map<String, dynamic> json) {
    return TopupSubmitResult(
      status: topupOrderStatusFrom(json['status']?.toString()),
      orderId: json['order_id']?.toString(),
    );
  }
}

int _asInt(Object? value) {
  if (value is int) return value;
  if (value is num) return value.toInt();
  return int.tryParse(value?.toString() ?? '') ?? 0;
}
