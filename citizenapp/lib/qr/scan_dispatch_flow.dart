import 'package:flutter/material.dart';

import 'package:citizenapp/qr/pages/qr_scan_page.dart';
import 'package:citizenapp/qr/pages/qr_sign_response_page.dart';
import 'package:citizenapp/signer/square_action_sign_service.dart';
import 'package:citizenapp/transaction/offchain-transaction/services/offchain_scan_flow.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 交易 tab「扫一扫」统一入口：扫码 → 按协议分派。
///
/// - 收款 / 链下支付码 → 现有链下支付流程（用交易页选的 [paymentWallet]）。
/// - 广场账户动作 signRequest → 签名响应方（用 QR `u` 对应的 owner 钱包，与付款钱包无关）。
/// - 未来其它类型只需在此加分支。
Future<void> openScanDispatchFlow({
  required BuildContext context,
  required WalletProfile? paymentWallet,
}) async {
  final scanned = await Navigator.of(context).push<Object?>(
    MaterialPageRoute(builder: (_) => const QrScanPage(mode: QrScanMode.dispatch)),
  );
  if (scanned == null || !context.mounted) return;

  if (scanned is QrScanTransferResult) {
    // 支付分支：此处才要求付款钱包（签名分支不需要）。
    if (paymentWallet == null) {
      _snack(context, '请先选择付款钱包');
      return;
    }
    await proceedOffchainPayment(
      context: context,
      wallet: paymentWallet,
      result: scanned,
    );
    return;
  }
  if (scanned is String) {
    await _handleSquareActionSignRequest(context, scanned);
  }
}

Future<void> _handleSquareActionSignRequest(BuildContext context, String raw) async {
  final service = SquareActionSignService();
  final walletManager = WalletManager();

  final SquareActionSignPrep prep;
  try {
    prep = await service.prepare(raw, walletManager);
  } on SquareActionSignException catch (e) {
    if (context.mounted) _snack(context, e.message);
    return;
  }
  if (!context.mounted) return;

  final confirmed = await _showActionConfirm(context, prep);
  if (confirmed != true || !context.mounted) return;

  final String responseJson;
  try {
    // 动钱动权 → 读硬件金库、弹一次生物识别。
    responseJson = await service.sign(prep, walletManager);
  } on WalletAuthException catch (e) {
    if (context.mounted) _snack(context, '签名已取消：${e.message}');
    return;
  }
  if (!context.mounted) return;

  await Navigator.of(context).push(
    MaterialPageRoute(
      builder: (_) => QrSignResponsePage(
        responseJson: responseJson,
        decoded: prep.decoded,
      ),
    ),
  );
}

Future<bool?> _showActionConfirm(BuildContext context, SquareActionSignPrep prep) {
  return showDialog<bool>(
    context: context,
    builder: (dialogContext) => AlertDialog(
      title: const Text('确认签名'),
      content: Text(
        '账户：${_shortAddress(prep.wallet.address)}\n'
        '操作：${prep.decoded.displayTitle}\n\n'
        '确认后将用本机钱包对此操作签名。',
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(dialogContext).pop(false),
          child: const Text('取消'),
        ),
        TextButton(
          onPressed: () => Navigator.of(dialogContext).pop(true),
          child: const Text('确认签名'),
        ),
      ],
    ),
  );
}

String _shortAddress(String address) {
  if (address.length <= 12) return address;
  return '${address.substring(0, 6)}…${address.substring(address.length - 6)}';
}

void _snack(BuildContext context, String message) {
  ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text(message)));
}
