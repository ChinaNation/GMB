import 'package:flutter/material.dart';

import 'package:citizenapp/qr/pages/qr_scan_page.dart';
import 'package:citizenapp/qr/pages/qr_sign_response_page.dart';
import 'package:citizenapp/qr/qr_protocols.dart';
import 'package:citizenapp/signer/square_action_sign_service.dart';
import 'package:citizenapp/signer/citizen_identity_sign_service.dart';
import 'package:citizenapp/signer/citizen_occupy_sign_service.dart';
import 'package:citizenapp/signer/qr_signer.dart';
import 'package:citizenapp/transaction/offchain-transaction/services/offchain_scan_flow.dart';
import 'package:citizenapp/wallet/core/secure_seed_store.dart';
import 'package:citizenapp/wallet/core/seed_sign_error.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 交易 tab「扫一扫」统一入口：扫码 → 按协议分派。
///
/// - 收款 / 链下支付码 → 现有链下支付流程（用交易页选的 [paymentWallet]）。
/// - 广场账户动作 signRequest → 签名响应方（用 QR `u` 对应的 accountId 钱包，与付款钱包无关）。
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
    } else if (QrActions.isSelfAccountDomainAction(action)) {
      await _handleOccupySignRequest(context, scanned);
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

/// 注册局占号/换绑:请求 b.u 留空,用户自选一个本机热账户绑定到该 CID,再签内层域签名。
Future<void> _handleOccupySignRequest(
  BuildContext context,
  String raw,
) async {
  final walletManager = WalletManager();
  final service = CitizenOccupySignService();

  final selected = await _pickBindingWallet(context, walletManager);
  if (selected == null || !context.mounted) return;

  try {
    final prep = await service.prepare(raw, selected);
    if (!context.mounted) return;
    final reviewEntries = <(String, String)>[
      ('身份CID', prep.cidNumber),
      ('绑定账户', _shortAddress(prep.wallet.accountId)),
    ];
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: Text(prep.actionLabel),
        content: Text(
          '${prep.isOccupy ? '把此 CID 占号绑定到你的账户' : '把此 CID 换绑到你的新账户'}\n'
          '身份CID：${prep.cidNumber}\n'
          '绑定账户：${_shortAddress(prep.wallet.accountId)}',
        ),
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
        reviewEntries: reviewEntries,
      ),
    ));
  } on CitizenOccupySignException catch (error) {
    if (context.mounted) _snack(context, error.message);
  } on SecureSeedException catch (error) {
    if (context.mounted) _snack(context, seedSignErrorMessage(error));
  } on WalletAuthException catch (error) {
    if (context.mounted) _snack(context, error.message);
  } on Exception catch (error) {
    if (context.mounted) _snack(context, '签名失败：$error');
  }
}

/// 占号/换绑:从本机热账户中选一个绑定到该 CID(占即绑一账户)。返回 null=取消。
Future<WalletProfile?> _pickBindingWallet(
  BuildContext context,
  WalletManager walletManager,
) async {
  final wallets =
      (await walletManager.getWallets()).where((w) => !w.isColdWallet).toList();
  if (!context.mounted) return null;
  if (wallets.isEmpty) {
    _snack(context, '本机没有可绑定的热账户');
    return null;
  }
  return showModalBottomSheet<WalletProfile>(
    context: context,
    builder: (sheetContext) => SafeArea(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Padding(
            padding: EdgeInsets.all(16),
            child: Text('选择要绑定到该 CID 的账户',
                style: TextStyle(fontSize: 16, fontWeight: FontWeight.w600)),
          ),
          for (final wallet in wallets)
            ListTile(
              title: Text(wallet.walletName),
              subtitle: Text(_shortAddress(wallet.accountId)),
              onTap: () => Navigator.of(sheetContext).pop(wallet),
            ),
          const SizedBox(height: 8),
        ],
      ),
    ),
  );
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
        '账户：${_shortAddress(prep.wallet.ss58Address)}\n'
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
