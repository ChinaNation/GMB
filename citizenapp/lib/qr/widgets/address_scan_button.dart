import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';

import 'package:citizenapp/qr/pages/qr_scan_page.dart';

/// 收款地址输入框内的扫码按钮，只用作 `InputDecoration.suffixIcon`。
///
/// 全仓「扫码填地址」的唯一实现：图标资产、扫码模式、页面 push 与空结果/unmounted
/// 守卫全部收在此处。此前多签转账页与安全基金转账页各抄了一份逐字节相同的写法。
///
/// 扫码固定走 [QrScanMode.transfer]：只认地址类码（用户码 / 账户码 / 收款码 / 裸地址）
/// 并只取其中的收款地址，**永远不进签名分支** —— 签名请求统一在「聊天 → 扫一扫」处理。
class AddressScanButton extends StatelessWidget {
  const AddressScanButton({super.key, required this.onAddressScanned});

  /// 扫到地址后回传 SS58。
  ///
  /// 只回传、不代写调用方的 controller：各页提交按钮可用态、错误文案清除等副作用
  /// 只有调用方自己清楚，代写会悄悄吃掉它们的 `setState`。
  final ValueChanged<String> onAddressScanned;

  @override
  Widget build(BuildContext context) {
    return IconButton(
      tooltip: '扫码填入收款地址',
      onPressed: () => _scan(context),
      icon: SvgPicture.asset(
        // 全仓扫码图标唯一资产。
        'assets/icons/scan-line.svg',
        width: 18,
        height: 18,
      ),
    );
  }

  Future<void> _scan(BuildContext context) async {
    final result = await Navigator.of(context).push<QrScanTransferResult>(
      MaterialPageRoute(
        builder: (_) => const QrScanPage(
          mode: QrScanMode.transfer,
          customTitle: '扫码填入收款地址',
        ),
      ),
    );
    if (result == null || !context.mounted) return;
    onAddressScanned(result.toSs58Address);
  }
}
