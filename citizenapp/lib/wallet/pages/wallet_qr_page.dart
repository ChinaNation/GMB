import 'package:flutter/material.dart';

import 'package:citizenapp/citizen/shared/account_derivation.dart';
import 'package:citizenapp/qr/bodies/wallet_code_body.dart';
import 'package:citizenapp/qr/envelope.dart';
import 'package:citizenapp/qr/qr_protocols.dart';
import 'package:citizenapp/qr/widgets/qr_display_scaffold.dart';

/// 钱包码展示页(`QR_V1 k=5 wallet_code`,固定码)。
///
/// 钱包码表达「账户」,载荷只有 `account_id` 一个字段。任意账户无条件生成,包括
/// CID 已绑定的身份账户——账户详情表达的是账户,身份由用户主页的用户码表达。
///
/// 固定码,无时效:钱包码必须在离线的 CitizenWallet 也能生成,而离线设备无 NTP,
/// 不得签发带绝对时间戳的凭证。
///
/// 合法扫码场景只有三类:按账户转账、OnChina 管理员登录第 1 步、OnChina 与
/// citizenchain node 前端的「扫码识别账户」。写入通讯录必须被拒绝(无 CID 真源)。
class WalletQrPage extends StatelessWidget {
  const WalletQrPage({
    super.key,
    required this.accountId,
    required this.accountLabel,
  });

  /// 账户唯一标识(0x + 64hex),载荷中唯一字段。
  final String accountId;

  /// 本机账户/钱包标签,只在本机顶部展示,**绝不进载荷**——本机标签用户可随意改写,
  /// 一旦进入二维码就会被扫码端当成对方公开身份显示。
  final String accountLabel;

  /// 展示态 SS58 地址,由 accountId 派生。
  String get _ss58Address => ss58FromAccountIdText(accountId);

  String _buildQrData() {
    return QrEnvelope<WalletCodeBody>(
      kind: QrKind.walletCode,
      id: null,
      issuedAt: null,
      expiresAt: null,
      body: WalletCodeBody(accountId: accountId),
    ).toRawJson();
  }

  @override
  Widget build(BuildContext context) {
    return QrDisplayScaffold(
      headline: accountLabel,
      qrData: _buildQrData(),
      ss58Address: _ss58Address,
      footerText: '钱包码：扫描可向本账户转账，或用于扫码登录',
    );
  }
}

/// 打开钱包码页。account_id 非法时不进页面,直接提示。
Future<void> openWalletQrPage(
  BuildContext context, {
  required String accountId,
  required String accountLabel,
}) async {
  final normalizedAccountId = accountId.trim();
  if (!isAccountIdText(normalizedAccountId)) {
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('账户标识无效，无法生成二维码')),
    );
    return;
  }
  await Navigator.of(context).push<void>(
    MaterialPageRoute<void>(
      builder: (_) => WalletQrPage(
        accountId: normalizedAccountId,
        accountLabel: accountLabel,
      ),
    ),
  );
}
