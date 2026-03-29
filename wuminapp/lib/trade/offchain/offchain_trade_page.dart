import 'package:flutter/material.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';

class OffchainTradePage extends StatelessWidget {
  const OffchainTradePage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('链下交易')),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Container(
          decoration: AppTheme.cardDecoration(),
          padding: const EdgeInsets.all(16),
          child: const Text(
            '链下交易为第二阶段开发内容。\n'
            '后续将在该模块实现账本、下单/撤单、撮合成交、清结算与自动对账。',
            style: TextStyle(height: 1.5),
          ),
        ),
      ),
    );
  }
}
