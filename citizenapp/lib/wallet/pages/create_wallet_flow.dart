import 'dart:async';

import 'package:flutter/material.dart';
import 'package:citizenapp/my/util/screenshot_guard.dart';
import 'package:citizenapp/rpc/chain_tx_monitor.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 本地 Isar/MDBX 繁忙属于钱包数据库问题，不能提示成区块链网络异常。
bool isWalletLocalStoreError(Object? error) {
  final raw = error.toString().toLowerCase();
  return raw.contains('isar') ||
      raw.contains('mdbx') ||
      raw.contains('active transaction') ||
      raw.contains('database');
}

String walletLocalStoreErrorMessage(Object? error) {
  if (isWalletLocalStoreError(error)) {
    return '本地钱包数据库繁忙，请稍后重试';
  }
  return '本地钱包读取失败：$error';
}

String walletOperationErrorMessage(Object error) {
  if (isWalletLocalStoreError(error)) {
    return walletLocalStoreErrorMessage(error);
  }
  return '$error';
}

/// 创建热钱包完整流程：生成密钥并落库 → 记基线余额 → 防截屏展示助记词备份弹窗。
///
/// 钱包页与首启强制创建页共用。创建失败向上抛出，由调用方决定错误展示；
/// 返回时钱包已落库（备份弹窗即使被进程杀死跳过，助记词仍可在钱包详情查看）。
Future<WalletCreationResult> runCreateWalletFlow(
  BuildContext context, {
  required int wordCount,
}) async {
  final created = await WalletManager().createWallet(wordCount: wordCount);
  unawaited(ChainTxMonitor.instance.initBaselineBalance(
    created.profile.address,
    created.profile.pubkeyHex,
  ));
  if (!context.mounted) {
    return created;
  }
  await ScreenshotGuard.enable();
  if (!context.mounted) {
    return created;
  }
  await showDialog<void>(
    context: context,
    barrierDismissible: false,
    builder: (context) {
      return AlertDialog(
        title: const Text('请备份助记词'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              '助记词已加密存储在本机，后续可在钱包详情中查看。\n'
              '请务必手抄备份并妥善保管，这是恢复钱包的唯一凭证。\n'
              '不支持复制，不支持截屏。',
            ),
            const SizedBox(height: 12),
            Text(
              created.mnemonic,
              style: const TextStyle(fontWeight: FontWeight.w600),
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('我已备份'),
          ),
        ],
      );
    },
  );
  await ScreenshotGuard.disable();
  return created;
}
