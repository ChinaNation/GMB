import 'package:flutter/material.dart';
import 'package:qr_flutter/qr_flutter.dart';

import '../../qr/bodies/wallet_code_body.dart';
import '../../qr/envelope.dart';
import '../../qr/qr_protocols.dart';
import '../app_theme.dart';

/// 钱包码（`QR_V1 k=5 wallet_code`）的构造与展示单源。
///
/// 账户详情页与钱包详情页的账户列表共用本文件：同一个账户在两处入口打开的二维码
/// 必须逐字节一致，不得各造一份。

/// 为账户生成固定钱包码（`k=5`）。
///
/// 钱包码只声明账户，不带时效、不带账户名、不带 CID/昵称。公民钱包完全离线、无 NTP，
/// 不得签发带绝对时间戳的凭证；也没有 CID↔AccountId 链上真源，不得伪造 `k=3` 用户码。
String buildWalletQr({required String accountId}) {
  return QrEnvelope<WalletCodeBody>(
    kind: QrKind.walletCode,
    id: null,
    expiresAt: null,
    body: WalletCodeBody(accountId: accountId),
  ).toRawJson();
}

/// 弹出该账户的钱包码。
///
/// [accountName] 只在弹窗顶部本机展示，**绝不进载荷**——本机标签用户可随意改写，
/// 一旦进入二维码就会被扫码端当成对方公开身份显示。
Future<void> showWalletQrDialog(
  BuildContext context, {
  required String accountId,
  required String accountName,
  required String ss58Address,
}) {
  final qrData = buildWalletQr(accountId: accountId);
  return showDialog<void>(
    context: context,
    builder: (context) => Dialog(
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(16),
      ),
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(accountName,
                style: const TextStyle(
                    fontSize: 16, fontWeight: FontWeight.w600)),
            const SizedBox(height: 16),
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: Colors.white,
                borderRadius: BorderRadius.circular(12),
                border: Border.all(color: AppTheme.border),
              ),
              child: QrImageView(
                data: qrData,
                version: QrVersions.auto,
                size: 240,
                eyeStyle: const QrEyeStyle(
                  eyeShape: QrEyeShape.square,
                  color: AppTheme.primaryDark,
                ),
                dataModuleStyle: const QrDataModuleStyle(
                  dataModuleShape: QrDataModuleShape.square,
                  color: AppTheme.primaryDark,
                ),
              ),
            ),
            const SizedBox(height: 12),
            const Text(
              '钱包码：扫描可向本账户转账，或用于扫码登录',
              style: TextStyle(
                fontSize: 12,
                color: AppTheme.textSecondary,
              ),
            ),
            const SizedBox(height: 8),
            SelectableText(
              ss58Address,
              style: const TextStyle(
                  fontSize: 11,
                  color: AppTheme.textSecondary,
                  fontFamily: 'monospace'),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 12),
            SizedBox(
              width: double.infinity,
              child: TextButton(
                onPressed: () => Navigator.of(context).pop(),
                child: const Text('关闭'),
              ),
            ),
          ],
        ),
      ),
    ),
  );
}
