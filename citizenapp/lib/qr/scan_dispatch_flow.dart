import 'package:flutter/material.dart';

import 'package:citizenapp/qr/pages/qr_scan_page.dart';
import 'package:citizenapp/qr/pages/qr_sign_response_page.dart';
import 'package:citizenapp/qr/qr_protocols.dart';
import 'package:citizenapp/signer/square_action_sign_service.dart';
import 'package:citizenapp/signer/citizen_identity_sign_service.dart';
import 'package:citizenapp/signer/qr_signer.dart';
import 'package:citizenapp/transaction/offchain-transaction/services/offchain_scan_flow.dart';
import 'package:citizenapp/wallet/core/secure_seed_store.dart';
import 'package:citizenapp/wallet/core/seed_sign_error.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 交易 tab「扫一扫」统一入口：扫码 → 按协议分派。
///
/// - 收款 / 链下支付码 → 现有链下支付流程（用交易页选的 [paymentWallet]）。
/// - 广场账户动作 signRequest → 签名响应方（用 QR `u` 对应的 owner 钱包，与付款钱包无关）。
/// - 未来其它类型只需在此加分支。
Future<void> openScanDispatchFlow({
  required BuildContext context,
  required WalletProfile? paymentWallet,
  WalletProfile? signingWallet,
}) async {
  final scanned = await Navigator.of(context).push<Object?>(
    MaterialPageRoute(
        builder: (_) => const QrScanPage(mode: QrScanMode.dispatch)),
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
    final action = QrSigner().parseRequest(scanned).body.action;
    if (action == QrActions.citizenIdentity) {
      await _handleCitizenIdentitySignRequest(context, scanned, signingWallet);
    } else {
      await _handleSquareActionSignRequest(context, scanned);
    }
  }
}

Future<void> _handleSquareActionSignRequest(
    BuildContext context, String raw) async {
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
  } on SecureSeedException catch (e) {
    // 生物识别取消 / 无锁屏 / 金库错误：此前只捕 WalletAuthException，
    // 这类异常会逃逸成无声失败（点签名后无任何反应）。
    if (context.mounted) _snack(context, seedSignErrorMessage(e));
    return;
  } on WalletAuthException catch (e) {
    if (context.mounted) _snack(context, e.message);
    return;
  } on Exception catch (e) {
    // 兜底：任何签名异常都必须有反馈，永不静默。
    if (context.mounted) _snack(context, '签名失败：$e');
    return;
  }
  if (!context.mounted) return;

  await Navigator.of(context).push(
    MaterialPageRoute(
      builder: (_) => QrSignResponsePage(
        responseJson: responseJson,
        actionLabel: prep.actionLabel,
        reviewEntries: prep.decoded.reviewFields!
            .map((field) => (field.label, field.value))
            .toList(),
      ),
    ),
  );
}

Future<void> _handleCitizenIdentitySignRequest(
  BuildContext context,
  String raw,
  WalletProfile? signingWallet,
) async {
  final service = CitizenIdentitySignService();
  final walletManager = WalletManager();
  try {
    final prep = await service.prepare(
      raw,
      walletManager,
      requiredWallet: signingWallet,
    );
    if (!context.mounted) return;
    final fields = prep.decoded.reviewEntries;
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: Text(prep.actionLabel),
        content:
            Text(fields.map((field) => '${field.$1}：${field.$2}').join('\n')),
        actions: [
          TextButton(
              onPressed: () => Navigator.pop(dialogContext, false),
              child: const Text('取消')),
          FilledButton(
              onPressed: () => Navigator.pop(dialogContext, true),
              child: const Text('确认签名')),
        ],
      ),
    );
    if (confirmed != true || !context.mounted) return;
    final response = await service.sign(prep, walletManager);
    if (!context.mounted) return;
    await Navigator.of(context).push(MaterialPageRoute(
      builder: (_) => QrSignResponsePage(
        responseJson: response,
        actionLabel: prep.actionLabel,
        reviewEntries: fields,
      ),
    ));
  } on CitizenIdentitySignException catch (error) {
    if (context.mounted) _snack(context, error.message);
  } on SecureSeedException catch (error) {
    if (context.mounted) _snack(context, seedSignErrorMessage(error));
  } on WalletAuthException catch (error) {
    if (context.mounted) _snack(context, error.message);
  } on Exception catch (error) {
    if (context.mounted) _snack(context, '签名失败：$error');
  }
}

Future<bool?> _showActionConfirm(
    BuildContext context, SquareActionSignPrep prep) {
  final fieldLines = prep.decoded.reviewFields!
      .map((field) => '${field.label}：${field.value}')
      .join('\n');
  return showDialog<bool>(
    context: context,
    builder: (dialogContext) => AlertDialog(
      title: const Text('确认签名'),
      content: Text(
        '账户：${_shortAddress(prep.wallet.address)}\n'
        '动作：${prep.actionLabel}\n'
        '$fieldLines\n\n'
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
