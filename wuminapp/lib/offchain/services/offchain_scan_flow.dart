import 'package:flutter/material.dart';
import 'package:wuminapp_mobile/offchain/pages/offchain_pay_page.dart';
import 'package:wuminapp_mobile/offchain/services/clearing_bank_directory.dart';
import 'package:wuminapp_mobile/qr/pages/qr_scan_page.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

/// 链下扫码支付入口流程。
///
/// 中文注释:
/// - 钱包页 / 交易页只负责放入口按钮,真正的扫码、校验清算行、查询收款方节点
///   与跳转付款确认页都收口在 offchain 功能域。
/// - 扫码结果必须携带 `UserTransferBody.bank`,该字段是收款方清算行 `shenfen_id`。
Future<void> openOffchainScanPaymentFlow({
  required BuildContext context,
  required WalletProfile? wallet,
}) async {
  if (wallet == null) {
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('请先选择付款钱包')),
    );
    return;
  }

  final result = await Navigator.of(context).push<QrScanTransferResult>(
    MaterialPageRoute(
      builder: (_) => const QrScanPage(mode: QrScanMode.transfer),
    ),
  );
  if (result == null || !context.mounted) return;

  if (result.bank == null || result.bank!.isEmpty) {
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('该收款码不支持扫码支付(未绑定清算行)')),
    );
    return;
  }

  const sfidBaseUrl = String.fromEnvironment(
    'SFID_BASE_URL',
    defaultValue: 'http://127.0.0.1:8080',
  );
  final directory = ClearingBankDirectory(sfidBaseUrl: sfidBaseUrl);
  final endpoint = await directory.fetchEndpoint(result.bank!);
  if (!context.mounted) return;
  if (endpoint == null) {
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('收款方清算行尚未声明节点,无法扫码支付')),
    );
    return;
  }

  await Navigator.of(context).push(
    MaterialPageRoute(
      builder: (_) => OffchainClearingPayPage(
        wallet: wallet,
        toAddress: result.toAddress,
        recipientBankShenfenId: result.bank!,
        clearingNodeWssUrl: endpoint.wssUrl,
        sfidBaseUrl: sfidBaseUrl,
        initialAmountYuan: result.amount,
        memo: result.memo,
      ),
    ),
  );
}
