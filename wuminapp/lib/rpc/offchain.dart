/// 链下快捷支付 RPC 调用模块（mock）。
///
/// 对接省储行节点的 offchain_submitSignedTx / offchain_queryTxStatus RPC。
/// 当前为 mock 实现，第2步 node 节点改造完成后替换为真实 WSS 调用。
class OffchainRpc {
  /// 提交签名交易到省储行节点。
  ///
  /// [bankShenfenId] 省储行 shenfen_id（用于定位省储行节点）。
  /// [payerAddress] 付款方地址。
  /// [recipientAddress] 收款方地址。
  /// [amountFen] 金额（分）。
  /// [signature] 交易签名（hex）。
  /// [txId] 交易唯一标识。
  ///
  /// 返回确认回执。
  static Future<OffchainTxReceipt> submitSignedTx({
    required String bankShenfenId,
    required String payerAddress,
    required String recipientAddress,
    required int amountFen,
    required String signature,
    required String txId,
  }) async {
    // mock：模拟省储行确认延迟
    await Future.delayed(const Duration(milliseconds: 500));
    return OffchainTxReceipt(
      txId: txId,
      status: OffchainTxStatus.confirmed,
      message: '省储行已确认（mock）',
    );
  }

  /// 查询链下交易状态。
  static Future<OffchainTxReceipt> queryTxStatus(String txId) async {
    // mock：始终返回已确认
    await Future.delayed(const Duration(milliseconds: 200));
    return OffchainTxReceipt(
      txId: txId,
      status: OffchainTxStatus.confirmed,
      message: '已确认（mock）',
    );
  }

  /// 预估链下交易手续费（元）。
  ///
  /// 费率 1-10 bp（0.01%-0.1%），此处取中间值 5 bp = 0.05%，最低 0.01 元。
  static double estimateOffchainFeeYuan(double amountYuan) {
    const int rateBp = 5;
    final fee = amountYuan * rateBp / 10000;
    return fee < 0.01 ? 0.01 : double.parse(fee.toStringAsFixed(2));
  }
}

/// 链下交易状态。
enum OffchainTxStatus {
  /// 已确认（省储行链下确认，待上链）。
  confirmed,

  /// 已上链（批量打包完成）。
  onchain,

  /// 失败。
  failed,
}

/// 链下交易确认回执。
class OffchainTxReceipt {
  const OffchainTxReceipt({
    required this.txId,
    required this.status,
    this.message,
  });

  final String txId;
  final OffchainTxStatus status;
  final String? message;
}
