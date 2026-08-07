import 'package:flutter/material.dart';

import 'package:citizenapp/citizen/shared/account_derivation.dart';
import 'package:citizenapp/qr/bodies/user_contact_body.dart';
import 'package:citizenapp/qr/envelope.dart';
import 'package:citizenapp/qr/qr_protocols.dart';
import 'package:citizenapp/qr/widgets/qr_display_scaffold.dart';

/// 用户码展示页(`QR_V1 k=3 user_contact`,固定码)。
///
/// 用户码表达「人」——永久 CID 与其当前绑定账户(码内不含昵称)。**唯一入口是用户主页**,
/// 只有链上 CID↔AccountId 闭环命中的身份账户才能出这张码。账户维度的展示走账户码
/// (`lib/wallet/pages/wallet_qr_page.dart`),一笔收款请求走收款码;三者按入口分类,
/// 本页不做任何「该出哪种码」的运行时判断。
///
/// 用户码是唯一能写入通讯录的码:通讯录关系必须锚永久 CID,不能锚会换绑的账户。
class UserQrPage extends StatelessWidget {
  const UserQrPage({
    super.key,
    required this.cidNumber,
    required this.displayName,
    required this.accountId,
  });

  /// 永久公民身份号,身份主键。
  final String cidNumber;

  /// 公开昵称,只作展示。
  final String displayName;

  /// CID 当前绑定的钱包账户(0x + 64hex)。
  final String accountId;

  /// 展示态 SS58 地址(accountId 为授权真源,ss58 仅用于展示与二维码载荷)。
  String get _ss58Address => ss58FromAccountIdText(accountId);

  String _buildQrData() {
    return QrEnvelope<UserContactBody>(
      kind: QrKind.userContact,
      id: null,
      issuedAt: null,
      expiresAt: null,
      // 码内只放身份主键与账户标识:昵称由扫码端按 CID 从服务端拉取(本机昵称
      // 可随意改写,进码即冒名风险);SS58 是展示形态,扫码端自行派生。
      body: UserContactBody(cidNumber: cidNumber, accountId: accountId),
    ).toRawJson();
  }

  @override
  Widget build(BuildContext context) {
    return QrDisplayScaffold(
      headline: displayName,
      qrData: _buildQrData(),
      ss58Address: _ss58Address,
      footerText: '扫描此二维码可加为联系人，或向其转账',
    );
  }
}
