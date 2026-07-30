import 'package:flutter/material.dart';

import 'package:citizenapp/citizen/shared/account_derivation.dart'
    show ss58FromAccountIdText;
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
/// - signRequest → 用 QR `u` 对应的本机 Account 签名，与付款钱包无关。
/// - 未来其它类型只需在此加分支。
Future<void> openScanDispatchFlow({
  required BuildContext context,
  required WalletProfile? paymentWallet,
  Account? signingAccount,
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
    await _dispatchSignRequest(context, scanned, signingAccount);
  }
}

/// 我的钱包账户卡“扫码签名”：保留扫码页原 UI，只把业务边界收紧为签名请求。
///
/// 使用 raw 模式是为了不让收款码进入支付分支；二维码返回后仍由 [QrSigner.parseRequest]
/// 完整校验 QR_V1 类型、字段和有效期。
Future<void> openAccountScanSignFlow({
  required BuildContext context,
  required Account account,
}) async {
  final scanned = await Navigator.of(context).push<String>(
    MaterialPageRoute(
      builder: (_) => const QrScanPage(
        mode: QrScanMode.raw,
        customTitle: '扫码签名',
      ),
    ),
  );
  if (scanned == null || !context.mounted) return;
  await _dispatchSignRequest(context, scanned, account);
}

Future<void> _dispatchSignRequest(
  BuildContext context,
  String raw,
  Account? requiredAccount,
) async {
  final int action;
  try {
    action = QrSigner().parseRequest(raw).body.action;
  } on QrSignException catch (error) {
    if (context.mounted) _snack(context, '请扫描签名请求二维码：${error.message}');
    return;
  }
  if (action == QrActions.citizenIdentity) {
    await _handleCitizenIdentitySignRequest(context, raw, requiredAccount);
  } else if (QrActions.isSelfAccountDomainAction(action)) {
    await _handleOccupySignRequest(context, raw, requiredAccount);
  } else {
    await _handleSquareActionSignRequest(context, raw, requiredAccount);
  }
}

Future<void> _handleSquareActionSignRequest(
  BuildContext context,
  String raw,
  Account? requiredAccount,
) async {
  final service = SquareActionSignService();
  final walletManager = WalletManager();

  final SquareActionSignPrep prep;
  try {
    prep = await service.prepare(
      raw,
      walletManager,
      requiredAccount: requiredAccount,
    );
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
  Account? signingAccount,
) async {
  final service = CitizenIdentitySignService();
  final walletManager = WalletManager();
  try {
    final prep = await service.prepare(
      raw,
      walletManager,
      requiredAccount: signingAccount,
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

/// 注册局占号/换绑：请求 b.u 留空，完整授权模板内的账户槽必须为零；用户选择本机
/// 热账户后，服务把账户原位填入并签名。确认页展示全部防重放字段，禁止盲签。
Future<void> _handleOccupySignRequest(
  BuildContext context,
  String raw,
  Account? requiredAccount,
) async {
  final walletManager = WalletManager();
  final service = CitizenOccupySignService();

  final selected =
      requiredAccount ?? await _pickBindingAccount(context, walletManager);
  if (selected == null || !context.mounted) return;

  try {
    final prep = await service.prepare(raw, selected);
    if (!context.mounted) return;
    final reviewEntries = <(String, String)>[
      ('创世哈希', prep.genesisHash),
      ('身份CID', prep.cidNumber),
      if (prep.expectedOldAccountId != null)
        ('当前绑定账户', ss58FromAccountIdText(prep.expectedOldAccountId!)),
      ('预期绑定版本', prep.expectedBindingRevision.toString()),
      ('过期时间（Unix 秒）', prep.expiresAt.toString()),
      (prep.isOccupy ? '绑定账户' : '新绑定账户', prep.account.ss58Address),
    ];
    final reviewText =
        reviewEntries.map((entry) => '${entry.$1}：${entry.$2}').join('\n');
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: Text(prep.actionLabel),
        content: SingleChildScrollView(
          child: SelectableText(
            '${prep.isOccupy ? '把此 CID 占号绑定到你的账户' : '把此 CID 换绑到你的新账户'}\n'
            '$reviewText',
          ),
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

/// 通用扫一扫遇到占号/换绑时，从唯一热钱包的全部账户中选一个；账户卡入口则直接
/// 使用卡片账户，不进入本选择器。
Future<Account?> _pickBindingAccount(
  BuildContext context,
  WalletManager walletManager,
) async {
  final wallets = await walletManager.getWallets();
  WalletProfile? hotWallet;
  for (final wallet in wallets) {
    if (wallet.isHotWallet) {
      hotWallet = wallet;
      break;
    }
  }
  final accounts = hotWallet == null
      ? const <Account>[]
      : await walletManager.getAccounts(hotWallet.accountId);
  if (!context.mounted) return null;
  if (accounts.isEmpty) {
    _snack(context, '本机没有可绑定的热账户');
    return null;
  }
  return showModalBottomSheet<Account>(
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
          for (final account in accounts)
            ListTile(
              title: Text(account.accountName),
              subtitle: Text(_shortAddress(account.ss58Address)),
              onTap: () => Navigator.of(sheetContext).pop(account),
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
        '账户：${_shortAddress(prep.account.ss58Address)}\n'
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
